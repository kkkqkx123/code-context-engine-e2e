use cce_e2e_tests::bench_data::EvaluationScope;
use cce_e2e_tests::benchmark_runner::run_standard_benchmark;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_standard_benchmark("once_cell", cce_e2e_tests::judgments::once_cell::once_cell_relevance_judgments(), EvaluationScope::All)
}
