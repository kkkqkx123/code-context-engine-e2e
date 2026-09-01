//! Workflow test entry point.
//!
//! The alignment workflow tests live under `tests/e2e/` and are included
//! here. The remaining e2e modules are not yet wired in (the suite drifted
//! while orphaned).

#[path = "e2e/alignment_workflow.rs"]
pub mod alignment_workflow;
#[path = "e2e/helper.rs"]
pub mod helper;
#[path = "e2e/plugin_workflow.rs"]
pub mod plugin_workflow;
#[path = "e2e/rerank_workflow.rs"]
pub mod rerank_workflow;

#[path = "e2e/retrieval_method_alignment.rs"]
pub mod retrieval_method_alignment;

// The hot-update suite (change detection, processor chain, resume/recovery)
// runs under the workflow test binary.
#[path = "e2e/hot_update.rs"]
pub mod hot_update;
// The index workflow helper used by the hot-update suite.
#[path = "e2e/index.rs"]
pub mod index;
