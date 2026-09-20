//! Aggregation-enhance benchmark for the flask fixture.
//!
//! Reuses `data/benchmark/full_pipeline/flask/bge-m3/bench_data.rkyv` and
//! compares plain base rankings against offline relation/summary boosted
//! rankings. Reports are written to
//! `outputs/benchmark/flask/aggregation_enhance/`.

use cce_e2e_tests::bench_data::EvaluationScope;
use cce_e2e_tests::judgments::flask::flask_relevance_judgments;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let all_judgments = flask_relevance_judgments();
    let scope = EvaluationScope::All;
    let judgments: Vec<_> = all_judgments
        .iter()
        .filter(|j| scope.includes_query_type(j.query_type))
        .cloned()
        .collect();

    println!("=== Aggregation-Enhance Benchmark Configuration ===");
    println!("Evaluation scope: {scope}");
    println!(
        "Total judgments: {} ({} excluded)",
        judgments.len(),
        all_judgments.len() - judgments.len()
    );

    cce_e2e_tests::aggregation_enhance::run_aggregation_enhance_benchmark("flask", &judgments)?;
    Ok(())
}
