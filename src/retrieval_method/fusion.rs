//! Cross-path fusion for the retrieval-method benchmark.
//!
//! The embedding and BM25 paths chunk independently: there is no chunk-level
//! 1:1 correspondence between them (chunk IDs are `{group_id}_{emb|bm25}_{index}`).
//! Following the production alignment contract (`post_processing/fusion.rs`),
//! results are aligned at the entity level through an alignment key derived
//! from the production chunk metadata now carried by `ChunkData`:
//!
//! - code chunks: `e:{entity_id}` (one key per contained entity)
//! - document/plain-text chunks: `s:{segment_id}`
//! - fallback: `c:{chunk_id}`
//!
//! The min-max family delegates directly to the production
//! `fuse_hybrid_results` (with the production `expand_multi_entity_results`
//! pre-step) so the benchmark measures exactly the shipped hybrid path. The
//! RRF family stays local — RRF is the candidate under evaluation, not the
//! production algorithm — but shares the same alignment-key semantics.
//!
//! # Measurement granularity
//!
//! **Single paths** (`emb` / `bm25`) measure the raw chunk ranking — each
//! chunk exactly once, exactly as the no-aggregation baseline benchmark does.
//! Entity-level alignment (multi-entity expansion, best chunk per key) applies
//! only to the **fused** methods, where the score carrier is the alignment
//! key, not the chunk. Fused entries are evaluated against the **union** of
//! the two paths' best chunks for that key, so a hit never depends on which
//! path's chunk happened to become the representative output chunk.

use std::collections::{HashMap, HashSet};

use crate::bench_data::ChunkData;
use cce_orchestrator::query::retrieval::{HybridFusionConfig, fuse_hybrid_results};
use cce_orchestrator::query::searcher::expand_multi_entity_results;
use cce_orchestrator::query::types::SearchResult;
use cce_types::EntityId;

/// Derive the primary cross-path alignment key for a chunk.
///
/// Mirrors the production key priority: entity id, then segment id, then chunk
/// id. `(file_path, entity_name)` was previously used as an approximate proxy;
/// this is replaced by the real entity/segment ids from the chunker.
pub fn alignment_key(chunk: &ChunkData) -> String {
    if let Some(eid) = chunk.entity_ids.first() {
        format!("e:{}", eid)
    } else if !chunk.segment_id.is_empty() {
        format!("s:{}", chunk.segment_id)
    } else {
        format!("c:{}", chunk.chunk_id)
    }
}

/// Alignment keys for a chunk, expanding multi-entity chunks one key per entity.
///
/// Mirrors the production `expand_multi_entity_results` pre-fusion step so a
/// chunk containing several entities can align to each of them independently.
/// Chunks with 0 or 1 entities fall back to the primary `alignment_key`.
pub fn alignment_keys(chunk: &ChunkData) -> Vec<String> {
    if chunk.entity_ids.len() > 1 {
        chunk
            .entity_ids
            .iter()
            .map(|id| format!("e:{}", id))
            .collect()
    } else {
        vec![alignment_key(chunk)]
    }
}

/// Alignment key of a production `SearchResult`.
///
/// The production fused results are pre-expanded (at most one entity each), so
/// the key resolves from the result's own entity id — not from the chunk it
/// points at, whose `entity_ids` may list several entities.
fn result_key(result: &SearchResult) -> String {
    if let Some(eid) = result.entity_ids.first() {
        format!("e:{}", eid.0)
    } else if let Some(seg) = result.segment_id.as_deref() {
        if seg.is_empty() {
            format!("c:{}", result.id)
        } else {
            format!("s:{}", seg)
        }
    } else {
        format!("c:{}", result.id)
    }
}

/// One retrieval path with its ranked chunks.
pub struct RankedPath<'a> {
    /// Chunk metadata for the path (indices in `ranked` refer to this slice).
    pub chunks: &'a [ChunkData],
    /// Ranked (chunk index, score) pairs, sorted by score descending.
    pub ranked: Vec<(usize, f64)>,
}

impl<'a> RankedPath<'a> {
    pub fn new(chunks: &'a [ChunkData], ranked: Vec<(usize, f64)>) -> Self {
        Self { chunks, ranked }
    }
}

/// A single fused result: one representative chunk per alignment key.
#[derive(Debug, Clone)]
pub struct FusedEntry {
    pub key: String,
    pub chunk: ChunkData,
    pub fused_score: f64,
    pub vector_score: Option<f64>,
    pub bm25_score: Option<f64>,
    /// Entity-level source coverage for range-based evaluation: the union of
    /// the best-scoring chunks for this key across both paths. The
    /// representative `chunk` is only a score/identity carrier; evaluation must
    /// not depend on which path's chunk won the representative slot.
    pub coverage: ChunkData,
}

/// Collapse a ranked list to at most one entry per chunk index.
///
/// Single-path methods (`emb` / `bm25`) must measure the raw chunk ranking:
/// each chunk appears exactly once, matching the no-aggregation baseline
/// benchmark. The ranked list is already sorted by score descending, so the
/// first occurrence of each chunk index is its best-scoring entry. (This used
/// to expand multi-entity chunks into one entry per contained entity, which
/// duplicated chunk indices and crowded genuinely relevant chunks out of the
/// top-k — the regression that made single-path rows diverge from the
/// baseline.)
pub fn dedup_ranked(ranked: &[(usize, f64)]) -> Vec<(usize, f64)> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for &(idx, score) in ranked {
        if seen.insert(idx) {
            out.push((idx, score));
        }
    }
    out
}

/// Best-scoring chunk index per alignment key for a ranked path.
///
/// The input is sorted by score descending, so the first occurrence of each
/// key is its best chunk. Multi-entity chunks contribute one entry per entity
/// key.
fn best_chunk_per_key(chunks: &[ChunkData], ranked: &[(usize, f64)]) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for &(idx, _) in ranked {
        for key in alignment_keys(&chunks[idx]) {
            out.entry(key).or_insert(idx);
        }
    }
    out
}

/// Merge the source coverage of a key's best chunks from both paths.
///
/// Both paths chunk independently, so the same entity's two best chunks may
/// cover different line ranges. Entity-level evaluation judges the union; the
/// representative chunk (whose line fields/identity the report shows) is the
/// base.
fn merge_coverage(
    representative: &ChunkData,
    vector_chunks: &[ChunkData],
    bm25_chunks: &[ChunkData],
    vector_best: Option<usize>,
    bm25_best: Option<usize>,
) -> ChunkData {
    let mut merged = representative.clone();
    let mut union = |other: Option<&ChunkData>| {
        let Some(other) = other else { return };
        if other.chunk_id == representative.chunk_id {
            return;
        }
        for range in &other.source_ranges {
            let duplicate = merged
                .source_ranges
                .iter()
                .any(|r| r.start_line == range.start_line && r.end_line == range.end_line);
            if !duplicate {
                merged.source_ranges.push(range.clone());
            }
        }
        merged.start_line = merged.start_line.min(other.start_line);
        merged.end_line = merged.end_line.max(other.end_line);
    };
    union(vector_best.map(|i| &vector_chunks[i]));
    union(bm25_best.map(|i| &bm25_chunks[i]));
    merged
}

/// Which retrieval path a `RankedPath` belongs to (controls score placement).
#[derive(Clone, Copy, PartialEq)]
enum PathKind {
    Vector,
    Bm25,
}

/// Map a ranked path onto production `SearchResult`s for fusion.
///
/// Score placement mirrors what the production searcher produces before fusion:
/// the vector path carries `vector_score`, the BM25 path carries `bm25_score`.
fn to_search_results(path: &RankedPath<'_>, kind: PathKind) -> Vec<SearchResult> {
    path.ranked
        .iter()
        .map(|&(idx, score)| {
            let chunk = &path.chunks[idx];
            let (vector_score, bm25_score) = match kind {
                PathKind::Vector => (score as f32, None),
                PathKind::Bm25 => (0.0, Some(score as f32)),
            };
            SearchResult {
                id: chunk.chunk_id.clone(),
                entity_ids: chunk
                    .entity_ids
                    .iter()
                    .map(|&id| EntityId(id as u64))
                    .collect(),
                segment_id: Some(chunk.segment_id.clone()),
                score: score as f32,
                original_score: score as f32,
                vector_score,
                bm25_score,
                sources: vec![],
                file_path: chunk.file_path.clone(),
                ..Default::default()
            }
        })
        .collect()
}

/// Fuse two ranked paths through the production `fuse_hybrid_results`.
///
/// Both paths are converted to `SearchResult`s, expanded per entity (the exact
/// production pre-fusion step), and fused with weighted min-max normalization.
/// The representative chunks are resolved back from the chunk id so the rest of
/// the benchmark can evaluate them range-based; the `coverage` field carries
/// the entity-level union of both paths' best chunks.
pub fn fuse_minmax<'a>(
    vector: &RankedPath<'a>,
    bm25: &RankedPath<'a>,
    vector_weight: f64,
    bm25_weight: f64,
    include_single_path: bool,
) -> Vec<FusedEntry> {
    PreparedFusion::prepare(vector, bm25).fuse_minmax(
        vector_weight,
        bm25_weight,
        include_single_path,
    )
}

/// Weight-independent fusion preprocessing for a pair of ranked paths.
///
/// The alignment lookups (best chunk per key, per-key rank maps) and the
/// expanded `SearchResult` inputs of the production fusion depend only on the
/// rankings, not on the fusion weights. Preparing them once and reusing them
/// across every weight variant avoids rebuilding the same tables for each
/// `minmax-*` / `rrf-*` method row.
pub struct PreparedFusion<'a> {
    vector_chunks: &'a [ChunkData],
    bm25_chunks: &'a [ChunkData],
    by_id: HashMap<String, ChunkData>,
    vector_best: HashMap<String, usize>,
    bm25_best: HashMap<String, usize>,
    vector_results: Vec<SearchResult>,
    bm25_results: Vec<SearchResult>,
    /// Per alignment key: (best chunk index, 1-based rank of that chunk, raw
    /// score). The ranked list is sorted by score descending, so the first
    /// occurrence of each key is its best chunk.
    vector_key_rank: HashMap<String, (usize, usize, f64)>,
    bm25_key_rank: HashMap<String, (usize, usize, f64)>,
}

impl<'a> PreparedFusion<'a> {
    /// Preprocess the alignment tables shared by every fusion weight variant.
    pub fn prepare(vector: &RankedPath<'a>, bm25: &RankedPath<'a>) -> Self {
        let mut by_id: HashMap<String, ChunkData> = HashMap::new();
        for chunk in vector.chunks.iter().chain(bm25.chunks.iter()) {
            by_id
                .entry(chunk.chunk_id.clone())
                .or_insert_with(|| chunk.clone());
        }
        let vector_best = best_chunk_per_key(vector.chunks, &vector.ranked);
        let bm25_best = best_chunk_per_key(bm25.chunks, &bm25.ranked);
        let vector_results =
            expand_multi_entity_results(to_search_results(vector, PathKind::Vector));
        let bm25_results = expand_multi_entity_results(to_search_results(bm25, PathKind::Bm25));
        let vector_key_rank = key_rank_map(vector.chunks, &vector.ranked);
        let bm25_key_rank = key_rank_map(bm25.chunks, &bm25.ranked);
        Self {
            vector_chunks: vector.chunks,
            bm25_chunks: bm25.chunks,
            by_id,
            vector_best,
            bm25_best,
            vector_results,
            bm25_results,
            vector_key_rank,
            bm25_key_rank,
        }
    }

    /// Fuse through the production `fuse_hybrid_results` with one weight pair.
    pub fn fuse_minmax(
        &self,
        vector_weight: f64,
        bm25_weight: f64,
        include_single_path: bool,
    ) -> Vec<FusedEntry> {
        let config = HybridFusionConfig {
            vector_weight: vector_weight as f32,
            bm25_weight: bm25_weight as f32,
            include_single_path,
            min_score: 0.0,
            dedup_by_chunk: false,
        };
        let fused = fuse_hybrid_results(
            self.vector_results.clone(),
            self.bm25_results.clone(),
            &config,
        );
        fused
            .into_iter()
            .map(|r| {
                let chunk = self
                    .by_id
                    .get(&r.id)
                    .cloned()
                    .expect("fused result references a known chunk");
                let key = result_key(&r);
                let coverage = merge_coverage(
                    &chunk,
                    self.vector_chunks,
                    self.bm25_chunks,
                    self.vector_best.get(&key).copied(),
                    self.bm25_best.get(&key).copied(),
                );
                FusedEntry {
                    key,
                    chunk,
                    fused_score: r.score as f64,
                    vector_score: (r.vector_score > 0.0).then_some(r.vector_score as f64),
                    bm25_score: r.bm25_score.map(|s| s as f64),
                    coverage,
                }
            })
            .collect()
    }

    /// Fuse with reciprocal-rank fusion at one k value.
    ///
    /// Only the rank-derived scores (`1 / (k + rank)`) depend on k; the
    /// per-key alignment was prepared once in `prepare`.
    pub fn fuse_rrf(&self, k: f64, include_single_path: bool) -> Vec<FusedEntry> {
        let rrf_of = |rank: usize| 1.0 / (k + rank as f64);
        let mut all_keys: Vec<&str> = self.vector_key_rank.keys().map(String::as_str).collect();
        let mut seen: HashSet<&str> = all_keys.iter().copied().collect();
        if include_single_path {
            for key in self.bm25_key_rank.keys() {
                if seen.insert(key.as_str()) {
                    all_keys.push(key.as_str());
                }
            }
        }

        let mut fused = Vec::with_capacity(all_keys.len());
        for key in all_keys {
            let vector_entry = self.vector_key_rank.get(key);
            let bm25_entry = self.bm25_key_rank.get(key);
            let (chunk, v_norm, b_norm, v_raw, b_raw) = match (vector_entry, bm25_entry) {
                (Some(&(vi, vr, vraw)), Some(&(_, br, braw))) => (
                    self.vector_chunks[vi].clone(),
                    rrf_of(vr),
                    rrf_of(br),
                    Some(vraw),
                    Some(braw),
                ),
                (Some(&(vi, vr, vraw)), None) => {
                    if include_single_path {
                        (
                            self.vector_chunks[vi].clone(),
                            rrf_of(vr),
                            0.0,
                            Some(vraw),
                            None,
                        )
                    } else {
                        continue;
                    }
                }
                (None, Some(&(bi, br, braw))) => {
                    if include_single_path {
                        (
                            self.bm25_chunks[bi].clone(),
                            0.0,
                            rrf_of(br),
                            None,
                            Some(braw),
                        )
                    } else {
                        continue;
                    }
                }
                (None, None) => continue,
            };
            let coverage = merge_coverage(
                &chunk,
                self.vector_chunks,
                self.bm25_chunks,
                vector_entry.map(|(vi, _, _)| *vi),
                bm25_entry.map(|(bi, _, _)| *bi),
            );
            let fused_score = 0.5 * v_norm + 0.5 * b_norm;
            fused.push(FusedEntry {
                key: key.to_string(),
                chunk,
                fused_score,
                vector_score: v_raw,
                bm25_score: b_raw,
                coverage,
            });
        }
        fused.sort_by(|a, b| b.fused_score.total_cmp(&a.fused_score));
        fused
    }
}

/// Per alignment key: best chunk index, its 1-based rank and raw score.
///
/// The ranked list is sorted by score descending, so the first occurrence of
/// each key is its best chunk; since the RRF score is monotone in rank, the
/// best chunk per key is independent of k.
fn key_rank_map(
    chunks: &[ChunkData],
    ranked: &[(usize, f64)],
) -> HashMap<String, (usize, usize, f64)> {
    let mut out = HashMap::new();
    for (rank, &(idx, raw)) in ranked.iter().enumerate() {
        for key in alignment_keys(&chunks[idx]) {
            out.entry(key).or_insert((idx, rank + 1, raw));
        }
    }
    out
}

/// Fuse two deduplicated paths with reciprocal-rank fusion.
///
/// Both paths contribute on the shared `1 / (k + rank)` scale with equal
/// weight, matching the standard RRF definition. Uses the same alignment-key
/// semantics (including multi-entity expansion) as the production fusion.
pub fn fuse_rrf<'a>(
    vector: &RankedPath<'a>,
    bm25: &RankedPath<'a>,
    k: f64,
    include_single_path: bool,
) -> Vec<FusedEntry> {
    PreparedFusion::prepare(vector, bm25).fuse_rrf(k, include_single_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: &str, file: &str, name: &str, entity_id: Option<i64>, segment: &str) -> ChunkData {
        ChunkData {
            chunk_id: id.to_string(),
            entity_name: name.to_string(),
            file_path: file.to_string(),
            start_line: 1,
            end_line: 10,
            source_ranges: Vec::new(),
            source_span_kind: String::new(),
            test_info: cce_types::TestInfo::unknown(),
            language: None,
            entity_ids: entity_id.into_iter().collect(),
            segment_id: segment.to_string(),
        }
    }

    fn chunk_with_ranges(
        id: &str,
        entity_ids: Vec<i64>,
        ranges: Vec<(usize, usize)>,
        start_line: usize,
        end_line: usize,
    ) -> ChunkData {
        ChunkData {
            chunk_id: id.to_string(),
            entity_name: String::new(),
            file_path: "src/lib.rs".to_string(),
            start_line,
            end_line,
            source_ranges: ranges
                .into_iter()
                .map(|(s, e)| crate::bench_data::ChunkSourceRange {
                    start_line: s,
                    end_line: e,
                })
                .collect(),
            source_span_kind: String::new(),
            test_info: cce_types::TestInfo::unknown(),
            language: None,
            entity_ids,
            segment_id: String::new(),
        }
    }

    fn code_chunk(id: &str, entity_id: i64) -> ChunkData {
        chunk(id, "src/lib.rs", "fn", Some(entity_id), "group_1")
    }

    fn doc_chunk(id: &str, segment: &str) -> ChunkData {
        chunk(id, "README.md", "", None, segment)
    }

    fn ranked<'a>(chunks: &'a [ChunkData], scores: &[f64]) -> RankedPath<'a> {
        let ranked: Vec<(usize, f64)> = scores.iter().copied().enumerate().collect();
        RankedPath::new(chunks, ranked)
    }

    #[test]
    fn alignment_key_uses_entity_then_segment_then_chunk() {
        assert_eq!(alignment_key(&code_chunk("c1", 42)), "e:42");
        assert_eq!(alignment_key(&doc_chunk("c2", "doc_sec_1")), "s:doc_sec_1");
        assert_eq!(alignment_key(&chunk("c3", "x", "", None, "")), "c:c3");
    }

    #[test]
    fn dedup_keeps_best_occurrence_per_chunk_index() {
        // Single paths measure the raw chunk ranking: each chunk appears
        // exactly once at its best (highest-scoring) position, regardless of
        // how many entities it contains.
        let ranked = vec![(0, 0.9), (1, 0.8), (2, 0.7), (0, 0.3)];
        let deduped = dedup_ranked(&ranked);
        assert_eq!(deduped, vec![(0, 0.9), (1, 0.8), (2, 0.7)]);
    }

    #[test]
    fn dedup_does_not_expand_multi_entity_chunks() {
        // Regression: multi-entity expansion used to emit one entry per
        // contained entity, duplicating the same chunk index and crowding
        // genuinely relevant chunks out of the top-k. Single paths must keep
        // each chunk once, matching the no-aggregation baseline benchmark. The
        // dedup key is the chunk index, so chunk #0 (however many entities it
        // contains) contributes exactly one entry.
        let ranked = vec![(0, 0.9), (1, 0.7), (0, 0.3)];
        let deduped = dedup_ranked(&ranked);

        assert_eq!(deduped, vec![(0, 0.9), (1, 0.7)]);
    }

    #[test]
    fn document_chunks_align_by_segment_id() {
        let vec_chunks = vec![
            doc_chunk("v_d1", "doc_sec_1"),
            doc_chunk("v_d2", "doc_sec_2"),
        ];
        let bm_chunks = vec![
            doc_chunk("b_d1", "doc_sec_1"),
            doc_chunk("b_d3", "doc_sec_3"),
        ];
        let vector = ranked(&vec_chunks, &[0.9, 0.5]);
        let bm25 = ranked(&bm_chunks, &[0.8, 0.7]);

        let fused = fuse_minmax(&vector, &bm25, 0.5, 0.5, true);
        assert_eq!(fused.len(), 3);
        let sec1 = fused
            .iter()
            .find(|e| e.key == "s:doc_sec_1")
            .expect("doc_sec_1 fused");
        assert!(sec1.vector_score.is_some());
        assert!(sec1.bm25_score.is_some());
        let sec3 = fused
            .iter()
            .find(|e| e.key == "s:doc_sec_3")
            .expect("doc_sec_3 fused");
        assert!(sec3.vector_score.is_none());
        assert!(sec3.bm25_score.is_some());
        assert!(sec3.fused_score < sec1.fused_score);
    }

    #[test]
    fn minmax_fusion_aligns_by_entity_id_via_production_fuse() {
        let vec_chunks = vec![code_chunk("v_a", 10), code_chunk("v_b", 11)];
        let bm_chunks = vec![code_chunk("b_a", 10), code_chunk("b_c", 12)];
        let vector = ranked(&vec_chunks, &[0.9, 0.5]);
        let bm25 = ranked(&bm_chunks, &[0.8, 0.7]);

        let fused = fuse_minmax(&vector, &bm25, 0.5, 0.5, true);
        assert_eq!(fused.len(), 3);
        let alpha = fused
            .iter()
            .find(|e| e.key == "e:10")
            .expect("entity 10 fused");
        assert!(alpha.vector_score.is_some());
        assert!(alpha.bm25_score.is_some());
        let gamma = fused
            .iter()
            .find(|e| e.key == "e:12")
            .expect("entity 12 fused");
        assert!(gamma.vector_score.is_none());
        assert!(gamma.bm25_score.is_some());
        assert!(gamma.fused_score < alpha.fused_score);
    }

    #[test]
    fn fused_coverage_unions_both_paths_and_is_representative_independent() {
        // Same entity on both paths, disjoint chunk ranges: the vector chunk
        // covers lines 1-10, the BM25 chunk covers lines 31-40. The judgment
        // range (lines 35-38) is covered ONLY by the BM25 chunk. Entity-level
        // evaluation must hit regardless of which path's chunk production
        // fusion selects as the representative output chunk.
        let v = chunk_with_ranges("e_emb_0", vec![1], vec![(1, 10)], 1, 10);
        let b = chunk_with_ranges("e_bm25_0", vec![1], vec![(31, 40)], 31, 40);
        let v_chunks = [v];
        let b_chunks = [b];
        let vector = ranked(&v_chunks, &[0.9]);
        let bm25 = ranked(&b_chunks, &[0.8]);

        // vector_weight=0.9: vector contribution dominates -> vector chunk is
        // the representative.
        let fused_v = fuse_minmax(&vector, &bm25, 0.9, 0.1, true);
        // bm25_weight=0.9: BM25 contribution dominates -> BM25 chunk is the
        // representative.
        let fused_b = fuse_minmax(&vector, &bm25, 0.1, 0.9, true);

        let ev = fused_v
            .iter()
            .find(|e| e.key == "e:1")
            .expect("entity 1 fused (vector-dominant)");
        let eb = fused_b
            .iter()
            .find(|e| e.key == "e:1")
            .expect("entity 1 fused (bm25-dominant)");

        // Representatives differ (identity is path-dependent)...
        assert_ne!(ev.chunk.chunk_id, eb.chunk.chunk_id);
        // ...but the entity-level coverage is identical and includes both
        // paths' line ranges.
        assert_eq!(ev.coverage.source_ranges.len(), 2);
        let range_pairs = |c: &ChunkData| -> Vec<(usize, usize)> {
            c.source_ranges
                .iter()
                .map(|r| (r.start_line, r.end_line))
                .collect()
        };
        let mut ev_ranges = range_pairs(&ev.coverage);
        let mut eb_ranges = range_pairs(&eb.coverage);
        ev_ranges.sort();
        eb_ranges.sort();
        assert_eq!(
            ev_ranges, eb_ranges,
            "coverage must be independent of the representative chunk"
        );
        let lines: Vec<(usize, usize)> = ev
            .coverage
            .source_ranges
            .iter()
            .map(|r| (r.start_line, r.end_line))
            .collect();
        assert!(lines.contains(&(1, 10)));
        assert!(lines.contains(&(31, 40)));
        assert_eq!(ev.coverage.start_line, 1);
        assert_eq!(ev.coverage.end_line, 40);
    }

    #[test]
    fn rrf_coverage_unions_both_paths() {
        let v = chunk_with_ranges("e_emb_0", vec![1], vec![(1, 10)], 1, 10);
        let b = chunk_with_ranges("e_bm25_0", vec![1], vec![(31, 40)], 31, 40);
        let v_chunks = [v];
        let b_chunks = [b];
        let vector = ranked(&v_chunks, &[1.0]);
        let bm25 = ranked(&b_chunks, &[1.0]);

        let fused = fuse_rrf(&vector, &bm25, 60.0, true);
        let e = fused.iter().find(|e| e.key == "e:1").expect("entity fused");
        assert_eq!(e.coverage.source_ranges.len(), 2);
        assert_eq!(e.coverage.start_line, 1);
        assert_eq!(e.coverage.end_line, 40);
    }

    #[test]
    fn rrf_fusion_prefers_keys_ranked_high_in_both_paths() {
        let vec_chunks = vec![code_chunk("v_a", 10), code_chunk("v_b", 11)];
        let bm_chunks = vec![code_chunk("b_b", 11), code_chunk("b_a", 10)];
        let vector = ranked(&vec_chunks, &[1.0, 0.5]);
        let bm25 = ranked(&bm_chunks, &[0.9, 0.6]);
        let fused = fuse_rrf(&vector, &bm25, 60.0, true);
        assert_eq!(fused.len(), 2);
        // Both keys rank #1 in one path, #2 in the other; fused score equal.
        assert!((fused[0].fused_score - fused[1].fused_score).abs() < 1e-9);
    }

    #[test]
    fn exclude_single_path_drops_partial_entries() {
        let vec_chunks = vec![code_chunk("v_a", 10)];
        let bm_chunks = vec![code_chunk("b_c", 12)];
        let vector = ranked(&vec_chunks, &[0.9]);
        let bm25 = ranked(&bm_chunks, &[0.7]);
        let fused = fuse_minmax(&vector, &bm25, 0.5, 0.5, false);
        assert!(fused.is_empty());
    }
}
