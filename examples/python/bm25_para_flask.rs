use std::path::PathBuf;

use cce_e2e_tests::bm25_parameter_benchmark::{
    aggregate_by_parameter_set, run_flask_bm25_parameter_sweep,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cce_e2e_tests::init_minimal_logging();
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs/benchmark/flask/bm25_parameter_sweep");
    let observations = run_flask_bm25_parameter_sweep(&output_dir).await?;

    let aggregates = aggregate_by_parameter_set(&observations);
    for aggregate in &aggregates {
        println!(
            "{}: MRR@10={:.4}, F1@10={:.4}, Recall@20={:.4}",
            aggregate.parameter_key.display_name(),
            aggregate.strong_mrr_at_10,
            aggregate.strong_f1_at_10,
            aggregate.recall_any_at_20,
        );
    }
    println!("Results written to {}", output_dir.display());
    Ok(())
}
