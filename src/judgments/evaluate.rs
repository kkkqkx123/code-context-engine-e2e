//! Shared benchmark evaluation logic
//!
//! Provides the scoring functions used by all benchmark runners:
//! embedding cosine-similarity, BM25, file-documentation ranking,
//! and per-query relevance-info collection.

use std::collections::HashMap;

use crate::bench_data::{
    BenchmarkData, ChunkData, QueryType, RelevanceJudgment, RelevanceLevel, compute_bm25_scores,
    cosine_similarity,
};
use crate::infra::{QueryForms, build_query_forms};
use crate::range_evaluator::{RangeBasedPerQueryScore, RankedScan, is_relevant_to_query};
use cce_storage_bm25::TermOperator;
use cce_types::{TestInfo, TestStatus};

pub const BASELINES: &[&str] = &[
    "full_pipeline",
    "full_pipeline_raw_source",
    "direct_chunking",
];
pub const TOP_K_VALUES: &[usize] = &[5, 10, 20, 30, 50];
pub const DEFAULT_TOP_K: usize = 5;

/// Evaluation variant: whether test chunks participate in retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalVariant {
    All,
    NoTest,
}

impl EvalVariant {
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::NoTest => "no_test",
        }
    }

    fn filter_test_chunks(self) -> bool {
        matches!(self, Self::NoTest)
    }
}

pub struct BaselineResult {
    pub baseline: String,
    pub variant: String,
    pub query_id: String,
    pub query_text: String,
    pub query_type: QueryType,
    pub retriever_type: String,
    pub top_k: usize,
    pub score: RangeBasedPerQueryScore,
}

pub struct RelevanceInfo {
    pub baseline: String,
    pub query_id: String,
    pub query_text: String,
    pub retriever_type: String,
    pub strong_chunks: Vec<(usize, String, RelevanceLevel)>,
    pub related_chunks: Vec<(usize, String, RelevanceLevel)>,
}

pub struct FileDocumentationScore {
    pub baseline: String,
    pub query_id: String,
    pub retriever: String,
    pub rank: usize,
    pub document_id: String,
    pub chunk_id: String,
    pub score: f64,
    pub is_expected: bool,
}

pub struct FileDocumentationMetric {
    pub baseline: String,
    pub query_id: String,
    pub retriever: String,
    pub rr: f64,
}

/// Determine whether a chunk originates from test code.
///
/// Consumes the end-to-end `test_info` marker carried by `ChunkData` (AST
/// detection in the grouper + per-language file-path rules). When the marker
/// is `Unknown`, fall back to the per-language path rules derived from the
/// chunk's language — never default to `Test`.
pub fn is_test_chunk(chunk: &ChunkData) -> bool {
    match chunk.test_info.status {
        TestStatus::Test => true,
        TestStatus::Unknown => {
            TestInfo::from_path(chunk.language.as_ref(), &chunk.file_path).is_test()
        }
    }
}

/// Per-retriever test-code diagnostics for one baseline.
pub struct TestDiagnostics {
    pub baseline: String,
    pub retriever: String,
    pub total_chunks: usize,
    pub test_chunks: usize,
    pub unknown_chunks: usize,
    /// Chunks removed by the `NoTest` variant (equals `test_chunks`).
    pub filtered_chunks: usize,
    /// Number of test chunks appearing in the top-5 ranking across all
    /// queries of the `All` variant (occurrences, not unique chunks).
    pub top_k_test_occurrences: usize,
    /// Number of queries whose top-5 contains at least one test chunk.
    pub top_k_test_queries_affected: usize,
}

/// Collect test-code diagnostics for a baseline across both retrievers.
///
/// Ranking is recomputed here (cosine / BM25) so the `All`-variant top-k
/// occurrence statistics are independent of the score bookkeeping inside
/// `evaluate_embedding`/`evaluate_bm25`.
pub fn collect_test_diagnostics(bench: &BenchmarkData) -> Vec<TestDiagnostics> {
    vec![
        collect_embedding_diagnostics(bench),
        collect_bm25_diagnostics(bench),
    ]
}

fn collect_embedding_diagnostics(bench: &BenchmarkData) -> TestDiagnostics {
    let dim = bench.embedding.dimension as usize;
    let mut diagnostics = TestDiagnostics {
        baseline: String::new(),
        retriever: "emb".to_string(),
        total_chunks: 0,
        test_chunks: 0,
        unknown_chunks: 0,
        filtered_chunks: 0,
        top_k_test_occurrences: 0,
        top_k_test_queries_affected: 0,
    };
    if dim == 0 || bench.embedding.vectors.is_empty() {
        return diagnostics;
    }
    let n_chunks = bench.embedding.vectors.len() / dim;
    diagnostics.total_chunks = n_chunks;
    for chunk in &bench.embedding.chunks[..n_chunks] {
        if is_test_chunk(chunk) {
            diagnostics.test_chunks += 1;
        }
        if chunk.test_info.is_unknown() {
            diagnostics.unknown_chunks += 1;
        }
    }
    diagnostics.filtered_chunks = diagnostics.test_chunks;

    let n_queries = bench.embedding.query_vectors.len() / dim;
    for q_idx in 0..n_queries.min(bench.queries.len()) {
        let q_start = q_idx * dim;
        let q_vec = &bench.embedding.query_vectors[q_start..q_start + dim];
        let mut ranked: Vec<(usize, f32)> = (0..n_chunks)
            .map(|c| {
                let c_start = c * dim;
                (
                    c,
                    cosine_similarity(&bench.embedding.vectors[c_start..c_start + dim], q_vec)
                        as f32,
                )
            })
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        let top5: Vec<(usize, f32)> = ranked.iter().take(DEFAULT_TOP_K).cloned().collect();
        let affected = top5
            .iter()
            .any(|(c, _)| is_test_chunk(&bench.embedding.chunks[*c]));
        diagnostics.top_k_test_occurrences += top5
            .iter()
            .filter(|(c, _)| is_test_chunk(&bench.embedding.chunks[*c]))
            .count();
        if affected {
            diagnostics.top_k_test_queries_affected += 1;
        }
    }
    diagnostics
}

/// Score all BM25 queries against the benchmark corpus with production
/// semantics: dual-form (raw + cleaned) queries tokenized by the shared
/// `MixedTokenizer`, three-field weighted scoring, `Or` operator.
pub fn bm25_scores_for_bench(bench: &BenchmarkData) -> Vec<Vec<f64>> {
    if bench.bm25_documents.is_empty() || bench.query_texts.is_empty() {
        return Vec::new();
    }
    let queries: Vec<QueryForms> = bench
        .query_texts
        .iter()
        .map(|text| build_query_forms(text))
        .collect();
    compute_bm25_scores(&bench.bm25_documents, &queries, TermOperator::Or)
}

fn collect_bm25_diagnostics(bench: &BenchmarkData) -> TestDiagnostics {
    let mut diagnostics = TestDiagnostics {
        baseline: String::new(),
        retriever: "BM25".to_string(),
        total_chunks: bench.bm25.chunks.len(),
        test_chunks: 0,
        unknown_chunks: 0,
        filtered_chunks: 0,
        top_k_test_occurrences: 0,
        top_k_test_queries_affected: 0,
    };
    for chunk in &bench.bm25.chunks {
        if is_test_chunk(chunk) {
            diagnostics.test_chunks += 1;
        }
        if chunk.test_info.is_unknown() {
            diagnostics.unknown_chunks += 1;
        }
    }
    diagnostics.filtered_chunks = diagnostics.test_chunks;

    let bm25_scores = bm25_scores_for_bench(bench);
    for scores in bm25_scores.iter().take(bench.queries.len()) {
        let mut ranked: Vec<(usize, f64)> = scores.iter().copied().enumerate().collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        let top5: Vec<(usize, f64)> = ranked.iter().take(DEFAULT_TOP_K).cloned().collect();
        diagnostics.top_k_test_occurrences += top5
            .iter()
            .filter(|(c, _)| is_test_chunk(&bench.bm25.chunks[*c]))
            .count();
        if top5
            .iter()
            .any(|(c, _)| is_test_chunk(&bench.bm25.chunks[*c]))
        {
            diagnostics.top_k_test_queries_affected += 1;
        }
    }
    diagnostics
}

/// Evaluation outputs of one retriever pass for a single test-code variant.
#[derive(Default)]
pub struct VariantEvaluation {
    pub results: Vec<BaselineResult>,
    pub relevance: Vec<RelevanceInfo>,
}

/// Select the chunks passing a variant's test-code filter.
///
/// Returns (original indices, cloned chunks); the two are index-aligned.
fn select_chunks(chunks: &[ChunkData], variant: EvalVariant) -> (Vec<usize>, Vec<ChunkData>) {
    let indices: Vec<usize> = (0..chunks.len())
        .filter(|&i| !variant.filter_test_chunks() || !is_test_chunk(&chunks[i]))
        .collect();
    let selected: Vec<ChunkData> = indices.iter().map(|&i| chunks[i].clone()).collect();
    (indices, selected)
}

/// Evaluate one variant of a query against an already-scored chunk set.
///
/// Scores and chunks are index-aligned and pre-filtered for the variant; a
/// single sort and a single scan produce every top-k cutoff row plus the
/// top-5 relevance detail.
#[allow(clippy::too_many_arguments)]
fn evaluate_variant_query(
    baseline: &str,
    query_data: &crate::bench_data::QueryData,
    retriever_type: &str,
    judgment: &RelevanceJudgment,
    chunks: &[ChunkData],
    scores: &[f64],
    variant: EvalVariant,
    out: &mut VariantEvaluation,
) {
    let mut ranked: Vec<usize> = (0..chunks.len()).collect();
    ranked.sort_by(|left, right| scores[*right].total_cmp(&scores[*left]));

    let max_cutoff = TOP_K_VALUES.iter().copied().max().unwrap_or(0);
    let mut scan = RankedScan::new(judgment, TOP_K_VALUES, chunks.len());
    for &index in ranked.iter().take(max_cutoff) {
        if let Some(chunk) = chunks.get(index) {
            scan.advance(chunk);
        }
    }
    let range_scores = scan.finish();

    for (&top_k, (range, _)) in TOP_K_VALUES.iter().zip(range_scores) {
        out.results.push(BaselineResult {
            baseline: baseline.to_string(),
            variant: variant.label().to_string(),
            query_id: query_data.id.clone(),
            query_text: query_data.text.clone(),
            query_type: judgment.query_type,
            retriever_type: retriever_type.to_string(),
            top_k,
            score: range,
        });
    }

    out.relevance.push(build_relevance_info_ranked(
        baseline,
        query_data,
        retriever_type,
        judgment,
        chunks,
        &ranked,
        DEFAULT_TOP_K,
    ));
}

pub fn evaluate_embedding(
    baseline: &str,
    bench: &BenchmarkData,
    judgments: &[RelevanceJudgment],
    results: &mut Vec<BaselineResult>,
    relevance: &mut Vec<RelevanceInfo>,
    variant: EvalVariant,
) {
    let dim = bench.embedding.dimension as usize;
    if dim == 0 || bench.embedding.vectors.is_empty() {
        return;
    }
    let n_chunks = bench.embedding.vectors.len() / dim;
    let n_queries = bench.embedding.query_vectors.len() / dim;

    let (chunk_indices, filtered_chunks) =
        select_chunks(&bench.embedding.chunks[..n_chunks], variant);
    let mut out = VariantEvaluation::default();

    for q_idx in 0..bench.queries.len().min(n_queries) {
        let query_data = &bench.queries[q_idx];
        let Some(judgment) = judgments.iter().find(|j| j.id == query_data.id) else {
            continue;
        };

        let q_start = q_idx * dim;
        let q_vec = &bench.embedding.query_vectors[q_start..q_start + dim];
        let scores: Vec<f64> = chunk_indices
            .iter()
            .map(|&c_idx| {
                let c_start = c_idx * dim;
                cosine_similarity(&bench.embedding.vectors[c_start..c_start + dim], q_vec)
            })
            .collect();

        evaluate_variant_query(
            baseline,
            query_data,
            "emb",
            judgment,
            &filtered_chunks,
            &scores,
            variant,
            &mut out,
        );
    }

    results.extend(out.results);
    relevance.extend(out.relevance);
}

/// Evaluate both test-code variants with shared score computation.
///
/// The embedding scores, BM25 scores and per-query sorts do not depend on the
/// test-code filter, so one pass serves both the `All` and `NoTest` variants.
pub fn evaluate_embedding_variants(
    baseline: &str,
    bench: &BenchmarkData,
    judgments: &[RelevanceJudgment],
) -> (VariantEvaluation, VariantEvaluation) {
    let dim = bench.embedding.dimension as usize;
    if dim == 0 || bench.embedding.vectors.is_empty() {
        return (VariantEvaluation::default(), VariantEvaluation::default());
    }
    let n_chunks = bench.embedding.vectors.len() / dim;
    let n_queries = bench.embedding.query_vectors.len() / dim;

    let all_chunks: Vec<ChunkData> = bench.embedding.chunks[..n_chunks].to_vec();
    let (no_test_indices, no_test_chunks) = select_chunks(&all_chunks, EvalVariant::NoTest);
    let mut all = VariantEvaluation::default();
    let mut no_test = VariantEvaluation::default();

    for q_idx in 0..bench.queries.len().min(n_queries) {
        let query_data = &bench.queries[q_idx];
        let Some(judgment) = judgments.iter().find(|j| j.id == query_data.id) else {
            continue;
        };

        let q_start = q_idx * dim;
        let q_vec = &bench.embedding.query_vectors[q_start..q_start + dim];
        let scores: Vec<f64> = (0..n_chunks)
            .map(|c_idx| {
                let c_start = c_idx * dim;
                cosine_similarity(&bench.embedding.vectors[c_start..c_start + dim], q_vec)
            })
            .collect();

        evaluate_variant_query(
            baseline,
            query_data,
            "emb",
            judgment,
            &all_chunks,
            &scores,
            EvalVariant::All,
            &mut all,
        );
        let no_test_scores: Vec<f64> = no_test_indices.iter().map(|&i| scores[i]).collect();
        evaluate_variant_query(
            baseline,
            query_data,
            "emb",
            judgment,
            &no_test_chunks,
            &no_test_scores,
            EvalVariant::NoTest,
            &mut no_test,
        );
    }

    (all, no_test)
}

pub fn evaluate_bm25(
    baseline: &str,
    bench: &BenchmarkData,
    judgments: &[RelevanceJudgment],
    results: &mut Vec<BaselineResult>,
    relevance: &mut Vec<RelevanceInfo>,
    variant: EvalVariant,
) {
    if bench.bm25_documents.is_empty() {
        return;
    }
    let bm25_scores = bm25_scores_for_bench(bench);

    let (chunk_indices, filtered_chunks) = select_chunks(&bench.bm25.chunks, variant);
    let mut out = VariantEvaluation::default();

    for (q_idx, scores) in bm25_scores
        .iter()
        .enumerate()
        .take(bench.queries.len().min(bm25_scores.len()))
    {
        let query_data = &bench.queries[q_idx];
        let Some(judgment) = judgments.iter().find(|j| j.id == query_data.id) else {
            continue;
        };

        let filtered_scores: Vec<f64> = chunk_indices.iter().map(|&i| scores[i]).collect();
        evaluate_variant_query(
            baseline,
            query_data,
            "BM25",
            judgment,
            &filtered_chunks,
            &filtered_scores,
            variant,
            &mut out,
        );
    }

    results.extend(out.results);
    relevance.extend(out.relevance);
}

/// Evaluate both test-code variants with shared score computation.
///
/// The BM25 scores and per-query sorts do not depend on the test-code filter,
/// so one scoring pass serves both the `All` and `NoTest` variants.
pub fn evaluate_bm25_variants(
    baseline: &str,
    bench: &BenchmarkData,
    judgments: &[RelevanceJudgment],
) -> (VariantEvaluation, VariantEvaluation) {
    if bench.bm25_documents.is_empty() {
        return (VariantEvaluation::default(), VariantEvaluation::default());
    }
    let bm25_scores = bm25_scores_for_bench(bench);

    let (_, all_chunks) = select_chunks(&bench.bm25.chunks, EvalVariant::All);
    let (no_test_indices, no_test_chunks) = select_chunks(&bench.bm25.chunks, EvalVariant::NoTest);
    let mut all = VariantEvaluation::default();
    let mut no_test = VariantEvaluation::default();

    for (q_idx, scores) in bm25_scores
        .iter()
        .enumerate()
        .take(bench.queries.len().min(bm25_scores.len()))
    {
        let query_data = &bench.queries[q_idx];
        let Some(judgment) = judgments.iter().find(|j| j.id == query_data.id) else {
            continue;
        };

        let all_scores: Vec<f64> = scores.to_vec();
        evaluate_variant_query(
            baseline,
            query_data,
            "BM25",
            judgment,
            &all_chunks,
            &all_scores,
            EvalVariant::All,
            &mut all,
        );
        let no_test_scores: Vec<f64> = no_test_indices.iter().map(|&i| scores[i]).collect();
        evaluate_variant_query(
            baseline,
            query_data,
            "BM25",
            judgment,
            &no_test_chunks,
            &no_test_scores,
            EvalVariant::NoTest,
            &mut no_test,
        );
    }

    (all, no_test)
}

pub fn build_relevance_info(
    baseline: &str,
    query_data: &crate::bench_data::QueryData,
    retriever_type: &str,
    judgment: &RelevanceJudgment,
    chunks: &[ChunkData],
    scores: &[f64],
    top_k: usize,
) -> RelevanceInfo {
    let mut indices: Vec<usize> = (0..chunks.len()).collect();
    indices.sort_by(|a, b| {
        scores[*b]
            .partial_cmp(&scores[*a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    build_relevance_info_ranked(
        baseline,
        query_data,
        retriever_type,
        judgment,
        chunks,
        &indices,
        top_k,
    )
}

/// Build top-k relevance detail rows from an already-ranked chunk list.
///
/// `ranked_indices` must be sorted by score descending.
fn build_relevance_info_ranked(
    baseline: &str,
    query_data: &crate::bench_data::QueryData,
    retriever_type: &str,
    judgment: &RelevanceJudgment,
    chunks: &[ChunkData],
    ranked_indices: &[usize],
    top_k: usize,
) -> RelevanceInfo {
    let mut strong = Vec::new();
    let mut related = Vec::new();

    for (rank, &idx) in ranked_indices.iter().enumerate().take(top_k) {
        let Some(chunk) = chunks.get(idx) else {
            continue;
        };
        if let Some(level) = is_relevant_to_query(chunk, judgment) {
            let loc = format!(
                "{}:{}-{}",
                chunk.file_path, chunk.start_line, chunk.end_line
            );
            match level {
                RelevanceLevel::Strong => strong.push((rank + 1, loc, level)),
                RelevanceLevel::Related => related.push((rank + 1, loc, level)),
                _ => {}
            }
        }
    }

    RelevanceInfo {
        baseline: baseline.to_string(),
        query_id: query_data.id.clone(),
        query_text: query_data.text.clone(),
        retriever_type: retriever_type.to_string(),
        strong_chunks: strong,
        related_chunks: related,
    }
}

pub fn document_group_id(chunk_id: &str) -> String {
    for marker in ["_bm25_", "_emb_"] {
        if let Some((document_id, _)) = chunk_id.rsplit_once(marker) {
            return document_id.to_string();
        }
    }
    chunk_id.to_string()
}

pub fn score_file_documentation(
    baseline: &str,
    bench: &BenchmarkData,
) -> Result<(Vec<FileDocumentationScore>, Vec<FileDocumentationMetric>), String> {
    if bench.queries.is_empty() || bench.bm25.chunks.is_empty() || bench.embedding.chunks.is_empty()
    {
        return Err(
            "File-documentation benchmark must contain queries and both retrieval paths".into(),
        );
    }

    let mut scores = Vec::new();
    let mut metrics = Vec::new();

    let bm25_scores = bm25_scores_for_bench(bench);
    for (query_index, query) in bench.queries.iter().enumerate() {
        let ranked: Vec<_> = bm25_scores
            .get(query_index)
            .into_iter()
            .flatten()
            .copied()
            .enumerate()
            .collect();
        score_file_documentation_ranking(
            baseline,
            query,
            "BM25",
            ranked,
            &bench.bm25.chunks,
            &mut scores,
            &mut metrics,
        );
    }

    let dimension = bench.embedding.dimension as usize;
    let embedding_valid = dimension > 0
        && bench.embedding.vectors.len() == bench.embedding.chunks.len() * dimension
        && bench.embedding.query_vectors.len() == bench.queries.len() * dimension;

    if embedding_valid {
        for (query_index, query) in bench.queries.iter().enumerate() {
            let query_start = query_index * dimension;
            let query_vector = &bench.embedding.query_vectors[query_start..query_start + dimension];
            let ranked: Vec<(usize, f64)> = bench
                .embedding
                .chunks
                .iter()
                .enumerate()
                .map(|(chunk_index, _)| {
                    let chunk_start = chunk_index * dimension;
                    (
                        chunk_index,
                        cosine_similarity(
                            query_vector,
                            &bench.embedding.vectors[chunk_start..chunk_start + dimension],
                        ),
                    )
                })
                .collect();
            score_file_documentation_ranking(
                baseline,
                query,
                "Embedding",
                ranked,
                &bench.embedding.chunks,
                &mut scores,
                &mut metrics,
            );
        }
    } else {
        eprintln!(
            "  WARNING: {baseline} file-doc embedding vectors missing or invalid; skipping embedding evaluation"
        );
    }

    Ok((scores, metrics))
}

fn score_file_documentation_ranking(
    baseline: &str,
    query: &crate::bench_data::QueryData,
    retriever: &str,
    ranked_chunks: Vec<(usize, f64)>,
    chunks: &[ChunkData],
    scores: &mut Vec<FileDocumentationScore>,
    metrics: &mut Vec<FileDocumentationMetric>,
) {
    let mut best_by_document: HashMap<String, (usize, f64)> = HashMap::new();
    for (chunk_index, score) in ranked_chunks {
        let Some(chunk) = chunks.get(chunk_index) else {
            continue;
        };
        let document_id = document_group_id(&chunk.chunk_id);
        let replace = best_by_document
            .get(&document_id)
            .is_none_or(|(_, current)| score > *current);
        if replace {
            best_by_document.insert(document_id, (chunk_index, score));
        }
    }

    let mut ranked_documents: Vec<_> = best_by_document.into_iter().collect();
    ranked_documents.sort_by(|left, right| {
        right
            .1
            .1
            .total_cmp(&left.1.1)
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.1.0.cmp(&right.1.0))
    });

    let mut rr = 0.0;

    for (index, (document_id, (chunk_index, score))) in ranked_documents.iter().enumerate() {
        let rank = index + 1;
        let Some(chunk) = chunks.get(*chunk_index) else {
            continue;
        };
        let is_expected = query.relevant_names.iter().any(|name| document_id == name);
        if is_expected && rr == 0.0 {
            rr = 1.0 / rank as f64;
        }
        scores.push(FileDocumentationScore {
            baseline: baseline.to_string(),
            query_id: query.id.clone(),
            retriever: retriever.to_string(),
            rank,
            document_id: document_id.clone(),
            chunk_id: chunk.chunk_id.clone(),
            score: *score,
            is_expected,
        });
    }

    metrics.push(FileDocumentationMetric {
        baseline: baseline.to_string(),
        query_id: query.id.clone(),
        retriever: retriever.to_string(),
        rr,
    });
}

/// Data paths for a benchmark project
pub struct BenchmarkPaths {
    pub project_name: &'static str,
}

impl BenchmarkPaths {
    pub fn new(project_name: &'static str) -> Self {
        Self { project_name }
    }

    pub fn output_dir(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("outputs")
            .join("benchmark")
            .join(self.project_name)
    }

    pub fn data_dir(&self, baseline: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("benchmark")
            .join(baseline)
            .join(self.project_name)
            .join("bge-m3")
            .join("bench_data.rkyv")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_data::{ChunkSourceRange, QueryData};

    fn chunk(chunk_id: &str) -> ChunkData {
        ChunkData {
            chunk_id: chunk_id.to_string(),
            entity_name: String::new(),
            file_path: String::new(),
            start_line: 0,
            end_line: 0,
            source_ranges: Vec::<ChunkSourceRange>::new(),
            source_span_kind: String::new(),
            test_info: TestInfo::unknown(),
            language: None,
            entity_ids: Vec::new(),
            segment_id: String::new(),
        }
    }

    #[test]
    fn file_documentation_ranking_breaks_score_ties_by_document_id() {
        let query = QueryData {
            id: "FD1".to_string(),
            text: "query".to_string(),
            query_type: crate::bench_data::QueryType::FileDocumentation,
            relevant_names: vec!["file_doc_a.rs".to_string()],
            irrelevant_names: Vec::new(),
        };
        let chunks = vec![chunk("file_doc_b.rs_emb_0"), chunk("file_doc_a.rs_emb_0")];
        let mut scores = Vec::new();
        let mut metrics = Vec::new();

        score_file_documentation_ranking(
            "baseline",
            &query,
            "Embedding",
            vec![(0, 0.0), (1, 0.0)],
            &chunks,
            &mut scores,
            &mut metrics,
        );

        assert_eq!(scores[0].document_id, "file_doc_a.rs");
        assert_eq!(scores[0].rank, 1);
        assert_eq!(metrics[0].rr, 1.0);
    }
}
