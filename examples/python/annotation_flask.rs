//! Annotation review export for the flask fixture.
//!
//! Same pipeline as `annotation_oncecell`, over the `full_pipeline`
//! flask benchmark snapshot. Requires
//! `data/benchmark/full_pipeline/flask/bge-m3/bench_data.rkyv`
//! (run `gen_bench_flask` first if missing).
//!
//! Output:
//!   outputs/scenarios/python/annotation/flask/
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
        project: "flask",
        language: "python",
        spec: FixtureSpec::python_flask(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
        filter: ReviewFilterOptions::default(),
    })
    .await
}
