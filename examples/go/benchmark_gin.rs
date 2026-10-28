use cce_e2e_tests::bench_data::EvaluationScope;
use cce_e2e_tests::benchmark_runner::run_standard_benchmark;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_standard_benchmark(
        "gin",
        cce_e2e_tests::judgments::gin::gin_relevance_judgments(),
        EvaluationScope::CoreRetrieval,
    )
}
