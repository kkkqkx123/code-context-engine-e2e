use cce_e2e_tests::bench_data::EvaluationScope;
use cce_e2e_tests::benchmark_runner::run_standard_benchmark;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_standard_benchmark(
        "express",
        cce_e2e_tests::judgments::express::express_relevance_judgments(),
        EvaluationScope::CoreRetrieval,
    )
}
