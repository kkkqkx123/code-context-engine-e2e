//! Assembly review export for the flask fixture.
//!
//! Same pipeline as `assembly_oncecell`, over the `full_pipeline`
//! flask benchmark snapshot. Requires
//! `data/benchmark/full_pipeline/flask/bge-m3/bench_data.rkyv`
//! (run `gen_bench_flask` first if missing).
//!
//! Output:
//!   outputs/scenarios/python/assembly/flask/
//!   ├── index.md
//!   └── {query_id}.md          — raw recall hits + assembled content per query

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::assembly_review::{
    AssemblyReviewConfig, ContentMode, RecallMode, run_assembly_review,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_assembly_review(AssemblyReviewConfig {
        project: "flask",
        language: "python",
        spec: FixtureSpec::python_flask(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
    })
    .await
}
