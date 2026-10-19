//! Full query-result visualization export for the gin fixture.
//!
//! Indexes `fixtures/go/review/gin`, runs every judgment query from
//! `src/judgments/gin.rs` through the real query pipeline, and renders
//! per-query complete results plus an aggregated-search demo.
//!
//! Output:
//!   outputs/scenarios/go/query_review/gin/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── {query_id}.md         — full hits with scores, content, annotation comparison
//!   └── aggregated_demo.md    — two-sub-query merge demo

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::gin::gin_relevance_judgments;
use cce_e2e_tests::query_review::{QueryReviewConfig, run_query_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_query_review(QueryReviewConfig {
        project: "gin",
        language: "go",
        spec: FixtureSpec::go_gin(),
        judgments: gin_relevance_judgments(),
        top_k: 10,
        scenario_suffix: None,
    })
    .await
}
