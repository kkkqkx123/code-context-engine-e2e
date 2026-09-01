//! Type inference output generator.
//!
//! Generates formatted symbol table and relation output for human review,
//! modeled after the `export_rs.rs` example. Outputs markdown-formatted
//! tables that can be used for debugging and verification.
//!
//! This module is kept for backward compatibility. New code should use
//! [`crate::structured_output`] which provides the full structured framework.

use cce_relation::RelationIndex;
use cce_types::ParsedFile;

/// Format a symbol table from a relation index as markdown.
///
/// Delegates to [`crate::structured_output::render_symbol_table`].
pub fn render_symbol_table(index: &RelationIndex, project_name: &str) -> String {
    crate::structured_output::render_symbol_table(index, project_name)
}

/// Format call graph relations from a relation index as markdown.
///
/// Delegates to [`crate::structured_output::render_relations`] and extracts
/// the call-graph section. Prefer [`crate::structured_output::render_relations`]
/// for the full output.
pub fn render_call_graph(index: &RelationIndex, project_name: &str) -> String {
    crate::structured_output::render_relations(index, project_name)
}

/// Render a complete type inference report for a project.
///
/// Delegates to [`crate::structured_output::render_summary`] plus the detailed
/// symbol and relation tables. For per-file variable inference use
/// [`crate::structured_output::render_type_inference`].
pub fn render_type_inference_report(index: &RelationIndex, project_name: &str) -> String {
    crate::structured_output::render_summary(index, &[], project_name)
}

/// Render type inference from parsed files (full detail).
///
/// Uses the same engine as the resolver so variable and return types reflect
/// the actual disambiguation context.
pub fn render_type_inference_from_files(files: &[ParsedFile], project_name: &str) -> String {
    crate::structured_output::render_type_inference(files, project_name)
}
