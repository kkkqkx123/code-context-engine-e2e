use cce_orchestrator::index::FileProcessor;
use cce_parser::ast_to_nl::chunker::{ChunkPath, ChunkedResult};
use cce_parser::grouper::GroupType;
use cce_relation::IndexBuilder;
use cce_types::{OutputMode, ParsedFile};

use crate::FixtureSpec;
use crate::bench_data::{CallEdgeData, ChunkData};
use crate::bench_gen::{
    GeneratedChunks, bm25_doc_records, chunk_data_from_result, collect_call_edges,
    fixture_source_path, scan_fixture, validate_unique_chunks,
};

/// Split a single-parse `OutputMode::Both` result into the two retrieval paths.
///
/// The production index parses each file **once** and generates both paths
/// from the same parse (see `IndexOrchestrator::decide_output_mode`), so both
/// paths share the same `entity_ids` and `segment_id`s — the cross-path
/// alignment contract documented in `chunker.rs` / `hybrid-fusion-design.md`.
/// The benchmark must mirror this: parsing once per mode would consume a fresh
/// slice of the global entity-id counter per path and silently break
/// entity-level alignment (0% alignment coverage).
fn split_by_path(results: Vec<ChunkedResult>) -> (Vec<ChunkedResult>, Vec<ChunkedResult>) {
    let mut emb = Vec::new();
    let mut bm25 = Vec::new();
    for chunk in results {
        match chunk.path {
            ChunkPath::Bm25 => bm25.push(chunk),
            ChunkPath::Embedding => emb.push(chunk),
        }
    }
    (emb, bm25)
}

pub async fn gen_full_pipeline_chunks(
    target_spec: FixtureSpec,
    distractor_spec: FixtureSpec,
) -> anyhow::Result<GeneratedChunks> {
    let target_entries = scan_fixture(target_spec.clone())?;
    let dist_entries = scan_fixture(distractor_spec)?;

    let mut processor = FileProcessor::new();

    let mut emb_results: Vec<ChunkedResult> = Vec::new();
    let mut bm25_results: Vec<ChunkedResult> = Vec::new();
    let mut target_parsed: Vec<ParsedFile> = Vec::new();

    // Target entries: collect chunks and the parse products used for the
    // relation-edge sidecar (distractors share relative paths and would
    // collide, so only the target fixture feeds the index).
    for entry in &target_entries {
        if let Ok(result) = processor
            .process_file_complete(entry, OutputMode::Both)
            .await
        {
            let (emb, bm25) = split_by_path(result.chunks);
            emb_results.extend(emb);
            bm25_results.extend(bm25);
            target_parsed.push(result.parsed_file);
        }
    }
    // Distractor entries: collect chunks only (no file-docs — same relative
    // paths would collide on chunk_id and file-doc queries target only).
    for entry in &dist_entries {
        if let Ok(result) = processor
            .process_file_complete(entry, OutputMode::Both)
            .await
        {
            let (emb, bm25) = split_by_path(result.chunks);
            emb_results.extend(emb);
            bm25_results.extend(bm25);
        }
    }

    exclude_file_documentation(&mut emb_results, "full-pipeline embedding");
    exclude_file_documentation(&mut bm25_results, "full-pipeline BM25");

    eprintln!(
        "=== FULL PIPELINE: {} embedding chunks, {} bm25 chunks ===",
        emb_results.len(),
        bm25_results.len(),
    );

    let embedding_chunks: Vec<ChunkData> = emb_results.iter().map(chunk_data_from_result).collect();
    let bm25_chunks: Vec<ChunkData> = bm25_results.iter().map(chunk_data_from_result).collect();
    let embedding_texts: Vec<String> = emb_results.iter().map(|cr| cr.text.clone()).collect();
    let bm25_texts: Vec<String> = bm25_results.iter().map(|cr| cr.text.clone()).collect();
    let bm25_documents = bm25_doc_records(&bm25_results);

    validate_unique_chunks(&embedding_chunks, "full_pipeline embedding");
    validate_unique_chunks(&bm25_chunks, "full_pipeline bm25");

    Ok(GeneratedChunks {
        embedding_chunks,
        embedding_texts,
        bm25_chunks,
        bm25_texts,
        bm25_documents,
        call_edges: build_call_edges(target_spec, &target_parsed),
    })
}

/// Build the relation index from the same parses that produced the chunks and
/// extract the in-project call-edge sidecar for offline assembly review.
fn build_call_edges(target_spec: FixtureSpec, parsed_files: &[ParsedFile]) -> Vec<CallEdgeData> {
    if parsed_files.is_empty() {
        return Vec::new();
    }
    let builder = IndexBuilder::new();
    let project_symbols = builder.create_project_symbol_table(fixture_source_path(target_spec));
    for pf in parsed_files {
        builder.add_file_symbols(pf, &project_symbols);
    }
    for pf in parsed_files {
        builder.register_file_entities(pf);
    }
    for pf in parsed_files {
        builder.resolve_file_relations(pf, &project_symbols);
    }
    let index = builder.build();
    let edges = collect_call_edges(&index);
    eprintln!(
        "=== FULL PIPELINE: {} in-project call edges over {} parsed files ===",
        edges.len(),
        parsed_files.len()
    );
    edges
}

pub fn exclude_file_documentation(chunks: &mut Vec<ChunkedResult>, label: &str) {
    let before = chunks.len();
    chunks.retain(|chunk| chunk.group_type != GroupType::FileDocumentation);
    let removed = before.saturating_sub(chunks.len());
    if removed > 0 {
        eprintln!("  Excluded {removed} file-documentation chunks from {label}");
    }
}
