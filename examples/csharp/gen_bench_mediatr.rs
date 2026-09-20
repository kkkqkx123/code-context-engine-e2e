use cce_e2e_tests::FixtureSpec;
use cce_e2e_tests::bench_data::QueryData;
use cce_e2e_tests::bench_gen::{ProjectConfig, run_bench_gen};
use cce_e2e_tests::judgments::mediatr::mediatr_relevance_judgments;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let queries: Vec<QueryData> = mediatr_relevance_judgments()
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
        name: "mediatr",
        target_spec: FixtureSpec::csharp_mediatr(),
        distractor_spec: FixtureSpec::csharp_basic(),
        queries,
    })
    .await
}
