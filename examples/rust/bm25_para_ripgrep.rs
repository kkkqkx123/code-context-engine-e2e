use std::path::PathBuf;

use cce_e2e_tests::bm25_parameter_benchmark::{
    aggregate_by_parameter_set, run_ripgrep_bm25_parameter_sweep,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cce_e2e_tests::init_minimal_logging();
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs/benchmark/ripgrep/bm25_parameter_sweep");
    let observations = run_ripgrep_bm25_parameter_sweep(&output_dir).await?;

    let aggregates = aggregate_by_parameter_set(&observations);
    for agg in &aggregates {
        println!(
            "{}: MRR@10={:.4}, F1@10={:.4}, Recall@20={:.4}",
            agg.parameter_key.display_name(),
            agg.strong_mrr_at_10,
            agg.strong_f1_at_10,
            agg.recall_any_at_20,
        );
    }
    println!("Results written to {}", output_dir.display());
    Ok(())
}
