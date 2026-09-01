//! Simplified index test helpers for E2E output and workflow tests
//!
//! This module provides test helpers that wrap existing integration test functionality.

use std::sync::Arc;

use crate::fixture::FixtureAccess;
use async_trait::async_trait;
use cce_orchestrator::index::{
    RelationPublication, RelationSnapshotPublisher, ResolutionPipelineService,
};
use cce_plugin::PluginRegistry;
use cce_storage_bm25::Bm25Client;
use cce_storage_sqlite::RelationSnapshotRepository;
use cce_storage_sqlite::SqliteClient;
use cce_types::{CanonicalRelationSnapshot, SnapshotDelta, StorageError};

/// Minimal relation-snapshot publisher for workflow tests.
///
/// Persists canonical snapshots through the real resolution pipeline so the
/// full-index relation path exercises genuine storage writes instead of being
/// short-circuited by a missing publisher.
pub struct TestRelationPublisher {
    sqlite: SqliteClient,
    pipeline: ResolutionPipelineService,
}

impl TestRelationPublisher {
    /// Create a publisher backed by a fresh temporary SQLite database.
    pub fn new() -> Self {
        let sqlite = SqliteClient::new_in_temp().expect("temporary sqlite client");
        Self::with_sqlite(sqlite)
    }

    /// Create a publisher bound to an existing SQLite database, so hot-update
    /// relation publications share the durable state of the test harness.
    pub fn with_sqlite(sqlite: SqliteClient) -> Self {
        Self {
            pipeline: ResolutionPipelineService::new(sqlite.clone()),
            sqlite,
        }
    }

    /// Ensure the project row exists so relation tables with foreign keys can
    /// reference it.
    fn ensure_project(&self, project_id: i64) -> Result<(), StorageError> {
        self.sqlite.with_transaction(|tx| {
            tx.execute(
                "INSERT OR IGNORE INTO projects (id, name, root_path, config_file_path, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '.cce/config.json', ?4, ?4)",
                rusqlite::params![project_id, format!("project-{project_id}"), format!("/project/{project_id}"), chrono::Utc::now().timestamp()],
            )
            .map_err(|error| StorageError::Insert(error.to_string()))?;
            Ok(())
        })
    }
}

impl Default for TestRelationPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RelationSnapshotPublisher for TestRelationPublisher {
    async fn publish(
        &self,
        project_id: i64,
        operation_id: &str,
        snapshot: CanonicalRelationSnapshot,
        _index: &cce_relation::index::RelationIndex,
    ) -> Result<RelationPublication, StorageError> {
        self.ensure_project(project_id)?;
        let relation_epoch =
            self.pipeline
                .allocate_and_write(project_id, operation_id, &snapshot)?;
        // Activate like the production publisher does: activation promotes the
        // ready epoch and records `active_relation_epoch` in project_meta, so
        // downstream delta flows have a CAS base.
        self.pipeline.activate(project_id, relation_epoch)?;
        Ok(RelationPublication { relation_epoch })
    }

    async fn publish_delta(
        &self,
        project_id: i64,
        operation_id: &str,
        delta: SnapshotDelta,
        _base: Option<cce_relation::index::LayeredSnapshotIndex>,
    ) -> Result<RelationPublication, StorageError> {
        self.ensure_project(project_id)?;

        // CAS: the candidate was built from the active epoch; reject it if a
        // concurrent publication advanced past the base.
        let active_epoch = self
            .sqlite
            .project_meta_get_int(project_id, "active_relation_epoch")
            .map_err(|error| {
                StorageError::Validation(format!("cannot read active epoch for CAS: {error}"))
            })?;
        if active_epoch != delta.base_epoch {
            return Err(StorageError::epoch_conflict(active_epoch, delta.base_epoch));
        }

        let epoch = self
            .sqlite
            .with_transaction(|tx| {
                let epoch = RelationSnapshotRepository::allocate_building(
                    tx,
                    project_id,
                    operation_id,
                    &delta.config_fingerprint,
                )?;
                RelationSnapshotRepository::write_delta(tx, project_id, epoch, &delta)?;
                Ok(epoch)
            })
            .map_err(|error| {
                StorageError::Validation(format!("failed to persist relation delta: {error}"))
            })?;

        Ok(RelationPublication {
            relation_epoch: epoch,
        })
    }
}

/// Index workflow test helper for workflow-level tests
pub struct IndexWorkflowTest<F: FixtureAccess> {
    pub fixture: F,
    pub project_id: i64,
    pub extensions: Vec<String>,
    pub excludes: Vec<String>,
    pub with_bm25: bool,
    pub with_vectors: bool,
    pub with_relations: bool,
    pub relation_publisher: Option<Arc<dyn RelationSnapshotPublisher>>,
    pub plugin_registry: Option<Arc<PluginRegistry>>,
    pub bm25_client: Option<Arc<tokio::sync::Mutex<Bm25Client>>>,
    pub(crate) orchestrator: Option<cce_orchestrator::IndexOrchestrator>,
    pub(crate) last_result: Option<cce_orchestrator::IndexResult>,
}

impl<F: FixtureAccess> IndexWorkflowTest<F> {
    pub fn new(fixture: F) -> Self {
        Self {
            fixture,
            project_id: 1,
            extensions: vec!["rs".to_string()],
            excludes: vec![],
            with_bm25: false,
            with_vectors: false,
            with_relations: false,
            relation_publisher: None,
            plugin_registry: None,
            bm25_client: None,
            orchestrator: None,
            last_result: None,
        }
    }

    pub fn with_project_id(mut self, project_id: i64) -> Self {
        self.project_id = project_id;
        self
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn with_bm25(mut self, enabled: bool) -> Self {
        self.with_bm25 = enabled;
        self
    }

    pub fn with_vectors(mut self, enabled: bool) -> Self {
        self.with_vectors = enabled;
        self
    }

    pub fn with_relations(mut self, enabled: bool) -> Self {
        self.with_relations = enabled;
        self
    }

    /// Install a custom relation-snapshot publisher.
    ///
    /// When relations are enabled without an explicit publisher, a
    /// [`TestRelationPublisher`] is created automatically.
    pub fn with_relation_publisher(
        mut self,
        publisher: Arc<dyn RelationSnapshotPublisher>,
    ) -> Self {
        self.relation_publisher = Some(publisher);
        self
    }

    /// Mount a plugin registry so the full index pipeline (`TextGen`,
    /// `EntityExtract`, `Group`/`GroupOverride`, `Chunk`, `FormatParse`)
    /// runs against real plugin implementations.
    pub fn with_plugin_registry(mut self, registry: Arc<PluginRegistry>) -> Self {
        self.plugin_registry = Some(registry);
        self
    }

    /// Use an externally-owned BM25 client so tests can inspect the stored
    /// documents after indexing (e.g. to assert plugin-generated text).
    pub fn with_bm25_client(mut self, client: Arc<tokio::sync::Mutex<Bm25Client>>) -> Self {
        self.bm25_client = Some(client);
        self
    }

    pub fn fixture(&self) -> &F {
        &self.fixture
    }

    pub async fn execute(&mut self) -> anyhow::Result<cce_orchestrator::IndexResult> {
        use cce_orchestrator::{IndexOptions, IndexOrchestrator};

        let mut orchestrator =
            IndexOrchestrator::new(self.project_id).expect("failed to create IndexOrchestrator");

        if self.with_relations {
            let publisher = self
                .relation_publisher
                .clone()
                .unwrap_or_else(|| Arc::new(TestRelationPublisher::new()));
            orchestrator = orchestrator.with_relation_publisher(publisher);
        }
        if let Some(registry) = &self.plugin_registry {
            orchestrator = orchestrator.with_plugin_registry(registry.clone());
        }
        if let Some(bm25) = &self.bm25_client {
            orchestrator = orchestrator.with_bm25_client(bm25.clone());
        }

        self.orchestrator = Some(orchestrator);

        let options = IndexOptions {
            root_dir: self.fixture.root_path().to_path_buf(),
            extensions: self.extensions.clone(),
            exclude_dirs: self.excludes.clone(),
            build_relations: self.with_relations,
            store_vectors: self.with_vectors,
            store_bm25: self.with_bm25,
            ..Default::default()
        };

        let result = self
            .orchestrator
            .as_mut()
            .expect("Orchestrator not initialized")
            .execute(options)
            .await?;

        self.last_result = Some(result.clone());
        Ok(result)
    }

    pub async fn reindex(&mut self) -> anyhow::Result<cce_orchestrator::IndexResult> {
        self.execute().await
    }

    pub async fn rebuild(&mut self) -> anyhow::Result<cce_orchestrator::IndexResult> {
        self.execute().await
    }
}
