//! Full query-result visualization export for the spring_boot fixture.
//!
//! Indexes `fixtures/java/review/springboot-minimal-demo`, runs every
//! judgment query from `src/judgments/spring_boot.rs` through the real
//! query pipeline, and renders per-query complete results plus an
//! aggregated-search demo.
//!
//! Output:
//!   outputs/scenarios/java/query_review/spring_boot/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── {query_id}.md         — full hits with scores, content, annotation comparison
//!   └── aggregated_demo.md    — two-sub-query merge demo

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::spring_boot::spring_boot_relevance_judgments;
use cce_e2e_tests::query_review::{QueryReviewConfig, run_query_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_query_review(QueryReviewConfig {
        project: "spring_boot",
        language: "java",
        spec: FixtureSpec::java_spring_boot(),
        judgments: spring_boot_relevance_judgments(),
        top_k: 10,
        scenario_suffix: None,
    })
    .await
}
