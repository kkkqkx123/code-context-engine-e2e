//! Assembly review export for the ripgrep fixture.
//!
//! Requires `data/benchmark/full_pipeline/ripgrep/bge-m3/bench_data.rkyv`
//! (run `gen_bench_ripgrep` first if missing).
//!
//! Output:
//!   outputs/scenarios/rust/assembly/ripgrep/
//!   ├── index.md
//!   └── {query_id}.md          — raw recall hits + assembled content per query

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::assembly_review::{
    AssemblyReviewConfig, ContentMode, RecallMode, run_assembly_review,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_assembly_review(AssemblyReviewConfig {
        project: "ripgrep",
        language: "rust",
        spec: FixtureSpec::rust_ripgrep(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
    })
    .await
}
