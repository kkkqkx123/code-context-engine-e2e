//! Full query-result visualization export for the ripgrep fixture.
//!
//! Indexes `fixtures/rust/review/ripgrep`, runs every judgment query from
//! `src/judgments/ripgrep.rs` through the real query pipeline, and renders
//! per-query complete results plus an aggregated-search demo for manual
//! review. Uses the deterministic mock embedder with the embedded local
//! vector engine, so hybrid output needs no external service.
//!
//! Output:
//!   outputs/scenarios/rust/query_review/ripgrep/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── {query_id}.md         — full hits with scores, content, annotation comparison
//!   └── aggregated_demo.md    — two-sub-query merge demo

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::ripgrep::ripgrep_relevance_judgments;
use cce_e2e_tests::query_review::{QueryReviewConfig, run_query_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_query_review(QueryReviewConfig {
        project: "ripgrep",
        language: "rust",
        spec: FixtureSpec::rust_ripgrep(),
        judgments: ripgrep_relevance_judgments(),
        top_k: 10,
        scenario_suffix: None,
    })
    .await
}
