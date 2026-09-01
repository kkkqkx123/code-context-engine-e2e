//! Relevance judgment definitions and benchmark evaluation infrastructure.
//!
//! Sub-modules:
//! - `flask`, `once_cell`, `ripgrep`: Ground-truth judgment definitions
//! - `evaluate`: Shared evaluation logic (scoring, ranking, file-doc metrics)
//! - `report`: Shared report formatting (CSV, Markdown, chunk dumping)

pub mod evaluate;
pub mod flask;
pub mod once_cell;
pub mod report;
pub mod ripgrep;

pub use flask::flask_relevance_judgments;
pub use once_cell::once_cell_relevance_judgments;
pub use ripgrep::ripgrep_relevance_judgments;
