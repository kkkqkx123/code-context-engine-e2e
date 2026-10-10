//! Full query-result visualization export for the flask fixture.
//!
//! Same pipeline as `query_review_oncecell`, over
//! `fixtures/python/review/flask` with `src/judgments/flask.rs` queries.
//! Without a reachable Qdrant the run is BM25-only and lands in the
//! `flask_bm25` directory (recorded in the manifest).
//!
//! Output:
//!   outputs/scenarios/python/query_review/flask/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── {query_id}.md         — full hits with scores, content, annotation comparison
//!   └── aggregated_demo.md    — two-sub-query merge demo

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::flask::flask_relevance_judgments;
use cce_e2e_tests::query_review::{QueryReviewConfig, run_query_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_query_review(QueryReviewConfig {
        project: "flask",
        language: "python",
        spec: FixtureSpec::python_flask(),
        judgments: flask_relevance_judgments(),
        top_k: 10,
        scenario_suffix: None,
    })
    .await
}
