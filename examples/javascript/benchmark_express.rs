use cce_e2e_tests::bench_data::{EvaluationScope, RelevanceJudgment, load_benchmark_data};
use cce_e2e_tests::judgments::evaluate::RelevanceInfo;
use cce_e2e_tests::judgments::evaluate::{
    BASELINES, BaselineResult, BenchmarkPaths, evaluate_bm25_variants, evaluate_embedding_variants,
};
use cce_e2e_tests::judgments::express::express_relevance_judgments;
use cce_e2e_tests::judgments::report::{
    generate_evaluation_scope_csv, print_console_summary, write_aggregate_reports,
    write_relevance_reports, write_run_manifest, write_test_diagnostics,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let all_judgments = express_relevance_judgments();
    let scope = EvaluationScope::CoreRetrieval;
    let judgments: Vec<RelevanceJudgment> = all_judgments
        .iter()
        .filter(|j| scope.includes_query_type(j.query_type))
        .cloned()
        .collect();

    if judgments.is_empty() {
        return Err("No queries match the evaluation scope".into());
    }

    println!("=== Benchmark Configuration ===");
    println!("Evaluation scope: {scope}");
    println!(
        "Total judgments: {} ({} excluded)",
        judgments.len(),
        all_judgments.len() - judgments.len()
    );

    let paths = BenchmarkPaths::new("express");
    let mut all_results: Vec<BaselineResult> = Vec::new();
    let mut no_test_results: Vec<BaselineResult> = Vec::new();
    let mut all_relevance: Vec<RelevanceInfo> = Vec::new();
    let mut no_test_relevance: Vec<RelevanceInfo> = Vec::new();
    let mut all_bench: Vec<(String, cce_e2e_tests::bench_data::BenchmarkData)> = Vec::new();

    for baseline in BASELINES {
        let data_path = paths.data_dir(baseline);
        if !data_path.exists() {
            eprintln!(
                "WARNING: {baseline} data not found at {}",
                data_path.display()
            );
            continue;
        }
        println!(
            "Loading baseline '{baseline}' from: {}",
            data_path.display()
        );
        let bench = load_benchmark_data(&data_path)?;
        println!(
            "  {} queries, {} embedding chunks, {} BM25 chunks",
            bench.queries.len(),
            bench.embedding.chunks.len(),
            bench.bm25.chunks.len()
        );

        let (emb_all, emb_no_test) = evaluate_embedding_variants(baseline, &bench, &judgments);
        all_results.extend(emb_all.results);
        all_relevance.extend(emb_all.relevance);
        no_test_results.extend(emb_no_test.results);
        no_test_relevance.extend(emb_no_test.relevance);

        let (bm25_all, bm25_no_test) = evaluate_bm25_variants(baseline, &bench, &judgments);
        all_results.extend(bm25_all.results);
        all_relevance.extend(bm25_all.relevance);
        no_test_results.extend(bm25_no_test.results);
        no_test_relevance.extend(bm25_no_test.relevance);

        all_bench.push((baseline.to_string(), bench));
    }

    if all_results.is_empty() {
        eprintln!("No baseline data loaded. Run `gen_bench_express` first.");
        return Ok(());
    }

    let out = paths.output_dir();
    std::fs::create_dir_all(&out)?;
    write_run_manifest(&out, &all_bench)?;

    generate_evaluation_scope_csv(&out, &all_judgments, &judgments)?;

    write_aggregate_reports(&out, &all_results, &no_test_results)?;
    write_test_diagnostics(&out, &all_bench)?;
    write_relevance_reports(&out, &all_relevance, &no_test_relevance)?;

    println!("\n✓ All evaluation files written to: {}", out.display());
    let mut summary_results = all_results;
    summary_results.extend(no_test_results);
    print_console_summary(&summary_results);

    Ok(())
}
