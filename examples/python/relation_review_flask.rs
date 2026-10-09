//! Relation-graph visualization export for the flask fixture.
//!
//! Same pipeline as `relation_review_oncecell`, over
//! `fixtures/python/review/flask` with `src/judgments/flask.rs` seeds.
//!
//! Output:
//!   outputs/scenarios/python/relation_review/flask/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── entity/{seed}.md      — callees/callers/chains/ego graph + mermaid
//!   └── graph/{seed}.json     — machine-readable ego graph snapshot

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::flask::flask_relevance_judgments;
use cce_e2e_tests::relation_review::{RelationReviewConfig, run_relation_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_relation_review(RelationReviewConfig {
        project: "flask",
        language: "python",
        spec: FixtureSpec::python_flask(),
        judgments: flask_relevance_judgments(),
        hub_names: vec!["route", "run", "Flask"],
        max_seeds: 8,
        depth: 2,
        limit: 20,
    })
    .await
}
