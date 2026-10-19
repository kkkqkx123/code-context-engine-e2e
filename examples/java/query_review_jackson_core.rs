//! Full query-result visualization export for the jackson_core fixture.
//!
//! Indexes `fixtures/java/review/jackson-core`, runs every judgment query
//! from `src/judgments/jackson_core.rs` through the real query pipeline,
//! and renders per-query complete results plus an aggregated-search demo.
//!
//! Output:
//!   outputs/scenarios/java/query_review/jackson_core/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── {query_id}.md         — full hits with scores, content, annotation comparison
//!   └── aggregated_demo.md    — two-sub-query merge demo

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::jackson_core::jackson_core_relevance_judgments;
use cce_e2e_tests::query_review::{QueryReviewConfig, run_query_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_query_review(QueryReviewConfig {
        project: "jackson_core",
        language: "java",
        spec: FixtureSpec::java_jackson_core(),
        judgments: jackson_core_relevance_judgments(),
        top_k: 10,
        scenario_suffix: None,
    })
    .await
}
