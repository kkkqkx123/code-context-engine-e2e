//! Assembly review export for the mediatr fixture.
//!
//! Requires `data/benchmark/full_pipeline/mediatr/bge-m3/bench_data.rkyv`
//! (run `gen_bench_mediatr` first if missing).
//!
//! Output:
//!   outputs/scenarios/csharp/assembly/mediatr/
//!   ├── index.md
//!   └── {query_id}.md          — raw recall hits + assembled content per query

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::assembly_review::{
    AssemblyReviewConfig, ContentMode, RecallMode, run_assembly_review,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_assembly_review(AssemblyReviewConfig {
        project: "mediatr",
        language: "csharp",
        spec: FixtureSpec::csharp_mediatr(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
    })
    .await
}
