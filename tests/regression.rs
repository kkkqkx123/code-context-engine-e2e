//! Regression test entry point.
//!
//! Regression tests live under `tests/regression/`.

#[path = "regression/bm25_parity.rs"]
pub mod bm25_parity;
#[path = "regression/e2e_outputs.rs"]
pub mod e2e_outputs;
#[path = "regression/helper.rs"]
pub mod helper;
#[path = "regression/rust_relation.rs"]
pub mod rust_relation;
#[path = "regression/rust_relation_diamond.rs"]
pub mod rust_relation_diamond;
#[path = "regression/rust_sidecar.rs"]
pub mod rust_sidecar;
#[path = "regression/test_marker_language_coverage.rs"]
pub mod test_marker_language_coverage;
#[path = "regression/type_inference_integration.rs"]
pub mod type_inference_integration;
