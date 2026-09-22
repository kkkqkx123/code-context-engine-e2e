//! Assembly review export for the once_cell fixture (primary example).
//!
//! Replays embedding recall offline over the `full_pipeline` benchmark
//! snapshot and renders per-query raw vs assembled markdown pairs for manual
//! review of the dormant SPSR-Graph assembler.
//!
//! Requires `data/benchmark/full_pipeline/once_cell/bge-m3/bench_data.rkyv`
//! (run `gen_bench_oncecell` first if missing).
//!
//! Output:
//!   outputs/scenarios/rust/assembly/once_cell/
//!   ├── index.md
//!   └── {query_id}.md          — raw recall hits + assembled content per query

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::assembly_review::{
    AssemblyReviewConfig, ContentMode, RecallMode, run_assembly_review,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_assembly_review(AssemblyReviewConfig {
        project: "once_cell",
        language: "rust",
        spec: FixtureSpec::rust_once_cell(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
    })
    .await
}
