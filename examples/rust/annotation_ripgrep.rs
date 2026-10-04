//! Annotation review export for the ripgrep fixture.
//!
//! Requires `data/benchmark/full_pipeline/ripgrep/bge-m3/bench_data.rkyv`
//! (run `gen_bench_ripgrep` first if missing).
//!
//! Output:
//!   outputs/scenarios/rust/annotation/ripgrep/
//!   ├── index.md
//!   └── {query_id}.md          — raw recall hits + annotated content per query

use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::annotation_review::{
    AnnotationReviewConfig, ContentMode, RecallMode, run_annotation_review,
};
use cce_e2e_tests::review_filter::ReviewFilterOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_annotation_review(AnnotationReviewConfig {
        project: "ripgrep",
        language: "rust",
        spec: FixtureSpec::rust_ripgrep(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
        filter: ReviewFilterOptions::default(),
    })
    .await
}
