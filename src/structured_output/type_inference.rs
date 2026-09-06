//! Type inference rendering for structured output.
//!
//! Provides `render_type_inference` and `render_type_inference_with_index`
//! for aggregated markdown type inference reports.

use std::collections::HashSet;
use std::fmt::Write as _;

use cce_relation::RelationIndex;
use cce_types::ParsedFile;

use super::types::{escape_md, span_str_from_span};

/// Render type inference results from parsed files.
///
/// For each file the full per-file `TypeInferenceEngine` is re-run so the
/// output reflects the same inference used by the relation resolver.
pub fn render_type_inference(files: &[ParsedFile], project_name: &str) -> String {
    render_type_inference_with_index(files, None, project_name)
}

/// Render type inference with an optional relation index for member index.
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

    let project_contexts = crate::type_inference_assert::infer_project_contexts(files);

    for (file, merged) in files.iter().zip(project_contexts.iter()) {
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
            let func_name = super::types::resolve_inferred_return_name(
                eid,
                Some(file),
                &[],
                &std::collections::BTreeMap::new(),
            );
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
