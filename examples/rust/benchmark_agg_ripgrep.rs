//! Aggregation-enhance benchmark for the ripgrep fixture.
//!
//! Reuses `data/benchmark/full_pipeline/ripgrep/bge-m3/bench_data.rkyv` and
//! compares plain base rankings against offline relation/summary boosted
//! rankings. Reports are written to
//! `outputs/benchmark/ripgrep/aggregation_enhance/`.

use cce_e2e_tests::bench_data::EvaluationScope;
use cce_e2e_tests::judgments::ripgrep::ripgrep_relevance_judgments;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let all_judgments = ripgrep_relevance_judgments();
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

    cce_e2e_tests::aggregation_enhance::run_aggregation_enhance_benchmark(
        "ripgrep",
        &judgments,
    )?;
    Ok(())
}