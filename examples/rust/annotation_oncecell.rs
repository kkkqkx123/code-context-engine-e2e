//! Annotation review export for the once_cell fixture (primary example).
//!
//! Replays embedding recall offline over the `full_pipeline` benchmark
//! snapshot and renders per-query raw vs annotated markdown pairs for manual
//! review of the relation annotator.
//!
//! Requires `data/benchmark/full_pipeline/once_cell/bge-m3/bench_data.rkyv`
//! (run `gen_bench_oncecell` first if missing).
//!
//! Output:
//!   outputs/scenarios/rust/annotation/once_cell/
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
        project: "once_cell",
        language: "rust",
        spec: FixtureSpec::rust_once_cell(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
        filter: ReviewFilterOptions::default(),
    })
    .await
}
