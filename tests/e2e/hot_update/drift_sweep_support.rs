//! Shared scaffolding for the chunking/embedding drift-sweep e2e scenarios.
//!
//! Builds a project over durable state (file-backed SQLite, real BM25 client,
//! capturing mock Qdrant) and drives embedding/BM25 processors through their
//! real prepare → process → commit lifecycle, exactly like a hot-update
//! operation would. No server, no network, no real LLM: vectors come from
//! [`HashEmbedder`], points land in the [`CapturingMockQdrant`].

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use tempfile::TempDir;
use tokio::sync::Mutex;

use cce_config::modules::{DistanceMetric, QdrantConfig};
use cce_config::{AstToNlConfig, NestProcessorConfig};
use cce_e2e_tests::mock_qdrant::CapturingMockQdrant;
use cce_e2e_tests::stub_embedder::HashEmbedder;
use cce_llm::Embedder;
use cce_orchestrator::hot_update::processors::ProcessorContext;
use cce_orchestrator::hot_update::{BatchChangeResult, FileChangeType, ParseResultWithChanges};
use cce_orchestrator::index::StorageCoordinator;
use cce_orchestrator::operation::{OperationContext, OperationProcessResult, OperationType};
use cce_orchestrator::{
    Bm25UpdateProcessor, CheckpointManager, EmbeddingUpdateProcessor, UpdateProcessor,
};
use cce_storage_bm25::{Bm25Client, Bm25Config};
use cce_storage_qdrant::{QdrantClient, types as qdrant_types};
use cce_storage_sqlite::SqliteClient;

/// Qdrant payload isolation group used for every scenario.
pub const GROUP_ID: &str = "drift-project-root";
const PROJECT_ID: i64 = 1;
const EMBEDDER_DIMENSION: usize = 8;

/// Durable state shared by all operations of one drift-sweep test.
pub struct DriftScaffold {
    pub fixture: crate::helper::EmptyFixture,
    pub sqlite: Arc<SqliteClient>,
    pub checkpoint_manager: Arc<CheckpointManager>,
    pub bm25: Arc<Mutex<Bm25Client>>,
    pub qdrant: CapturingMockQdrant,
    entity_id_seed: Arc<AtomicU64>,
    _db_dir: TempDir,
    _bm25_dir: TempDir,
}

impl DriftScaffold {
    pub const GROUP_ID: &str = GROUP_ID;

    pub async fn new() -> anyhow::Result<Self> {
        let fixture = crate::helper::EmptyFixture::new().context("failed to create fixture")?;
        let db_dir = TempDir::new().context("failed to create db temp dir")?;
        let scoped = SqliteClient::with_path(db_dir.path().join("cce.db").to_string_lossy())
            .context("failed to create file-backed SQLite")?
            .for_project(PROJECT_ID)
            .context("failed to scope SQLite to project")?;

        // Production semantics: root_path is the real project directory;
        // chunking-drift sweeps resolve swept files against it. The row was
        // already seeded by `for_project`, so update instead of insert.
        scoped
            .with_transaction(|tx| {
                tx.execute(
                    "UPDATE projects SET root_path = ?2 WHERE id = ?1",
                    rusqlite::params![PROJECT_ID, fixture.root().to_string_lossy(),],
                )
                .map(|_| ())
                .map_err(|e| cce_types::StorageError::update("projects", e.to_string()))
            })
            .context("failed to point the project row at the fixture root")?;

        let bm25_dir = TempDir::new().context("failed to create bm25 temp dir")?;
        let bm25_config = Bm25Config::default()
            .enabled()
            .with_index_name("default")
            .with_index_path(bm25_dir.path().join("index").to_string_lossy());
        let mut bm25 = Bm25Client::new(bm25_config);
        bm25.connect().await.context("failed to connect BM25")?;

        Ok(Self {
            fixture,
            sqlite: scoped.clone(),
            checkpoint_manager: Arc::new(CheckpointManager::new_for_project(PROJECT_ID, scoped)),
            bm25: Arc::new(Mutex::new(bm25)),
            qdrant: CapturingMockQdrant::spawn().await,
            entity_id_seed: Arc::new(AtomicU64::new(0)),
            _db_dir: db_dir,
            _bm25_dir: bm25_dir,
        })
    }

    pub fn root(&self) -> &Path {
        self.fixture.root()
    }

    pub fn add_file(&self, relative: &str, content: &str) -> anyhow::Result<PathBuf> {
        self.fixture
            .add_file(relative, content)
            .context("failed to add fixture file")
    }

    /// Storage coordinator pointed at the shared durable state. The epoch is
    /// only a fallback: `prepare_operation` advances it to the candidate
    /// epoch before any write happens.
    pub fn storage_with_embedder(&self, embedder: Arc<dyn Embedder>) -> Arc<StorageCoordinator> {
        self.storage_inner(Some(embedder))
    }

    /// Storage coordinator without an embedder (BM25-only scenarios).
    pub fn storage_without_embedder(&self) -> Arc<StorageCoordinator> {
        self.storage_inner(None)
    }

    fn storage_inner(&self, embedder: Option<Arc<dyn Embedder>>) -> Arc<StorageCoordinator> {
        let active_epoch = self.active_epoch();
        let qdrant_config = QdrantConfig {
            url: self.qdrant.url().to_string(),
            vector_size: EMBEDDER_DIMENSION,
            distance_metric: DistanceMetric::Cosine,
            timeout_ms: 5000,
            max_retries: 0,
            retry_delay_ms: 10,
            enabled: true,
            ..Default::default()
        };
        let qdrant =
            Arc::new(QdrantClient::new(qdrant_config, ".").expect("qdrant client must build"));
        let mut storage = StorageCoordinator::new(PROJECT_ID)
            .expect("valid project id")
            .with_metadata_store(self.sqlite.clone())
            .with_bm25(self.bm25.clone())
            .with_qdrant(qdrant)
            .with_project_group_id(GROUP_ID)
            .with_epoch(active_epoch);
        if let Some(embedder) = embedder {
            storage = storage.with_embedder(embedder);
        }
        Arc::new(storage)
    }

    fn active_epoch(&self) -> i64 {
        use cce_storage_sqlite::ProjectIndexManifestRepository;
        let conn = self.sqlite.read_connection().expect("read connection");
        ProjectIndexManifestRepository::get_active(&conn, PROJECT_ID)
            .ok()
            .flatten()
            .map(|manifest| manifest.data_epoch)
            .unwrap_or(0)
    }

    /// The currently published (active) data epoch, or 0 when never published.
    pub fn published_epoch(&self) -> i64 {
        self.active_epoch()
    }

    /// Processor context for the shared SQLite database. `ast_to_nl`
    /// overrides the chunking configuration (used to construct a drifted
    /// pipeline fingerprint).
    ///
    /// Both chunk paths are enabled so the embedding processor has vectors to
    /// write; the default config only emits BM25-path chunks.
    pub fn context(
        &self,
        storage: Arc<StorageCoordinator>,
        ast_to_nl: Option<&AstToNlConfig>,
    ) -> Arc<ProcessorContext> {
        let config = match ast_to_nl {
            Some(config) => config.clone(),
            None => Self::base_ast_to_nl_config(),
        };
        let index_processor = cce_orchestrator::index::FileProcessor::with_config(&config)
            .with_project_id(PROJECT_ID);
        let context = ProcessorContext::new(storage, NestProcessorConfig::default());
        let mut context = context.with_checkpoint_manager(self.checkpoint_manager.clone());
        context.file_processor = Arc::new(tokio::sync::Mutex::new(index_processor));
        Arc::new(context)
    }

    /// Base AST-to-NL config with both chunk output paths enabled.
    pub fn base_ast_to_nl_config() -> AstToNlConfig {
        AstToNlConfig {
            default_mode: cce_types::OutputMode::Both,
            ..AstToNlConfig::default()
        }
    }

    /// Embedding + BM25 processor pair sharing `context`.
    pub fn processor_pair(context: Arc<ProcessorContext>) -> Vec<Arc<dyn UpdateProcessor>> {
        vec![
            Arc::new(EmbeddingUpdateProcessor::new(Arc::clone(&context))),
            Arc::new(Bm25UpdateProcessor::new(context)),
        ]
    }

    /// Parse a fixture file with the hot-update pipeline, mirroring
    /// production's monotonic entity-ID seeding across files.
    pub async fn parse_file(&self, relative: &str) -> anyhow::Result<ParseResultWithChanges> {
        use cce_orchestrator::hot_update::FileProcessor;
        let absolute = self.root().join(relative);
        let seed = self.entity_id_seed.load(Ordering::Relaxed);
        let mut processor = FileProcessor::with_entity_id_seed(seed);
        let result = processor
            .process_file_change_at(
                &absolute,
                relative,
                FileChangeType::Modified,
                &Some(self.sqlite.clone()),
                PROJECT_ID,
            )
            .await
            .context("failed to parse fixture file")?;
        if let Some(max) = result.parsed_file.entities.iter().map(|e| e.id.0).max() {
            self.entity_id_seed
                .store(max.saturating_add(1), Ordering::Relaxed);
        }
        Ok(result)
    }

    /// Run one full hot-update operation (prepare → process → commit) over
    /// `batch` with the given processors. Mirrors production wiring by
    /// creating the operation checkpoint first — module progress markers
    /// carry a foreign key to it.
    pub async fn run_operation(
        &self,
        processors: &[Arc<dyn UpdateProcessor>],
        batch: BatchChangeResult,
        operation_id: &str,
    ) -> anyhow::Result<Vec<OperationProcessResult>> {
        use cce_orchestrator::operation::checkpoint::CreateCheckpointParams;
        use cce_types::OperationKind;

        self.checkpoint_manager
            .create_checkpoint(CreateCheckpointParams {
                operation_id,
                operation_type: OperationKind::HotUpdate,
                root_dir: &self.root().to_string_lossy(),
                total_files: batch.parse_results.len() as u32,
                batch_size: 1,
                file_list_hash: "",
            })
            .await
            .context("failed to create operation checkpoint")?;
        let path_of =
            |result: &ParseResultWithChanges| result.file_path.to_string_lossy().into_owned();
        let first_file = batch.parse_results.first().map(path_of).unwrap_or_default();
        let last_file = batch.parse_results.last().map(path_of).unwrap_or_default();
        self.checkpoint_manager
            .create_batch_checkpoint(
                operation_id,
                0,
                &first_file,
                &last_file,
                batch.parse_results.len() as u32,
            )
            .await
            .context("failed to create batch checkpoint")?;

        let ctx = OperationContext::new(
            PROJECT_ID,
            operation_id.to_string(),
            OperationType::HotUpdate,
            batch.parse_results.len(),
        );
        for processor in processors {
            processor
                .prepare_operation(&ctx)
                .await
                .context("prepare_operation failed")?;
        }
        let mut batch = batch;
        let mut results = Vec::new();
        for processor in processors {
            results.push(
                processor
                    .process_operation(&ctx, &mut batch)
                    .await
                    .context("process_operation failed")?,
            );
        }
        for processor in processors {
            processor
                .commit_operation(&ctx)
                .await
                .context("commit_operation failed")?;
        }

        // Mirror the change detector's post-publication hash backfill:
        // candidate file rows are written without content hashes and only
        // get them once the manifest is active.
        let published = self.published_epoch();
        let conn = self.sqlite.write_connection().expect("write connection");
        for result in &batch.parse_results {
            if let Some(hash) = result.parsed_file.file_hash.as_deref() {
                conn.execute(
                    "UPDATE files SET content_hash = ?1
                     WHERE project_id = ?2 AND epoch = ?3 AND path = ?4",
                    rusqlite::params![
                        hash,
                        PROJECT_ID,
                        published,
                        result.file_path.to_string_lossy()
                    ],
                )
                .expect("backfill file content hash");
            }
        }

        Ok(results)
    }

    pub fn meta_string(&self, key: &str) -> Option<String> {
        self.sqlite
            .project_meta_get_string_optional(PROJECT_ID, key)
            .expect("read project meta")
    }

    /// Chunk rows of one file at one epoch: `(chunk_id, entity_ids_json)`.
    pub fn chunks_at_epoch(&self, epoch: i64, path: &str) -> Vec<(String, String)> {
        let conn = self.sqlite.read_connection().expect("read connection");
        let mut stmt = conn
            .prepare(
                "SELECT chunk_id, entity_ids FROM chunks
                 WHERE project_id = ?1 AND epoch = ?2 AND file_path = ?3
                 ORDER BY chunk_id",
            )
            .expect("prepare chunk query");
        let rows = stmt
            .query_map(rusqlite::params![PROJECT_ID, epoch, path], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .expect("query chunks");
        rows.collect::<Result<Vec<_>, _>>().expect("collect chunks")
    }

    /// Entity detail mappings of one file at one epoch:
    /// `(source_entity_db_id, [qdrant point ids])`.
    pub fn mappings_at_epoch(&self, epoch: i64, path: &str) -> Vec<(i64, Vec<String>)> {
        let conn = self.sqlite.read_connection().expect("read connection");
        let mut stmt = conn
            .prepare(
                "SELECT m.entity_id, m.qdrant_point_ids FROM entity_detail_mappings m
                 JOIN entities e ON e.id = m.entity_id
                 JOIN files f ON f.id = e.file_id
                 WHERE m.project_id = ?1 AND m.epoch = ?2 AND f.path = ?3
                 ORDER BY m.entity_id",
            )
            .expect("prepare mapping query");
        let rows = stmt
            .query_map(rusqlite::params![PROJECT_ID, epoch, path], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .expect("query mappings");
        rows.collect::<Result<Vec<_>, _>>()
            .expect("collect mappings")
            .into_iter()
            .map(|(entity_id, ids_json)| {
                let ids: Vec<String> =
                    serde_json::from_str(&ids_json).unwrap_or_else(|_| Vec::new());
                (entity_id, ids)
            })
            .collect()
    }

    /// Number of file rows for `path` at `epoch`.
    pub fn file_rows_at_epoch(&self, epoch: i64, path: &str) -> i64 {
        let conn = self.sqlite.read_connection().expect("read connection");
        conn.query_row(
            "SELECT COUNT(*) FROM files WHERE project_id = ?1 AND epoch = ?2 AND path = ?3",
            rusqlite::params![PROJECT_ID, epoch, path],
            |row| row.get::<_, i64>(0),
        )
        .expect("count file rows")
    }

    /// Chunk row contents of one file at one epoch (embedding path).
    pub fn chunk_contents_at_epoch(&self, epoch: i64, path: &str) -> Vec<String> {
        let conn = self.sqlite.read_connection().expect("read connection");
        let mut stmt = conn
            .prepare(
                "SELECT content FROM chunks
                 WHERE project_id = ?1 AND epoch = ?2 AND file_path = ?3 AND path = 'emb'",
            )
            .expect("prepare chunk content query");
        let rows = stmt
            .query_map(rusqlite::params![PROJECT_ID, epoch, path], |row| {
                row.get::<_, String>(0)
            })
            .expect("query chunk contents");
        rows.collect::<Result<Vec<_>, _>>()
            .expect("collect chunk contents")
    }

    /// Total physical chunk rows of one file across all epochs.
    pub fn total_chunks_for_path(&self, path: &str) -> i64 {
        let conn = self.sqlite.read_connection().expect("read connection");
        conn.query_row(
            "SELECT COUNT(*) FROM chunks WHERE project_id = ?1 AND file_path = ?2",
            rusqlite::params![PROJECT_ID, path],
            |row| row.get::<_, i64>(0),
        )
        .expect("count total chunks")
    }

    /// BM25 documents visible at one epoch.
    pub async fn bm25_docs_at_epoch(&self, epoch: i64) -> usize {
        self.bm25
            .lock()
            .await
            .snapshot_documents(PROJECT_ID, epoch)
            .await
            .expect("snapshot bm25 documents")
            .len()
    }

    /// The UUID the client derives from a logical point id.
    pub fn uuid_of(logical_point_id: &str) -> String {
        qdrant_types::to_qdrant_point_id(logical_point_id).to_string()
    }

    /// Deterministic embedder bound to `model_name`.
    pub fn embedder(model_name: &str) -> Arc<HashEmbedder> {
        Arc::new(HashEmbedder::new(model_name, EMBEDDER_DIMENSION))
    }
}

/// Convenience: paths changed by an operation (for skip-set assertions).
pub fn changed_paths(batch: &BatchChangeResult) -> HashSet<String> {
    batch
        .parse_results
        .iter()
        .map(|result| result.file_path.to_string_lossy().to_string())
        .collect()
}
