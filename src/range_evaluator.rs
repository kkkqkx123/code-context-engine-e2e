//! Source-range retrieval evaluation.
//!
//! Relevance is determined by source-range overlap. `SourceSpanKind` is a
//! provenance diagnostic only: a fallback group range still represents a
//! source fragment and is relevant when it overlaps a labeled source range.

use std::collections::{HashMap, HashSet};

use crate::bench_data::{
    ChunkData, ChunkSourceRange, RelevanceJudgment, RelevanceLevel, SourceRange,
};

/// All relevance targets matched by one retrieved chunk.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChunkRelevance {
    pub matched_targets: Vec<(usize, RelevanceLevel)>,
}

impl ChunkRelevance {
    pub fn highest_level(&self) -> Option<RelevanceLevel> {
        self.matched_targets.iter().map(|(_, level)| *level).max()
    }
}

/// Return every judgment target represented by this chunk.
pub fn relevance_for_chunk(chunk: &ChunkData, judgment: &RelevanceJudgment) -> ChunkRelevance {
    let chunk_file = normalize_path(&chunk.file_path);
    let normalized_files: Vec<String> = judgment
        .relevant_ranges
        .iter()
        .map(|(range, _)| normalize_path(&range.file))
        .collect();
    relevance_matches(
        &chunk_file,
        &chunk.source_ranges,
        &judgment.relevant_ranges,
        &normalized_files,
    )
}

/// Match a chunk (with pre-normalized file path) against judgment ranges
/// whose file paths are also pre-normalized.
///
/// Evaluators that scan many chunks against one judgment normalize each side
/// once and reuse the result here.
fn relevance_matches(
    chunk_file: &str,
    chunk_ranges: &[ChunkSourceRange],
    ranges: &[(SourceRange, RelevanceLevel)],
    normalized_files: &[String],
) -> ChunkRelevance {
    let mut matched_targets = Vec::new();
    for (index, ((range, level), range_file)) in ranges.iter().zip(normalized_files).enumerate() {
        if chunk_file != range_file {
            continue;
        }
        let overlaps = chunk_ranges.iter().any(|chunk_range| {
            chunk_range.start_line > 0
                && chunk_range.end_line >= chunk_range.start_line
                && chunk_range.start_line <= range.end_line
                && chunk_range.end_line >= range.start_line
        });
        if overlaps {
            matched_targets.push((index, *level));
        }
    }
    ChunkRelevance { matched_targets }
}

/// Compatibility helper for reports that only need a chunk's highest level.
pub fn is_relevant_to_query(
    chunk: &ChunkData,
    judgment: &RelevanceJudgment,
) -> Option<RelevanceLevel> {
    relevance_for_chunk(chunk, judgment).highest_level()
}

fn normalize_path(path: &str) -> String {
    crate::bench_data::normalize_path(path)
}

fn harmonic_mean(precision: f64, recall: f64) -> f64 {
    if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    }
}

#[derive(Debug, Clone)]
pub struct RangeBasedPerQueryScore {
    /// Retrieved chunks whose highest hit is a Strong target.
    pub strong_matches: usize,
    /// Retrieved chunks whose highest hit is a Related (non-Strong) target.
    pub related_matches: usize,
    /// Retrieved chunks containing at least one labeled target.
    pub any_matches: usize,
    /// Unique labeled targets recovered by the top-k chunks.
    pub total_relevant: usize,

    pub precision_strong: f64,
    pub precision_related: f64,
    pub precision_any: f64,
    pub recall_strong: f64,
    pub recall_related: f64,
    pub recall_any: f64,
    pub f1_strong: f64,
    pub f1_related: f64,
    pub f1_any: f64,
    pub f1_score: f64,
}

/// Evaluate source-fragment recovery at a fixed cutoff.
///
/// Precision is unweighted per-chunk. Recall is unweighted per-target and
/// reported separately for the Strong and Related layers plus the combined
/// `any` layer.
pub fn evaluate_query_range_based(
    judgment: &RelevanceJudgment,
    chunks: &[ChunkData],
    scores: &[f64],
    top_k: usize,
) -> RangeBasedPerQueryScore {
    evaluate_query_range_based_cutoffs(judgment, chunks, scores, &[top_k])
        .into_iter()
        .next()
        .expect("a non-empty cutoff list always yields a score")
}

/// Evaluate source-fragment recovery at several top-k cutoffs with a single
/// sort of the dense score vector.
pub fn evaluate_query_range_based_cutoffs(
    judgment: &RelevanceJudgment,
    chunks: &[ChunkData],
    scores: &[f64],
    cutoffs: &[usize],
) -> Vec<RangeBasedPerQueryScore> {
    let mut indices: Vec<usize> = (0..chunks.len()).collect();
    indices.sort_by(|left, right| scores[*right].total_cmp(&scores[*left]));

    evaluate_ranked_chunks_range_based_cutoffs(judgment, chunks, &indices, cutoffs)
}

/// Evaluate source-fragment recovery from chunks that are already ranked.
///
/// This avoids materializing and sorting a dense score vector when a retrieval
/// backend has already returned its ranked top results.
pub fn evaluate_ranked_chunks_range_based(
    judgment: &RelevanceJudgment,
    chunks: &[ChunkData],
    ranked_indices: &[usize],
    top_k: usize,
) -> RangeBasedPerQueryScore {
    evaluate_ranked_chunks_range_based_cutoffs(judgment, chunks, ranked_indices, &[top_k])
        .into_iter()
        .next()
        .expect("a non-empty cutoff list always yields a score")
}

/// Evaluate source-fragment recovery at several top-k cutoffs with a single
/// pass over an already-ranked chunk list.
pub fn evaluate_ranked_chunks_range_based_cutoffs(
    judgment: &RelevanceJudgment,
    chunks: &[ChunkData],
    ranked_indices: &[usize],
    cutoffs: &[usize],
) -> Vec<RangeBasedPerQueryScore> {
    let mut scan = RankedScan::new(judgment, cutoffs, chunks.len());
    let max_cutoff = cutoffs.iter().copied().max().unwrap_or(0);
    for &index in ranked_indices.iter().take(max_cutoff) {
        if let Some(chunk) = chunks.get(index) {
            scan.advance(chunk);
        }
    }
    scan.finish().into_iter().map(|(range, _)| range).collect()
}

/// First-hit and redundant-coverage diagnostics captured at a cutoff during a
/// single-pass ranked scan.
#[derive(Debug, Clone, Copy)]
pub struct CutoffDiagnostics {
    /// Average first-hit rank across the judgment's relevant ranges.
    pub avg_first_hit_rank: Option<f64>,
    /// Number of (chunk, relevant range) overlaps inside the top-k.
    pub redundant_coverage: usize,
}

/// Monotonic counters accumulated over a ranked prefix.
#[derive(Default)]
struct PrefixCounts {
    strong_matches: usize,
    related_matches: usize,
    any_matches: usize,
    covered_targets: HashSet<usize>,
}

impl PrefixCounts {
    fn accumulate(&mut self, relevance: &ChunkRelevance) {
        let Some(highest) = relevance.highest_level() else {
            return;
        };
        for (target_index, _) in &relevance.matched_targets {
            self.covered_targets.insert(*target_index);
        }
        self.any_matches += 1;
        // The three layer counters are disjoint: a chunk whose highest hit is
        // Strong only counts as strong, one whose highest hit is Related
        // counts as related, so `strong + related == any` always holds.
        match highest {
            RelevanceLevel::Strong => self.strong_matches += 1,
            RelevanceLevel::Related => self.related_matches += 1,
            RelevanceLevel::Irrelevant => {}
        }
    }
}

fn finalize_range_score(
    target_levels: &[RelevanceLevel],
    counts: &PrefixCounts,
    top_k: usize,
    total_chunks: usize,
) -> RangeBasedPerQueryScore {
    // Unweighted recall: each relevant target counts once toward total and
    // toward coverage, and the two relevance layers are measured separately.
    let exact_recall = |wanted: RelevanceLevel| -> f64 {
        let total = target_levels
            .iter()
            .filter(|level| **level == wanted)
            .count();
        if total == 0 {
            return 0.0;
        }
        let covered = counts
            .covered_targets
            .iter()
            .filter(|index| {
                target_levels
                    .get(**index)
                    .map(|level| *level == wanted)
                    .unwrap_or(false)
            })
            .count();
        covered as f64 / total as f64
    };
    let any_recall = if target_levels.is_empty() {
        0.0
    } else {
        counts.covered_targets.len() as f64 / target_levels.len() as f64
    };

    let denominator = top_k.min(total_chunks);
    let precision = |matches| {
        if denominator == 0 {
            0.0
        } else {
            matches as f64 / denominator as f64
        }
    };

    let precision_strong = precision(counts.strong_matches);
    let precision_related = precision(counts.related_matches);
    let precision_any = precision(counts.any_matches);
    let recall_strong = exact_recall(RelevanceLevel::Strong);
    let recall_related = exact_recall(RelevanceLevel::Related);
    let recall_any = any_recall;

    RangeBasedPerQueryScore {
        strong_matches: counts.strong_matches,
        related_matches: counts.related_matches,
        any_matches: counts.any_matches,
        total_relevant: counts.covered_targets.len(),
        precision_strong,
        precision_related,
        precision_any,
        recall_strong,
        recall_related,
        recall_any,
        f1_strong: harmonic_mean(precision_strong, recall_strong),
        f1_related: harmonic_mean(precision_related, recall_related),
        f1_any: harmonic_mean(precision_any, recall_any),
        f1_score: harmonic_mean(precision_any, recall_any),
    }
}

/// Single-pass evaluator over an already-ranked chunk list.
///
/// Range-based counters, first-hit positions and redundant-coverage counts
/// all accumulate monotonically as the scan advances, so one pass over the
/// ranked prefix (up to the largest requested cutoff) yields the results for
/// every cutoff. Each `advance` returns the chunk's relevance so callers that
/// build top-k detail rows do not re-evaluate it. Judgment range paths and
/// each chunk's file path are normalized once per scan.
pub struct RankedScan<'j> {
    judgment: &'j RelevanceJudgment,
    target_levels: Vec<RelevanceLevel>,
    normalized_range_files: Vec<String>,
    counts: PrefixCounts,
    first_hit: Vec<Option<usize>>,
    redundant: usize,
    total_chunks: usize,
    requested_cutoffs: Vec<usize>,
    ordered_cutoffs: Vec<usize>,
    snapshots: Vec<(RangeBasedPerQueryScore, CutoffDiagnostics)>,
    next_cutoff: usize,
    position: usize,
}

impl<'j> RankedScan<'j> {
    /// Prepare a scan over a ranked list of `total_chunks` entries.
    ///
    /// `cutoffs` may be unsorted or duplicated; results come back in the
    /// requested order.
    pub fn new(judgment: &'j RelevanceJudgment, cutoffs: &[usize], total_chunks: usize) -> Self {
        let mut ordered_cutoffs: Vec<usize> = cutoffs.to_vec();
        ordered_cutoffs.sort_unstable();
        ordered_cutoffs.dedup();
        Self {
            judgment,
            target_levels: judgment
                .relevant_ranges
                .iter()
                .map(|(_, level)| *level)
                .collect(),
            normalized_range_files: judgment
                .relevant_ranges
                .iter()
                .map(|(range, _)| normalize_path(&range.file))
                .collect(),
            counts: PrefixCounts::default(),
            first_hit: vec![None; judgment.relevant_ranges.len()],
            redundant: 0,
            total_chunks,
            requested_cutoffs: cutoffs.to_vec(),
            ordered_cutoffs,
            snapshots: Vec::new(),
            next_cutoff: 0,
            position: 0,
        }
    }

    /// Advance the scan by one ranked position.
    pub fn advance(&mut self, chunk: &ChunkData) -> ChunkRelevance {
        self.position += 1;
        let chunk_file = normalize_path(&chunk.file_path);
        let relevance = relevance_matches(
            &chunk_file,
            &chunk.source_ranges,
            &self.judgment.relevant_ranges,
            &self.normalized_range_files,
        );
        self.counts.accumulate(&relevance);
        for (target, _) in &relevance.matched_targets {
            if self.first_hit[*target].is_none() {
                self.first_hit[*target] = Some(self.position);
            }
        }
        self.redundant += relevance.matched_targets.len();
        self.snapshot_if_due();
        relevance
    }

    fn snapshot_if_due(&mut self) {
        while self.next_cutoff < self.ordered_cutoffs.len()
            && self.position >= self.ordered_cutoffs[self.next_cutoff]
        {
            let top_k = self.ordered_cutoffs[self.next_cutoff];
            self.snapshots.push(self.snapshot_at(top_k));
            self.next_cutoff += 1;
        }
    }

    fn snapshot_at(&self, top_k: usize) -> (RangeBasedPerQueryScore, CutoffDiagnostics) {
        let range =
            finalize_range_score(&self.target_levels, &self.counts, top_k, self.total_chunks);
        let hits: Vec<f64> = self
            .first_hit
            .iter()
            .flatten()
            .map(|&rank| rank as f64)
            .collect();
        let avg = if hits.is_empty() {
            None
        } else {
            Some(hits.iter().sum::<f64>() / hits.len() as f64)
        };
        (
            range,
            CutoffDiagnostics {
                avg_first_hit_rank: avg,
                redundant_coverage: self.redundant,
            },
        )
    }

    /// Complete the scan and return per-cutoff results in requested order.
    pub fn finish(mut self) -> Vec<(RangeBasedPerQueryScore, CutoffDiagnostics)> {
        while self.next_cutoff < self.ordered_cutoffs.len() {
            let top_k = self.ordered_cutoffs[self.next_cutoff];
            self.snapshots.push(self.snapshot_at(top_k));
            self.next_cutoff += 1;
        }
        let by_cutoff: HashMap<usize, (RangeBasedPerQueryScore, CutoffDiagnostics)> = self
            .ordered_cutoffs
            .iter()
            .copied()
            .zip(self.snapshots)
            .collect();
        self.requested_cutoffs
            .iter()
            .filter_map(|k| by_cutoff.get(k).cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench_data::{ChunkSourceRange, QueryType, SourceRange};

    fn range(
        start_line: usize,
        end_line: usize,
        level: RelevanceLevel,
    ) -> (SourceRange, RelevanceLevel) {
        (
            SourceRange {
                file: "src/lib.rs".to_string(),
                start_line,
                end_line,
            },
            level,
        )
    }

    fn chunk(kind: &str, ranges: Vec<ChunkSourceRange>) -> ChunkData {
        ChunkData {
            chunk_id: "chunk".to_string(),
            entity_name: "entity".to_string(),
            file_path: "src/lib.rs".to_string(),
            start_line: 1,
            end_line: 100,
            source_ranges: ranges,
            source_span_kind: kind.to_string(),
            test_info: cce_types::TestInfo::unknown(),
            language: None,
            entity_ids: Vec::new(),
            segment_id: String::new(),
        }
    }

    #[test]
    fn group_fallback_range_is_a_source_match() {
        let judgment = RelevanceJudgment {
            id: "coverage".to_string(),
            query_text: "coverage".to_string(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![range(5, 10, RelevanceLevel::Strong)],
        };
        let chunk = chunk(
            "group_fallback",
            vec![ChunkSourceRange {
                start_line: 5,
                end_line: 10,
            }],
        );

        assert_eq!(
            is_relevant_to_query(&chunk, &judgment),
            Some(RelevanceLevel::Strong)
        );
    }

    #[test]
    fn all_overlapping_targets_contribute_to_any_recall() {
        let judgment = RelevanceJudgment {
            id: "coverage".to_string(),
            query_text: "coverage".to_string(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![
                range(5, 10, RelevanceLevel::Strong),
                range(30, 35, RelevanceLevel::Related),
            ],
        };
        let chunk = chunk(
            "exact_entities",
            vec![
                ChunkSourceRange {
                    start_line: 5,
                    end_line: 10,
                },
                ChunkSourceRange {
                    start_line: 30,
                    end_line: 35,
                },
            ],
        );

        let relevance = relevance_for_chunk(&chunk, &judgment);
        assert_eq!(relevance.matched_targets.len(), 2);
        let score = evaluate_query_range_based(&judgment, &[chunk], &[1.0], 1);
        assert_eq!(score.total_relevant, 2);
        assert_eq!(score.recall_any, 1.0);
        assert_eq!(score.recall_strong, 1.0);
        assert_eq!(score.recall_related, 1.0);
    }

    #[test]
    fn unavailable_ranges_do_not_use_navigation_span() {
        let judgment = RelevanceJudgment {
            id: "coverage".to_string(),
            query_text: "coverage".to_string(),
            query_type: QueryType::Semantic,
            fuzzy_subtype: None,
            relevant_ranges: vec![range(5, 10, RelevanceLevel::Strong)],
        };
        let chunk = chunk("unavailable", Vec::new());

        assert_eq!(is_relevant_to_query(&chunk, &judgment), None);
    }
}
