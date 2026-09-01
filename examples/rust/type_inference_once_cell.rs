//! Type inference example for the once_cell fixture.
//!
//! Runs the indexing pipeline on the once_cell Rust fixture, then uses the
//! `type_inference_output` module to render symbol table, call graph, and
//! a combined type inference report.
//!
//! Output:
//!   outputs/relation/rust/presentation/once_cell/symbol_table.md
//!   outputs/relation/rust/presentation/once_cell/call_graph.md
//!   outputs/relation/rust/presentation/once_cell/type_inference_report.md
//!   outputs/relation/rust/presentation/once_cell/SUMMARY.txt

use std::fs;
use std::path::PathBuf;

use cce_orchestrator::{IndexOptions, IndexOrchestrator};
use cce_relation::index::RelationQueryOps;

use cce_e2e_tests::{TestFixture, init_minimal_logging, type_inference_output};

#[tokio::main]
async fn main() {
    init_minimal_logging();

    let fixture = TestFixture::rust_once_cell().expect("Failed to load once_cell fixture");
    let project_root = fixture.root_path().to_path_buf();

    println!("=== Indexing once_cell fixture ===");

    let options = IndexOptions {
        root_dir: project_root.clone(),
        extensions: vec!["rs".to_string()],
        store_vectors: false,
        store_bm25: false,
        store_summaries: false,
        build_relations: true,
        respect_gitignore: false,
        additional_ignore_patterns: Vec::new(),
        custom_gitignore_path: None,
        ..IndexOptions::new(&project_root)
    };

    let mut orchestrator = IndexOrchestrator::new(1).expect("Failed to create IndexOrchestrator");
    let result = orchestrator
        .execute(options)
        .await
        .expect("Indexing failed");

    println!(
        "Indexed: {} files, {} entities",
        result.total_files, result.total_entities
    );

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available after indexing");

    let relation_count = relation_index.resolved_relation_count();
    println!("Relation index: {} relations", relation_count);

    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("outputs")
        .join("relation")
        .join("rust")
        .join("presentation")
        .join("once_cell");
    fs::create_dir_all(&output_dir).expect("Failed to create output dir");

    let symbol_table = type_inference_output::render_symbol_table(&relation_index, "once_cell");
    fs::write(output_dir.join("symbol_table.md"), &symbol_table)
        .expect("Failed to write symbol_table.md");
    println!("  Wrote symbol_table.md ({} bytes)", symbol_table.len());

    let call_graph = type_inference_output::render_call_graph(&relation_index, "once_cell");
    fs::write(output_dir.join("call_graph.md"), &call_graph)
        .expect("Failed to write call_graph.md");
    println!("  Wrote call_graph.md ({} bytes)", call_graph.len());

    let report = type_inference_output::render_type_inference_report(&relation_index, "once_cell");
    fs::write(output_dir.join("type_inference_report.md"), &report)
        .expect("Failed to write type_inference_report.md");
    println!("  Wrote type_inference_report.md ({} bytes)", report.len());

    let summary = format!(
        "# Type Inference Output for once_cell\n\n\
         Files indexed: {}\n\
         Entities indexed: {}\n\
         Relations indexed: {relation_count}\n\n\
         Files:\n\
         - symbol_table.md\n\
         - call_graph.md\n\
         - type_inference_report.md\n",
        result.total_files, result.total_entities
    );
    fs::write(output_dir.join("SUMMARY.txt"), &summary).expect("Failed to write SUMMARY.txt");
    println!("  Wrote SUMMARY.txt");

    println!("\n=== Output written to {} ===", output_dir.display());
}
