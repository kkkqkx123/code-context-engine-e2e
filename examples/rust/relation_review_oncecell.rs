//! Relation-graph visualization export for the once_cell fixture.
//!
//! Indexes `fixtures/rust/review/once_cell`, resolves seed entities from the
//! judgment expectation ranges plus hub names, and renders per-seed call
//! relations, call chains, and ego graphs for manual review. Fully offline
//! (no vector backend needed).
//!
//! Output:
//!   outputs/scenarios/rust/relation_review/once_cell/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── entity/{seed}.md      — callees/callers/chains/ego graph + mermaid
//!   └── graph/{seed}.json     — machine-readable ego graph snapshot

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::once_cell::once_cell_relevance_judgments;
use cce_e2e_tests::relation_review::{RelationReviewConfig, run_relation_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_relation_review(RelationReviewConfig {
        project: "once_cell",
        language: "rust",
        spec: FixtureSpec::rust_once_cell(),
        judgments: once_cell_relevance_judgments(),
        hub_names: vec!["new", "get", "set", "get_or_init"],
        max_seeds: 8,
        depth: 2,
        limit: 20,
    })
    .await
}
