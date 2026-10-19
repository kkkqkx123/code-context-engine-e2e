//! Relation-graph visualization export for the media_server fixture.
//!
//! Indexes `fixtures/cpp/review/media-server`, resolves seed entities from
//! the judgment expectation ranges plus hub names, and renders per-seed call
//! relations, call chains, and ego graphs for manual review. Fully offline.
//!
//! Output:
//!   outputs/scenarios/cpp/relation_review/media_server/
//!   ├── run_manifest.txt
//!   ├── index.md
//!   ├── entity/{seed}.md      — callees/callers/chains/ego graph + mermaid
//!   └── graph/{seed}.json     — machine-readable ego graph snapshot

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::judgments::media_server::media_server_relevance_judgments;
use cce_e2e_tests::relation_review::{RelationReviewConfig, run_relation_review};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_relation_review(RelationReviewConfig {
        project: "media_server",
        language: "cpp",
        spec: FixtureSpec::cpp_media_server(),
        judgments: media_server_relevance_judgments(),
        hub_names: vec!["create", "open", "read", "write"],
        max_seeds: 8,
        depth: 2,
        limit: 20,
    })
    .await
}
