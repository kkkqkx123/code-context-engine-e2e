//! Annotation review export for the spring_boot fixture.
//!
//! Requires `data/benchmark/full_pipeline/spring_boot/bge-m3/bench_data.rkyv`
//! (run `gen_bench_spring_boot` first if missing).
//!
//! Output:
//!   outputs/scenarios/java/annotation/spring_boot/
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
        project: "spring_boot",
        language: "java",
        spec: FixtureSpec::java_spring_boot(),
        top_k: 10,
        content_mode: ContentMode::Chunk,
        recall: RecallMode::Emb,
        expansion: true,
        filter: ReviewFilterOptions::default(),
    })
    .await
}
