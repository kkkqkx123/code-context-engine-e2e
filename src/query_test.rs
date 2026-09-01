//! Query workflow test helper
//!
//! This module provides a test helper for query workflow tests.

use anyhow::Result;
use std::sync::Arc;
use tempfile::TempDir;

use cce_config::project_registry::ProjectScope;
use cce_llm_client::OpenAICompatibleProvider;
use cce_orchestrator::{
    IndexOptions, IndexOrchestrator, IndexResult, QueryCoordinator, QueryOptions, QueryResult,
    SearchConfig, SearchSources,
};
use cce_storage_bm25::Bm25Config;
use cce_storage_qdrant::QdrantConfig;

use super::EmbeddingConfig;
use crate::fixture::FixtureAccess;

/// Query workflow test helper
///
/// Provides utilities for testing query workflows including:
/// - Vector search
/// - BM25 search
/// - Relation queries
/// - Hybrid search
pub struct QueryWorkflowTest<F: FixtureAccess> {
    /// Test fixture
    fixture: F,
    /// Embedding configuration
    embedding_config: EmbeddingConfig,
    /// Project ID for isolation
    project_id: i64,
    /// Index orchestrator
    index_orchestrator: Option<IndexOrchestrator>,
    /// Query coordinator
    query_coordinator: Option<QueryCoordinator>,
    /// Last index result
    last_index_result: Option<IndexResult>,
    /// Last query result
    last_query_result: Option<QueryResult>,
    /// Search sources configuration
    search_sources: SearchSources,
    /// Search configuration
    search_config: SearchConfig,
    /// Shared BM25 client (index + query use same instance)
    bm25_client: Option<Arc<tokio::sync::Mutex<cce_storage_bm25::Bm25Client>>>,
    /// Shared Qdrant client (index + query use same instance)
    qdrant_client: Option<Arc<cce_storage_qdrant::QdrantClient>>,
    /// Shared SQLite database (index + query use same instance)
    sqlite_db: Option<Arc<cce_storage_sqlite::SqliteClient>>,
    /// Shared embedder for vector index and query (optional; without it,
    /// vector storage is skipped during indexing)
    embedder: Option<Arc<OpenAICompatibleProvider>>,
    /// Metrics registry wired into the query searcher (optional)
    metrics_registry: Option<Arc<cce_metrics::MetricsRegistry>>,
    /// Generative rerank handler wired into the query searcher (optional)
    rerank_handler: Option<Arc<cce_llm_client::ProductionRerankHandler>>,
    /// Per-query rerank override used when building query options
    enable_rerank: Option<bool>,
    /// File extensions to index (defaults to the orchestrator defaults)
    extensions: Option<Vec<String>>,
    /// BM25 index temp directory, dropped after test completes
    _bm25_temp_dir: Option<TempDir>,
    /// Project group id shared by the index fingerprint and the query scope
    project_group_id: Option<String>,
    /// Optional stable scenario key. When set, the Qdrant group id derives
    /// from it instead of the per-run temp directory, so repeated runs of the
    /// same scenario reuse one partition of the shared collection (and stale
    /// points are deleted before indexing instead of accumulating).
    scenario_name: Option<String>,
}

impl<F: FixtureAccess> QueryWorkflowTest<F> {
    /// Create a new query workflow test
    pub fn new(fixture: F, embedding_config: EmbeddingConfig) -> Self {
        Self {
            fixture,
            embedding_config,
            project_id: 1,
            index_orchestrator: None,
            query_coordinator: None,
            last_index_result: None,
            last_query_result: None,
            search_sources: SearchSources::default(),
            search_config: SearchConfig::default(),
            bm25_client: None,
            qdrant_client: None,
            sqlite_db: None,
            embedder: None,
            metrics_registry: None,
            rerank_handler: None,
            enable_rerank: None,
            extensions: None,
            _bm25_temp_dir: None,
            project_group_id: None,
            scenario_name: None,
        }
    }

    /// Set the project ID for this test
    #[allow(dead_code)]
    pub fn with_project_id(mut self, project_id: i64) -> Self {
        self.project_id = project_id;
        self
    }

    /// Set search sources
    #[allow(dead_code)]
    pub fn with_sources(mut self, sources: SearchSources) -> Self {
        self.search_sources = sources;
        self
    }

    /// Set search configuration
    #[allow(dead_code)]
    pub fn with_config(mut self, config: SearchConfig) -> Self {
        self.search_config = config;
        self
    }

    /// Set a shared embedder used for both vector indexing and querying
    #[allow(dead_code)]
    pub fn with_embedder(mut self, embedder: Arc<OpenAICompatibleProvider>) -> Self {
        self.embedder = Some(embedder);
        self
    }

    /// Set a metrics registry wired into the query searcher (hybrid alignment
    /// coverage observation). A separate `SearchMetrics` handle created from
    /// the same registry can read the recorded values.
    #[allow(dead_code)]
    pub fn with_metrics_registry(mut self, registry: Arc<cce_metrics::MetricsRegistry>) -> Self {
        self.metrics_registry = Some(registry);
        self
    }

    /// Wire a generative rerank handler into the query searcher (simulating
    /// `[rerank] enabled = true` at the config layer).
    #[allow(dead_code)]
    pub fn with_rerank_handler(
        mut self,
        handler: Arc<cce_llm_client::ProductionRerankHandler>,
    ) -> Self {
        self.rerank_handler = Some(handler);
        self
    }

    /// Force reranking on/off for the next queries (request-level override).
    #[allow(dead_code)]
    pub fn with_enable_rerank(mut self, enable: bool) -> Self {
        self.enable_rerank = Some(enable);
        self
    }

    /// Override the file extensions to index (e.g. to include markdown)
    #[allow(dead_code)]
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Bind a stable scenario key used to derive the Qdrant group id.
    ///
    /// Without it the group id is hashed from the per-run temp directory, so
    /// every run writes a fresh partition that is never reused. With a stable
    /// key repeated runs of the same scenario share one partition of the
    /// shared `cce_vectors` collection, keeping storage bounded.
    #[allow(dead_code)]
    pub fn with_scenario_name(mut self, scenario: impl Into<String>) -> Self {
        self.scenario_name = Some(scenario.into());
        self
    }

    /// The Qdrant group id this test indexes into (if an index has run).
    #[allow(dead_code)]
    pub fn project_group_id(&self) -> Option<&str> {
        self.project_group_id.as_deref()
    }

    /// Get the test fixture
    pub fn fixture(&self) -> &F {
        &self.fixture
    }

    /// Get the last index result
    pub fn last_index_result(&self) -> Option<&IndexResult> {
        self.last_index_result.as_ref()
    }

    /// Execute indexing
    ///
    /// Configures shared BM25 and Qdrant clients for both indexing and subsequent queries.
    /// Uses a unique per-instance BM25 index path to prevent cross-test data pollution.
    pub async fn index(&mut self) -> Result<&IndexResult> {
        use cce_orchestrator::CheckpointManager;
        use cce_storage_sqlite::SqliteClient;

        let mut orchestrator =
            IndexOrchestrator::new(self.project_id).expect("failed to create IndexOrchestrator");

        // The project group id must be shared by the index and query sides so
        // stored payloads and the query scope reference the same partition.
        // Mirror the production wiring (crates/cce_server/src/engine.rs):
        // `generate_project_group_id(project_id, root_path)` is used both for
        // `with_project_fingerprint` (index) and `ProjectScope` (query).
        // A stable scenario name (when set) replaces the per-run temp path as
        // the workspace key so repeated runs reuse one partition.
        let workspace_key = self
            .scenario_name
            .clone()
            .unwrap_or_else(|| self.fixture.root_path().to_string_lossy().to_string());
        let project_group_id =
            cce_storage_qdrant::generate_project_group_id(self.project_id, &workspace_key);
        self.project_group_id = Some(project_group_id.clone());
        orchestrator = orchestrator.with_project_fingerprint(project_group_id);

        // Set up checkpoint manager for operation progress tracking
        let sqlite_db = Arc::new(SqliteClient::in_memory()?);
        let checkpoint_manager = Arc::new(CheckpointManager::new_for_project(
            self.project_id,
            sqlite_db.clone(),
        ));
        self.sqlite_db = Some(sqlite_db);
        orchestrator = orchestrator.with_checkpoint_manager(checkpoint_manager);

        // Each test instance gets a unique BM25 index temp directory to prevent cross-test
        // data pollution (Tantivy lock contention, stale data from other tests).
        // TempDir is automatically cleaned up when QueryWorkflowTest is dropped.
        let bm25_temp_dir = TempDir::new()?;
        let bm25_path = bm25_temp_dir.path().to_string_lossy().to_string();
        let index_name = "default";
        let bm25_config = Bm25Config::default()
            .enabled()
            .with_index_name(index_name)
            .with_index_path(&bm25_path);
        let mut bm25 = cce_storage_bm25::Bm25Client::new(bm25_config);
        bm25.connect().await?;
        let bm25 = Arc::new(tokio::sync::Mutex::new(bm25));
        self.bm25_client = Some(bm25.clone());
        self._bm25_temp_dir = Some(bm25_temp_dir);
        orchestrator = orchestrator.with_bm25_client(bm25);

        if self.search_sources.vector {
            // Match the collection dimension to the configured embedder.
            let mut qdrant_config = QdrantConfig::with_url("http://localhost:6333");
            qdrant_config.vector_size = self.embedding_config.dimension;
            let qdrant = Arc::new(cce_storage_qdrant::QdrantClient::new(
                qdrant_config,
                "test",
            )?);

            // Verify Qdrant is reachable before starting the index.
            // Vector search requires a running Qdrant service on localhost:6333.
            qdrant.initialize().await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to connect to Qdrant at localhost:6333. \
                     Vector search is enabled but Qdrant is not running. \
                     Please start the Qdrant service or disable vector search. \
                     Underlying error: {}",
                    e
                )
            })?;

            self.qdrant_client = Some(qdrant.clone());
            orchestrator = orchestrator.with_qdrant_client(qdrant);
        }
        if let Some(ref embedder) = self.embedder {
            orchestrator = orchestrator.with_embedder(embedder.clone());
        }
        self.index_orchestrator = Some(orchestrator);

        let options = IndexOptions {
            root_dir: self.fixture.root_path().to_path_buf(),
            extensions: self
                .extensions
                .clone()
                .unwrap_or_else(|| IndexOptions::default().extensions),
            build_relations: true,
            // Always enable BM25 so search_bm25() queries return results
            store_bm25: true,
            store_vectors: self.search_sources.vector,
            ..Default::default()
        };

        let result = self
            .index_orchestrator
            .as_mut()
            .expect("Orchestrator not initialized")
            .execute(options)
            .await?;

        self.last_index_result = Some(result);
        Ok(self.last_index_result.as_ref().expect("Result not set"))
    }

    /// Execute a query
    pub async fn query(&mut self, query_text: &str) -> Result<&QueryResult> {
        // Ensure index is available
        if self.last_index_result.is_none() {
            self.index().await?;
        }

        // Create query options
        let options = QueryOptions {
            query: query_text.to_string(),
            project_id: self.project_id,
            sources: self.search_sources,
            config: self.search_config.clone(),
            query_intent: None,
            directory_prefix: None,
            exclude_content_types: Vec::new(),
            include_categories: Vec::new(),
            exclude_categories: Vec::new(),
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            with_source: true,
            enable_rerank: self.enable_rerank,
        };

        // Initialize query coordinator if not already done
        if self.query_coordinator.is_none() {
            self.init_query_coordinator().await?;
        }

        // Execute the query
        if let Some(ref coordinator) = self.query_coordinator {
            let result = coordinator.search(&options).await?;
            self.last_query_result = Some(result);
        }

        Ok(self.last_query_result.as_ref().expect("Result not set"))
    }

    /// Execute a vector search query
    pub async fn search_vector(&mut self, query_text: &str, top_k: usize) -> Result<&QueryResult> {
        self.search_sources = SearchSources::none().with_vector();
        self.search_config.vector.top_k = top_k;
        self.query(query_text).await
    }

    /// Execute a BM25 search query
    pub async fn search_bm25(&mut self, query_text: &str, limit: usize) -> Result<&QueryResult> {
        self.search_sources = SearchSources::none().with_bm25();
        self.search_config.result.limit = limit;
        self.query(query_text).await
    }

    /// Execute a hybrid search query
    pub async fn search_hybrid(&mut self, query_text: &str, limit: usize) -> Result<&QueryResult> {
        self.search_sources = SearchSources::default();
        self.search_config.result.limit = limit;
        self.query(query_text).await
    }

    /// Get the query coordinator
    #[allow(dead_code)]
    pub fn coordinator(&self) -> &QueryCoordinator {
        self.query_coordinator
            .as_ref()
            .expect("Query coordinator not initialized")
    }

    /// Best-effort removal of this test's Qdrant points.
    ///
    /// All tests share the single `cce_vectors` collection; logical isolation
    /// comes from the `group_id` payload field. Deleting the group's points at
    /// teardown keeps the collection from growing without bound across test
    /// runs. BM25 (Tantivy) state lives in a per-instance `TempDir` that is
    /// removed on drop, so no explicit cleanup is needed there.
    ///
    /// Callers should invoke this in their teardown path; failures are logged
    /// and swallowed so a missing Qdrant cannot fail an otherwise-passing test.
    pub async fn cleanup(&mut self) {
        let Some(group_id) = self.project_group_id.clone() else {
            return;
        };
        let Some(client) = self.qdrant_client.clone() else {
            return;
        };
        if let Err(e) = client.delete_by_group(&group_id).await {
            tracing::warn!("Qdrant cleanup for group '{}' failed: {e}", group_id);
        }
    }

    /// Execute an aggregated search query
    #[allow(dead_code)]
    pub async fn search_aggregated(
        &mut self,
        query_text: &str,
        _sub_queries: Vec<String>,
    ) -> Result<&QueryResult> {
        let options = QueryOptions {
            query: query_text.to_string(),
            project_id: self.project_id,
            sources: self.search_sources,
            config: self.search_config.clone(),
            query_intent: None,
            directory_prefix: None,
            exclude_content_types: Vec::new(),
            include_categories: Vec::new(),
            exclude_categories: Vec::new(),
            exclude_patterns: Vec::new(),
            include_patterns: Vec::new(),
            with_source: true,
            enable_rerank: None,
        };

        if self.query_coordinator.is_none() {
            self.init_query_coordinator().await?;
        }

        if let Some(ref coordinator) = self.query_coordinator {
            let result = coordinator.search(&options).await?;
            self.last_query_result = Some(result);
        }

        Ok(self.last_query_result.as_ref().expect("Result not set"))
    }

    /// Get callees of a function
    #[allow(dead_code)]
    pub async fn get_callees(
        &mut self,
        entity_id: cce_types::EntityId,
    ) -> Result<Vec<cce_types::ResolvedRelation>> {
        if self.query_coordinator.is_none() {
            self.init_query_coordinator().await?;
        }

        if let Some(ref coordinator) = self.query_coordinator {
            coordinator
                .get_callees(entity_id)
                .map_err(|e| anyhow::anyhow!("Failed to get callees: {}", e))
        } else {
            Ok(vec![])
        }
    }

    /// Get callers of a function
    #[allow(dead_code)]
    pub async fn get_callers(
        &mut self,
        entity_id: cce_types::EntityId,
    ) -> Result<Vec<cce_types::EntityId>> {
        if self.query_coordinator.is_none() {
            self.init_query_coordinator().await?;
        }

        if let Some(ref coordinator) = self.query_coordinator {
            coordinator
                .get_callers(entity_id)
                .map_err(|e| anyhow::anyhow!("Failed to get callers: {}", e))
        } else {
            Ok(vec![])
        }
    }

    /// Initialize query coordinator from index orchestrator
    async fn init_query_coordinator(&mut self) -> Result<()> {
        use cce_config::AppConfig;
        use cce_config::modules::{EmbeddingModelConfig, ProviderConfig};
        use cce_llm_client::OpenAICompatibleProvider;
        use cce_orchestrator::query::{IndexCapabilities, QueryCoordinator};
        use cce_relation::{CallChainQuery, RelationIndex};
        use cce_storage_qdrant::QdrantClient;
        use std::sync::Arc;

        // Create in-memory storage clients for testing
        let qdrant = if let Some(client) = self.qdrant_client.clone() {
            client
        } else {
            let mut qdrant_config = QdrantConfig::with_url("http://localhost:6333");
            qdrant_config.vector_size = self.embedding_config.dimension;
            Arc::new(QdrantClient::new(qdrant_config, "test")?)
        };

        let bm25 = if let Some(client) = self.bm25_client.clone() {
            client
        } else {
            // Fallback: create standalone BM25 (no index data)
            let bm25_config = Bm25Config::default().enabled().with_index_name("default");
            let mut bm25 = cce_storage_bm25::Bm25Client::new(bm25_config);
            bm25.connect().await.expect("Failed to connect BM25 client");
            Arc::new(tokio::sync::Mutex::new(bm25))
        };

        // Create OpenAICompatibleProvider for testing
        // Use the new model-based architecture
        let mut providers = std::collections::HashMap::new();
        let base_url = if self.embedding_config.is_mock() {
            "http://mock.local".to_string()
        } else {
            self.embedding_config.endpoint.clone()
        };

        let api_key = if self.embedding_config.api_key.is_empty() {
            "test-key".to_string()
        } else {
            self.embedding_config.api_key.clone()
        };

        providers.insert(
            "test-provider".to_string(),
            ProviderConfig {
                id: "test-provider".to_string(),
                name: "Test Provider".to_string(),
                base_url,
                api_keys: vec![api_key],
                ..Default::default()
            },
        );

        let mut models = std::collections::HashMap::new();
        models.insert(
            self.embedding_config.model.clone(),
            EmbeddingModelConfig {
                provider_id: "test-provider".to_string(),
                model: self.embedding_config.model.clone(),
                vector_dimension: self.embedding_config.dimension,
                ..Default::default()
            },
        );

        // Create AppConfig with LLM configuration
        let mut app_config = AppConfig::default();
        app_config.llm.providers = providers;
        app_config.llm.embedding_models = models;
        app_config.embedder.default_model = self.embedding_config.model.clone();
        app_config.embedder.use_base64 = false;

        // Create embedder using from_model (or reuse the externally provided
        // one so the index and query sides share the same embedder)
        let embedder = match self.embedder.clone() {
            Some(embedder) => embedder,
            None => Arc::new(
                OpenAICompatibleProvider::from_model(&app_config, &self.embedding_config.model)
                    .map_err(|e| anyhow::anyhow!("Failed to create embedder: {}", e))?,
            ),
        };

        // Create scope sharing the same project group id used during indexing.
        // Falls back to the canonical fingerprint when no index has run yet.
        let project_group_id = self.project_group_id.clone().unwrap_or_else(|| {
            let workspace_key = self
                .scenario_name
                .clone()
                .unwrap_or_else(|| self.fixture.root_path().to_string_lossy().to_string());
            cce_storage_qdrant::generate_project_group_id(self.project_id, &workspace_key)
        });
        let scope =
            ProjectScope::new(self.project_id, project_group_id).expect("valid project scope");

        // Get relation index from the index orchestrator after indexing
        let relation_index = if let Some(ref orchestrator) = self.index_orchestrator {
            if let Some(index) = orchestrator.get_relation_index() {
                tracing::info!("Using relation index from orchestrator with relations enabled");
                Some(index)
            } else {
                tracing::warn!(
                    "No relation index available from orchestrator (relations may not be built)"
                );
                None
            }
        } else {
            tracing::warn!("Index orchestrator not initialized, creating empty relation index");
            None
        };

        // Create relation searcher with real or empty index
        let (call_chain_query, has_relations) = if let Some(index) = &relation_index {
            (Arc::new(CallChainQuery::from_index(index.clone())), true)
        } else {
            // Fallback to empty index
            let empty_index = RelationIndex::new();
            (Arc::new(CallChainQuery::from_index(empty_index)), false)
        };

        // Build capabilities based on what was actually indexed. Using the
        // storage presence (not the current query's source selection) means a
        // test may run vector-only and bm25-only queries against one shared
        // coordinator, matching production where all indexed sources stay
        // available regardless of the per-query source mix.
        let capabilities = IndexCapabilities::new()
            .with_vectors(self.qdrant_client.is_some())
            .with_bm25(self.bm25_client.is_some())
            .with_relations(has_relations);

        // Create query coordinator using builder pattern
        let mut coordinator_builder = QueryCoordinator::builder(
            qdrant.clone(),
            embedder,
            bm25.clone(),
            call_chain_query,
            scope,
        )
        .with_capabilities(capabilities);

        if let Some(ref sqlite_db) = self.sqlite_db {
            coordinator_builder = coordinator_builder.with_sqlite(sqlite_db.clone());
        }
        if let Some(ref registry) = self.metrics_registry {
            coordinator_builder = coordinator_builder.with_metrics_registry(registry.clone());
        }
        if let Some(ref handler) = self.rerank_handler {
            coordinator_builder = coordinator_builder.with_rerank(handler.clone());
        }

        let coordinator = coordinator_builder.build();

        self.query_coordinator = Some(coordinator);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmptyFixture;

    #[tokio::test]
    async fn test_query_workflow_basic() {
        let fixture = EmptyFixture::new().expect("Failed to create fixture");
        fixture
            .add_file(
                "src/lib.rs",
                r#"
/// Calculate the sum of two numbers
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
            )
            .expect("Failed to add file");

        let mut query_test =
            QueryWorkflowTest::new(fixture.into_test_fixture(), EmbeddingConfig::mock())
                .with_sources(SearchSources::none().with_bm25());

        // Index (BM25 only, no Qdrant dependency)
        let index_result = query_test.index().await.expect("Index failed");
        assert!(index_result.total_entities >= 1);

        // Use BM25-only search (no Qdrant dependency)
        let _query_result = query_test
            .search_bm25("add numbers", 10)
            .await
            .expect("Query failed");
    }
}
