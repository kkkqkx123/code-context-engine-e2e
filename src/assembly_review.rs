//! Offline assembly-review export for the SPSR-Graph assembler.
//!
//! For every query in the `full_pipeline` benchmark snapshot this job replays
//! embedding recall offline (cosine over the stored chunk/query vectors, no
//! Qdrant, no network) and renders one self-contained markdown file per query
//! under:
//!
//! ```text
//! outputs/scenarios/{lang}/assembly/{project}/
//! ├── index.md
//! └── {query_id}.md
//! ```
//!
//! Each `{query_id}.md` holds the plain top-K recall hits (control group)
//! side by side with the same hits passed through
//! `SPSRGraphAssembler::assemble_single` (assembly enabled), plus the
//! `AssemblyMetadata` so reviewers can tell "no effect" apart from
//! "assembly switched off". When expansion is enabled, one call-graph hop
//! (callees first, then callers) is resolved from the benchmark's
//! `call_edges` sidecar and attached to every hit. See
//! `docs/plan/tests/assembly-review-export-design.md`.
//!
//! Only `full_pipeline` snapshots are consumed: the other baselines carry
//! raw source slices with no entity structure and no call edges, so
//! assembly is meaningless on them. The baseline therefore appears nowhere
//! in the output paths.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use cce_orchestrator::query::assembly::{
    AssembledResult, ExpandedUnit, ExpansionOrigin, SPSRGraphAssembler, SPSRGraphConfig,
    SearchResultInput,
};
use cce_types::EntityId;

use crate::bench_data::{BenchmarkData, ChunkData, cosine_similarity, load_benchmark_data};
use crate::bench_gen::fixture_source_path;
use crate::judgments::evaluate::BenchmarkPaths;
use crate::{FixtureSpec, OutputCategory, OutputManager};

/// Which text the assembler consumes for a hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentMode {
    /// Use the benchmark chunk text (`extract_unit_from_content` path).
    /// Fully offline and reproducible.
    Chunk,
    /// Read the real source file under the fixture root and slice the
    /// chunk's absolute line range (same extractor entry point, richer unit).
    Source,
}

impl ContentMode {
    fn label(self) -> &'static str {
        match self {
            Self::Chunk => "chunk",
            Self::Source => "source",
        }
    }
}

/// Which recall path feeds the review. First cut implements embedding only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecallMode {
    Emb,
}

impl RecallMode {
    fn label(self) -> &'static str {
        match self {
            Self::Emb => "emb",
        }
    }
}

/// Benchmark snapshot consumed by the review. Fixed: only `full_pipeline`
/// carries NL-converted chunk text with entity structure; the other
/// baselines hold raw source slices where assembly is meaningless.
const BASELINE: &str = "full_pipeline";

/// One assembly-review export job.
pub struct AssemblyReviewConfig {
    /// Benchmark project name, also the rkyv directory and output fixture
    /// name (e.g. `"once_cell"`, `"flask"`).
    pub project: &'static str,
    /// Language directory under `outputs/scenarios/` (e.g. `"rust"`).
    pub language: &'static str,
    /// Fixture used to resolve real source files in `ContentMode::Source`.
    pub spec: FixtureSpec,
    /// Number of recall hits rendered per query.
    pub top_k: usize,
    pub content_mode: ContentMode,
    pub recall: RecallMode,
    /// Attach one call-graph hop (callees + callers) from the benchmark's
    /// `call_edges` sidecar to every hit. Requires a snapshot generated
    /// with the call-edge sidecar.
    pub expansion: bool,
}

/// Call-graph adjacency over the benchmark entity-id space (`i64` mirrors
/// `ChunkData::entity_ids`).
struct CallGraph {
    /// caller entity id -> edge indices (forward: who calls whom)
    forward: HashMap<i64, Vec<usize>>,
    /// callee entity id -> edge indices (backward: who is called by whom)
    backward: HashMap<i64, Vec<usize>>,
    /// entity id -> embedding chunk index; primary entity of a chunk wins
    chunk_of_entity: HashMap<i64, usize>,
}

impl CallGraph {
    fn build(bench: &BenchmarkData) -> Self {
        let mut forward: HashMap<i64, Vec<usize>> = HashMap::new();
        let mut backward: HashMap<i64, Vec<usize>> = HashMap::new();
        for (idx, edge) in bench.call_edges.iter().enumerate() {
            forward.entry(edge.caller_entity_id).or_default().push(idx);
            backward.entry(edge.callee_entity_id).or_default().push(idx);
        }
        let mut chunk_of_entity: HashMap<i64, usize> = HashMap::new();
        for (idx, chunk) in bench.embedding.chunks.iter().enumerate() {
            if let Some(&primary) = chunk.entity_ids.first() {
                chunk_of_entity.entry(primary).or_insert(idx);
            }
        }
        for (idx, chunk) in bench.embedding.chunks.iter().enumerate() {
            for &id in chunk.entity_ids.iter().skip(1) {
                chunk_of_entity.entry(id).or_insert(idx);
            }
        }
        Self {
            forward,
            backward,
            chunk_of_entity,
        }
    }
}

/// One scored recall hit: embedding chunk index plus cosine score.
struct ScoredHit {
    chunk_idx: usize,
    score: f64,
}

/// Rank every query against the embedding chunks with cosine similarity.
///
/// Mirrors the embedding half of `retrieval_method::rank_recall` without the
/// BM25 side (review needs no BM25 scoring yet).
fn rank_embedding(bench: &BenchmarkData) -> Vec<Vec<ScoredHit>> {
    let dim = bench.embedding.dimension as usize;
    let n_chunks = if dim > 0 {
        bench
            .embedding
            .vectors
            .len()
            .checked_div(dim)
            .unwrap_or(0)
            .min(bench.embedding.chunks.len())
            .min(bench.embedding.texts.len())
    } else {
        0
    };
    (0..bench.queries.len())
        .map(|q_idx| {
            let Some(q_vec) = bench
                .embedding
                .query_vectors
                .get(q_idx * dim..(q_idx + 1) * dim)
                .filter(|_| dim > 0)
            else {
                return Vec::new();
            };
            let mut ranked: Vec<ScoredHit> = (0..n_chunks)
                .map(|c_idx| {
                    let c_vec = &bench.embedding.vectors[c_idx * dim..(c_idx + 1) * dim];
                    ScoredHit {
                        chunk_idx: c_idx,
                        score: cosine_similarity(c_vec, q_vec),
                    }
                })
                .collect();
            ranked.sort_by(|a, b| b.score.total_cmp(&a.score));
            ranked
        })
        .collect()
}

/// Assembler content plus the line window and display path for one hit.
struct HitContent {
    content: String,
    start_line: u32,
    end_line: u32,
    /// Path handed to the assembler. Absolute in source mode so the
    /// file-coverage replacement can read the file; the benchmark-relative
    /// path is always used for display.
    assembler_path: String,
}

/// Resolve the text the assembler consumes for a hit.
fn resolve_hit_content(
    chunk: &ChunkData,
    chunk_text: &str,
    mode: ContentMode,
    fixture_root: &Path,
) -> Result<HitContent, String> {
    match mode {
        ContentMode::Chunk => {
            let line_count = chunk_text.lines().count() as u32;
            if line_count == 0 {
                return Err("chunk text is empty".to_string());
            }
            Ok(HitContent {
                content: chunk_text.to_string(),
                start_line: 1,
                end_line: line_count,
                assembler_path: chunk.file_path.clone(),
            })
        }
        ContentMode::Source => {
            let abs = fixture_root.join(&chunk.file_path);
            let content = std::fs::read_to_string(&abs)
                .map_err(|e| format!("read {}: {e}", abs.display()))?;
            let line_count = content.lines().count() as u32;
            if line_count == 0 {
                return Err(format!("source file {} is empty", chunk.file_path));
            }
            let start = chunk.start_line as u32;
            let end = chunk.end_line as u32;
            if start == 0 || end == 0 || start > end || end > line_count {
                return Err(format!(
                    "chunk range {start}-{end} outside {} ({} lines)",
                    chunk.file_path, line_count
                ));
            }
            Ok(HitContent {
                content,
                start_line: start,
                end_line: end,
                assembler_path: abs.to_string_lossy().replace('\\', "/"),
            })
        }
    }
}

/// Build the assembler input for one hit.
fn build_assembler_input(chunk: &ChunkData, hit: &HitContent, score: f64) -> SearchResultInput {
    SearchResultInput {
        id: chunk.chunk_id.clone(),
        entity_id: chunk.entity_ids.first().map(|&id| EntityId(id as u64)),
        name: chunk.entity_name.clone(),
        kind: String::new(),
        file_path: hit.assembler_path.clone(),
        start_line: hit.start_line,
        end_line: hit.end_line,
        content: hit.content.clone(),
        score: score as f32,
    }
}

/// Resolve one call-graph neighbour as an expansion unit.
///
/// The neighbour is rendered from its own embedding chunk: NL text with
/// 1-based remapped lines in chunk mode, absolute source lines in source
/// mode. Neighbours without a chunk, or already present in the query's
/// top-K hits, are skipped (the assembler dedups and caps the rest).
#[allow(clippy::too_many_arguments)]
fn neighbor_unit(
    bench: &BenchmarkData,
    graph: &CallGraph,
    source_cache: &mut HashMap<String, Option<Vec<String>>>,
    neighbor_id: i64,
    origin: ExpansionOrigin,
    edge_label: &str,
    top_chunk_ids: &HashSet<String>,
    mode: ContentMode,
    fixture_root: &Path,
) -> Option<ExpandedUnit> {
    let chunk_idx = *graph.chunk_of_entity.get(&neighbor_id)?;
    let chunk = &bench.embedding.chunks[chunk_idx];
    if top_chunk_ids.contains(&chunk.chunk_id) {
        return None;
    }
    let (code, start_line, end_line, file_path) = match mode {
        ContentMode::Chunk => {
            let text = bench.embedding.texts.get(chunk_idx).map_or("", |t| t);
            let line_count = text.lines().count() as u32;
            if line_count == 0 {
                return None;
            }
            (text.to_string(), 1, line_count, chunk.file_path.clone())
        }
        ContentMode::Source => {
            let lines = source_cache
                .entry(chunk.file_path.clone())
                .or_insert_with(|| {
                    std::fs::read_to_string(fixture_root.join(&chunk.file_path))
                        .map(|content| content.lines().map(String::from).collect())
                        .ok()
                })
                .as_ref()?;
            let start = chunk.start_line.max(1);
            let end = chunk.end_line.min(lines.len());
            if end < start {
                return None;
            }
            let code = lines[start - 1..end].join("\n");
            let abs = fixture_root
                .join(&chunk.file_path)
                .to_string_lossy()
                .replace('\\', "/");
            (code, start as u32, end as u32, abs)
        }
    };
    let name = display_name(chunk).to_string();
    Some(
        ExpandedUnit::new(code, file_path, start_line, end_line, name)
            .with_entity_id(EntityId(neighbor_id as u64))
            .with_expansion(origin, edge_label),
    )
}

/// Resolve the one-hop forward (callee) and backward (caller) expansion
/// units for one hit chunk.
fn resolve_expansion(
    bench: &BenchmarkData,
    graph: &CallGraph,
    chunk: &ChunkData,
    top_chunk_ids: &HashSet<String>,
    mode: ContentMode,
    fixture_root: &Path,
    source_cache: &mut HashMap<String, Option<Vec<String>>>,
) -> (Vec<ExpandedUnit>, Vec<ExpandedUnit>) {
    let Some(&primary_id) = chunk.entity_ids.first() else {
        return (Vec::new(), Vec::new());
    };
    let mut forward = Vec::new();
    let mut backward = Vec::new();
    if let Some(edges) = graph.forward.get(&primary_id) {
        for &edge_idx in edges {
            let edge = &bench.call_edges[edge_idx];
            if let Some(unit) = neighbor_unit(
                bench,
                graph,
                source_cache,
                edge.callee_entity_id,
                ExpansionOrigin::Forward,
                "calls",
                top_chunk_ids,
                mode,
                fixture_root,
            ) {
                forward.push(unit);
            }
        }
    }
    if let Some(edges) = graph.backward.get(&primary_id) {
        for &edge_idx in edges {
            let edge = &bench.call_edges[edge_idx];
            if let Some(unit) = neighbor_unit(
                bench,
                graph,
                source_cache,
                edge.caller_entity_id,
                ExpansionOrigin::Backward,
                "called by",
                top_chunk_ids,
                mode,
                fixture_root,
            ) {
                backward.push(unit);
            }
        }
    }
    (forward, backward)
}

/// Rendered outcome for one hit in the `assembled/` tree.
enum AssembledOutcome {
    Assembled(AssembledResult),
    Fallback { note: String, raw: String },
}

fn sanitize_query_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn entity_ids_list(chunk: &ChunkData) -> String {
    chunk
        .entity_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn display_name(chunk: &ChunkData) -> &str {
    if chunk.entity_name.is_empty() {
        &chunk.chunk_id
    } else {
        &chunk.entity_name
    }
}

fn render_hit_header(rank: usize, chunk: &ChunkData, score: f64) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "### {}. {}  score={:.4}\n\n",
        rank + 1,
        display_name(chunk),
        score
    ));
    out.push_str(&format!(
        "- file: `{}:{}-{}`\n",
        chunk.file_path, chunk.start_line, chunk.end_line
    ));
    out.push_str(&format!(
        "- entity_ids: [{}], segment_id: {}\n",
        entity_ids_list(chunk),
        if chunk.segment_id.is_empty() {
            "(none)"
        } else {
            &chunk.segment_id
        }
    ));
    out.push('\n');
    out
}

fn render_query_header(
    query_id: &str,
    query_text: &str,
    query_type: &str,
    mode_label: &str,
    content_label: &str,
) -> String {
    format!(
        "# Query: {query_id}\n\n- **Query**: {query_text}\n- **Type**: {query_type}\n- **Mode**: {mode_label}\n- **Content**: {content_label}\n\n## Results\n\n"
    )
}

/// Run the review: offline recall, per-query assembly, markdown export.
pub async fn run_assembly_review(config: AssemblyReviewConfig) -> anyhow::Result<()> {
    let paths = BenchmarkPaths::new(config.project);
    let data_path = paths.data_dir(BASELINE);
    if !data_path.exists() {
        anyhow::bail!(
            "benchmark data not found at {}; run the matching gen_bench example first",
            data_path.display()
        );
    }
    let bench = load_benchmark_data(&data_path).map_err(anyhow::Error::msg)?;
    println!(
        "Loaded {} queries, {} embedding chunks from {}",
        bench.queries.len(),
        bench.embedding.chunks.len(),
        data_path.display()
    );

    // Assembly must be enabled: the default config takes the verbatim
    // shortcut, which would make every assembled file identical to raw.
    let mut spsr_config = SPSRGraphConfig::new().enable(true);
    if config.expansion {
        spsr_config = spsr_config.with_expansion(true);
    }
    let assembler = SPSRGraphAssembler::new(spsr_config);
    let graph = CallGraph::build(&bench);
    if config.expansion && bench.call_edges.is_empty() {
        println!(
            "warning: expansion requested but {BASELINE}/{} snapshot carries no call edges; \
             regenerate the benchmark data with the current gen_bench example",
            config.project
        );
    }
    let ranked = rank_embedding(&bench);
    let fixture_root = fixture_source_path(config.spec.clone());
    let mut source_cache: HashMap<String, Option<Vec<String>>> = HashMap::new();

    let base = OutputManager::builder()
        .category(OutputCategory::Scenarios)
        .language(config.language)
        .scenario(format!("assembly/{}", config.project))
        .build();
    let base_dir = base.ensure_output_dir()?;

    let mode_label = format!("{}_top{}", config.recall.label(), config.top_k);
    let mut index_rows = Vec::new();

    for (q_idx, query) in bench.queries.iter().enumerate() {
        let top: Vec<&ScoredHit> = ranked
            .get(q_idx)
            .map(|hits| hits.iter().take(config.top_k).collect())
            .unwrap_or_default();
        let top_chunk_ids: HashSet<String> = top
            .iter()
            .map(|hit| bench.embedding.chunks[hit.chunk_idx].chunk_id.clone())
            .collect();

        let mut assembled_md = render_query_header(
            &query.id,
            &query.text,
            &query.query_type.to_string(),
            &mode_label,
            config.content_mode.label(),
        );

        for (rank, hit) in top.iter().enumerate() {
            let chunk = &bench.embedding.chunks[hit.chunk_idx];
            let chunk_text = bench.embedding.texts.get(hit.chunk_idx).map_or("", |t| t);

            assembled_md.push_str(&render_hit_header(rank, chunk, hit.score));
            assembled_md.push_str("#### Content (raw)\n\n");
            assembled_md.push_str(chunk_text);
            assembled_md.push_str("\n\n#### Content (assembled)\n\n");

            let outcome =
                match resolve_hit_content(chunk, chunk_text, config.content_mode, &fixture_root) {
                    Ok(resolved) => {
                        let input = build_assembler_input(chunk, &resolved, hit.score);
                        let (forward, backward) = if config.expansion {
                            resolve_expansion(
                                &bench,
                                &graph,
                                chunk,
                                &top_chunk_ids,
                                config.content_mode,
                                &fixture_root,
                                &mut source_cache,
                            )
                        } else {
                            (Vec::new(), Vec::new())
                        };
                        match assembler.assemble_single(input, forward, backward).await {
                            Ok(result) => AssembledOutcome::Assembled(result),
                            Err(e) => AssembledOutcome::Fallback {
                                note: format!("assembly failed ({e}); content identical to raw."),
                                raw: chunk_text.to_string(),
                            },
                        }
                    }
                    Err(note) => AssembledOutcome::Fallback {
                        note: format!("{note}; content identical to raw."),
                        raw: chunk_text.to_string(),
                    },
                };
            match outcome {
                AssembledOutcome::Assembled(result) => {
                    let meta = &result.metadata;
                    assembled_md.push_str(&format!(
                        "- assembly: expanded={}, expanded_nodes={} (fwd={}, bwd={}), files={}, original_length={}, assembled_length={}, truncated={}\n",
                        meta.expanded,
                        meta.expanded_nodes,
                        meta.forward_nodes,
                        meta.backward_nodes,
                        meta.file_count,
                        meta.original_length,
                        meta.assembled_length,
                        meta.truncated
                    ));
                    if !meta.expanded {
                        assembled_md.push_str("> not expanded — content identical to raw.\n");
                    }
                    assembled_md.push('\n');
                    assembled_md.push_str(&result.assembled_content);
                    assembled_md.push_str("\n\n");
                }
                AssembledOutcome::Fallback { note, raw } => {
                    assembled_md.push_str(&format!("> {note}\n\n{raw}\n\n"));
                }
            }
        }

        let file_name = format!("{}.md", sanitize_query_id(&query.id));
        std::fs::write(base_dir.join(&file_name), &assembled_md)?;

        let top1 = top.first().map(|hit| {
            let chunk = &bench.embedding.chunks[hit.chunk_idx];
            format!(
                "{}:{}-{} ({:.4})",
                chunk.file_path, chunk.start_line, chunk.end_line, hit.score
            )
        });
        index_rows.push((query, top1));
    }

    let mut index = format!(
        "# Assembly review: {} ({}, {}, content={}, expansion={})\n\n| query_id | type | query | top-1 hit |\n|---|---|---|---|\n",
        config.project,
        BASELINE,
        mode_label,
        config.content_mode.label(),
        if config.expansion { "on" } else { "off" }
    );
    for (query, top1) in &index_rows {
        index.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            query.id,
            query.query_type,
            query.text.replace('|', "\\|"),
            top1.as_deref().unwrap_or("(no hits)")
        ));
    }
    std::fs::write(base_dir.join("index.md"), &index)?;

    println!(
        "Assembly review written for {} queries -> {}",
        bench.queries.len(),
        base_dir.display()
    );
    Ok(())
}
