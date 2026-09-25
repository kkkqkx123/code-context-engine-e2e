use std::collections::HashMap;

use cce_orchestrator::index::FileProcessor;
use cce_parser::ast_to_nl::chunker::{ChunkPath, ChunkedResult};
use cce_parser::grouper::GroupType;
use cce_types::OutputMode;

use crate::bench_data::ChunkData;
use crate::bench_gen::{
    GeneratedChunks, bm25_doc_records, chunk_data_from_result, normalize_file_path, scan_fixture,
    validate_unique_chunks,
};

/// Split a single-parse `OutputMode::Both` result into the two retrieval paths
/// (see `baselines/full_pipeline.rs` for the alignment rationale).
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

pub async fn gen_full_pipeline_raw_source_chunks(
    target_spec: crate::FixtureSpec,
    distractor_spec: crate::FixtureSpec,
) -> anyhow::Result<GeneratedChunks> {
    let target_entries = scan_fixture(target_spec)?;
    let dist_entries = scan_fixture(distractor_spec)?;

    let mut source_map: HashMap<String, String> = HashMap::new();
    for entry in target_entries.iter().chain(dist_entries.iter()) {
        let path_str = normalize_file_path(&entry.relative_path.to_string_lossy());
        if let Ok(content) = std::fs::read_to_string(&entry.path) {
            source_map.entry(path_str).or_insert(content);
        }
    }

    let mut processor = FileProcessor::new();

    let mut emb_results: Vec<cce_parser::ast_to_nl::chunker::ChunkedResult> = Vec::new();
    let mut bm25_results: Vec<cce_parser::ast_to_nl::chunker::ChunkedResult> = Vec::new();

    for entry in target_entries.iter().chain(dist_entries.iter()) {
        if let Ok(result) = processor
            .process_file_complete(entry, OutputMode::Both)
            .await
        {
            let (emb, bm25) = split_by_path(result.chunks);
            emb_results.extend(emb);
            bm25_results.extend(bm25);
        }
    }

    emb_results.retain(|chunk| chunk.group_type != GroupType::FileDocumentation);
    for chunk in &mut emb_results {
        chunk.text = extract_raw_source(chunk, &source_map);
    }

    bm25_results.retain(|chunk| chunk.group_type != GroupType::FileDocumentation);
    for chunk in &mut bm25_results {
        chunk.text = extract_raw_source(chunk, &source_map);
    }

    eprintln!(
        "=== FULL PIPELINE RAW SOURCE: {} embedding chunks, {} bm25 chunks ===",
        emb_results.len(),
        bm25_results.len(),
    );

    let embedding_chunks: Vec<ChunkData> = emb_results.iter().map(chunk_data_from_result).collect();
    let bm25_chunks: Vec<ChunkData> = bm25_results.iter().map(chunk_data_from_result).collect();
    let embedding_texts: Vec<String> = emb_results.iter().map(|cr| cr.text.clone()).collect();
    let bm25_texts: Vec<String> = bm25_results.iter().map(|cr| cr.text.clone()).collect();
    let bm25_documents = bm25_doc_records(&bm25_results);

    validate_unique_chunks(&embedding_chunks, "full_pipeline_raw_source embedding");
    validate_unique_chunks(&bm25_chunks, "full_pipeline_raw_source bm25");

    Ok(GeneratedChunks {
        embedding_chunks,
        embedding_texts,
        bm25_chunks,
        bm25_texts,
        bm25_documents,
        call_edges: Vec::new(),
    })
}

fn extract_raw_source(
    chunk: &cce_parser::ast_to_nl::chunker::ChunkedResult,
    source_map: &HashMap<String, String>,
) -> String {
    const MAX_TOKENS: usize = 7200;
    // Proportional truncation is shared with the dead-letter truncate-retry
    // path via `cce_utils::token_estimation::truncate_to_token_budget`:
    // when the estimate exceeds the limit, cut to 80% of the current length
    // (at a line boundary) and re-check, with an absolute split as the final
    // fallback. The estimator can underestimate code density, so the
    // conservative ratio keeps the result under the provider cap.

    let file_path = normalize_file_path(&chunk.metadata.file_path);
    let source = source_map.get(&file_path).unwrap_or_else(|| {
        panic!(
            "raw-source extraction: file not in source map: {} (chunk_id={})",
            file_path, chunk.chunk_id
        )
    });
    let lines: Vec<&str> = source.lines().collect();
    let ranges = chunk.metadata.source_ranges();
    let mut parts: Vec<String> = Vec::new();
    for span in ranges {
        let (start, end) = match span.line_range_opt() {
            Some(r) => r,
            None => continue,
        };
        if start == 0 || end < start || start > lines.len() {
            continue;
        }
        let end = end.min(lines.len());
        parts.push(lines[start - 1..end].join("\n"));
    }
    let raw = parts.join("\n");

    // The raw baseline must contain only raw source. An empty extraction
    // means the chunk carries no source coverage (GroupFallback/Unavailable)
    // — an upstream chunking invariant violation that must fail loudly, never
    // silently substitute NL text.
    assert!(
        !raw.trim().is_empty(),
        "raw-source extraction: chunk has no source coverage: file={} chunk_id={} entity={}",
        file_path,
        chunk.chunk_id,
        chunk.bm25_title.as_deref().unwrap_or("unknown"),
    );

    let result = cce_utils::token_estimation::truncate_to_token_budget(&raw, MAX_TOKENS);
    if result.truncated {
        eprintln!(
            "  raw-source: chunk {} truncated from {} to {} bytes (estimate {} > {} tokens)",
            chunk.chunk_id,
            result.original_len,
            result.final_len,
            result.original_estimate,
            MAX_TOKENS
        );
    }
    result.text
}
