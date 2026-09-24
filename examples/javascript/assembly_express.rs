//! Assembly review export for the express fixture.
//!
//! Requires `data/benchmark/full_pipeline/express/bge-m3/bench_data.rkyv`
//! (run `gen_bench_express` first if missing).
//!
//! Output:
//!   outputs/scenarios/javascript/assembly/express/
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
        project: "express",
        language: "javascript",
        spec: FixtureSpec::javascript_express(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
        filter: ReviewFilterOptions::default(),
    })
    .await
}
