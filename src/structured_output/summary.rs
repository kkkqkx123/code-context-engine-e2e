//! Summary rendering for structured output.
//!
//! Provides `render_summary` for the aggregated markdown project summary
//! and `collect_summary` for JSON export.

use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;

use cce_relation::RelationIndex;
use cce_relation::index::{EntityIndexOps, FileIndexOps, RelationQueryOps};
use cce_types::{ParsedFile, RelationType};

use super::types::ProjectSummary;

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

    let call_count = relation_count;
    let type_rel_count = {
        let mut n = 0usize;
        for entry in index.resolved_relation_index().iter() {
            for r in entry.value().iter() {
                if matches!(
                    r.relation_type,
                    RelationType::Inheritance
                        | RelationType::Implementation
                        | RelationType::TraitBound
                        | RelationType::TypeReference
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
                    RelationType::Inheritance
                        | RelationType::Implementation
                        | RelationType::TraitBound
                        | RelationType::TypeReference
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
