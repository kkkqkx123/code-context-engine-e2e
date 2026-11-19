//! Full index pipeline performance probe.
//!
//! Indexes a deterministic synthetic project through the production query
//! workflow: scanner, parser, dual-path text generation, summaries, mock
//! embeddings, local vector store, local BM25 index, SQLite metadata and
//! relation publication. Resource counters are sampled before and after the
//! timed phase so the cost of all backend stages is visible in one row.
//!
//! Run with `cargo run --release -p cce-e2e-tests --example full_pipeline_perf`.
//! `CCE_PERF_SIZE=medium` switches from the 500-file default to 5,000 files;
//! `CCE_PERF_SIZE=large` uses 50,000 files.
//! Overall trends are appended to `outputs/benchmark/full_pipeline_perf.tsv`;
//! per-stage timings are appended to `outputs/benchmark/full_pipeline_stages.tsv`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use cce_bench_kit::{
    DEFAULT_LANG_DIST, ProjectSize, SyntheticProject, cpu_ms, resource_delta, resource_snapshot,
};
use cce_e2e_tests::fixture::FixtureAccess;
use cce_e2e_tests::{EmbeddingConfig, QueryWorkflowTest};
use cce_orchestrator::SearchSources;

struct SyntheticFixture {
    root: PathBuf,
}

impl FixtureAccess for SyntheticFixture {
    fn root(&self) -> &Path {
        &self.root
    }

    fn file<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        self.root.join(relative_path)
    }

    fn add_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        relative_path: P,
        content: C,
    ) -> io::Result<PathBuf> {
        let path = self.file(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(path)
    }

    fn modify_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        relative_path: P,
        content: C,
    ) -> io::Result<PathBuf> {
        self.add_file(relative_path, content)
    }

    fn delete_file<P: AsRef<Path>>(&self, relative_path: P) -> io::Result<()> {
        fs::remove_file(self.file(relative_path))
    }
}

fn size_from_env() -> ProjectSize {
    match std::env::var("CCE_PERF_SIZE").as_deref() {
        Ok("medium") => ProjectSize::Medium,
        Ok("large") => ProjectSize::Large,
        _ => ProjectSize::Small,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = size_from_env();
    let root = tempfile::TempDir::new()?;
    let project = SyntheticProject::generate(size, DEFAULT_LANG_DIST);
    let file_count = project.files.len();
    project.materialize(root.path())?;
    println!(
        "full pipeline perf: {file_count} files at {:?}",
        root.path()
    );

    let fixture = SyntheticFixture {
        root: root.path().to_path_buf(),
    };
    let mut workflow = QueryWorkflowTest::new(fixture, EmbeddingConfig::mock())
        .with_sources(SearchSources::none().with_vector().with_bm25())
        .with_extensions(vec![
            "rs".to_string(),
            "py".to_string(),
            "go".to_string(),
            "ts".to_string(),
        ]);

    let before = resource_snapshot();
    let started = Instant::now();
    let index_result = workflow.index().await?;
    let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let after = resource_snapshot();
    let resources = resource_delta(before, after);

    let indexed_files = index_result.indexed_files;
    let failed_files = index_result.failed_files;
    let entities = index_result.total_entities;
    let relations = index_result.total_relations;
    let vectors = index_result.total_vectors;
    let reported_ms = index_result.elapsed_ms;
    let stages = &index_result.stages;

    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("outputs/benchmark");
    fs::create_dir_all(&output_dir)?;
    let output = output_dir.join("full_pipeline_perf.tsv");
    let mut report = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&output)?;
    if report.metadata()?.len() == 0 {
        use std::io::Write as _;
        writeln!(
            report,
            "# tier\tfiles\tindexed\tfailed\tentities\trelations\tvectors\treported_ms\twall_ms\tfiles_per_sec\tcpu_ms\tpeak_rss_kb\tpeak_rss_delta_kb\tread_bytes_delta\twrite_bytes_delta"
        )?;
    }
    use std::io::Write as _;
    writeln!(
        report,
        "{}\t{file_count}\t{indexed_files}\t{failed_files}\t{entities}\t{relations}\t{vectors}\t{reported_ms}\t{wall_ms:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{}",
        size.label(),
        indexed_files as f64 / (wall_ms / 1000.0),
        cpu_ms(&resources).unwrap_or(0),
        after.peak_rss_kb.unwrap_or(0),
        resources.peak_rss_kb.unwrap_or(0),
        resources.read_bytes.unwrap_or(0),
        resources.write_bytes.unwrap_or(0),
    )?;

    let mut stage_report = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_dir.join("full_pipeline_stages.tsv"))?;
    if stage_report.metadata()?.len() == 0 {
        use std::io::Write as _;
        writeln!(
            stage_report,
            "# tier\tfiles\twall_ms\tscan_ms\tbatch_process_ms\tbatch_setup_ms\tentity_store_ms\tvector_store_ms\tbm25_store_ms\tsummary_ms\trelation_ms\trelation_spool_ms\trelation_spool_write_ms\trelation_symbols_ms\tsymbol_modules_ms\tsymbol_type_index_ms\tsymbol_overload_ms\tsymbol_flat_ms\tsymbol_cross_file_ms\texport_spool_ms\texport_ms\tmanifest_ms\tbatch_overhead_ms\tbatches\tvector_calls\tbm25_calls"
        )?;
    }
    writeln!(
        stage_report,
        "{}\t{file_count}\t{wall_ms:.1}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        size.label(),
        stages.scan_ms,
        stages.batch_process_ms,
        stages.batch_setup_ms,
        stages.entity_store_ms,
        stages.vector_store_ms,
        stages.bm25_store_ms,
        stages.summary_ms,
        stages.relation_ms,
        stages.relation_spool_ms,
        stages.relation_spool_write_ms,
        stages.relation_symbols_ms,
        stages.symbol_modules_ms,
        stages.symbol_type_index_ms,
        stages.symbol_overload_ms,
        stages.symbol_flat_ms,
        stages.symbol_cross_file_ms,
        stages.export_spool_ms,
        stages.export_ms,
        stages.manifest_ms,
        stages.batch_overhead_ms,
        stages.batches,
        stages.vector_store_calls,
        stages.bm25_store_calls,
    )?;

    println!(
        "indexed {indexed_files}/{file_count} files in {wall_ms:.1} ms, {} entities, {} vectors, peak RSS {} KiB",
        entities,
        vectors,
        after.peak_rss_kb.unwrap_or(0)
    );
    workflow.cleanup().await;
    Ok(())
}
