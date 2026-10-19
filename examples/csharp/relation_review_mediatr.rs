//! Relation-graph visualization export for the mediatr fixture.
//!
//! Indexes `fixtures/csharp/review/MediatR`, resolves seed entities from the
//! judgment expectation ranges plus hub names, and renders per-seed call
//! relations, call chains, and ego graphs for manual review. Fully offline.
//!
//! Output:
//!   outputs/scenarios/csharp/relation_review/mediatr/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── entity/{seed}.md      — callees/callers/chains/ego graph + mermaid
//!   └── graph/{seed}.json     — machine-readable ego graph snapshot

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::mediatr::mediatr_relevance_judgments;
use cce_e2e_tests::relation_review::{RelationReviewConfig, run_relation_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_relation_review(RelationReviewConfig {
        project: "mediatr",
        language: "csharp",
        spec: FixtureSpec::csharp_mediatr(),
        judgments: mediatr_relevance_judgments(),
        hub_names: vec!["Send", "Publish", "Mediator"],
        max_seeds: 8,
        depth: 2,
        limit: 20,
    })
    .await
}
