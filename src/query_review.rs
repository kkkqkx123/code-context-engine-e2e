//! Full-result visualization export for the real semantic query pipeline.
//!
//! For every judgment query this job indexes the review fixture once with
//! [`QueryWorkflowTest`], runs the production [`QueryCoordinator::search`]
//! path (retrieval, fusion, enrichment, boost, sort, threshold), and renders
//! one self-contained markdown file per query under:
//!
//! ```text
//! outputs/scenarios/{lang}/query_review/{project}/
//! ├── run_manifest.txt
//! ├── index.md
//! ├── {query_id}.md
//! └── aggregated_demo.md
//! ```
//!
//! Each `{query_id}.md` carries the judgment expectation (relevant ranges),
//! the top-K score table, every hit with complete metadata and full content,
//! and a raw-vs-annotated comparison produced by the production
//! [`RelationAnnotator`]. The aggregated demo exercises
//! [`QueryCoordinator::search_aggregated`] with two sub-queries derived from
//! the same judgment set. Outputs are for manual review only and never
//! participate in assertions.
//!
//! A run without a reachable Qdrant degrades to BM25-only and writes to
//! `query_review/{project}_bm25/` instead, so the default directory always
//! carries hybrid output.

use std::sync::Arc;

use anyhow::Context;
use cce_codegraph::index::SnapshotEntityQueryOps;
use cce_llm_client::OpenAICompatibleProvider;
use cce_orchestrator::query::annotation::{
    AnnotatedResult, ExpandedUnit, ExpansionOrigin, RelationAnnotationConfig, RelationAnnotator,
    SearchResultInput,
};
use cce_orchestrator::query::relation_searcher::RelationQueryOptions;
use cce_orchestrator::query::types::ExecutionStrategy;
use cce_orchestrator::query::types::SearchResult;
use cce_orchestrator::{
    AggregatedQueryOptions, QueryCoordinator, QueryResult, SearchConfig, SearchSources, SubQuery,
};

use crate::bench_data::{RelevanceJudgment, RelevanceLevel};
use crate::embedding::EmbeddingConfig;
use crate::fixture::{FixtureSpec, TestFixture};
use crate::mock_embedding_server::{
    MOCK_EMBEDDING_DIMENSION, MockEmbeddingServer, mock_embedding_config,
};
use crate::output_manager::{OutputBuilder, OutputCategory};
use crate::query_test::QueryWorkflowTest;

/// One query-review export job over a single fixture and judgment set.
pub struct QueryReviewConfig {
    /// Benchmark project name, also the output directory name.
    pub project: &'static str,
    /// Language directory under `outputs/scenarios/`.
    pub language: &'static str,
    /// Fixture holding the indexed corpus.
    pub spec: FixtureSpec,
    /// Query set with human-judged expectations.
    pub judgments: Vec<RelevanceJudgment>,
    /// Number of hits rendered per query.
    pub top_k: usize,
    /// Optional output directory suffix for single-source comparison runs.
    pub scenario_suffix: Option<&'static str>,
}

/// Rendered per-query outcome kept for the index page.
struct QueryOutcome {
    id: String,
    query_type: String,
    total: usize,
    elapsed_ms: u64,
    top_file: String,
    top_score: f32,
}

/// Run the full query-review export.
pub async fn run_query_review(config: QueryReviewConfig) -> anyhow::Result<()> {
    // Hybrid requires a reachable Qdrant; without it the run degrades to
    // BM25-only and must not pose as the default hybrid output, so an unset
    // suffix becomes the explicit bm25 marker.
    let hybrid_available = probe_qdrant().await.is_ok();
    let effective_suffix = match config.scenario_suffix {
        Some(suffix) => Some(suffix),
        None if hybrid_available => None,
        None => Some("bm25"),
    };
    let scenario = match effective_suffix {
        Some(suffix) => format!("query_review/{}_{}", config.project, suffix),
        None => format!("query_review/{}", config.project),
    };
    let manager = OutputBuilder::new()
        .category(OutputCategory::Scenarios)
        .language(config.language)
        .scenario(&scenario)
        .build();
    manager.clean().ok();

    let server = MockEmbeddingServer::start().await;
    let embedder = Arc::new(
        OpenAICompatibleProvider::from_model(&mock_embedding_config(&server.base_url), "mock")
            .context("mock embedder")?,
    );

    let sources = if hybrid_available {
        SearchSources::default()
    } else {
        SearchSources::none().with_bm25()
    };

    let mut search_config = SearchConfig::default();
    search_config.result.limit = config.top_k;

    let fixture = TestFixture::load(config.spec.clone()).context("load review fixture")?;
    let mut query_test = QueryWorkflowTest::new(fixture, EmbeddingConfig::mock())
        .with_embedder(embedder)
        .with_sources(sources)
        .with_config(search_config.clone());
    let index_result = query_test.index().await.context("index fixture")?.clone();

    let annotator = RelationAnnotator::new(
        RelationAnnotationConfig::new()
            .enable(true)
            .with_expansion(true)
            .with_max_expanded_units(6)
            .with_caller_expansion(true),
    );

    let mut outcomes = Vec::new();
    for judgment in &config.judgments {
        let result = query_test
            .query(&judgment.query_text)
            .await
            .with_context(|| format!("query {}", judgment.id))?
            .clone();
        let fixture_root = query_test.fixture().root_path().to_path_buf();
        let page = render_query_page(
            judgment,
            &result,
            sources,
            &search_config,
            &annotator,
            query_test.coordinator(),
            &fixture_root,
        )
        .await;
        manager.write(&format!("{}.md", judgment.id), &page)?;
        outcomes.push(QueryOutcome {
            id: judgment.id.clone(),
            query_type: judgment.query_type.to_string(),
            total: result.total,
            elapsed_ms: result.elapsed_ms,
            top_file: result
                .items
                .first()
                .map(|item| format!("{}:{}-{}", item.file_path, item.start_line, item.end_line))
                .unwrap_or_else(|| "(none)".to_string()),
            top_score: result.items.first().map(|item| item.score).unwrap_or(0.0),
        });
    }

    let aggregated_note =
        render_aggregated_demo(&mut query_test, &config, sources, &search_config).await?;
    if let Some(page) = aggregated_note {
        manager.write("aggregated_demo.md", &page)?;
    }

    manager.write(
        "run_manifest.txt",
        &render_manifest(
            &config,
            sources,
            hybrid_available,
            &search_config,
            &index_result,
        ),
    )?;
    manager.write("index.md", &render_index(&config, &outcomes))?;

    println!("Query review written to {}", manager.output_dir().display());
    query_test.cleanup().await;
    server.stop().await;
    Ok(())
}

/// Demonstrate the aggregated path with two sub-queries from the same set.
async fn render_aggregated_demo<F>(
    query_test: &mut QueryWorkflowTest<F>,
    config: &QueryReviewConfig,
    sources: SearchSources,
    search_config: &SearchConfig,
) -> anyhow::Result<Option<String>>
where
    F: crate::fixture::FixtureAccess,
{
    if config.judgments.len() < 2 {
        return Ok(None);
    }
    let first = &config.judgments[0];
    let second = &config.judgments[1];
    let agg = AggregatedQueryOptions {
        original_query: format!("{} + {}", first.query_text, second.query_text),
        project_id: 1,
        sub_queries: vec![
            SubQuery {
                text: first.query_text.clone(),
                sources,
                weight: 1.0,
            },
            SubQuery {
                text: second.query_text.clone(),
                sources,
                weight: 0.7,
            },
        ],
        global_config: search_config.clone(),
        filters: None,
        exclude_patterns: Vec::new(),
        include_patterns: Vec::new(),
        enable_rerank: None,
    };
    let result = query_test
        .coordinator()
        .search_aggregated(&agg)
        .await
        .context("aggregated search")?;

    let mut page = String::from("# Aggregated Search Demo\n\n");
    page.push_str(&format!(
        "- Sub-query 1 (weight 1.0): {} `{}`\n",
        first.id, first.query_text
    ));
    page.push_str(&format!(
        "- Sub-query 2 (weight 0.7): {} `{}`\n",
        second.id, second.query_text
    ));
    page.push_str(&format!(
        "- Merged total: {} | elapsed: {}ms | sources: {} | failed: {:?}\n\n",
        result.total,
        result.elapsed_ms,
        result.sources.join(","),
        result.failed_sub_queries,
    ));
    page.push_str(&render_hit_table(&result));
    for (rank, item) in result.items.iter().enumerate() {
        page.push_str(&render_hit_detail(
            rank,
            item,
            AnnotationOutcome::Skipped("aggregated demo omits annotation comparison"),
        ));
    }
    Ok(Some(page))
}

/// Render one self-contained per-query markdown page.
async fn render_query_page(
    judgment: &RelevanceJudgment,
    result: &QueryResult,
    sources: SearchSources,
    search_config: &SearchConfig,
    annotator: &RelationAnnotator,
    coordinator: &QueryCoordinator,
    fixture_root: &std::path::Path,
) -> String {
    let strategy = ExecutionStrategy::from_sources(&sources, search_config);
    let mut page = format!("# Query: {}\n\n", judgment.id);
    page.push_str(&format!("- Text: `{}`\n", judgment.query_text));
    page.push_str(&format!("- Type: {}\n", judgment.query_type));
    if let Some(subtype) = &judgment.fuzzy_subtype {
        page.push_str(&format!("- Fuzzy subtype: {subtype}\n"));
    }
    page.push_str(&format!("- Request sources: {sources}\n"));
    page.push_str(&format!("- Execution strategy: {strategy}\n"));
    page.push_str("- Intent: default hybrid (no per-query override)\n");
    page.push_str(&format!(
        "- Fusion: vector_weight={:.2} bm25_weight={:.2} algorithm={:?}\n",
        search_config.fusion.vector_weight,
        search_config.fusion.bm25_weight,
        search_config.fusion.algorithm,
    ));
    page.push_str(&format!(
        "- Total: {} | elapsed: {}ms | result sources: {} | cache: {}\n",
        result.total,
        result.elapsed_ms,
        result.sources.join(","),
        if result.from_cache { "hit" } else { "miss" },
    ));
    page.push_str(&format!(
        "- Config: limit={} min_score={:.2} vector_top_k={} vector_min_score={:.2} bm25_min_score={:.2} timeout_ms={}\n",
        search_config.result.limit,
        search_config.result.min_score,
        search_config.vector.top_k,
        search_config.vector.min_score,
        search_config.bm25.min_score,
        search_config.timeout_ms,
    ));
    if !result.failed_sub_queries.is_empty() {
        page.push_str(&format!(
            "- Failed sub-queries: {:?}\n",
            result.failed_sub_queries
        ));
    }

    page.push_str("\n## Expected ranges\n\n");
    if judgment.relevant_ranges.is_empty() {
        page.push_str("(no judged ranges)\n");
    } else {
        for (range, level) in &judgment.relevant_ranges {
            let marker = match level {
                RelevanceLevel::Strong => "strong",
                RelevanceLevel::Related => "related",
                RelevanceLevel::Irrelevant => "irrelevant",
            };
            page.push_str(&format!(
                "- `{}`:{}-{} ({marker})\n",
                range.file, range.start_line, range.end_line
            ));
        }
    }

    page.push_str("\n## Hits (top-K)\n\n");
    page.push_str(&render_hit_table(result));

    for (rank, item) in result.items.iter().enumerate() {
        let annotation = annotate_hit(item, coordinator, fixture_root, annotator).await;
        page.push_str(&render_hit_detail(rank, item, annotation));
    }
    page
}

/// Annotation outcome for one hit, kept distinct so disabled expansion and
/// genuine failures read differently in review output.
enum AnnotationOutcome {
    Annotated(Box<AnnotatedResult>),
    Skipped(&'static str),
    Failed,
}

/// Annotate one hit with real relation expansion.
///
/// The primary unit uses the identity range over the enriched body (matching
/// production), while forward/backward units resolve the hit's first entity
/// through the relation searcher and read snippets from the fixture sources.
async fn annotate_hit(
    item: &SearchResult,
    coordinator: &QueryCoordinator,
    fixture_root: &std::path::Path,
    annotator: &RelationAnnotator,
) -> AnnotationOutcome {
    if item.content_state.is_reference() {
        return AnnotationOutcome::Skipped("reference content carries no annotatable body");
    }
    if item.content.trim().is_empty() {
        return AnnotationOutcome::Skipped("empty body carries no annotatable unit");
    }
    if item.entity_ids.is_empty() {
        return AnnotationOutcome::Skipped("hit carries no entity for expansion lookup");
    };
    let (forward, backward) = expansion_units(
        coordinator,
        fixture_root,
        &item.file_path,
        item.start_line,
        item.end_line,
    );
    match annotator
        .annotate_single(search_result_input(item), forward, backward)
        .await
    {
        Ok(annotated) => AnnotationOutcome::Annotated(Box::new(annotated)),
        Err(_) => AnnotationOutcome::Failed,
    }
}

/// Resolve up to three call-domain callees and three call-domain callers
/// into annotator units.
///
/// Seeds resolve through the snapshot line-range lookup (the relation index
/// ID space), not the hit's retrieval entity IDs (the SQLite row space), so
/// method groups represented by their impl block still surface member call
/// edges instead of reading empty. Hit line numbers are one-based
/// presentation values while the snapshot index compares zero-based
/// tree-sitter rows, so the range is translated back before the lookup.
///
/// Both directions read the production paginated path with a call-domain
/// filter (`relation_domains=["call"]`, `include_external=false`), so
/// structural edges (impl association, trait bound, inheritance) and
/// dependency edges (import/use) are never rendered as calls. The producer
/// returns the edge behind each caller, so the labels `calls` / `called by`
/// carry real call semantics and the units carry the real relation type.
fn expansion_units(
    coordinator: &QueryCoordinator,
    fixture_root: &std::path::Path,
    file_path: &str,
    start_line: u32,
    end_line: u32,
) -> (Vec<ExpandedUnit>, Vec<ExpandedUnit>) {
    const PER_DIRECTION_LIMIT: usize = 3;
    let snapshot = coordinator.relation_searcher().query().index();
    let options = RelationQueryOptions {
        relation_domains: vec!["call".to_string()],
        include_external: false,
        limit: PER_DIRECTION_LIMIT,
        ..RelationQueryOptions::default()
    };

    // Snapshot spans are zero-based rows; hit lines are one-based.
    let first_row = start_line.min(end_line).saturating_sub(1) as usize;
    let last_row = start_line.max(end_line).saturating_sub(1) as usize;
    let (first_row, last_row) = (first_row.min(last_row), first_row.max(last_row));
    let mut seeds = snapshot.get_entities_in_line_range(file_path, first_row, last_row);
    seeds.sort_by_key(|id| id.0);
    seeds.truncate(4);
    let mut forward = Vec::new();
    let mut seen_forward = std::collections::HashSet::new();
    for entity_id in &seeds {
        if forward.len() >= PER_DIRECTION_LIMIT {
            break;
        }
        for relation in coordinator
            .get_callees_paginated(*entity_id, &options)
            .unwrap_or_default()
        {
            let Some(callee) = relation.callee_id else {
                continue;
            };
            if forward.len() >= PER_DIRECTION_LIMIT || !seen_forward.insert(callee.0) {
                continue;
            }
            if let Some(unit) = snippet_unit(
                snapshot,
                fixture_root,
                callee,
                ExpansionOrigin::Forward,
                "calls",
                &relation,
            ) {
                forward.push(unit);
            }
        }
    }

    let mut backward = Vec::new();
    let mut seen_backward = std::collections::HashSet::new();
    for entity_id in &seeds {
        if backward.len() >= PER_DIRECTION_LIMIT {
            break;
        }
        for relation in coordinator
            .get_callers_paginated(*entity_id, &options)
            .unwrap_or_default()
        {
            let caller = relation.caller;
            if backward.len() >= PER_DIRECTION_LIMIT || !seen_backward.insert(caller.0) {
                continue;
            }
            if let Some(unit) = snippet_unit(
                snapshot,
                fixture_root,
                caller,
                ExpansionOrigin::Backward,
                "called by",
                &relation,
            ) {
                backward.push(unit);
            }
        }
    }
    (forward, backward)
}

/// Read one entity's source snippet from the fixture tree as an annotator unit,
/// tagged with the edge that produced it.
fn snippet_unit(
    snapshot: &cce_codegraph::index::LayeredSnapshotIndex,
    fixture_root: &std::path::Path,
    entity_id: cce_types::EntityId,
    origin: ExpansionOrigin,
    edge_label: &str,
    relation: &cce_types::ResolvedRelation,
) -> Option<ExpandedUnit> {
    let entity = snapshot.get_function_by_entity_id(entity_id)?;
    let file_path = snapshot.get_file_path_by_entity(entity_id)?;
    let text = std::fs::read_to_string(fixture_root.join(&file_path)).ok()?;
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let start = (entity.span.start_position.row + 1).max(1);
    let end = (entity.span.end_position.row + 1).max(start);
    let end = end.min(lines.len());
    let start = start.min(end);
    let code = lines[start - 1..end].join("\n");
    if code.trim().is_empty() {
        return None;
    }
    Some(
        ExpandedUnit::new(
            code,
            file_path,
            start as u32,
            end as u32,
            entity.name.clone(),
        )
        .with_expansion(origin, edge_label)
        .with_entity_id(entity_id)
        .with_relation_type(relation.relation_type)
        .with_external(relation.is_external)
        .with_stdlib(relation.is_stdlib()),
    )
}

/// Compact score table over the returned hits.
fn render_hit_table(result: &QueryResult) -> String {
    if result.items.is_empty() {
        return String::from("(no hits)\n");
    }
    let mut table = String::from(
        "| rank | score | original | vector | bm25 | sources | boosted | boost reason | key | file | entity |\n|---:|---:|---:|---:|---:|---|---|---|---|---|---|\n",
    );
    for (rank, item) in result.items.iter().enumerate() {
        let bm25 = item
            .bm25_score
            .map(|score| format!("{score:.4}"))
            .unwrap_or_else(|| "-".to_string());
        table.push_str(&format!(
            "| {} | {:.4} | {:.4} | {:.4} | {} | {} | {} | {} | {} | {}:{}-{} | {} {} |\n",
            rank + 1,
            item.score,
            item.original_score,
            item.vector_score,
            bm25,
            item.sources.join("+"),
            if item.is_boosted { "yes" } else { "no" },
            cell(item.boost_reason.as_deref().unwrap_or("-")),
            cell(&alignment_key_of(item)),
            cell(&item.file_path),
            item.start_line,
            item.end_line,
            cell(&item.kind),
            cell(&item.name),
        ));
    }
    table.push('\n');
    table
}

/// Complete metadata plus full content and the annotation comparison for one hit.
fn render_hit_detail(rank: usize, item: &SearchResult, annotation: AnnotationOutcome) -> String {
    let mut out = format!(
        "### {}. {} `{}`\n\n",
        rank + 1,
        cell(&item.kind),
        cell(&item.name)
    );
    out.push_str(&format!("- id: `{}`\n", item.id));
    out.push_str(&format!(
        "- file: `{}:{}-{}`\n",
        item.file_path, item.start_line, item.end_line
    ));
    out.push_str(&format!(
        "- entity_ids: [{}] | segment_id: {}\n",
        item.entity_ids
            .iter()
            .map(|id| id.0.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        item.segment_id.as_deref().unwrap_or("(none)"),
    ));
    out.push_str(&format!("- alignment key: `{}`\n", alignment_key_of(item)));
    out.push_str(&format!(
        "- score: {:.4} | original: {:.4} | vector: {:.4} | bm25: {} | sources: {}\n",
        item.score,
        item.original_score,
        item.vector_score,
        item.bm25_score
            .map(|score| format!("{score:.4}"))
            .unwrap_or_else(|| "-".to_string()),
        item.sources.join("+"),
    ));
    out.push_str(&format!(
        "- boosted: {} | reason: {}\n",
        item.is_boosted,
        item.boost_reason.as_deref().unwrap_or("-"),
    ));
    out.push_str(&format!(
        "- content_state: {:?} | truncated: {} | category: {}\n",
        item.content_state,
        item.truncated,
        item.category.as_deref().unwrap_or("-"),
    ));
    if !item.metadata.is_empty() {
        let mut entries: Vec<String> = item
            .metadata
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        entries.sort();
        out.push_str(&format!("- metadata: {}\n", entries.join(", ")));
    }
    if let Some(pattern) = &item.pattern_info {
        out.push_str(&format!("- pattern_info: `{}`\n", cell(pattern)));
    }

    out.push_str("\n#### Content\n\n```text\n");
    out.push_str(&fenced(&item.content));
    out.push_str("\n```\n");

    match annotation {
        AnnotationOutcome::Annotated(result) => {
            out.push_str("\n#### Annotated content\n\n");
            out.push_str(&format!(
                "- expanded: {} | expanded_nodes: {} (forward {}, backward {}) | files: {} | truncated: {}\n",
                result.metadata.expanded,
                result.metadata.expanded_nodes,
                result.metadata.forward_nodes,
                result.metadata.backward_nodes,
                result.metadata.file_count,
                result.metadata.truncated,
            ));
            out.push_str("```text\n");
            out.push_str(&fenced(&result.annotated_content));
            out.push_str("\n```\n");
        }
        AnnotationOutcome::Skipped(reason) => {
            out.push_str(&format!(
                "\n#### Annotated content\n\n(annotation skipped: {reason}; see raw content above)\n"
            ));
        }
        AnnotationOutcome::Failed => {
            out.push_str("\n#### Annotated content\n\n(annotation failed; see raw content above)\n")
        }
    }
    out.push('\n');
    out
}

/// Manifest describing how the export was produced.
fn render_manifest(
    config: &QueryReviewConfig,
    sources: SearchSources,
    hybrid_available: bool,
    search_config: &SearchConfig,
    index_result: &cce_orchestrator::IndexResult,
) -> String {
    let mut manifest = format!("project: {}\n", config.project);
    manifest.push_str(&format!("language: {}\n", config.language));
    manifest.push_str(&format!("queries: {}\n", config.judgments.len()));
    manifest.push_str(&format!("top_k: {}\n", config.top_k));
    manifest.push_str(&format!("sources: {sources}\n"));
    manifest.push_str(&format!(
        "qdrant_available: {}\n",
        if hybrid_available { "yes" } else { "no" }
    ));
    manifest.push_str("embedder: deterministic mock (no LLM service)\n");
    manifest.push_str("rerank: disabled (request-level override unset)\n");
    manifest.push_str(&format!(
        "config: limit={} min_score={:.2} vector_top_k={} timeout_ms={}\n",
        search_config.result.limit,
        search_config.result.min_score,
        search_config.vector.top_k,
        search_config.timeout_ms,
    ));
    manifest.push_str(&format!(
        "fusion: vector_weight={:.2} bm25_weight={:.2} algorithm={:?}\n",
        search_config.fusion.vector_weight,
        search_config.fusion.bm25_weight,
        search_config.fusion.algorithm,
    ));
    manifest.push_str(&format!(
        "index: files={} entities={} relations={} vectors={}\n",
        index_result.total_files,
        index_result.total_entities,
        index_result.total_relations,
        index_result.total_vectors,
    ));
    manifest.push_str(&format!(
        "outcome: {}\n",
        if index_result.is_success() {
            "success".to_string()
        } else {
            "incomplete".to_string()
        },
    ));
    if index_result.errors().is_empty() {
        manifest.push_str("errors: none\n");
    } else {
        manifest.push_str("errors:\n");
        for error in index_result.errors().iter().take(20) {
            manifest.push_str(&format!("- {error}\n"));
        }
        if index_result.errors().len() > 20 {
            manifest.push_str(&format!(
                "- ... ({} more)\n",
                index_result.errors().len() - 20
            ));
        }
    }
    if hybrid_available {
        manifest.push_str("capability: hybrid default (vector+bm25)\n");
    } else {
        manifest.push_str(
            "capability: bm25-only fallback; semantic (G2), fuzzy (FZ), and cross-language (G4) conclusions are not representative without vectors\n",
        );
    }
    manifest.push_str(&format!(
        "expectations: src/judgments/{}.rs relevant_ranges\n",
        config.project
    ));
    manifest
}

/// Entry page linking every query file with its top-1 hit.
fn render_index(config: &QueryReviewConfig, outcomes: &[QueryOutcome]) -> String {
    let mut page = format!("# Query Review: {}\n\n", config.project);
    page.push_str(&format!("{} queries, top-{}. Expected ranges come from the judgment set; open each file to compare hits against them.\n\n", outcomes.len(), config.top_k));
    page.push_str("| query | type | total | elapsed_ms | top score | top hit |\n|---|---|---:|---:|---:|---|\n");
    for outcome in outcomes {
        page.push_str(&format!(
            "| [{}](./{}.md) | {} | {} | {} | {:.4} | `{}` |\n",
            outcome.id,
            outcome.id,
            outcome.query_type,
            outcome.total,
            outcome.elapsed_ms,
            outcome.top_score,
            cell(&outcome.top_file),
        ));
    }
    if config.judgments.len() >= 2 {
        page.push_str(
            "\n- [aggregated_demo.md](./aggregated_demo.md): two-sub-query merge demo.\n",
        );
    }
    page
}

/// Production alignment key for cross-path comparison.
fn alignment_key_of(item: &SearchResult) -> String {
    cce_types::alignment_key(&item.entity_ids, item.segment_id.as_deref(), &item.id)
        .unwrap_or_else(|| format!("c:{}", item.id))
}

/// Adapt a search hit to the annotator input shape.
///
/// The body is already the exact unit, so a 1-based whole-unit range makes
/// extraction identity (matching production); absolute line metadata stays
/// owned by the result itself.
fn search_result_input(item: &SearchResult) -> SearchResultInput {
    let line_count = item.content.lines().count();
    let end_line = u32::try_from(line_count).unwrap_or(u32::MAX).max(1);
    SearchResultInput {
        id: item.id.clone(),
        entity_id: item.entity_ids.first().copied(),
        name: item.name.clone(),
        kind: item.kind.clone(),
        file_path: item.file_path.clone(),
        start_line: 1,
        end_line,
        content: item.content.clone(),
        score: item.score,
    }
}

/// Keep markdown table cells on one line.
fn cell(text: &str) -> String {
    text.replace('|', "/").replace('\n', " ")
}

/// Keep hit content inside fenced blocks.
fn fenced(text: &str) -> String {
    text.replace("```", "~~~")
}

/// Probe Qdrant availability without touching the index.
async fn probe_qdrant() -> Result<(), String> {
    let mut config = cce_storage_vector_qdrant::QdrantConfig::with_url("http://localhost:6333");
    config.vector_size = MOCK_EMBEDDING_DIMENSION;
    let client =
        cce_storage_vector_qdrant::QdrantClient::new(config, "test").map_err(|e| e.to_string())?;
    client.initialize().await.map_err(|e| e.to_string())?;
    Ok(())
}
