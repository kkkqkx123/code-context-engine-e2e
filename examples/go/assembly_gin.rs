//! Assembly review export for the gin fixture.
//!
//! Requires `data/benchmark/full_pipeline/gin/bge-m3/bench_data.rkyv`
//! (run `gen_bench_gin` first if missing).
//!
//! Output:
//!   outputs/scenarios/go/assembly/gin/
//!   ├── index.md
//!   └── {query_id}.md          — raw recall hits + assembled content per query

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::assembly_review::{
    AssemblyReviewConfig, ContentMode, RecallMode, run_assembly_review,
};
use cce_e2e_tests::review_filter::ReviewFilterOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_assembly_review(AssemblyReviewConfig {
        project: "gin",
        language: "go",
        spec: FixtureSpec::go_gin(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
        filter: ReviewFilterOptions::default(),
    })
    .await
}
