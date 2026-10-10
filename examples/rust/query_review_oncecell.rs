//! Full query-result visualization export for the once_cell fixture.
//!
//! Indexes `fixtures/rust/review/once_cell`, runs every judgment query from
//! `src/judgments/once_cell.rs` through the real query pipeline, and renders
//! per-query complete results plus an aggregated-search demo for manual
//! review. Uses the deterministic mock embedder; hybrid search runs when
//! Qdrant is reachable, otherwise BM25-only into the `once_cell_bm25`
//! directory (recorded in the manifest).
//!
//! Output:
//!   outputs/scenarios/rust/query_review/once_cell/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── {query_id}.md         — full hits with scores, content, annotation comparison
//!   └── aggregated_demo.md    — two-sub-query merge demo

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::once_cell::once_cell_relevance_judgments;
use cce_e2e_tests::query_review::{QueryReviewConfig, run_query_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_query_review(QueryReviewConfig {
        project: "once_cell",
        language: "rust",
        spec: FixtureSpec::rust_once_cell(),
        judgments: once_cell_relevance_judgments(),
        top_k: 10,
        scenario_suffix: None,
    })
    .await
}
