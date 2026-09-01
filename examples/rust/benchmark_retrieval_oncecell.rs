//! Retrieval-method benchmark for the once_cell fixture.
//!
//! Reuses `data/benchmark/{baseline}/once_cell/bge-m3/bench_data.rkyv` and
//! compares emb / bm25 / minmax-* / rrf-* recall methods. Reports are written
//! to `outputs/benchmark/once_cell/retrieval_method/`.

use cce_e2e_tests::bench_data::EvaluationScope;
use cce_e2e_tests::judgments::once_cell::once_cell_relevance_judgments;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let all_judgments = once_cell_relevance_judgments();
    let scope = EvaluationScope::All;
    let judgments: Vec<_> = all_judgments
        .iter()
        .filter(|j| scope.includes_query_type(j.query_type))
        .cloned()
        .collect();

    println!("=== Retrieval-Method Benchmark Configuration ===");
    println!("Evaluation scope: {scope}");
    println!(
        "Total judgments: {} ({} excluded)",
        judgments.len(),
        all_judgments.len() - judgments.len()
    );

    cce_e2e_tests::retrieval_method::run_retrieval_method_benchmark("once_cell", &judgments)?;
    Ok(())
}
