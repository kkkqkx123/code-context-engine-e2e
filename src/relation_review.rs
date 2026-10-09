//! Relation-graph visualization export for the real relation pipeline.
//!
//! Seeds are resolved deterministically against the indexed snapshot: every
//! judgment's first strong expectation range contributes its host entity
//! (via line-range lookup) and the caller-supplied hub names contribute
//! `get_function_ids_by_name` matches. For every seed this job runs the
//! production relation and graph queries (`get_callees`, `get_callers`,
//! forward/backward call chains, ego graph) and renders one self-contained
//! markdown file plus a machine-readable graph snapshot under:
//!
//! ```text
//! outputs/scenarios/{lang}/relation_review/{project}/
//! ├── run_manifest.txt
//! ├── index.md
//! ├── entity/{seed_slug}.md
//! └── graph/{seed_slug}.json
//! ```
//!
//! Relation queries need no vector backend, so this export runs fully
//! offline on BM25-only sources. Outputs are for manual review only and
//! never participate in assertions.

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use cce_codegraph::index::SnapshotEntityQueryOps;
use cce_llm_client::OpenAICompatibleProvider;
use cce_orchestrator::SearchSources;
use cce_orchestrator::query::graph::{GraphDirection, SubGraph};
use cce_orchestrator::query::relation_searcher::RelationQueryOptions;
use cce_types::{EntityId, ResolvedRelation};

use crate::bench_data::{RelevanceJudgment, RelevanceLevel};
use crate::embedding::EmbeddingConfig;
use crate::fixture::{FixtureAccess, FixtureSpec, TestFixture};
use crate::mock_embedding_server::{MockEmbeddingServer, mock_embedding_config};
use crate::output_manager::{OutputBuilder, OutputCategory};
use crate::query_test::QueryWorkflowTest;

/// One relation-review export job over a single fixture.
pub struct RelationReviewConfig {
    /// Benchmark project name, also the output directory name.
    pub project: &'static str,
    /// Language directory under `outputs/scenarios/`.
    pub language: &'static str,
    /// Fixture holding the indexed corpus.
    pub spec: FixtureSpec,
    /// Judgment set used to derive seed entities from expectation ranges.
    pub judgments: Vec<RelevanceJudgment>,
    /// Hub entity names resolved via the snapshot function index.
    pub hub_names: Vec<&'static str>,
    /// Maximum number of seeds rendered.
    pub max_seeds: usize,
    /// Traversal depth for call chains and ego graphs.
    pub depth: usize,
    /// Per-direction neighbor limit.
    pub limit: usize,
}

/// A resolved seed entity with its provenance.
struct Seed {
    id: EntityId,
    name: String,
    provenance: String,
}

/// Rendered per-seed outcome kept for the index page.
struct SeedOutcome {
    slug: String,
    name: String,
    provenance: String,
    callees: usize,
    callers: usize,
    forward: usize,
    backward: usize,
    ego_nodes: usize,
    ego_edges: usize,
}

/// Run the full relation-review export.
pub async fn run_relation_review(config: RelationReviewConfig) -> anyhow::Result<()> {
    let manager = OutputBuilder::new()
        .category(OutputCategory::Scenarios)
        .language(config.language)
        .scenario(format!("relation_review/{}", config.project))
        .build();
    manager.clean().ok();

    let server = MockEmbeddingServer::start().await;
    let embedder = Arc::new(
        OpenAICompatibleProvider::from_model(&mock_embedding_config(&server.base_url), "mock")
            .context("mock embedder")?,
    );

    let fixture = TestFixture::load(config.spec.clone()).context("load review fixture")?;
    let mut query_test = QueryWorkflowTest::new(fixture, EmbeddingConfig::mock())
        .with_embedder(embedder)
        .with_sources(SearchSources::none().with_bm25());
    let index_result = query_test.index().await.context("index fixture")?.clone();
    query_test
        .ensure_coordinator()
        .await
        .context("init query coordinator")?;

    let seeds = resolve_seeds(&query_test, &config);
    let relation_options = RelationQueryOptions {
        max_depth: config.depth,
        limit: config.limit,
        ..RelationQueryOptions::default()
    };

    let mut outcomes = Vec::new();
    let mut skipped = Vec::new();
    for seed in seeds {
        let coordinator = query_test.coordinator();
        let seed_name = seed.name.clone();
        let describe = |id: EntityId| describe_entity(coordinator, id);

        let callees = coordinator.get_callees(seed.id).unwrap_or_default();
        let callers = coordinator.get_callers(seed.id).unwrap_or_default();
        let forward = coordinator
            .query_forward(seed.id, &relation_options)
            .unwrap_or_default();
        let backward = coordinator
            .query_backward(seed.id, &relation_options)
            .unwrap_or_default();
        let ego = coordinator
            .graph_ego(seed.id, config.depth, GraphDirection::Both)
            .unwrap_or_default();

        let slug = seed_slug(&seed_name, seed.id);
        let page = render_seed_page(
            &seed, &describe, &callees, &callers, &forward, &backward, &ego,
        );
        manager.write(&format!("entity/{slug}.md"), &page)?;
        manager.write(
            &format!("graph/{slug}.json"),
            &serde_json::to_string_pretty(&SeedGraphJson::from_ego(&seed, &ego))
                .context("serialize seed graph")?,
        )?;
        outcomes.push(SeedOutcome {
            slug,
            name: seed_name,
            provenance: seed.provenance.clone(),
            callees: callees.len(),
            callers: callers.len(),
            forward: forward.len(),
            backward: backward.len(),
            ego_nodes: ego.nodes.len(),
            ego_edges: ego.edges.len(),
        });
    }
    for hub in &config.hub_names {
        if !hub_resolved(&query_test, hub, &outcomes) {
            skipped.push(format!("hub `{hub}`: no entity matched"));
        }
    }

    let export = query_test
        .coordinator()
        .graph_export(500)
        .unwrap_or_default();
    manager.write(
        "run_manifest.txt",
        &render_manifest(
            &config,
            &outcomes,
            &skipped,
            export.nodes.len(),
            export.edges.len(),
            &index_result,
        ),
    )?;
    manager.write("index.md", &render_index(&config, &outcomes, &skipped))?;

    println!(
        "Relation review written to {}",
        manager.output_dir().display()
    );
    query_test.cleanup().await;
    server.stop().await;
    Ok(())
}

/// Resolve seed entities from judgment ranges first, then hub names.
fn resolve_seeds<F>(query_test: &QueryWorkflowTest<F>, config: &RelationReviewConfig) -> Vec<Seed>
where
    F: FixtureAccess,
{
    let mut seeds = Vec::new();
    let mut seen = HashSet::new();

    let snapshot = query_test.coordinator().relation_searcher().query().index();
    for judgment in &config.judgments {
        if seeds.len() >= config.max_seeds {
            break;
        }
        let Some((range, _)) = judgment
            .relevant_ranges
            .iter()
            .find(|(_, level)| *level == RelevanceLevel::Strong)
            .or_else(|| judgment.relevant_ranges.first())
        else {
            continue;
        };
        let mut ids =
            snapshot.get_entities_in_line_range(&range.file, range.start_line, range.end_line);
        ids.sort_by_key(|id| id.0);
        if let Some(id) = ids.into_iter().next() {
            let name = snapshot
                .get_function_by_entity_id(id)
                .map(|entity| entity.name.clone())
                .unwrap_or_else(|| format!("entity-{}", id.0));
            push_seed(
                &mut seen,
                &mut seeds,
                id,
                name,
                format!(
                    "judgment {} range {}:{}-{}",
                    judgment.id, range.file, range.start_line, range.end_line
                ),
            );
        }
    }

    for hub in &config.hub_names {
        if seeds.len() >= config.max_seeds {
            break;
        }
        let mut ids = snapshot.get_function_ids_by_name(hub);
        ids.sort_by_key(|id| id.0);
        for id in ids.into_iter().take(2) {
            let name = snapshot
                .get_function_by_entity_id(id)
                .map(|entity| entity.name.clone())
                .unwrap_or_else(|| hub.to_string());
            push_seed(&mut seen, &mut seeds, id, name, format!("hub `{hub}`"));
        }
    }
    seeds
}

/// Insert a seed unless its entity was already selected.
fn push_seed(
    seen: &mut HashSet<u64>,
    seeds: &mut Vec<Seed>,
    id: EntityId,
    name: String,
    provenance: String,
) {
    if seen.insert(id.0) {
        seeds.push(Seed {
            id,
            name,
            provenance,
        });
    }
}

/// Whether a hub name contributed at least one rendered seed.
fn hub_resolved<F>(query_test: &QueryWorkflowTest<F>, hub: &str, outcomes: &[SeedOutcome]) -> bool
where
    F: FixtureAccess,
{
    let snapshot = query_test.coordinator().relation_searcher().query().index();
    let ids: HashSet<u64> = snapshot
        .get_function_ids_by_name(hub)
        .into_iter()
        .map(|id| id.0)
        .collect();
    outcomes.iter().any(|outcome| {
        outcome.provenance == format!("hub `{hub}`")
            && ids
                .iter()
                .any(|id| outcome.slug.ends_with(&format!("-{id:x}")))
    })
}

/// One-line human description of an entity id for neighbor tables.
fn describe_entity(
    coordinator: &cce_orchestrator::QueryCoordinator,
    id: EntityId,
) -> (String, String, String) {
    let snapshot = coordinator.relation_searcher().query().index();
    match snapshot.get_function_by_entity_id(id) {
        Some(entity) => {
            let file = snapshot
                .get_file_path_by_entity(id)
                .unwrap_or_else(|| "(unknown file)".to_string());
            (
                entity.name,
                format!("{:?}", entity.kind).to_lowercase(),
                format!(
                    "{file}:{}-{}",
                    entity.span.start_position.row + 1,
                    entity.span.end_position.row + 1
                ),
            )
        }
        None => (
            format!("entity-{}", id.0),
            "unknown".to_string(),
            "(unknown)".to_string(),
        ),
    }
}

/// Render one self-contained per-seed markdown page.
fn render_seed_page(
    seed: &Seed,
    describe: &dyn Fn(EntityId) -> (String, String, String),
    callees: &[ResolvedRelation],
    callers: &[EntityId],
    forward: &[cce_codegraph::CallChainNode],
    backward: &[cce_codegraph::CallChainNode],
    ego: &SubGraph,
) -> String {
    let (name, kind, location) = describe(seed.id);
    let mut page = format!("# Seed: {name} (`{kind}`)\n\n");
    page.push_str(&format!("- Entity id: `{}`\n", seed.id.0));
    page.push_str(&format!("- Location: `{location}`\n"));
    page.push_str(&format!("- Provenance: {}\n", seed.provenance));

    page.push_str(&format!("\n## Callees ({})\n\n", callees.len()));
    if callees.is_empty() {
        page.push_str("(none)\n");
    } else {
        page.push_str("| callee | relation | location |\n|---|---|---|\n");
        for relation in callees {
            let (target, _, target_location) =
                relation.callee_id.map(describe).unwrap_or_else(|| {
                    (
                        relation.callee_name.clone(),
                        "external".to_string(),
                        "(external)".to_string(),
                    )
                });
            page.push_str(&format!(
                "| {} | {:?} | `{}` |\n",
                cell(&target),
                relation.relation_type,
                cell(&target_location),
            ));
        }
    }

    page.push_str(&format!("\n## Callers ({})\n\n", callers.len()));
    if callers.is_empty() {
        page.push_str("(none)\n");
    } else {
        page.push_str("| caller | kind | location |\n|---|---|---|\n");
        for caller in callers {
            let (caller_name, caller_kind, caller_location) = describe(*caller);
            page.push_str(&format!(
                "| {} | {} | `{}` |\n",
                cell(&caller_name),
                cell(&caller_kind),
                cell(&caller_location),
            ));
        }
    }

    page.push_str(&format!("\n## Forward chain ({})\n\n", forward.len()));
    page.push_str(&render_chain(forward, describe));
    page.push_str(&format!("\n## Backward chain ({})\n\n", backward.len()));
    page.push_str(&render_chain(backward, describe));

    page.push_str(&format!(
        "\n## Ego graph ({} nodes, {} edges)\n\n",
        ego.nodes.len(),
        ego.edges.len()
    ));
    if ego.nodes.is_empty() {
        page.push_str("(empty)\n");
    } else {
        page.push_str("| node | kind | source |\n|---|---|---|\n");
        for node in &ego.nodes {
            page.push_str(&format!(
                "| {} | {} | `{}` |\n",
                cell(&node.label),
                cell(&node.kind),
                cell(&node_source(node)),
            ));
        }
        page.push_str("\n| from | relation | to |\n|---|---|---|\n");
        for edge in &ego.edges {
            page.push_str(&format!(
                "| {} | {} | {} |\n",
                cell(&edge.source),
                cell(&edge.relation),
                cell(&edge.target),
            ));
        }
        page.push_str("\n```mermaid\n");
        page.push_str(&render_mermaid(ego));
        page.push_str("```\n");
    }
    page
}

/// Indented call-chain listing.
fn render_chain(
    nodes: &[cce_codegraph::CallChainNode],
    describe: &dyn Fn(EntityId) -> (String, String, String),
) -> String {
    if nodes.is_empty() {
        return String::from("(none)\n");
    }
    let mut out = String::new();
    for node in nodes {
        let (name, _, _) = describe(node.function_id);
        out.push_str(&format!(
            "{}- {} `{}` depth={} relation={:?} call_line={}\n",
            "  ".repeat(node.depth.min(8)),
            cell(&name),
            cell(&node.file_path),
            node.depth,
            node.relation_type,
            node.call_line
                .map(|line| line.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ));
    }
    out
}

/// Mermaid flowchart for the ego graph.
fn render_mermaid(ego: &SubGraph) -> String {
    let mut out = String::from("flowchart TD\n");
    for node in &ego.nodes {
        out.push_str(&format!(
            "    {}[\"{}\"]\n",
            mermaid_id(&node.id),
            mermaid_label(&node.label),
        ));
    }
    for edge in &ego.edges {
        out.push_str(&format!(
            "    {} -->|{}| {}\n",
            mermaid_id(&edge.source),
            mermaid_label(&edge.relation),
            mermaid_id(&edge.target),
        ));
    }
    out
}

/// Manifest describing how the export was produced.
fn render_manifest(
    config: &RelationReviewConfig,
    outcomes: &[SeedOutcome],
    skipped: &[String],
    export_nodes: usize,
    export_edges: usize,
    index_result: &cce_orchestrator::IndexResult,
) -> String {
    let mut manifest = format!("project: {}\n", config.project);
    manifest.push_str(&format!("language: {}\n", config.language));
    manifest.push_str(&format!(
        "seeds: {} (max {})\n",
        outcomes.len(),
        config.max_seeds
    ));
    manifest.push_str(&format!(
        "depth: {} | limit: {}\n",
        config.depth, config.limit
    ));
    manifest.push_str(&format!("hubs: {}\n", config.hub_names.join(", ")));
    manifest.push_str("backend: embedded snapshot (no vector service needed)\n");
    manifest.push_str(&format!(
        "index: files={} entities={} relations={}\n",
        index_result.total_files, index_result.total_entities, index_result.total_relations,
    ));
    manifest.push_str(&format!(
        "project_graph: nodes={export_nodes} edges={export_edges} (capped at 500 nodes)\n",
    ));
    if skipped.is_empty() {
        manifest.push_str("skipped: none\n");
    } else {
        manifest.push_str("skipped:\n");
        for entry in skipped {
            manifest.push_str(&format!("- {entry}\n"));
        }
    }
    manifest
}

/// Entry page linking every seed file with degree counts.
fn render_index(
    config: &RelationReviewConfig,
    outcomes: &[SeedOutcome],
    skipped: &[String],
) -> String {
    let mut page = format!("# Relation Review: {}\n\n", config.project);
    page.push_str(&format!(
        "{} seeds (depth {}, limit {}). Open each file for chains and the ego graph.\n\n",
        outcomes.len(),
        config.depth,
        config.limit
    ));
    page.push_str("| seed | provenance | callees | callers | forward | backward | ego nodes/edges |\n|---|---|---:|---:|---:|---:|---|\n");
    for outcome in outcomes {
        page.push_str(&format!(
            "| [{}](./entity/{}.md) | {} | {} | {} | {} | {} | {}/{} |\n",
            cell(&outcome.name),
            outcome.slug,
            cell(&outcome.provenance),
            outcome.callees,
            outcome.callers,
            outcome.forward,
            outcome.backward,
            outcome.ego_nodes,
            outcome.ego_edges,
        ));
    }
    if !skipped.is_empty() {
        page.push_str("\n## Skipped\n\n");
        for entry in skipped {
            page.push_str(&format!("- {entry}\n"));
        }
    }
    page
}

/// Filesystem-safe slug for a seed entity.
fn seed_slug(name: &str, id: EntityId) -> String {
    let safe: String = name
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || char == '-' || char == '_' {
                char
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = safe.trim_matches('_');
    let stem = if trimmed.is_empty() {
        "entity"
    } else {
        trimmed
    };
    format!("{stem}-{:x}", id.0)
}

/// Keep markdown table cells on one line.
fn cell(text: &str) -> String {
    text.replace('|', "/").replace('\n', " ")
}

/// Node source rendering for the ego node table.
fn node_source(node: &cce_orchestrator::query::graph::GraphNode) -> String {
    if node.source_file.is_empty() {
        node.source_location.clone()
    } else if node.source_location.is_empty() {
        node.source_file.clone()
    } else {
        format!("{} {}", node.source_file, node.source_location)
    }
}

/// Mermaid-safe node identifier.
fn mermaid_id(id: &str) -> String {
    let safe: String = id
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char
            } else {
                '_'
            }
        })
        .collect();
    format!("n{safe}")
}

/// Mermaid-safe label text.
fn mermaid_label(label: &str) -> String {
    label
        .replace('"', "'")
        .replace(['<', '>', '|', '{', '}', '[', ']'], " ")
}

/// Machine-readable snapshot of one seed ego graph.
#[derive(serde::Serialize)]
struct SeedGraphJson {
    seed_id: u64,
    seed_name: String,
    nodes: Vec<cce_orchestrator::query::graph::GraphNode>,
    edges: Vec<cce_orchestrator::query::graph::GraphEdge>,
}

impl SeedGraphJson {
    fn from_ego(seed: &Seed, ego: &SubGraph) -> Self {
        Self {
            seed_id: seed.id.0,
            seed_name: seed.name.clone(),
            nodes: ego.nodes.clone(),
            edges: ego.edges.clone(),
        }
    }
}
