//! Structured output framework for symbol tables, relations, and type inference.
//!
//! Provides markdown renderers that are used by the `export_rs` example and by
//! standalone type-inference diagnostics. The design reuses tree-sitter parse
//! products (`ParsedFile`) and the `RelationIndex` so that export and relation
//! pipelines share the same underlying data.
//!
//! Output layout (per project, hierarchical):
//!   SUMMARY.md                        — project-level summary with counts
//!   <rel_path>.txt                    — per-file report: symbols + relations + type inference
//!                                       (preserves original directory structure,
//!                                        e.g. `crates/cli/src/decompress.rs.txt`)
//!   <dir>.dir.txt                     — per-directory overview (sibling to the
//!                                       directory, e.g. `crates/cli/src.dir.txt`,
//!                                       `crates/cli.dir.txt`) to avoid clashing with
//!                                       normal files.
//!
//! Inside each per-file report, symbols are grouped by kind domain (struct/enum,
//! impl blocks, functions/methods, variables, etc.) following the same table
//! hierarchy as the previous aggregated `SYMBOL_TABLE.md`. Relations are
//! organized per file — outgoing calls, type relationships, containment,
//! dependencies, and module metadata are colocated with their owning file so the
//! reader does not need to cross-reference a global `RELATIONS.md`. Type
//! inference for the file is appended in the same file.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write as _;
use std::path::Path;

use cce_relation::RelationIndex;
use cce_relation::index::{EntityIndexOps, FileIndexOps, RelationQueryOps};
use cce_types::{Entity, EntityId, EntityKind, ParsedFile};

use crate::output_manager::{OutputCategory, OutputManager};

// ---------------------------------------------------------------------------
// Public data structures (for JSON / programmatic use)
// ---------------------------------------------------------------------------

/// Serializable symbol entry for structured output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolEntry {
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub file_path: String,
    pub visibility: String,
    pub span: String,
    pub parent: Option<String>,
    pub return_type: Option<String>,
    pub parameters: Vec<(String, Option<String>)>,
    pub doc_present: bool,
}

/// Serializable relation entry for structured output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelationEntry {
    pub caller: String,
    pub caller_file: String,
    pub callee: String,
    pub callee_file: Option<String>,
    pub relation_type: String,
    pub span: String,
    pub is_external: bool,
}

/// Project summary for structured output.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectSummary {
    pub project_name: String,
    pub file_count: usize,
    pub entity_count: usize,
    pub relation_count: usize,
    pub type_relation_count: usize,
    pub call_count: usize,
    pub dependency_count: usize,
    pub languages: Vec<String>,
    pub entities_by_kind: BTreeMap<String, usize>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn visibility_of(entity: &Entity) -> String {
    if entity.modifiers.is_empty() {
        "private".to_string()
    } else {
        // Keep the original ordering but join for display
        entity.modifiers.join(" ")
    }
}

fn span_str(entity: &Entity) -> String {
    if let Some((s, e)) = entity.span.line_range_opt() {
        if s == e {
            format!("L{s}")
        } else {
            format!("L{s}-L{e}")
        }
    } else {
        "n/a".to_string()
    }
}

fn span_str_from_span(span: &cce_types::Span) -> String {
    if let Some((s, e)) = span.line_range_opt() {
        if s == e {
            format!("L{s}")
        } else {
            format!("L{s}-L{e}")
        }
    } else {
        "n/a".to_string()
    }
}

fn ordered_files(index: &RelationIndex) -> BTreeMap<String, Vec<(EntityId, Entity)>> {
    let mut map: BTreeMap<String, Vec<(EntityId, Entity)>> = BTreeMap::new();
    for entry in index.function_index().iter() {
        let id = *entry.key();
        let entity = entry.value().clone();
        let file = index
            .get_file_path_by_entity(id)
            .unwrap_or_else(|| "unknown".to_string());
        map.entry(file).or_default().push((id, entity));
    }
    for vec in map.values_mut() {
        vec.sort_by_key(|(_, e)| e.span.start_position.row);
    }
    map
}

#[allow(dead_code)]
fn entity_label(entity: &Entity) -> String {
    if entity.signature.is_empty() {
        entity.name.clone()
    } else {
        entity.signature.clone()
    }
}

// ---------------------------------------------------------------------------
// Symbol table rendering
// ---------------------------------------------------------------------------

/// Render a full symbol table as markdown.
///
/// Groups entities per file and per kind category, showing fields for type
/// definitions and signatures for callables. Visibility and span are derived
/// from the entity metadata so the output can be audited without re-parsing.
pub fn render_symbol_table(index: &RelationIndex, project_name: &str) -> String {
    let mut out = String::new();
    writeln!(out, "# Symbol Table for {project_name}").expect("write");
    writeln!(out).expect("write");
    writeln!(
        out,
        "_Generated from the relation index ({} entities, {} files)_",
        index.function_count(),
        index.file_count()
    )
    .expect("write");
    writeln!(out).expect("write");

    let files = ordered_files(index);
    if files.is_empty() {
        writeln!(out, "(no entities)").expect("write");
        return out;
    }

    // Build parent -> children index for field resolution
    let mut id_to_entity: BTreeMap<EntityId, Entity> = BTreeMap::new();
    for entities in files.values() {
        for (id, e) in entities {
            id_to_entity.insert(*id, e.clone());
        }
    }
    let mut parent_to_children: BTreeMap<EntityId, Vec<Entity>> = BTreeMap::new();
    for entities in files.values() {
        for (_, e) in entities {
            if let Some(pid) = e.parent {
                parent_to_children.entry(pid).or_default().push(e.clone());
            }
        }
    }

    writeln!(out, "## File Overview").expect("write");
    writeln!(out).expect("write");
    writeln!(out, "| File | Entities |").expect("write");
    writeln!(out, "|------|----------|").expect("write");
    for (file, entities) in &files {
        writeln!(out, "| {} | {} |", file, entities.len()).expect("write");
    }
    writeln!(out).expect("write");

    for (file, entities) in &files {
        writeln!(out, "## File: {file}").expect("write");
        writeln!(out).expect("write");

        // Group by kind domain for readable sections
        let mut by_kind: BTreeMap<String, Vec<&Entity>> = BTreeMap::new();
        for (_, e) in entities {
            by_kind.entry(e.kind.to_string()).or_default().push(e);
        }

        // Emit type definitions first (struct/class/enum/trait etc.)
        let type_kinds = [
            "struct",
            "class",
            "enum",
            "trait",
            "interface",
            "type_alias",
            "union",
        ];
        for kind in type_kinds {
            if let Some(list) = by_kind.get(kind) {
                writeln!(out, "### {}s", capitalize(kind)).expect("write");
                writeln!(out).expect("write");
                writeln!(out, "| Name | Signature | Fields | Visibility | Span |").expect("write");
                writeln!(out, "|------|-----------|--------|------------|------|").expect("write");
                for e in list {
                    let fields = parent_to_children
                        .get(&e.id)
                        .map(|children| {
                            children
                                .iter()
                                .filter(|c| {
                                    c.kind == EntityKind::Field
                                        || c.kind == EntityKind::Property
                                        || c.kind == EntityKind::Variable
                                })
                                .map(|c| {
                                    let ty = c
                                        .metadata
                                        .get("field_type")
                                        .or_else(|| c.metadata.get("type_annotation"))
                                        .or_else(|| c.metadata.get("variable_type"))
                                        .cloned()
                                        .unwrap_or_else(|| "-".to_string());
                                    if ty == "-" {
                                        c.name.clone()
                                    } else {
                                        format!("{}: {}", c.name, ty)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    let fields = if fields.is_empty() {
                        "-".to_string()
                    } else {
                        fields
                    };
                    let sig = if e.signature.is_empty() {
                        e.name.clone()
                    } else {
                        e.signature.clone()
                    };
                    writeln!(
                        out,
                        "| {} | {} | {} | {} | {} |",
                        escape_md(&e.name),
                        escape_md(&sig),
                        escape_md(&fields),
                        escape_md(&visibility_of(e)),
                        span_str(e)
                    )
                    .expect("write");
                }
                writeln!(out).expect("write");
            }
        }

        // Inherent / trait impl blocks
        for kind in ["inherent_impl", "trait_impl"] {
            if let Some(list) = by_kind.get(kind) {
                writeln!(out, "### {}s", capitalize(kind)).expect("write");
                writeln!(out).expect("write");
                writeln!(out, "| Name | Signature | Visibility | Span |").expect("write");
                writeln!(out, "|------|-----------|------------|------|").expect("write");
                for e in list {
                    let sig = if e.signature.is_empty() {
                        e.name.clone()
                    } else {
                        e.signature.clone()
                    };
                    writeln!(
                        out,
                        "| {} | {} | {} | {} |",
                        escape_md(&e.name),
                        escape_md(&sig),
                        escape_md(&visibility_of(e)),
                        span_str(e)
                    )
                    .expect("write");
                }
                writeln!(out).expect("write");
            }
        }

        // Callables: function / method / constructor / operator
        let callable_kinds = ["function", "method", "constructor", "operator"];
        let mut has_callable = false;
        for kind in callable_kinds {
            if by_kind.contains_key(kind) {
                has_callable = true;
                break;
            }
        }
        if has_callable {
            writeln!(out, "### Functions / Methods").expect("write");
            writeln!(out).expect("write");
            writeln!(
                out,
                "| Name | Kind | Parameters | Return Type | Visibility | Span |"
            )
            .expect("write");
            writeln!(
                out,
                "|------|------|------------|-------------|------------|------|"
            )
            .expect("write");
            for kind in callable_kinds {
                if let Some(list) = by_kind.get(kind) {
                    for e in list {
                        let params = if e.parameters.is_empty() {
                            "-".to_string()
                        } else {
                            e.parameters
                                .iter()
                                .map(|(n, t)| {
                                    if let Some(ty) = t {
                                        format!("{n}: {ty}")
                                    } else {
                                        n.clone()
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        let ret = e.return_type.as_deref().unwrap_or("-");
                        writeln!(
                            out,
                            "| {} | {} | {} | {} | {} | {} |",
                            escape_md(&e.name),
                            kind,
                            escape_md(&params),
                            escape_md(ret),
                            escape_md(&visibility_of(e)),
                            span_str(e)
                        )
                        .expect("write");
                    }
                }
            }
            writeln!(out).expect("write");
        }

        // Variables / fields / constants that are not part of a parent struct
        // (standalone). For nested fields we already rendered them inline above.
        let var_kinds = ["field", "variable", "constant", "property", "enum_variant"];
        let mut has_vars = false;
        for kind in var_kinds {
            if by_kind.contains_key(kind) {
                has_vars = true;
                break;
            }
        }
        if has_vars {
            writeln!(out, "### Variables / Fields / Constants").expect("write");
            writeln!(out).expect("write");
            writeln!(out, "| Name | Kind | Type | Visibility | Span | Parent |").expect("write");
            writeln!(out, "|------|------|------|------------|------|--------|").expect("write");
            for kind in var_kinds {
                if let Some(list) = by_kind.get(kind) {
                    for e in list {
                        let ty = e
                            .metadata
                            .get("field_type")
                            .or_else(|| e.metadata.get("type_annotation"))
                            .or_else(|| e.metadata.get("variable_type"))
                            .or_else(|| e.metadata.get("literal_type"))
                            .or_else(|| e.metadata.get("constructor_type"))
                            .cloned()
                            .unwrap_or_else(|| "-".to_string());
                        let parent_name = e
                            .parent
                            .and_then(|pid| id_to_entity.get(&pid).map(|p| p.name.clone()))
                            .unwrap_or_else(|| "-".to_string());
                        writeln!(
                            out,
                            "| {} | {} | {} | {} | {} | {} |",
                            escape_md(&e.name),
                            kind,
                            escape_md(&ty),
                            escape_md(&visibility_of(e)),
                            span_str(e),
                            escape_md(&parent_name)
                        )
                        .expect("write");
                    }
                }
            }
            writeln!(out).expect("write");
        }

        // Fallback for any remaining kinds (module, import, macro, etc.)
        let covered: HashSet<&str> = [
            "struct",
            "class",
            "enum",
            "trait",
            "interface",
            "type_alias",
            "union",
            "inherent_impl",
            "trait_impl",
            "function",
            "method",
            "constructor",
            "operator",
            "field",
            "variable",
            "constant",
            "property",
            "enum_variant",
        ]
        .into_iter()
        .collect();
        let mut others: Vec<(&String, &Vec<&Entity>)> = by_kind
            .iter()
            .filter(|(k, _)| !covered.contains(k.as_str()))
            .collect();
        if !others.is_empty() {
            others.sort_by(|a, b| a.0.cmp(b.0));
            writeln!(out, "### Other Entities").expect("write");
            writeln!(out).expect("write");
            writeln!(out, "| Name | Kind | Signature | Visibility | Span |").expect("write");
            writeln!(out, "|------|------|-----------|------------|------|").expect("write");
            for (kind, list) in others {
                for e in list {
                    let sig = if e.signature.is_empty() {
                        e.name.clone()
                    } else {
                        e.signature.clone()
                    };
                    writeln!(
                        out,
                        "| {} | {} | {} | {} | {} |",
                        escape_md(&e.name),
                        escape_md(kind),
                        escape_md(&sig),
                        escape_md(&visibility_of(e)),
                        span_str(e)
                    )
                    .expect("write");
                }
            }
            writeln!(out).expect("write");
        }
    }

    out
}

/// Collect symbol entries for JSON export.
pub fn collect_symbols(index: &RelationIndex) -> Vec<SymbolEntry> {
    let mut entries = Vec::new();
    let files = ordered_files(index);
    let mut id_to_parent: BTreeMap<EntityId, String> = BTreeMap::new();
    for entities in files.values() {
        for (id, e) in entities {
            if let Some(pid) = e.parent {
                // resolve parent name if available
                let parent_name = files
                    .values()
                    .flat_map(|v| v.iter())
                    .find(|(eid, _)| *eid == pid)
                    .map(|(_, pe)| pe.name.clone())
                    .unwrap_or_else(|| pid.0.to_string());
                id_to_parent.insert(*id, parent_name);
            }
        }
    }
    for (file, entities) in files {
        for (id, e) in entities {
            entries.push(SymbolEntry {
                name: e.name.clone(),
                kind: e.kind.to_string(),
                signature: e.signature.clone(),
                file_path: file.clone(),
                visibility: visibility_of(&e),
                span: span_str(&e),
                parent: id_to_parent.get(&id).cloned(),
                return_type: e.return_type.clone(),
                parameters: e.parameters.clone(),
                doc_present: e.doc_comment.is_some(),
            });
        }
    }
    entries.sort_by(|a, b| {
        a.file_path
            .cmp(&b.file_path)
            .then_with(|| a.name.cmp(&b.name))
    });
    entries
}

// ---------------------------------------------------------------------------
// Relations rendering
// ---------------------------------------------------------------------------

/// Render call graph, type relationships and file dependencies as markdown.
pub fn render_relations(index: &RelationIndex, project_name: &str) -> String {
    let mut out = String::new();
    writeln!(out, "# Relations for {project_name}").expect("write");
    writeln!(out).expect("write");
    writeln!(
        out,
        "_Resolved relations: {}, file-level relations: {}, dependencies: {}_",
        index.resolved_relation_count(),
        index
            .file_records()
            .read()
            .keys()
            .filter_map(|k| index.file_relations(k))
            .map(|v| v.len())
            .sum::<usize>(),
        index.dependency_graph.get_all_files().len()
    )
    .expect("write");
    writeln!(out).expect("write");

    // Build entity id -> (name, file) map for quick lookup
    let mut id_to_info: BTreeMap<EntityId, (String, String, String)> = BTreeMap::new();
    for entry in index.function_index().iter() {
        let id = *entry.key();
        let e = entry.value();
        let file = index.get_file_path_by_entity(id).unwrap_or_default();
        id_to_info.insert(id, (e.name.clone(), file, e.kind.to_string()));
    }

    // ---- Direct / method calls ----
    writeln!(out, "## Call Graph").expect("write");
    writeln!(out).expect("write");
    // Separate internal vs external — only call relations belong to the call
    // graph; structural / dependency / reference relations are rendered under
    // Type Relationships (see below). This prevents `type_reference` or
    // `dependency.use` from polluting the internal call count.
    let mut internal_calls: Vec<(String, String, String, String, String, String)> = Vec::new();
    let mut external_calls: Vec<(String, String, String, String)> = Vec::new();
    for entry in index.resolved_relation_index().iter() {
        let caller_id = *entry.key();
        let caller_info = id_to_info.get(&caller_id);
        let caller_name = caller_info
            .map(|(n, _, _)| n.clone())
            .unwrap_or_else(|| format!("<?>:{}", caller_id.0));
        let caller_file = caller_info.map(|(_, f, _)| f.clone()).unwrap_or_default();
        for rel in entry.value().iter() {
            if !rel.relation_type.is_call() {
                continue;
            }
            let callee_file = rel
                .callee_id
                .and_then(|cid| id_to_info.get(&cid).map(|(_, f, _)| f.clone()));
            let callee_name = if let Some(cid) = rel.callee_id {
                id_to_info
                    .get(&cid)
                    .map(|(n, _, _)| n.clone())
                    .unwrap_or_else(|| rel.callee_name.clone())
            } else {
                rel.callee_name.clone()
            };
            let span = span_str_from_span(&rel.span);
            if rel.is_external {
                external_calls.push((
                    caller_name.clone(),
                    callee_name,
                    rel.relation_type.to_string(),
                    span,
                ));
            } else {
                let callee_kind = rel
                    .callee_id
                    .and_then(|cid| id_to_info.get(&cid).map(|(_, _, k)| k.clone()))
                    .unwrap_or_default();
                internal_calls.push((
                    caller_name.clone(),
                    caller_file.clone(),
                    callee_name,
                    callee_file.clone().unwrap_or_default(),
                    rel.relation_type.to_string(),
                    span,
                ));
                let _ = callee_kind;
            }
        }
    }
    // Include file-level relations as well — only calls survive into the call
    // graph; file-level dependencies are shown separately under File Dependencies
    // and Type Relationships.
    for file_path in index
        .file_records()
        .read()
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        if let Some(rels) = index.file_relations(&file_path) {
            let caller_file = file_path.clone();
            for rel in rels.iter() {
                if !rel.relation_type.is_call() {
                    continue;
                }
                let callee_file = rel
                    .callee_id
                    .and_then(|cid| id_to_info.get(&cid).map(|(_, f, _)| f.clone()));
                let callee_name = if let Some(cid) = rel.callee_id {
                    id_to_info
                        .get(&cid)
                        .map(|(n, _, _)| n.clone())
                        .unwrap_or_else(|| rel.callee_name.clone())
                } else {
                    rel.callee_name.clone()
                };
                let span = span_str_from_span(&rel.span);
                if rel.is_external {
                    external_calls.push((
                        caller_file.clone(),
                        callee_name,
                        rel.relation_type.to_string(),
                        span,
                    ));
                } else {
                    internal_calls.push((
                        caller_file.clone(),
                        caller_file.clone(),
                        callee_name,
                        callee_file.clone().unwrap_or_default(),
                        rel.relation_type.to_string(),
                        span,
                    ));
                }
            }
        }
    }
    internal_calls.sort();
    external_calls.sort();

    writeln!(out, "### Internal Calls ({})", internal_calls.len()).expect("write");
    writeln!(out).expect("write");
    if internal_calls.is_empty() {
        writeln!(out, "_No internal calls_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(
            out,
            "| Caller | Caller File | Callee | Callee File | Type | Span |"
        )
        .expect("write");
        writeln!(
            out,
            "|--------|-------------|--------|-------------|------|------|"
        )
        .expect("write");
        for (caller, caller_file, callee, callee_file, ty, span) in &internal_calls {
            writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} |",
                escape_md(caller),
                escape_md(caller_file),
                escape_md(callee),
                escape_md(callee_file),
                escape_md(ty),
                escape_md(span)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    writeln!(out, "### External Calls ({})", external_calls.len()).expect("write");
    writeln!(out).expect("write");
    if external_calls.is_empty() {
        writeln!(out, "_No external calls_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Caller | Callee | Type | Span |").expect("write");
        writeln!(out, "|--------|--------|------|------|").expect("write");
        for (caller, callee, ty, span) in &external_calls {
            writeln!(
                out,
                "| {} | {} | {} | {} |",
                escape_md(caller),
                escape_md(callee),
                escape_md(ty),
                escape_md(span)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    // ---- Type relationships: inheritance, implementation, contains ----
    writeln!(out, "## Type Relationships").expect("write");
    writeln!(out).expect("write");

    // Collect inheritance / implementation edges plus any non-call
    // structural / dependency / reference relations that were previously
    // mixed into the call graph. All `!is_call()` relations belong here.
    let mut hierarchy_rels: Vec<(String, String, String, String)> = Vec::new();
    for entry in index.resolved_relation_index().iter() {
        for rel in entry.value().iter() {
            if rel.relation_type.is_call() {
                continue;
            }
            if rel.relation_type.is_structural()
                || rel.relation_type.is_reference()
                || rel.relation_type.is_dependency()
                || matches!(
                    rel.relation_type,
                    cce_types::RelationType::Inheritance
                        | cce_types::RelationType::Implementation
                        | cce_types::RelationType::TraitBound
                        | cce_types::RelationType::ImplAssociation
                )
            {
                let caller = id_to_info
                    .get(&rel.caller)
                    .map(|(n, _, _)| n.clone())
                    .unwrap_or_else(|| rel.caller.0.to_string());
                let callee = rel
                    .callee_id
                    .and_then(|cid| id_to_info.get(&cid).map(|(n, _, _)| n.clone()))
                    .unwrap_or_else(|| rel.callee_name.clone());
                let caller_file = id_to_info
                    .get(&rel.caller)
                    .map(|(_, f, _)| f.clone())
                    .unwrap_or_default();
                hierarchy_rels.push((caller, callee, rel.relation_type.to_string(), caller_file));
            }
        }
    }
    for file_path in index
        .file_records()
        .read()
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        if let Some(rels) = index.file_relations(&file_path) {
            for rel in rels.iter() {
                if rel.relation_type.is_call() {
                    continue;
                }
                if rel.relation_type.is_structural()
                    || rel.relation_type.is_reference()
                    || rel.relation_type.is_dependency()
                {
                    let callee = rel
                        .callee_id
                        .and_then(|cid| id_to_info.get(&cid).map(|(n, _, _)| n.clone()))
                        .unwrap_or_else(|| rel.callee_name.clone());
                    let caller = file_path.clone();
                    hierarchy_rels.push((
                        caller.clone(),
                        callee,
                        rel.relation_type.to_string(),
                        caller,
                    ));
                }
            }
        }
    }
    hierarchy_rels.sort();
    writeln!(
        out,
        "### Inheritance / Implementation ({})",
        hierarchy_rels.len()
    )
    .expect("write");
    writeln!(out).expect("write");
    if hierarchy_rels.is_empty() {
        writeln!(out, "_No hierarchy relations_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Type | Target | Kind | File |").expect("write");
        writeln!(out, "|------|--------|------|------|").expect("write");
        for (ty, target, kind, file) in &hierarchy_rels {
            writeln!(
                out,
                "| {} | {} | {} | {} |",
                escape_md(ty),
                escape_md(target),
                escape_md(kind),
                escape_md(file)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    // Contains (parent -> children)
    writeln!(out, "### Contains (Parent → Child)").expect("write");
    writeln!(out).expect("write");
    // Parent-child from entity tree
    let mut contains: Vec<(String, String, String, String)> = Vec::new();
    for entry in index.function_index().iter() {
        let e = entry.value();
        if let Some(pid) = e.parent {
            if let Some(parent) = index.function_index().get(&pid) {
                let parent_file = index.get_file_path_by_entity(pid).unwrap_or_default();
                contains.push((
                    parent.name.clone(),
                    e.name.clone(),
                    e.kind.to_string(),
                    parent_file,
                ));
            }
        }
    }
    contains.sort();
    if contains.is_empty() {
        writeln!(out, "_No parent-child relations_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Parent | Child | Kind | File |").expect("write");
        writeln!(out, "|--------|-------|------|------|").expect("write");
        for (parent, child, kind, file) in &contains {
            writeln!(
                out,
                "| {} | {} | {} | {} |",
                escape_md(parent),
                escape_md(child),
                escape_md(kind),
                escape_md(file)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    // ---- File dependencies ----
    writeln!(out, "## File Dependencies").expect("write");
    writeln!(out).expect("write");
    let all_files = index.dependency_graph.get_all_files();
    if all_files.is_empty() {
        writeln!(out, "_No file dependencies_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| From | To |").expect("write");
        writeln!(out, "|------|----|").expect("write");
        let mut deps: Vec<(String, String)> = Vec::new();
        for f in all_files {
            for to in index.dependency_graph.get_dependencies(&f) {
                deps.push((f.clone(), to));
            }
        }
        deps.sort();
        deps.dedup();
        for (from, to) in deps {
            writeln!(out, "| {} | {} |", escape_md(&from), escape_md(&to)).expect("write");
        }
        writeln!(out).expect("write");
    }

    // Exports / Imports per file
    writeln!(out, "## Module Metadata").expect("write");
    writeln!(out).expect("write");
    writeln!(out, "### Exports").expect("write");
    writeln!(out).expect("write");
    let mut has_exports = false;
    for entry in index.file_records().read().iter() {
        if !entry.1.exports.is_empty() {
            has_exports = true;
            writeln!(
                out,
                "- **{}** ({} exports):",
                escape_md(entry.0),
                entry.1.exports.len()
            )
            .expect("write");
            for ex in entry.1.exports.iter() {
                writeln!(
                    out,
                    "  - {} ({:?})",
                    escape_md(&ex.function_name),
                    ex.export_type
                )
                .expect("write");
            }
        }
    }
    if !has_exports {
        writeln!(out, "_No exports_").expect("write");
    }
    writeln!(out).expect("write");
    writeln!(out, "### Imports").expect("write");
    writeln!(out).expect("write");
    let mut has_imports = false;
    for entry in index.file_records().read().iter() {
        if !entry.1.imports.standardized_imports.is_empty() {
            has_imports = true;
            writeln!(
                out,
                "- **{}** ({} imports):",
                escape_md(entry.0),
                entry.1.imports.standardized_imports.len()
            )
            .expect("write");
            for imp in &entry.1.imports.standardized_imports {
                writeln!(out, "  - {:?} -> {}", imp.kind, escape_md(&imp.source)).expect("write");
            }
        }
    }
    if !has_imports {
        writeln!(out, "_No imports_").expect("write");
    }
    writeln!(out).expect("write");

    out
}

/// Collect relation entries for JSON.
pub fn collect_relations(index: &RelationIndex) -> Vec<RelationEntry> {
    let mut entries = Vec::new();
    let mut id_to_info: BTreeMap<EntityId, (String, String)> = BTreeMap::new();
    for entry in index.function_index().iter() {
        let id = *entry.key();
        let e = entry.value();
        let file = index.get_file_path_by_entity(id).unwrap_or_default();
        id_to_info.insert(id, (e.name.clone(), file));
    }
    for entry in index.resolved_relation_index().iter() {
        let caller = *entry.key();
        let caller_info = id_to_info.get(&caller);
        let caller_name = caller_info
            .map(|(n, _)| n.clone())
            .unwrap_or_else(|| caller.0.to_string());
        let caller_file = caller_info.map(|(_, f)| f.clone()).unwrap_or_default();
        for rel in entry.value().iter() {
            let callee_file = rel
                .callee_id
                .and_then(|cid| id_to_info.get(&cid).map(|(_, f)| f.clone()));
            let callee_name = rel
                .callee_id
                .and_then(|cid| id_to_info.get(&cid).map(|(n, _)| n.clone()))
                .unwrap_or_else(|| rel.callee_name.clone());
            entries.push(RelationEntry {
                caller: caller_name.clone(),
                caller_file: caller_file.clone(),
                callee: callee_name,
                callee_file,
                relation_type: rel.relation_type.to_string(),
                span: span_str_from_span(&rel.span),
                is_external: rel.is_external,
            });
        }
    }
    entries.sort_by(|a, b| {
        a.caller
            .cmp(&b.caller)
            .then_with(|| a.callee.cmp(&b.callee))
    });
    entries
}

// ---------------------------------------------------------------------------
// Type inference rendering
// ---------------------------------------------------------------------------

/// Render type inference results from parsed files.
///
/// For each file the full per-file `TypeInferenceEngine` is re-run so the
/// output reflects the same inference used by the relation resolver. This
/// relies only on `ParsedFile` and reuses the tree-sitter entities, so no
/// second parse is needed.
pub fn render_type_inference(files: &[ParsedFile], project_name: &str) -> String {
    render_type_inference_with_index(files, None, project_name)
}

/// Render type inference with an optional relation index for member index.
///
/// When a `RelationIndex` is available its symbol information can be used to
/// enhance member-lookup for discriminated unions.
pub fn render_type_inference_with_index(
    files: &[ParsedFile],
    _index: Option<&RelationIndex>,
    project_name: &str,
) -> String {
    let mut out = String::new();
    writeln!(out, "# Type Inference for {project_name}").expect("write");
    writeln!(out).expect("write");
    writeln!(
        out,
        "_Inferred per-file using the same engine as the resolver_"
    )
    .expect("write");
    writeln!(out).expect("write");

    if files.is_empty() {
        writeln!(out, "(no files)").expect("write");
        return out;
    }

    let mut total_bindings = 0usize;
    let mut total_narrowed = 0usize;
    let mut total_shapes: HashSet<String> = HashSet::new();

    for file in files {
        // Run inference
        let ctx = cce_relation::type_inference::TypeInferenceEngine::infer_types(
            file,
            &cce_relation::type_inference::traits::InferenceContext::default(),
        );
        // Alternative two-pass for richer results if file has many functions
        let ctx_two = cce_relation::type_inference::TypeInferenceEngine::infer_types_two_pass(
            file,
            &cce_relation::type_inference::traits::InferenceContext::default(),
        );
        // Merge both contexts for maximum coverage (two-pass may have forward refs)
        let mut merged = ctx.clone();
        merged.merge_from(&ctx_two);

        writeln!(out, "## File: {}", file.path).expect("write");
        writeln!(out).expect("write");
        writeln!(out, "- Language: {}", file.language).expect("write");
        writeln!(out, "- Entities: {}", file.entities.len()).expect("write");
        writeln!(
            out,
            "- Control-flow facts: {}",
            if file.control_flow.is_empty() { 0 } else { 1 }
        )
        .expect("write");
        writeln!(out).expect("write");

        if merged.is_empty() {
            writeln!(out, "_No inferred types_").expect("write");
            writeln!(out).expect("write");
            continue;
        }

        // Variables
        writeln!(out, "### Variables").expect("write");
        writeln!(out).expect("write");
        writeln!(
            out,
            "| Variable | Inferred Type | Origin | Priority | Shape | Span |"
        )
        .expect("write");
        writeln!(
            out,
            "|----------|---------------|--------|----------|-------|------|"
        )
        .expect("write");
        let mut var_count = 0usize;
        for frame in merged.frames_iter() {
            for (name, binding) in &frame.bindings {
                let origin = binding
                    .origin
                    .map(|o| format!("{:?}", o))
                    .unwrap_or_else(|| "-".to_string());
                let priority = binding
                    .origin
                    .map(|o| {
                        cce_relation::type_inference::types::origin_priority(Some(o)).to_string()
                    })
                    .unwrap_or_else(|| "0".to_string());
                let shape = binding
                    .shape
                    .as_ref()
                    .map(|s| s.to_type_string())
                    .unwrap_or_else(|| "-".to_string());
                total_shapes.insert(shape.clone());
                writeln!(
                    out,
                    "| {} | {} | {} | {} | {} | {} |",
                    escape_md(name),
                    escape_md(&binding.type_name),
                    escape_md(&origin),
                    priority,
                    escape_md(&shape),
                    span_str_from_span(&binding.span)
                )
                .expect("write");
                var_count += 1;
            }
        }
        if var_count == 0 {
            writeln!(out, "| - | - | - | - | - | - |").expect("write");
        }
        total_bindings += var_count;
        writeln!(out).expect("write");

        // Return types
        writeln!(out, "### Function Returns").expect("write");
        writeln!(out).expect("write");
        writeln!(
            out,
            "| Function (EntityId) | Return Type | Origin | Shape |"
        )
        .expect("write");
        writeln!(out, "|-----|-------------|--------|-------|").expect("write");
        let mut ret_count = 0usize;
        for (eid, binding) in merged.return_types_iter() {
            let shape = binding
                .shape
                .as_ref()
                .map(|s| s.to_type_string())
                .unwrap_or_else(|| "-".to_string());
            total_shapes.insert(shape.clone());
            let origin = binding
                .origin
                .map(|o| format!("{:?}", o))
                .unwrap_or_else(|| "-".to_string());
            // Find function name if possible
            let func_name = file
                .entities
                .iter()
                .find(|e| e.id == *eid)
                .map(|e| e.name.clone())
                .unwrap_or_else(|| format!("EntityId({})", eid.0));
            writeln!(
                out,
                "| {} ({}) | {} | {} | {} |",
                escape_md(&func_name),
                eid.0,
                escape_md(&binding.type_name),
                escape_md(&origin),
                escape_md(&shape)
            )
            .expect("write");
            ret_count += 1;
        }
        if ret_count == 0 {
            writeln!(out, "| - | - | - | - |").expect("write");
        }
        writeln!(out).expect("write");

        // Control-flow narrowing
        writeln!(out, "### Control-Flow Narrowing").expect("write");
        writeln!(out).expect("write");
        writeln!(out, "| Variable | Narrowed Type | Origin | Span |").expect("write");
        writeln!(out, "|----------|---------------|--------|------|").expect("write");
        let mut narrow_count = 0usize;
        for frame in merged.frames_iter() {
            for (name, list) in &frame.narrowed {
                for binding in list {
                    let origin = binding
                        .origin
                        .map(|o| format!("{:?}", o))
                        .unwrap_or_else(|| "-".to_string());
                    writeln!(
                        out,
                        "| {} | {} | {} | {} |",
                        escape_md(name),
                        escape_md(&binding.type_name),
                        escape_md(&origin),
                        span_str_from_span(&binding.span)
                    )
                    .expect("write");
                    narrow_count += 1;
                }
            }
        }
        if narrow_count == 0 {
            writeln!(out, "| - | - | - | - |").expect("write");
        }
        total_narrowed += narrow_count;
        writeln!(out).expect("write");

        // Type shapes summary per file
        writeln!(out, "### Type Shapes (distinct)").expect("write");
        writeln!(out).expect("write");
        let mut shapes: Vec<String> = merged
            .frames_iter()
            .flat_map(|f| {
                f.bindings
                    .values()
                    .filter_map(|b| b.shape.as_ref().map(|s| s.to_type_string()))
            })
            .collect();
        shapes.extend(
            merged
                .return_types_iter()
                .filter_map(|(_, b)| b.shape.as_ref().map(|s| s.to_type_string())),
        );
        shapes.sort();
        shapes.dedup();
        if shapes.is_empty() {
            writeln!(out, "_No complex shapes_").expect("write");
            writeln!(out).expect("write");
        } else {
            writeln!(out, "| Shape |").expect("write");
            writeln!(out, "|-------|").expect("write");
            for s in shapes {
                writeln!(out, "| {} |", escape_md(&s)).expect("write");
            }
            writeln!(out).expect("write");
        }
    }

    writeln!(out, "## Aggregate").expect("write");
    writeln!(out).expect("write");
    writeln!(out, "- Files with inference: {}", files.len()).expect("write");
    writeln!(out, "- Total variable bindings: {total_bindings}").expect("write");
    writeln!(out, "- Total narrowed bindings: {total_narrowed}").expect("write");
    writeln!(out, "- Distinct shapes: {}", total_shapes.len()).expect("write");
    writeln!(out).expect("write");

    out
}

// ---------------------------------------------------------------------------
// Summary rendering
// ---------------------------------------------------------------------------

/// Render a project-level summary covering symbols, relations and type inference.
pub fn render_summary(index: &RelationIndex, files: &[ParsedFile], project_name: &str) -> String {
    let mut out = String::new();
    writeln!(out, "# Summary for {project_name}").expect("write");
    writeln!(out).expect("write");

    let file_count = index.file_count().max(files.len());
    let entity_count = index.function_count();
    let relation_count = index.resolved_relation_count();
    let file_rel_count: usize = index
        .file_records()
        .read()
        .keys()
        .filter_map(|k| index.file_relations(k))
        .map(|v| v.len())
        .sum();
    let total_rels = relation_count + file_rel_count;
    let dep_count: usize = {
        let all = index.dependency_graph.get_all_files();
        let mut n = 0usize;
        for f in &all {
            n += index.dependency_graph.get_dependencies(f).len();
        }
        n
    };

    // Entities by kind
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut languages: HashSet<String> = HashSet::new();
    for entry in index.function_index().iter() {
        let k = entry.value().kind.to_string();
        *by_kind.entry(k).or_default() += 1;
    }
    for f in files {
        languages.insert(f.language.to_string());
    }
    for entry in index.file_records().read().iter() {
        languages.insert(entry.1.info.language.clone());
    }

    let call_count = relation_count; // Approximates call count
    let type_rel_count = {
        let mut n = 0usize;
        for entry in index.resolved_relation_index().iter() {
            for r in entry.value().iter() {
                if matches!(
                    r.relation_type,
                    cce_types::RelationType::Inheritance
                        | cce_types::RelationType::Implementation
                        | cce_types::RelationType::TraitBound
                        | cce_types::RelationType::TypeReference
                ) {
                    n += 1;
                }
            }
        }
        n
    };

    writeln!(out, "## Counts").expect("write");
    writeln!(out).expect("write");
    writeln!(out, "| Metric | Count |").expect("write");
    writeln!(out, "|--------|-------|").expect("write");
    writeln!(out, "| Files | {} |", file_count).expect("write");
    writeln!(out, "| Entities | {} |", entity_count).expect("write");
    writeln!(out, "| Relations (total) | {} |", total_rels).expect("write");
    writeln!(out, "| Relations (entity) | {} |", relation_count).expect("write");
    writeln!(out, "| Relations (file-level) | {} |", file_rel_count).expect("write");
    writeln!(out, "| Call relations | {} |", call_count).expect("write");
    writeln!(out, "| Type relations | {} |", type_rel_count).expect("write");
    writeln!(out, "| File dependencies | {} |", dep_count).expect("write");
    writeln!(out).expect("write");

    writeln!(out, "## Languages").expect("write");
    writeln!(out).expect("write");
    let mut langs: Vec<String> = languages.into_iter().collect();
    langs.sort();
    if langs.is_empty() {
        writeln!(out, "_No language info_").expect("write");
    } else {
        for l in &langs {
            writeln!(out, "- {}", l).expect("write");
        }
    }
    writeln!(out).expect("write");

    writeln!(out, "## Entities by Kind").expect("write");
    writeln!(out).expect("write");
    writeln!(out, "| Kind | Count |").expect("write");
    writeln!(out, "|------|-------|").expect("write");
    let mut sorted: Vec<(&String, &usize)> = by_kind.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    for (k, c) in sorted {
        writeln!(out, "| {} | {} |", k, c).expect("write");
    }
    writeln!(out).expect("write");

    writeln!(out, "## Files (from index)").expect("write");
    writeln!(out).expect("write");
    let mut file_list: Vec<String> = index.file_records().read().keys().cloned().collect();
    if file_list.is_empty() {
        // Fallback to parsed files
        file_list = files.iter().map(|f| f.path.clone()).collect();
    }
    file_list.sort();
    for f in file_list {
        writeln!(out, "- {}", f).expect("write");
    }
    writeln!(out).expect("write");

    writeln!(out, "## Artifacts").expect("write");
    writeln!(out).expect("write");
    writeln!(out, "- `SUMMARY.md` — this file (project summary)").expect("write");
    writeln!(
        out,
        "- `<path>.txt` — per-file report (symbols + relations + type inference), preserves original directory tree"
    )
    .expect("write");
    writeln!(
        out,
        "- `<dir>.dir.txt` — per-directory overview (sibling to directory, e.g. `crates/cli/src.dir.txt`)"
    )
    .expect("write");
    writeln!(out).expect("write");

    out
}

/// Collect summary for JSON.
pub fn collect_summary(
    index: &RelationIndex,
    files: &[ParsedFile],
    project_name: &str,
) -> ProjectSummary {
    let file_count = index.file_count().max(files.len());
    let entity_count = index.function_count();
    let relation_count = index.resolved_relation_count();
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for entry in index.function_index().iter() {
        let k = entry.value().kind.to_string();
        *by_kind.entry(k).or_default() += 1;
    }
    let mut languages: Vec<String> = files.iter().map(|f| f.language.to_string()).collect();
    for entry in index.file_records().read().iter() {
        languages.push(entry.1.info.language.clone());
    }
    languages.sort();
    languages.dedup();
    let dep_count: usize = {
        let all = index.dependency_graph.get_all_files();
        let mut n = 0usize;
        for f in &all {
            n += index.dependency_graph.get_dependencies(f).len();
        }
        n
    };
    let call_count = relation_count;
    let type_rel_count = {
        let mut n = 0usize;
        for entry in index.resolved_relation_index().iter() {
            for r in entry.value().iter() {
                if matches!(
                    r.relation_type,
                    cce_types::RelationType::Inheritance
                        | cce_types::RelationType::Implementation
                        | cce_types::RelationType::TraitBound
                        | cce_types::RelationType::TypeReference
                ) {
                    n += 1;
                }
            }
        }
        n
    };
    ProjectSummary {
        project_name: project_name.to_string(),
        file_count,
        entity_count,
        relation_count,
        type_relation_count: type_rel_count,
        call_count,
        dependency_count: dep_count,
        languages,
        entities_by_kind: by_kind,
    }
}

// ---------------------------------------------------------------------------
// Per-file report rendering (hierarchical)
// ---------------------------------------------------------------------------

/// Render a per-file report that colocates symbols, relations, and type
/// inference for a single source file.
///
/// This keeps markdown tables (the most readable format) but scoped to one
/// file so the result remains skimmable. Relations are organized per file —
/// outgoing calls, type relationships, containment, file dependencies and
/// module metadata all live beside the symbols they refer to.
pub fn render_file_report(
    file_path: &str,
    entities: &[(EntityId, Entity)],
    index: &RelationIndex,
    parsed_file: Option<&ParsedFile>,
    global_id_to_entity: &BTreeMap<EntityId, Entity>,
) -> String {
    let mut out = String::new();
    writeln!(out, "# File: {file_path}").expect("write");
    writeln!(out).expect("write");
    writeln!(
        out,
        "_Entities: {} | Language: {}_",
        entities.len(),
        parsed_file
            .map(|pf| pf.language.to_string())
            .or_else(|| index
                .file_records()
                .read()
                .get(file_path)
                .map(|r| r.info.language.clone()))
            .unwrap_or_else(|| "unknown".to_string())
    )
    .expect("write");
    writeln!(out).expect("write");

    // Build parent -> children for this file only (for field inline rendering)
    let mut parent_to_children: BTreeMap<EntityId, Vec<&Entity>> = BTreeMap::new();
    for (_, e) in entities {
        if let Some(pid) = e.parent {
            parent_to_children.entry(pid).or_default().push(e);
        }
    }
    let mut by_kind: BTreeMap<String, Vec<&Entity>> = BTreeMap::new();
    for (_, e) in entities {
        by_kind.entry(e.kind.to_string()).or_default().push(e);
    }
    for list in by_kind.values_mut() {
        list.sort_by_key(|e| e.span.start_position.row);
    }

    if entities.is_empty() {
        writeln!(out, "_No entities in this file_").expect("write");
        writeln!(out).expect("write");
    } else {
        // Type definitions
        let type_kinds = [
            "struct",
            "class",
            "enum",
            "trait",
            "interface",
            "type_alias",
            "union",
        ];
        for kind in type_kinds {
            if let Some(list) = by_kind.get(kind) {
                writeln!(out, "## {}s", capitalize(kind)).expect("write");
                writeln!(out).expect("write");
                writeln!(out, "| Name | Signature | Fields | Visibility | Span |").expect("write");
                writeln!(out, "|------|-----------|--------|------------|------|").expect("write");
                for e in list {
                    let fields = parent_to_children
                        .get(&e.id)
                        .map(|children| {
                            children
                                .iter()
                                .filter(|c| {
                                    c.kind == EntityKind::Field
                                        || c.kind == EntityKind::Property
                                        || c.kind == EntityKind::Variable
                                })
                                .map(|c| {
                                    let ty = c
                                        .metadata
                                        .get("field_type")
                                        .or_else(|| c.metadata.get("type_annotation"))
                                        .or_else(|| c.metadata.get("variable_type"))
                                        .cloned()
                                        .unwrap_or_else(|| "-".to_string());
                                    if ty == "-" {
                                        c.name.clone()
                                    } else {
                                        format!("{}: {}", c.name, ty)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    let fields = if fields.is_empty() {
                        "-".to_string()
                    } else {
                        fields
                    };
                    let sig = if e.signature.is_empty() {
                        e.name.clone()
                    } else {
                        e.signature.clone()
                    };
                    writeln!(
                        out,
                        "| {} | {} | {} | {} | {} |",
                        escape_md(&e.name),
                        escape_md(&sig),
                        escape_md(&fields),
                        escape_md(&visibility_of(e)),
                        span_str(e)
                    )
                    .expect("write");
                }
                writeln!(out).expect("write");
            }
        }

        for kind in ["inherent_impl", "trait_impl"] {
            if let Some(list) = by_kind.get(kind) {
                writeln!(out, "## {}s", capitalize(kind)).expect("write");
                writeln!(out).expect("write");
                writeln!(out, "| Name | Signature | Visibility | Span |").expect("write");
                writeln!(out, "|------|-----------|------------|------|").expect("write");
                for e in list {
                    let sig = if e.signature.is_empty() {
                        e.name.clone()
                    } else {
                        e.signature.clone()
                    };
                    writeln!(
                        out,
                        "| {} | {} | {} | {} |",
                        escape_md(&e.name),
                        escape_md(&sig),
                        escape_md(&visibility_of(e)),
                        span_str(e)
                    )
                    .expect("write");
                }
                writeln!(out).expect("write");
            }
        }

        let callable_kinds = ["function", "method", "constructor", "operator"];
        let has_callable = callable_kinds.iter().any(|k| by_kind.contains_key(*k));
        if has_callable {
            writeln!(out, "## Functions / Methods").expect("write");
            writeln!(out).expect("write");
            writeln!(
                out,
                "| Name | Kind | Parameters | Return Type | Visibility | Span |"
            )
            .expect("write");
            writeln!(
                out,
                "|------|------|------------|-------------|------------|------|"
            )
            .expect("write");
            for kind in callable_kinds {
                if let Some(list) = by_kind.get(kind) {
                    for e in list {
                        let params = if e.parameters.is_empty() {
                            "-".to_string()
                        } else {
                            e.parameters
                                .iter()
                                .map(|(n, t)| {
                                    if let Some(ty) = t {
                                        format!("{n}: {ty}")
                                    } else {
                                        n.clone()
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        let ret = e.return_type.as_deref().unwrap_or("-");
                        writeln!(
                            out,
                            "| {} | {} | {} | {} | {} | {} |",
                            escape_md(&e.name),
                            kind,
                            escape_md(&params),
                            escape_md(ret),
                            escape_md(&visibility_of(e)),
                            span_str(e)
                        )
                        .expect("write");
                    }
                }
            }
            writeln!(out).expect("write");
        }

        let var_kinds = ["field", "variable", "constant", "property", "enum_variant"];
        let has_vars = var_kinds.iter().any(|k| by_kind.contains_key(*k));
        if has_vars {
            writeln!(out, "## Variables / Fields / Constants").expect("write");
            writeln!(out).expect("write");
            writeln!(out, "| Name | Kind | Type | Visibility | Span | Parent |").expect("write");
            writeln!(out, "|------|------|------|------------|------|--------|").expect("write");
            for kind in var_kinds {
                if let Some(list) = by_kind.get(kind) {
                    for e in list {
                        let ty = e
                            .metadata
                            .get("field_type")
                            .or_else(|| e.metadata.get("type_annotation"))
                            .or_else(|| e.metadata.get("variable_type"))
                            .or_else(|| e.metadata.get("literal_type"))
                            .or_else(|| e.metadata.get("constructor_type"))
                            .cloned()
                            .unwrap_or_else(|| "-".to_string());
                        let parent_name = e
                            .parent
                            .and_then(|pid| global_id_to_entity.get(&pid).map(|p| p.name.clone()))
                            .unwrap_or_else(|| "-".to_string());
                        writeln!(
                            out,
                            "| {} | {} | {} | {} | {} | {} |",
                            escape_md(&e.name),
                            kind,
                            escape_md(&ty),
                            escape_md(&visibility_of(e)),
                            span_str(e),
                            escape_md(&parent_name)
                        )
                        .expect("write");
                    }
                }
            }
            writeln!(out).expect("write");
        }

        let covered: HashSet<&str> = [
            "struct",
            "class",
            "enum",
            "trait",
            "interface",
            "type_alias",
            "union",
            "inherent_impl",
            "trait_impl",
            "function",
            "method",
            "constructor",
            "operator",
            "field",
            "variable",
            "constant",
            "property",
            "enum_variant",
        ]
        .into_iter()
        .collect();
        let mut others: Vec<(&String, &Vec<&Entity>)> = by_kind
            .iter()
            .filter(|(k, _)| !covered.contains(k.as_str()))
            .collect();
        if !others.is_empty() {
            others.sort_by(|a, b| a.0.cmp(b.0));
            writeln!(out, "## Other Entities").expect("write");
            writeln!(out).expect("write");
            writeln!(out, "| Name | Kind | Signature | Visibility | Span |").expect("write");
            writeln!(out, "|------|------|-----------|------------|------|").expect("write");
            for (kind, list) in others {
                for e in list {
                    let sig = if e.signature.is_empty() {
                        e.name.clone()
                    } else {
                        e.signature.clone()
                    };
                    writeln!(
                        out,
                        "| {} | {} | {} | {} | {} |",
                        escape_md(&e.name),
                        escape_md(kind),
                        escape_md(&sig),
                        escape_md(&visibility_of(e)),
                        span_str(e)
                    )
                    .expect("write");
                }
            }
            writeln!(out).expect("write");
        }
    }

    // ---- Relations colocated with this file ----
    writeln!(out, "## Relations").expect("write");
    writeln!(out).expect("write");

    let mut id_to_info: BTreeMap<EntityId, (String, String, String)> = BTreeMap::new();
    for entry in index.function_index().iter() {
        let id = *entry.key();
        let e = entry.value();
        let file = index.get_file_path_by_entity(id).unwrap_or_default();
        id_to_info.insert(id, (e.name.clone(), file, e.kind.to_string()));
    }

    let mut internal_calls: Vec<(String, String, String, String)> = Vec::new();
    let mut external_calls: Vec<(String, String, String)> = Vec::new();
    let mut hierarchy_rels: Vec<(String, String, String)> = Vec::new();

    for entry in index.resolved_relation_index().iter() {
        let caller_id = *entry.key();
        let caller_file = index.get_file_path_by_entity(caller_id).unwrap_or_default();
        if caller_file != file_path {
            continue;
        }
        let caller_name = id_to_info
            .get(&caller_id)
            .map(|(n, _, _)| n.clone())
            .unwrap_or_else(|| format!("<?>:{}", caller_id.0));
        for rel in entry.value().iter() {
            if rel.relation_type.is_call() {
                let callee_name = if let Some(cid) = rel.callee_id {
                    id_to_info
                        .get(&cid)
                        .map(|(n, _, _)| n.clone())
                        .unwrap_or_else(|| rel.callee_name.clone())
                } else {
                    rel.callee_name.clone()
                };
                let callee_file = rel
                    .callee_id
                    .and_then(|cid| id_to_info.get(&cid).map(|(_, f, _)| f.clone()))
                    .unwrap_or_default();
                let span = span_str_from_span(&rel.span);
                if rel.is_external {
                    external_calls.push((caller_name.clone(), callee_name, span));
                } else {
                    internal_calls.push((caller_name.clone(), callee_name, callee_file, span));
                }
            } else if rel.relation_type.is_structural()
                || rel.relation_type.is_reference()
                || rel.relation_type.is_dependency()
                || matches!(
                    rel.relation_type,
                    cce_types::RelationType::Inheritance
                        | cce_types::RelationType::Implementation
                        | cce_types::RelationType::TraitBound
                        | cce_types::RelationType::ImplAssociation
                )
            {
                let callee = rel
                    .callee_id
                    .and_then(|cid| id_to_info.get(&cid).map(|(n, _, _)| n.clone()))
                    .unwrap_or_else(|| rel.callee_name.clone());
                hierarchy_rels.push((
                    id_to_info
                        .get(&rel.caller)
                        .map(|(n, _, _)| n.clone())
                        .unwrap_or_else(|| rel.caller.0.to_string()),
                    callee,
                    rel.relation_type.to_string(),
                ));
            }
        }
    }
    if let Some(rels) = index.file_relations(file_path) {
        for rel in rels.iter() {
            if rel.relation_type.is_call() {
                let callee_name = if let Some(cid) = rel.callee_id {
                    id_to_info
                        .get(&cid)
                        .map(|(n, _, _)| n.clone())
                        .unwrap_or_else(|| rel.callee_name.clone())
                } else {
                    rel.callee_name.clone()
                };
                let callee_file = rel
                    .callee_id
                    .and_then(|cid| id_to_info.get(&cid).map(|(_, f, _)| f.clone()))
                    .unwrap_or_default();
                let span = span_str_from_span(&rel.span);
                if rel.is_external {
                    external_calls.push((file_path.to_string(), callee_name, span));
                } else {
                    internal_calls.push((file_path.to_string(), callee_name, callee_file, span));
                }
            } else if rel.relation_type.is_structural()
                || rel.relation_type.is_reference()
                || rel.relation_type.is_dependency()
            {
                let callee = rel
                    .callee_id
                    .and_then(|cid| id_to_info.get(&cid).map(|(n, _, _)| n.clone()))
                    .unwrap_or_else(|| rel.callee_name.clone());
                hierarchy_rels.push((file_path.to_string(), callee, rel.relation_type.to_string()));
            }
        }
    }
    internal_calls.sort();
    external_calls.sort();
    hierarchy_rels.sort();

    writeln!(out, "### Internal Calls ({})", internal_calls.len()).expect("write");
    writeln!(out).expect("write");
    if internal_calls.is_empty() {
        writeln!(out, "_No internal calls_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Caller | Callee | Callee File | Span |").expect("write");
        writeln!(out, "|--------|--------|-------------|------|").expect("write");
        for (caller, callee, callee_file, span) in &internal_calls {
            writeln!(
                out,
                "| {} | {} | {} | {} |",
                escape_md(caller),
                escape_md(callee),
                escape_md(callee_file),
                escape_md(span)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    writeln!(out, "### External Calls ({})", external_calls.len()).expect("write");
    writeln!(out).expect("write");
    if external_calls.is_empty() {
        writeln!(out, "_No external calls_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Caller | Callee | Span |").expect("write");
        writeln!(out, "|--------|--------|------|").expect("write");
        for (caller, callee, span) in &external_calls {
            writeln!(
                out,
                "| {} | {} | {} |",
                escape_md(caller),
                escape_md(callee),
                escape_md(span)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    writeln!(out, "### Type Relationships ({})", hierarchy_rels.len()).expect("write");
    writeln!(out).expect("write");
    if hierarchy_rels.is_empty() {
        writeln!(out, "_No type relationships_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Source | Target | Kind |").expect("write");
        writeln!(out, "|--------|--------|------|").expect("write");
        for (src, target, kind) in &hierarchy_rels {
            writeln!(
                out,
                "| {} | {} | {} |",
                escape_md(src),
                escape_md(target),
                escape_md(kind)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    // Contains
    writeln!(out, "### Contains (Parent → Child)").expect("write");
    writeln!(out).expect("write");
    let mut contains: Vec<(String, String, String)> = Vec::new();
    for (_, e) in entities {
        if let Some(pid) = e.parent {
            if let Some(parent) = global_id_to_entity.get(&pid) {
                let parent_file = index.get_file_path_by_entity(pid).unwrap_or_default();
                if parent_file == file_path {
                    contains.push((parent.name.clone(), e.name.clone(), e.kind.to_string()));
                }
            }
        }
    }
    contains.sort();
    if contains.is_empty() {
        writeln!(out, "_No parent-child relations_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Parent | Child | Kind |").expect("write");
        writeln!(out, "|--------|-------|------|").expect("write");
        for (parent, child, kind) in &contains {
            writeln!(
                out,
                "| {} | {} | {} |",
                escape_md(parent),
                escape_md(child),
                escape_md(kind)
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    // File dependencies (outgoing from this file)
    writeln!(out, "### File Dependencies").expect("write");
    writeln!(out).expect("write");
    let deps = index.dependency_graph.get_dependencies(file_path);
    if deps.is_empty() {
        writeln!(out, "_No file dependencies_").expect("write");
        writeln!(out).expect("write");
    } else {
        let mut sorted = deps.clone();
        sorted.sort();
        sorted.dedup();
        writeln!(out, "| From | To |").expect("write");
        writeln!(out, "|------|----|").expect("write");
        for to in sorted {
            writeln!(out, "| {} | {} |", escape_md(file_path), escape_md(&to)).expect("write");
        }
        writeln!(out).expect("write");
    }

    // Module metadata (exports/imports for this file)
    writeln!(out, "### Module Metadata").expect("write");
    writeln!(out).expect("write");
    if let Some(rec) = index.file_records().read().get(file_path) {
        writeln!(out, "#### Exports ({})", rec.exports.len()).expect("write");
        writeln!(out).expect("write");
        if rec.exports.is_empty() {
            writeln!(out, "_No exports_").expect("write");
            writeln!(out).expect("write");
        } else {
            for ex in rec.exports.iter() {
                writeln!(
                    out,
                    "- {} ({:?})",
                    escape_md(&ex.function_name),
                    ex.export_type
                )
                .expect("write");
            }
            writeln!(out).expect("write");
        }
        writeln!(
            out,
            "#### Imports ({})",
            rec.imports.standardized_imports.len()
        )
        .expect("write");
        writeln!(out).expect("write");
        if rec.imports.standardized_imports.is_empty() {
            writeln!(out, "_No imports_").expect("write");
            writeln!(out).expect("write");
        } else {
            for imp in &rec.imports.standardized_imports {
                writeln!(out, "- {:?} -> {}", imp.kind, escape_md(&imp.source)).expect("write");
            }
            writeln!(out).expect("write");
        }
    } else {
        writeln!(out, "_No module metadata_").expect("write");
        writeln!(out).expect("write");
    }

    // Type inference for this single file
    writeln!(out, "## Type Inference").expect("write");
    writeln!(out).expect("write");
    if let Some(pf) = parsed_file {
        let ctx = cce_relation::type_inference::TypeInferenceEngine::infer_types(
            pf,
            &cce_relation::type_inference::traits::InferenceContext::default(),
        );
        let ctx_two = cce_relation::type_inference::TypeInferenceEngine::infer_types_two_pass(
            pf,
            &cce_relation::type_inference::traits::InferenceContext::default(),
        );
        let mut merged = ctx.clone();
        merged.merge_from(&ctx_two);
        if merged.is_empty() {
            writeln!(out, "_No inferred types_").expect("write");
            writeln!(out).expect("write");
        } else {
            writeln!(out, "### Variables").expect("write");
            writeln!(out).expect("write");
            writeln!(
                out,
                "| Variable | Inferred Type | Origin | Priority | Shape | Span |"
            )
            .expect("write");
            writeln!(
                out,
                "|----------|---------------|--------|----------|-------|------|"
            )
            .expect("write");
            let mut var_count = 0usize;
            for frame in merged.frames_iter() {
                for (name, binding) in &frame.bindings {
                    let origin = binding
                        .origin
                        .map(|o| format!("{:?}", o))
                        .unwrap_or_else(|| "-".to_string());
                    let priority = binding
                        .origin
                        .map(|o| {
                            cce_relation::type_inference::types::origin_priority(Some(o))
                                .to_string()
                        })
                        .unwrap_or_else(|| "0".to_string());
                    let shape = binding
                        .shape
                        .as_ref()
                        .map(|s| s.to_type_string())
                        .unwrap_or_else(|| "-".to_string());
                    writeln!(
                        out,
                        "| {} | {} | {} | {} | {} | {} |",
                        escape_md(name),
                        escape_md(&binding.type_name),
                        escape_md(&origin),
                        priority,
                        escape_md(&shape),
                        span_str_from_span(&binding.span)
                    )
                    .expect("write");
                    var_count += 1;
                }
            }
            if var_count == 0 {
                writeln!(out, "| - | - | - | - | - | - |").expect("write");
            }
            writeln!(out).expect("write");

            writeln!(out, "### Function Returns").expect("write");
            writeln!(out).expect("write");
            writeln!(
                out,
                "| Function (EntityId) | Return Type | Origin | Shape |"
            )
            .expect("write");
            writeln!(out, "|-----|-------------|--------|-------|").expect("write");
            let mut ret_count = 0usize;
            for (eid, binding) in merged.return_types_iter() {
                let shape = binding
                    .shape
                    .as_ref()
                    .map(|s| s.to_type_string())
                    .unwrap_or_else(|| "-".to_string());
                let origin = binding
                    .origin
                    .map(|o| format!("{:?}", o))
                    .unwrap_or_else(|| "-".to_string());
                let func_name = pf
                    .entities
                    .iter()
                    .find(|e| e.id == *eid)
                    .map(|e| e.name.clone())
                    .unwrap_or_else(|| format!("EntityId({})", eid.0));
                writeln!(
                    out,
                    "| {} ({}) | {} | {} | {} |",
                    escape_md(&func_name),
                    eid.0,
                    escape_md(&binding.type_name),
                    escape_md(&origin),
                    escape_md(&shape)
                )
                .expect("write");
                ret_count += 1;
            }
            if ret_count == 0 {
                writeln!(out, "| - | - | - | - |").expect("write");
            }
            writeln!(out).expect("write");

            writeln!(out, "### Control-Flow Narrowing").expect("write");
            writeln!(out).expect("write");
            writeln!(out, "| Variable | Narrowed Type | Origin | Span |").expect("write");
            writeln!(out, "|----------|---------------|--------|------|").expect("write");
            let mut narrow_count = 0usize;
            for frame in merged.frames_iter() {
                for (name, list) in &frame.narrowed {
                    for binding in list {
                        let origin = binding
                            .origin
                            .map(|o| format!("{:?}", o))
                            .unwrap_or_else(|| "-".to_string());
                        writeln!(
                            out,
                            "| {} | {} | {} | {} |",
                            escape_md(name),
                            escape_md(&binding.type_name),
                            escape_md(&origin),
                            span_str_from_span(&binding.span)
                        )
                        .expect("write");
                        narrow_count += 1;
                    }
                }
            }
            if narrow_count == 0 {
                writeln!(out, "| - | - | - | - |").expect("write");
            }
            writeln!(out).expect("write");

            writeln!(out, "### Type Shapes (distinct)").expect("write");
            writeln!(out).expect("write");
            let mut shapes: Vec<String> = merged
                .frames_iter()
                .flat_map(|f| {
                    f.bindings
                        .values()
                        .filter_map(|b| b.shape.as_ref().map(|s| s.to_type_string()))
                })
                .collect();
            shapes.extend(
                merged
                    .return_types_iter()
                    .filter_map(|(_, b)| b.shape.as_ref().map(|s| s.to_type_string())),
            );
            shapes.sort();
            shapes.dedup();
            if shapes.is_empty() {
                writeln!(out, "_No complex shapes_").expect("write");
                writeln!(out).expect("write");
            } else {
                writeln!(out, "| Shape |").expect("write");
                writeln!(out, "|-------|").expect("write");
                for s in shapes {
                    writeln!(out, "| {} |", escape_md(&s)).expect("write");
                }
                writeln!(out).expect("write");
            }
        }
    } else {
        writeln!(out, "_No parsed file available for type inference_").expect("write");
        writeln!(out).expect("write");
    }

    out
}

/// Render a per-directory overview file.
///
/// The file lives sibling to the directory (e.g. `crates/cli/src.dir.txt`)
/// so it does not clash with a normal source file. It summarizes the subtree
/// rooted at `dir_path`: file counts, entity counts, direct children, and
/// aggregated entities by kind and dependencies.
fn render_directory_report(
    dir_path: &str,
    ordered: &BTreeMap<String, Vec<(EntityId, Entity)>>,
    index: &RelationIndex,
) -> String {
    let mut out = String::new();
    writeln!(out, "# Directory: {dir_path}").expect("write");
    writeln!(out).expect("write");

    let prefix = format!("{dir_path}/");
    let mut subtree_files: Vec<String> = Vec::new();
    for fp in ordered.keys() {
        if fp == dir_path || fp.starts_with(&prefix) {
            subtree_files.push(fp.clone());
        }
    }
    // Include file_records that may have no entities but exist in index
    for fp in index.file_records().read().keys() {
        if (fp == dir_path || fp.starts_with(&prefix)) && !subtree_files.contains(fp) {
            subtree_files.push(fp.clone());
        }
    }
    subtree_files.sort();
    let subtree_entity_count: usize = subtree_files
        .iter()
        .filter_map(|f| ordered.get(f))
        .map(|v| v.len())
        .sum();

    writeln!(
        out,
        "_Files in subtree: {} | Entities: {}_",
        subtree_files.len(),
        subtree_entity_count
    )
    .expect("write");
    writeln!(out).expect("write");

    // Direct children files and subdirectories
    let mut direct_files: Vec<(String, usize)> = Vec::new();
    let mut direct_subdirs: BTreeSet<String> = BTreeSet::new();
    for fp in &subtree_files {
        let rel = if fp == dir_path {
            ""
        } else if let Some(stripped) = fp.strip_prefix(&prefix) {
            stripped
        } else {
            continue;
        };
        if rel.is_empty() {
            continue;
        }
        if let Some((first, rest)) = rel.split_once('/') {
            direct_subdirs.insert(first.to_string());
            let _ = rest;
        } else {
            let cnt = ordered.get(fp).map(|v| v.len()).unwrap_or(0);
            direct_files.push((fp.clone(), cnt));
        }
    }
    // Also detect subdirs that contain no files with entities but are present as empty dirs via file_records? Already handled.

    writeln!(out, "## Direct Files ({})", direct_files.len()).expect("write");
    writeln!(out).expect("write");
    if direct_files.is_empty() {
        writeln!(out, "_No direct files_").expect("write");
        writeln!(out).expect("write");
    } else {
        direct_files.sort();
        writeln!(out, "| File | Entities |").expect("write");
        writeln!(out, "|------|----------|").expect("write");
        for (f, cnt) in &direct_files {
            writeln!(out, "| {} | {} |", escape_md(f), cnt).expect("write");
        }
        writeln!(out).expect("write");
    }

    writeln!(out, "## Direct Subdirectories ({})", direct_subdirs.len()).expect("write");
    writeln!(out).expect("write");
    if direct_subdirs.is_empty() {
        writeln!(out, "_No subdirectories_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(
            out,
            "| Directory | Files (recursive) | Entities (recursive) |"
        )
        .expect("write");
        writeln!(
            out,
            "|-----------|-------------------|----------------------|"
        )
        .expect("write");
        let mut subdir_sorted: Vec<String> = direct_subdirs.into_iter().collect();
        subdir_sorted.sort();
        for sub in subdir_sorted {
            let sub_prefix = format!("{dir_path}/{sub}/");
            let mut cnt_files = 0usize;
            let mut cnt_entities = 0usize;
            for fp in &subtree_files {
                if fp.starts_with(&sub_prefix) || fp == &format!("{dir_path}/{sub}") {
                    cnt_files += 1;
                    if let Some(v) = ordered.get(fp) {
                        cnt_entities += v.len();
                    }
                }
            }
            // Also files directly named as sub (if sub is a file?) Not needed.
            writeln!(
                out,
                "| {} | {} | {} |",
                escape_md(&sub),
                cnt_files,
                cnt_entities
            )
            .expect("write");
        }
        writeln!(out).expect("write");
    }

    writeln!(out, "## All Files in Subtree").expect("write");
    writeln!(out).expect("write");
    if subtree_files.is_empty() {
        writeln!(out, "_No files_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| File | Entities |").expect("write");
        writeln!(out, "|------|----------|").expect("write");
        for f in &subtree_files {
            let cnt = ordered.get(f).map(|v| v.len()).unwrap_or(0);
            writeln!(out, "| {} | {} |", escape_md(f), cnt).expect("write");
        }
        writeln!(out).expect("write");
    }

    // Entities by kind aggregated for subtree
    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for fp in &subtree_files {
        if let Some(ents) = ordered.get(fp) {
            for (_, e) in ents {
                *by_kind.entry(e.kind.to_string()).or_default() += 1;
            }
        }
    }
    writeln!(out, "## Entities by Kind (subtree)").expect("write");
    writeln!(out).expect("write");
    if by_kind.is_empty() {
        writeln!(out, "_No entities_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| Kind | Count |").expect("write");
        writeln!(out, "|------|-------|").expect("write");
        let mut sorted: Vec<(&String, &usize)> = by_kind.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        for (k, c) in sorted {
            writeln!(out, "| {} | {} |", k, c).expect("write");
        }
        writeln!(out).expect("write");
    }

    // File dependencies within subtree
    writeln!(out, "## File Dependencies (outgoing from subtree)").expect("write");
    writeln!(out).expect("write");
    let mut deps: Vec<(String, String)> = Vec::new();
    for fp in &subtree_files {
        for to in index.dependency_graph.get_dependencies(fp) {
            deps.push((fp.clone(), to));
        }
    }
    deps.sort();
    deps.dedup();
    if deps.is_empty() {
        writeln!(out, "_No file dependencies_").expect("write");
        writeln!(out).expect("write");
    } else {
        writeln!(out, "| From | To |").expect("write");
        writeln!(out, "|------|----|").expect("write");
        for (from, to) in deps {
            writeln!(out, "| {} | {} |", escape_md(&from), escape_md(&to)).expect("write");
        }
        writeln!(out).expect("write");
    }

    out
}

// ---------------------------------------------------------------------------
// Writer (orchestrates OutputManager)
// ---------------------------------------------------------------------------

/// High-level writer that emits hierarchical structured outputs.
///
/// Reuses `OutputManager` for directory management so the layout stays
/// consistent with the rest of the E2E `outputs/` tree.
pub struct StructuredOutputWriter {
    manager: OutputManager,
    project_name: String,
}

impl StructuredOutputWriter {
    /// Create a writer that targets `outputs/scenarios/<language>/structured/<project>`
    /// via `OutputManager`.
    pub fn for_scenarios(language: &str, project_name: &str) -> Self {
        let manager = OutputManager::builder()
            .category(OutputCategory::Scenarios)
            .language(language)
            .scenario(format!("structured/{project_name}"))
            .build();
        Self {
            manager,
            project_name: project_name.to_string(),
        }
    }

    /// Create with a custom `OutputManager`.
    pub fn new(manager: OutputManager, project_name: impl Into<String>) -> Self {
        Self {
            manager,
            project_name: project_name.into(),
        }
    }

    /// Ensure the output directory exists and return its path.
    pub fn ensure_dir(&self) -> std::io::Result<std::path::PathBuf> {
        self.manager.ensure_output_dir()
    }

    /// Write all hierarchical files from an index + parsed files pair.
    ///
    /// Produces `SUMMARY.md` plus per-file `<path>.txt` reports (preserving the
    /// original directory tree) and per-directory `<dir>.dir.txt` overviews
    /// colocated sibling to each directory. Relations are colocated inside each
    /// per-file report rather than aggregated into a global `RELATIONS.md`.
    /// JSON outputs are intentionally omitted — markdown tables are the canonical
    /// human-readable form.
    pub fn write_all(
        &self,
        index: &RelationIndex,
        files: &[ParsedFile],
    ) -> std::io::Result<Vec<std::path::PathBuf>> {
        let base_dir = self.manager.ensure_output_dir()?;

        // Clean up legacy aggregated files and JSON left from previous runs.
        for legacy in [
            "SYMBOL_TABLE.md",
            "RELATIONS.md",
            "TYPE_INFERENCE.md",
            "symbols.json",
            "relations.json",
            "summary.json",
        ] {
            let p = base_dir.join(legacy);
            let _ = std::fs::remove_file(p);
        }

        let mut paths = Vec::new();
        let summary = render_summary(index, files, &self.project_name);
        paths.push(self.manager.write("SUMMARY.md", &summary)?);

        let ordered = ordered_files(index);

        let mut pf_map: BTreeMap<String, &ParsedFile> = BTreeMap::new();
        for pf in files {
            pf_map.insert(pf.path.clone(), pf);
        }

        let mut global_id_to_entity: BTreeMap<EntityId, Entity> = BTreeMap::new();
        for ents in ordered.values() {
            for (id, e) in ents {
                global_id_to_entity.insert(*id, e.clone());
            }
        }

        let mut all_files_set: BTreeSet<String> = BTreeSet::new();
        for k in ordered.keys() {
            all_files_set.insert(k.clone());
        }
        for k in pf_map.keys() {
            all_files_set.insert(k.clone());
        }
        for path in index.file_records().read().keys() {
            all_files_set.insert(path.clone());
        }

        // Distinct directories present in the file set.
        let mut dirs: BTreeSet<String> = BTreeSet::new();
        for fp in &all_files_set {
            let mut p = Path::new(fp);
            while let Some(parent) = p.parent() {
                if parent == Path::new("") || parent == Path::new(".") {
                    break;
                }
                dirs.insert(parent.to_string_lossy().to_string());
                p = parent;
            }
        }

        for file_path in &all_files_set {
            let entities = ordered.get(file_path).map(|v| v.as_slice()).unwrap_or(&[]);
            // Need owned Vec<(EntityId, Entity)> for render? Pass slice of pairs.
            // Build owned vector for helper (expects &[(EntityId, Entity)] slice)
            let owned: Vec<(EntityId, Entity)> = entities.to_vec();
            let pf_opt = pf_map.get(file_path).copied();
            let content =
                render_file_report(file_path, &owned, index, pf_opt, &global_id_to_entity);
            let rel_path = format!("{file_path}.txt");
            let full_path = base_dir.join(&rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
            paths.push(full_path);
        }

        for dir_path in &dirs {
            let content = render_directory_report(dir_path, &ordered, index);
            let rel_path = format!("{dir_path}.dir.txt");
            let full_path = base_dir.join(&rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
            paths.push(full_path);
        }

        Ok(paths)
    }

    /// Write symbol table only (legacy single-file aggregated).
    pub fn write_symbol_table(&self, index: &RelationIndex) -> std::io::Result<std::path::PathBuf> {
        let content = render_symbol_table(index, &self.project_name);
        self.manager.write("SYMBOL_TABLE.md", &content)
    }

    /// Write relations only (legacy single-file aggregated).
    pub fn write_relations(&self, index: &RelationIndex) -> std::io::Result<std::path::PathBuf> {
        let content = render_relations(index, &self.project_name);
        self.manager.write("RELATIONS.md", &content)
    }

    /// Write type inference only (from parsed files, legacy aggregated).
    pub fn write_type_inference(
        &self,
        files: &[ParsedFile],
    ) -> std::io::Result<std::path::PathBuf> {
        let content = render_type_inference(files, &self.project_name);
        self.manager.write("TYPE_INFERENCE.md", &content)
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn escape_md(s: &str) -> String {
    // Escape pipe for markdown tables
    s.replace('|', "\\|").replace('\n', " ").replace('\r', "")
}
