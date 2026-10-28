use cce_e2e_tests::bench_data::EvaluationScope;
use cce_e2e_tests::benchmark_runner::run_standard_benchmark;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_standard_benchmark(
        "jackson_core",
        cce_e2e_tests::judgments::jackson_core::jackson_core_relevance_judgments(),
        EvaluationScope::CoreRetrieval,
    )
}
