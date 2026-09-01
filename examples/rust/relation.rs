//! Generates relation presentation snapshots for relation_demo and relation_diamond
//! fixtures into `outputs/relation/rust/presentation/relation_demo/` and
//! `outputs/relation/rust/presentation/relation_diamond/`.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use cce_orchestrator::{IndexOptions, IndexOrchestrator};
use cce_relation::index::{EntityIndexOps, FileLevelOps, RelationQueryOps};
use cce_relation::{BuildConfigParser, CallChainNode, CallChainQuery};
use cce_types::language::Language;
use cce_types::{EntityId, RelationType, ResolvedRelation};

use cce_e2e_tests::{OutputCategory, OutputManager, TestFixture, init_minimal_logging};

fn relation_target(
    project_root: &Path,
    index: &cce_relation::RelationIndex,
    relation: &ResolvedRelation,
) -> (String, String, String) {
    if let Some(callee_id) = relation.callee_id {
        let display_id = relation
            .callee_symbol
            .as_ref()
            .and_then(|symbol| symbol.entity_id)
            .unwrap_or(callee_id);

        if let Some(entity_ref) = index.get_function_by_entity_id(callee_id) {
            let entity = entity_ref.value();
            let file_path = index
                .get_file_path_by_entity(callee_id)
                .map(|path| display_path(project_root, &path))
                .unwrap_or_default();

            if !file_path.is_empty() {
                return (display_id.to_string(), entity.name.clone(), file_path);
            }

            if let Some(symbol) = relation.callee_symbol.as_ref() {
                return (
                    display_id.to_string(),
                    symbol.name.clone(),
                    display_path(project_root, &symbol.location.file_path),
                );
            }

            return (callee_id.to_string(), entity.name.clone(), file_path);
        }

        if let Some(symbol) = relation.callee_symbol.as_ref() {
            return (
                display_id.to_string(),
                symbol.name.clone(),
                display_path(project_root, &symbol.location.file_path),
            );
        }

        return (
            callee_id.to_string(),
            relation.callee_name.clone(),
            String::new(),
        );
    }

    if let Some(symbol) = relation.callee_symbol.as_ref() {
        let display_id = symbol
            .entity_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| symbol.symbol_id.to_string());
        return (
            display_id,
            symbol.name.clone(),
            display_path(project_root, &symbol.location.file_path),
        );
    }

    (
        String::from("external"),
        relation.callee_name.clone(),
        String::from("external"),
    )
}

fn relation_source(
    project_root: &Path,
    index: &cce_relation::RelationIndex,
    caller_id: EntityId,
) -> (String, String) {
    if let Some(entity_ref) = index.get_function_by_entity_id(caller_id) {
        let entity = entity_ref.value();
        let file_path = index
            .get_file_path_by_entity(caller_id)
            .map(|path| display_path(project_root, &path))
            .unwrap_or_default();
        return (entity.name.clone(), file_path);
    }

    (caller_id.to_string(), String::new())
}

fn display_path(project_root: &Path, path: &str) -> String {
    Path::new(path)
        .strip_prefix(project_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.replace('\\', "/"))
}

fn render_relation_label(relation_type: RelationType) -> &'static str {
    if relation_type.is_call() {
        "entity"
    } else {
        match relation_type {
            RelationType::IncludeLocal
            | RelationType::ImportStandard
            | RelationType::ImportNamed
            | RelationType::ImportDefault
            | RelationType::ImportNamespace
            | RelationType::ImportDynamic
            | RelationType::Use
            | RelationType::Using
            | RelationType::MacroDependency
            | RelationType::ModuleDependency => "file",
            _ => "entity",
        }
    }
}

fn is_file_level_relation(relation_type: RelationType) -> bool {
    matches!(
        relation_type,
        RelationType::IncludeLocal
            | RelationType::ImportStandard
            | RelationType::ImportNamed
            | RelationType::ImportDefault
            | RelationType::ImportNamespace
            | RelationType::ImportDynamic
            | RelationType::Use
            | RelationType::Using
            | RelationType::MacroDependency
            | RelationType::ModuleDependency
    )
}

fn find_entity_id(index: &cce_relation::RelationIndex, name: &str, file_suffix: &str) -> EntityId {
    let mut candidates = index.get_function_ids_by_name(name);
    candidates.sort_by_key(|entity_id| {
        index
            .get_file_path_by_entity(*entity_id)
            .unwrap_or_default()
    });

    candidates
        .into_iter()
        .find(|entity_id| {
            index
                .get_file_path_by_entity(*entity_id)
                .as_deref()
                .is_some_and(|path| path.ends_with(file_suffix))
        })
        .unwrap_or_else(|| {
            let mut candidates = index
                .function_index()
                .iter()
                .map(|entry| {
                    let file_path = index
                        .get_file_path_by_entity(*entry.key())
                        .unwrap_or_default();
                    format!("{} @ {}", entry.value().name, file_path)
                })
                .collect::<Vec<_>>();
            candidates.sort();
            panic!(
                "Failed to find entity `{}` in file suffix `{}`; available entities: {:?}",
                name, file_suffix, candidates
            )
        })
}

fn render_config_scan(_project_root: &Path, parser: &BuildConfigParser) -> String {
    let mut out = String::new();
    out.push_str("# Build Config Scan\n\n");
    out.push_str("Root: relation_demo\n");

    let rust_deps = parser.dependencies_for_language(Language::Rust);
    if rust_deps.is_empty() {
        out.push_str("Dependencies: (none)\n");
        return out;
    }

    let mut deps = rust_deps.into_iter().collect::<Vec<_>>();
    deps.sort_by(|a, b| {
        a.name
            .cmp(&b.name)
            .then_with(|| a.package_type.to_string().cmp(&b.package_type.to_string()))
    });

    let _ = writeln!(out, "\nRust dependencies:");
    for dep in deps {
        let _ = writeln!(out, "- {} ({})", dep.name, dep.package_type);
    }

    out
}

fn render_direct_relations(project_root: &Path, index: &cce_relation::RelationIndex) -> String {
    let mut rows = Vec::new();

    for relation_entry in index.resolved_relation_index().iter() {
        let caller_id = *relation_entry.key();
        if caller_id.0 == 0 {
            continue;
        }
        let (caller_name, caller_file) = relation_source(project_root, index, caller_id);

        for relation in relation_entry.value() {
            if is_file_level_relation(relation.relation_type) {
                continue;
            }
            if relation.callee_id.is_some_and(|callee_id| callee_id.0 == 0) {
                continue;
            }
            let line = relation.span.start_position.row + 1;
            let (callee_id, callee_name, callee_file) =
                relation_target(project_root, index, relation);

            rows.push((
                caller_file.clone(),
                caller_name.clone(),
                callee_file,
                callee_name,
                relation.relation_type,
                line,
                caller_id.to_string(),
                callee_id,
                render_relation_label(relation.relation_type).to_string(),
            ));
        }
    }

    rows.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
            .then_with(|| a.3.cmp(&b.3))
            .then_with(|| a.4.to_string().cmp(&b.4.to_string()))
            .then_with(|| a.5.cmp(&b.5))
            .then_with(|| a.6.cmp(&b.6))
            .then_with(|| a.7.cmp(&b.7))
    });

    let mut out = String::new();
    out.push_str("# Direct Relations\n\n");
    out.push_str(
        "Ordered by caller file, caller name, callee target, relation type, and line.\n\n",
    );

    for (idx, row) in rows.iter().enumerate() {
        let (
            caller_file,
            caller_name,
            callee_file,
            callee_name,
            relation_type,
            line,
            caller_id,
            callee_id,
            level,
        ) = row;
        let target = if callee_file == "external" {
            format!("external::{}", callee_name)
        } else {
            format!("{}::{}", callee_file, callee_name)
        };

        let _ = writeln!(
            out,
            "{:>2}. [{}] {}::{} (id={}, call_site_line={}) -> {} (id={}, level={}, type={}, call_site_line={})",
            idx + 1,
            level,
            caller_file,
            caller_name,
            caller_id,
            line,
            target,
            callee_id,
            level,
            relation_type,
            line
        );
    }

    out
}

fn render_node(project_root: &Path, node: &CallChainNode) -> String {
    let line = node
        .call_line
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string());
    let file_path = if node.file_path.is_empty() {
        String::new()
    } else {
        display_path(project_root, &node.file_path)
    };

    format!(
        "depth={:>2} | id={} | {}::{} | line={} | relation={} ",
        node.depth, node.function_id, file_path, node.function_name, line, node.relation_type
    )
}

fn render_traversal(
    project_root: &Path,
    title: &str,
    start_label: &str,
    nodes: &[CallChainNode],
) -> String {
    let mut sorted_nodes = nodes.to_vec();
    sorted_nodes.sort_by(|a, b| {
        a.depth
            .cmp(&b.depth)
            .then_with(|| a.file_path.cmp(&b.file_path))
            .then_with(|| a.function_name.cmp(&b.function_name))
            .then_with(|| a.function_id.cmp(&b.function_id))
            .then_with(|| a.call_line.cmp(&b.call_line))
    });
    sorted_nodes.retain(|node| node.function_id.0 != 0);

    let mut out = String::new();
    let _ = writeln!(out, "# {}", title);
    let _ = writeln!(out, "Start: {}", start_label);
    let _ = writeln!(out);

    if sorted_nodes.is_empty() {
        out.push_str("(none)\n");
        return out;
    }

    for node in &sorted_nodes {
        let _ = writeln!(out, "{}", render_node(project_root, node));
    }

    out
}

fn render_path(project_root: &Path, title: &str, path: &[CallChainNode]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}", title);
    let _ = writeln!(out);

    for (idx, node) in path.iter().enumerate() {
        let line = node
            .call_line
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string());
        let _ = writeln!(
            out,
            "{:>2}. depth={:>2} | id={} | {}::{} | line={} | relation={}",
            idx + 1,
            node.depth,
            node.function_id,
            display_path(project_root, &node.file_path),
            node.function_name,
            line,
            node.relation_type
        );
    }

    out
}

fn render_optional_path(
    project_root: &Path,
    title: &str,
    path: Option<&[CallChainNode]>,
) -> String {
    match path {
        Some(nodes) => render_path(project_root, title, nodes),
        None => {
            let mut out = String::new();
            let _ = writeln!(out, "# {}", title);
            out.push_str("\nPath not available for the selected start/end pair.\n");
            out
        }
    }
}

fn render_scope_entities(project_root: &Path, index: &cce_relation::RelationIndex) -> String {
    let mut files: Vec<cce_types::FileInfo> = index
        .file_records()
        .read()
        .values()
        .map(|record| record.info.clone())
        .collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut out = String::new();
    out.push_str("# Scope Entities and Relations\n\n");

    for file in files {
        let entities = index.get_entities_by_file(&file.path);
        let mut entity_rows = entities
            .into_iter()
            .map(|(entity_id, entity)| {
                let outgoing = index
                    .get_resolved_relations_by_caller(entity_id)
                    .map(|relations| relations.value().len())
                    .unwrap_or(0);
                let incoming = index.get_callers_by_callee_entity(entity_id).len();
                let signature = if entity.signature.is_empty() {
                    "-".to_string()
                } else {
                    entity.signature.clone()
                };
                (
                    entity_id.to_string(),
                    entity.kind.to_string(),
                    entity.name.clone(),
                    signature,
                    outgoing,
                    incoming,
                )
            })
            .collect::<Vec<_>>();

        entity_rows.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.0.cmp(&b.0)));

        let mut scoped_relations = index.get_resolved_relations_by_file(&file.path);
        scoped_relations.sort_by_key(|a| a.0);

        let _ = writeln!(out, "## {}", display_path(project_root, &file.path));
        let _ = writeln!(
            out,
            "entities: {} | relations: {} | imports: {} | depends_on: {}",
            file.entity_count,
            file.relation_count,
            file.import_count,
            if file.depends_on.is_empty() {
                "(none)".to_string()
            } else {
                file.depends_on
                    .iter()
                    .map(|dep| display_path(project_root, dep))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        );
        out.push('\n');

        out.push_str("Entities:\n");
        if entity_rows.is_empty() {
            out.push_str("- (none)\n");
        } else {
            for (idx, row) in entity_rows.iter().enumerate() {
                let (entity_id, kind, name, signature, outgoing, incoming) = row;
                let _ = writeln!(
                    out,
                    "{:>2}. id={} | kind={} | name={} | signature={} | outgoing={} | incoming={}",
                    idx + 1,
                    entity_id,
                    kind,
                    name,
                    signature,
                    outgoing,
                    incoming
                );
            }
        }

        out.push_str("\nRelations:\n");
        let mut relation_rows = Vec::new();
        for (caller_id, relations) in scoped_relations {
            let (caller_name, caller_file) = relation_source(project_root, index, caller_id);
            for relation in relations {
                if is_file_level_relation(relation.relation_type) {
                    continue;
                }
                if relation.callee_id.is_some_and(|callee_id| callee_id.0 == 0) {
                    continue;
                }
                let line = relation.span.start_position.row + 1;
                let (callee_id, callee_name, callee_file) =
                    relation_target(project_root, index, &relation);
                relation_rows.push((
                    caller_file.clone(),
                    caller_name.clone(),
                    callee_file,
                    callee_name,
                    relation.relation_type,
                    line,
                    caller_id.to_string(),
                    callee_id,
                ));
            }
        }
        relation_rows.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| a.3.cmp(&b.3))
                .then_with(|| a.4.to_string().cmp(&b.4.to_string()))
                .then_with(|| a.5.cmp(&b.5))
                .then_with(|| a.6.cmp(&b.6))
                .then_with(|| a.7.cmp(&b.7))
        });

        if relation_rows.is_empty() {
            out.push_str("- (none)\n");
        } else {
            for (idx, row) in relation_rows.iter().enumerate() {
                let (
                    caller_file,
                    caller_name,
                    callee_file,
                    callee_name,
                    relation_type,
                    line,
                    caller_id,
                    callee_id,
                ) = row;
                let target = if callee_file == "external" {
                    format!("external::{}", callee_name)
                } else {
                    format!("{}::{}", callee_file, callee_name)
                };

                let _ = writeln!(
                    out,
                    "{:>2}. [{}] {}::{} (id={}) -> {} (id={}, type={}, call_site_line={})",
                    idx + 1,
                    render_relation_label(*relation_type),
                    caller_file,
                    caller_name,
                    caller_id,
                    target,
                    callee_id,
                    relation_type,
                    line
                );
            }
        }

        out.push('\n');
    }

    out
}

fn render_file_dependencies(project_root: &Path, index: &cce_relation::RelationIndex) -> String {
    let mut files: Vec<cce_types::FileInfo> = index
        .file_records()
        .read()
        .values()
        .map(|record| record.info.clone())
        .collect();
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let mut out = String::new();
    out.push_str("# File Dependencies\n\n");

    for file in files {
        let mut deps = BTreeSet::new();
        for dep in file.depends_on {
            deps.insert(dep);
        }

        let _ = writeln!(out, "- {}", display_path(project_root, &file.path));
        let _ = writeln!(
            out,
            "  entities: {} | relations: {} | imports: {}",
            file.entity_count, file.relation_count, file.import_count
        );

        if deps.is_empty() {
            let _ = writeln!(out, "  depends_on: (none)");
        } else {
            let _ = writeln!(out, "  depends_on:");
            for dep in deps {
                let _ = writeln!(out, "  - {}", display_path(project_root, &dep));
            }
        }
        let _ = writeln!(out);
    }

    out
}

fn render_diamond_relations(project_root: &Path, index: &cce_relation::RelationIndex) -> String {
    let mut out = String::new();
    out.push_str("# Diamond Relations\n\n");

    let entries: Vec<_> = index.resolved_relation_index().iter().collect();
    if entries.is_empty() {
        out.push_str("(no relations)\n");
        return out;
    }

    for entry in entries {
        let caller_id = *entry.key();
        if caller_id.0 == 0 {
            continue;
        }
        let caller_name = index
            .get_function_by_entity_id(caller_id)
            .map(|e| e.value().name.clone())
            .unwrap_or_default();
        let caller_file = index.get_file_path_by_entity(caller_id).unwrap_or_default();

        for rel in entry.value() {
            if rel.callee_id.is_some_and(|id| id.0 == 0) {
                continue;
            }
            let callee_name = rel
                .callee_symbol
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let callee_file = rel
                .callee_symbol
                .as_ref()
                .map(|s| s.location.file_path.clone())
                .unwrap_or_default();
            let line = rel.span.start_position.row + 1;
            let _ = writeln!(
                out,
                "  {}::{} (line {}) -> {}::{} [{}]",
                display_path(project_root, &caller_file),
                caller_name,
                line,
                display_path(project_root, &callee_file),
                callee_name,
                rel.relation_type,
            );
        }
    }

    out
}

fn render_call_chain(project_root: &Path, title: &str, nodes: &[CallChainNode]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# {}", title);

    if nodes.is_empty() {
        out.push_str("  (no nodes)\n");
        return out;
    }

    let mut sorted = nodes.to_vec();
    sorted.sort_by(|a, b| {
        a.depth
            .cmp(&b.depth)
            .then_with(|| a.function_name.cmp(&b.function_name))
    });
    for node in &sorted {
        if node.function_id.0 == 0 {
            continue;
        }
        let _ = writeln!(
            out,
            "  depth={} | {}::{} | line={}",
            node.depth,
            display_path(project_root, &node.file_path),
            node.function_name,
            node.call_line
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string()),
        );
    }

    out
}

async fn run_relation_demo() {
    let fixture = TestFixture::rust_relation_demo().expect("Failed to load relation_demo fixture");
    let project_root = fixture.root_path().to_path_buf();

    let mut config_parser = BuildConfigParser::new();
    config_parser
        .scan_project(&project_root, 0)
        .expect("Build config scan should succeed");

    let config_scan = render_config_scan(&project_root, &config_parser);

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

    let mut orchestrator = IndexOrchestrator::new(1).expect("failed to create IndexOrchestrator");
    let result = orchestrator
        .execute(options)
        .await
        .expect("Indexing relation demo should succeed");

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available after indexing");

    let call_query = CallChainQuery::from_index(relation_index.clone());

    let main_id = find_entity_id(&relation_index, "main", "src/main.rs");
    let load_user_id = find_entity_id(&relation_index, "load_user", "src/repository.rs");
    let format_user_id = find_entity_id(&relation_index, "format_user", "src/utils.rs");
    let _create_user_id = find_entity_id(&relation_index, "create", "src/model.rs");

    let direct_relations = render_direct_relations(&project_root, &relation_index);

    let main_relations = relation_index
        .get_resolved_relations_by_caller(main_id)
        .expect("Main should have resolved relations");
    let load_and_format_user_relation_id = main_relations
        .iter()
        .find(|relation| relation.callee_name == "load_and_format_user")
        .and_then(|relation| relation.callee_id)
        .expect("Main should resolve load_and_format_user");

    let forward_main = call_query
        .query_forward_by_entity(main_id, 6)
        .expect("Forward traversal from main should succeed");
    let forward_repo = call_query
        .query_forward_by_entity(load_and_format_user_relation_id, 5)
        .expect("Forward traversal from load_and_format_user should succeed");
    let backward_load_user = call_query
        .query_backward_by_entity(load_user_id, 5)
        .expect("Backward traversal from load_user should succeed");
    let path_main_to_load_user = call_query
        .find_call_chain(main_id, load_user_id, 6)
        .expect("Path query from main to load_user should not error");

    let scope_entities = render_scope_entities(&project_root, &relation_index);
    let mut call_chains = String::new();
    call_chains.push_str("# Call Chains\n\n");
    call_chains.push_str(&format!(
        "Start nodes: main={}, load_and_format_user={}, load_user={}, format_user={}\n\n",
        main_id, load_and_format_user_relation_id, load_user_id, format_user_id
    ));
    call_chains.push_str(&render_traversal(
        &project_root,
        "Forward traversal from main",
        "src/main.rs::main",
        &forward_main,
    ));
    call_chains.push('\n');
    call_chains.push_str(&render_traversal(
        &project_root,
        "Forward traversal from load_and_format_user",
        "src/repository.rs::load_and_format_user",
        &forward_repo,
    ));
    call_chains.push('\n');
    call_chains.push_str(&render_traversal(
        &project_root,
        "Backward traversal from repository::load_user",
        "src/repository.rs::load_user",
        &backward_load_user,
    ));
    call_chains.push('\n');
    call_chains.push_str(&render_optional_path(
        &project_root,
        "Path main -> repository::load_user",
        path_main_to_load_user.as_deref(),
    ));

    let file_dependencies = render_file_dependencies(&project_root, &relation_index);

    let output_mgr = OutputManager::builder()
        .category(OutputCategory::Relation)
        .language("rust")
        .scenario("presentation/relation_demo")
        .build();

    output_mgr
        .write("config_scan.txt", &config_scan)
        .expect("Failed to write config scan snapshot");
    output_mgr
        .write("scope_entities.txt", &scope_entities)
        .expect("Failed to write scope entity snapshot");
    output_mgr
        .write("relations.txt", &direct_relations)
        .expect("Failed to write direct relations snapshot");
    output_mgr
        .write("call_chains.txt", &call_chains)
        .expect("Failed to write call chain snapshot");
    output_mgr
        .write("file_dependencies.txt", &file_dependencies)
        .expect("Failed to write file dependency snapshot");

    let mut summary = String::new();
    summary.push_str("# Relation Presentation Output for relation_demo\n\n");
    summary.push_str(
        "This directory contains the relation presentation snapshot for manual review.\n\n",
    );
    summary.push_str(&format!("Files indexed: {}\n", result.total_files));
    summary.push_str(&format!("Entities indexed: {}\n", result.total_entities));
    summary.push_str(&format!(
        "Relations indexed: {}\n\n",
        result.total_relations
    ));
    summary.push_str("Files:\n");
    summary.push_str("- config_scan.txt\n");
    summary.push_str("- scope_entities.txt\n");
    summary.push_str("- relations.txt\n");
    summary.push_str("- call_chains.txt\n");
    summary.push_str("- file_dependencies.txt\n");

    output_mgr
        .write("SUMMARY.txt", &summary)
        .expect("Failed to write summary");
}

async fn run_relation_diamond() {
    let fixture =
        TestFixture::rust_relation_diamond().expect("Failed to load relation_diamond fixture");
    let project_root = fixture.root_path().to_path_buf();

    let mut config_parser = BuildConfigParser::new();
    config_parser
        .scan_project(&project_root, 0)
        .expect("Build config scan should succeed");

    let mut config_scan = String::new();
    config_scan.push_str("# Build Config Scan\n\n");
    let deps = config_parser.dependencies_for_language(Language::Rust);
    config_scan.push_str(&format!("Dependencies: {:?}\n", deps));

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

    let mut orchestrator = IndexOrchestrator::new(1).expect("failed to create IndexOrchestrator");
    let result = orchestrator
        .execute(options)
        .await
        .expect("Indexing relation_diamond should succeed");

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available after indexing");

    let call_query = CallChainQuery::from_index(relation_index.clone());

    let main_id = find_entity_id(&relation_index, "main", "src/main.rs");
    let _save_user_id = find_entity_id(&relation_index, "save_user", "src/repository.rs");
    let _get_user_id = find_entity_id(&relation_index, "get_user", "src/repository.rs");

    let forward_main = call_query
        .query_forward_by_entity(main_id, 10)
        .expect("Forward traversal from main should succeed");

    let output_mgr = OutputManager::builder()
        .category(OutputCategory::Relation)
        .language("rust")
        .scenario("presentation/relation_diamond")
        .build();

    output_mgr
        .write("config_scan.txt", &config_scan)
        .expect("Failed to write config scan snapshot");

    let relations_snapshot = render_diamond_relations(&project_root, &relation_index);
    output_mgr
        .write("relations.txt", &relations_snapshot)
        .expect("Failed to write relations snapshot");

    let chain_snapshot =
        render_call_chain(&project_root, "Forward traversal from main", &forward_main);
    output_mgr
        .write("call_chains.txt", &chain_snapshot)
        .expect("Failed to write call chain snapshot");

    let mut summary = String::new();
    summary.push_str("# Relation Diamond Presentation Output\n\n");
    summary.push_str(&format!("Files indexed: {}\n", result.total_files));
    summary.push_str(&format!("Entities indexed: {}\n", result.total_entities));
    summary.push_str(&format!(
        "Relations indexed: {}\n\n",
        result.total_relations
    ));
    summary.push_str("Files:\n");
    summary.push_str("- config_scan.txt\n");
    summary.push_str("- relations.txt\n");
    summary.push_str("- call_chains.txt\n");
    output_mgr
        .write("SUMMARY.txt", &summary)
        .expect("Failed to write summary");
}

#[tokio::main]
async fn main() {
    init_minimal_logging();

    run_relation_demo().await;
    run_relation_diamond().await;
}
