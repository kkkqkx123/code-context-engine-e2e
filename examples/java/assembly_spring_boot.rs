//! Assembly review export for the spring_boot fixture.
//!
//! Requires `data/benchmark/full_pipeline/spring_boot/bge-m3/bench_data.rkyv`
//! (run `gen_bench_spring_boot` first if missing).
//!
//! Output:
//!   outputs/scenarios/java/assembly/spring_boot/
//!   ├── index.md
//!   └── {query_id}.md          — raw recall hits + assembled content per query

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::assembly_review::{
    AssemblyReviewConfig, ContentMode, RecallMode, run_assembly_review,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_assembly_review(AssemblyReviewConfig {
        project: "spring_boot",
        language: "java",
        spec: FixtureSpec::java_spring_boot(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
    })
    .await
}
