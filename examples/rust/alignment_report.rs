//! Hybrid alignment review report.
//!
//! Generates a human-review alignment report under
//! `outputs/scenarios/rust/alignment/`:
//!
//! - `run_manifest.txt`: fixture, queries, Qdrant availability, alignment key
//!   convention.
//! - `alignment_summary.md`: per-query key counts, cross-path matched keys,
//!   and the resulting alignment ratio.
//! - `per_query_alignment.md`: per-query vector/BM25/hybrid key lists.
//!
//! Uses the deterministic mock embedding server, so no LLM service is needed.
//! Hybrid queries run only when Qdrant is reachable at localhost:6333; the
//! manifest records which mode ran.
//!
//! Run: `cargo run --example alignment_report -p cce-e2e-tests`

use std::sync::Arc;

use cce_e2e_tests::embedding::EmbeddingConfig;
use cce_e2e_tests::mock_embedding_server::MockEmbeddingServer;
use cce_e2e_tests::output_manager::{OutputBuilder, OutputCategory};
use cce_e2e_tests::query_test::QueryWorkflowTest;
use cce_llm_client::OpenAICompatibleProvider;
use cce_orchestrator::SearchSources;
use cce_orchestrator::query::types::SearchResult;

const FIXTURE: &str = "rust/basic";
const QUERIES: &[&str] = &["process", "internal", "process internal result"];

fn alignment_key_of(item: &SearchResult) -> String {
    if let Some(entity_id) = item.entity_ids.first() {
        format!("e:{}", entity_id.0)
    } else if let Some(segment_id) = &item.segment_id {
        format!("s:{}", segment_id)
    } else {
        format!("c:{}", item.id)
    }
}

fn fmt_keys(items: &[SearchResult]) -> String {
    let mut keys: Vec<String> = items.iter().map(alignment_key_of).collect();
    keys.sort_unstable();
    keys.dedup();
    if keys.is_empty() {
        "(none)".to_string()
    } else {
        keys.join(", ")
    }
}

fn main() {
    let manager = OutputBuilder::new()
        .category(OutputCategory::Scenarios)
        .language("rust")
        .scenario("alignment")
        .build();

    let _ = manager.clean();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let server = rt.block_on(MockEmbeddingServer::start());

    // Load the existing fixture.
    let fixture =
        cce_e2e_tests::fixture::TestFixture::load(cce_e2e_tests::fixture::FixtureSpec::new(
            cce_e2e_tests::fixture::FixtureCategory::Rust,
            "basic",
        ))
        .expect("load rust/basic fixture");

    let embedder = Arc::new(
        OpenAICompatibleProvider::from_model(
            &cce_e2e_tests::mock_embedding_server::mock_embedding_config(&server.base_url),
            "mock",
        )
        .expect("mock embedder"),
    );

    let mut query_test =
        QueryWorkflowTest::new(fixture, EmbeddingConfig::mock()).with_embedder(embedder);

    // Probe Qdrant availability: hybrid indexing fails fast with a clear
    // message when the service is absent; fall back to BM25-only then.
    let hybrid_available = rt
        .block_on(probe_qdrant())
        .map_err(|e| eprintln!("Qdrant unavailable, BM25-only mode: {e}"))
        .is_ok();

    if !hybrid_available {
        query_test = query_test.with_sources(SearchSources::none().with_bm25());
    }

    rt.block_on(query_test.index()).expect("index failed");

    let mut manifest = String::new();
    manifest.push_str(&format!("fixture: {FIXTURE}\n"));
    manifest.push_str(&format!(
        "qdrant_available: {}\n",
        if hybrid_available { "yes" } else { "no" }
    ));
    manifest.push_str("embedder: deterministic mock (no LLM service)\n");
    manifest.push_str("alignment_key: e:<entity_id> | s:<segment_id> | c:<chunk_id>\n");
    manifest.push_str(&format!(
        "queries: {}\n",
        QUERIES
            .iter()
            .map(|q| format!("\"{q}\""))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    let mut summary = String::from("# Hybrid Fusion Alignment Summary\n\n");
    let mut per_query = String::from("# Per-Query Alignment Detail\n\n");

    for query in QUERIES {
        let vector_items: Vec<SearchResult> = if hybrid_available {
            rt.block_on(query_test.search_vector(query, 20))
                .expect("vector search")
                .items
                .clone()
        } else {
            Vec::new()
        };
        let bm25_items: Vec<SearchResult> = rt
            .block_on(query_test.search_bm25(query, 20))
            .expect("bm25 search")
            .items
            .clone();
        let hybrid_items: Vec<SearchResult> = if hybrid_available {
            rt.block_on(query_test.search_hybrid(query, 20))
                .expect("hybrid search")
                .items
                .clone()
        } else {
            Vec::new()
        };

        let vector_keys: std::collections::HashSet<String> =
            vector_items.iter().map(alignment_key_of).collect();
        let bm25_keys: std::collections::HashSet<String> =
            bm25_items.iter().map(alignment_key_of).collect();
        let matched = vector_keys.intersection(&bm25_keys).count();
        let ratio = if vector_keys.is_empty() {
            0.0
        } else {
            matched as f64 / vector_keys.len() as f64
        };

        summary.push_str(&format!("## Query: \"{query}\"\n"));
        summary.push_str(&format!(
            "- vector keys: {} ({} items)\n",
            vector_keys.len(),
            vector_items.len()
        ));
        summary.push_str(&format!(
            "- bm25 keys: {} ({} items)\n",
            bm25_keys.len(),
            bm25_items.len()
        ));
        summary.push_str(&format!("- matched keys: {matched} (ratio {:.2})\n", ratio));
        summary.push_str(&format!(
            "- hybrid items: {}; hybrid key ⊆ (vector ∪ bm25): {}\n",
            hybrid_items.len(),
            hybrid_items.iter().all(|r| {
                let k = alignment_key_of(r);
                vector_keys.contains(&k) || bm25_keys.contains(&k)
            })
        ));
        summary.push('\n');

        per_query.push_str(&format!("## Query: \"{query}\"\n"));
        per_query.push_str(&format!("- vector keys: {}\n", fmt_keys(&vector_items)));
        per_query.push_str(&format!("- bm25 keys: {}\n", fmt_keys(&bm25_items)));
        per_query.push_str(&format!("- hybrid keys: {}\n", fmt_keys(&hybrid_items)));
        per_query.push('\n');
    }

    manager
        .write("run_manifest.txt", &manifest)
        .expect("write run_manifest.txt");
    manager
        .write("alignment_summary.md", &summary)
        .expect("write alignment_summary.md");
    manager
        .write("per_query_alignment.md", &per_query)
        .expect("write per_query_alignment.md");

    println!(
        "Alignment report written to {}",
        manager.output_dir().display()
    );

    rt.block_on(server.stop());
}

/// Probe Qdrant availability without touching the index.
async fn probe_qdrant() -> Result<(), String> {
    let mut config = cce_storage_qdrant::QdrantConfig::with_url("http://localhost:6333");
    config.vector_size = cce_e2e_tests::mock_embedding_server::MOCK_EMBEDDING_DIMENSION;
    let client =
        cce_storage_qdrant::QdrantClient::new(config, "test").map_err(|e| e.to_string())?;
    client.initialize().await.map_err(|e| e.to_string())?;
    Ok(())
}
