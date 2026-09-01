use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::bench_data::QueryData;
use cce_e2e_tests::bench_gen::{ProjectConfig, run_bench_gen};
use cce_e2e_tests::judgments::once_cell::once_cell_relevance_judgments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Keep generated queries and range-based evaluation judgments in lockstep.
    // The legacy `once_cell_queries` set uses different IDs and text, which
    // would otherwise make the generated embeddings evaluate against unrelated
    // relevance ranges.
    let queries: Vec<QueryData> = once_cell_relevance_judgments()
        .iter()
        .map(|q| QueryData {
            id: q.id.clone(),
            text: q.query_text.clone(),
            query_type: q.query_type,
            relevant_names: vec![],
            irrelevant_names: vec![],
        })
        .collect();

    run_bench_gen(ProjectConfig {
        name: "once_cell",
        target_spec: FixtureSpec::rust_once_cell(),
        distractor_spec: FixtureSpec::rust_distractor(),
        queries,
    })
    .await
}
