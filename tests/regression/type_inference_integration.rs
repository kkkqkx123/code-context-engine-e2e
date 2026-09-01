//! Type inference integration tests.
//!
//! Tests the full pipeline: source code parsing -> type inference -> symbol table construction.
//! Each language fixture contains realistic code snippets exercising type inference patterns.

use cce_orchestrator::{IndexOptions, IndexOrchestrator};
use cce_relation::index::{EntityIndexOps, FileIndexOps, RelationQueryOps};

use crate::helper::{TestFixture, init_minimal_logging};

/// Helper to run the full indexing pipeline on a fixture.
async fn run_index(fixture: TestFixture, extensions: Vec<String>) -> IndexOrchestrator {
    let project_root = fixture.root_path().to_path_buf();

    let options = IndexOptions {
        root_dir: project_root.clone(),
        extensions,
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
    orchestrator
        .execute(options)
        .await
        .expect("Indexing should succeed");
    orchestrator
}

// ==================== Rust type inference tests ====================

#[tokio::test]
async fn test_rust_generics_type_inference() {
    init_minimal_logging();

    let fixture =
        TestFixture::rust_type_inference_generics().expect("Failed to load rust generics fixture");
    let orchestrator = run_index(fixture, vec!["rs".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );
    let _relation_count = relation_index.resolved_relation_count();

    let function_ids = relation_index.function_index();
    assert!(
        !function_ids.is_empty(),
        "Should have extracted function entities"
    );
}

#[tokio::test]
async fn test_rust_control_flow_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::rust_type_inference_control_flow()
        .expect("Failed to load rust control flow fixture");
    let orchestrator = run_index(fixture, vec!["rs".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );

    let function_ids = relation_index.function_index();
    assert!(
        !function_ids.is_empty(),
        "Should have extracted function entities from control flow fixture"
    );
}

// ==================== Python type inference tests ====================

#[tokio::test]
async fn test_python_type_hints_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::python_type_inference_type_hints()
        .expect("Failed to load python type hints fixture");
    let orchestrator = run_index(fixture, vec!["py".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );

    let function_ids = relation_index.function_index();
    assert!(
        !function_ids.is_empty(),
        "Should have extracted function entities from Python type hints"
    );
}

#[tokio::test]
async fn test_python_control_flow_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::python_type_inference_control_flow()
        .expect("Failed to load python control flow fixture");
    let orchestrator = run_index(fixture, vec!["py".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );
}

// ==================== TypeScript type inference tests ====================

#[tokio::test]
async fn test_typescript_generics_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::typescript_type_inference_generics()
        .expect("Failed to load typescript generics fixture");
    let orchestrator = run_index(fixture, vec!["ts".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );

    let function_ids = relation_index.function_index();
    assert!(
        !function_ids.is_empty(),
        "Should have extracted function entities from TypeScript generics"
    );
}

#[tokio::test]
async fn test_typescript_unions_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::typescript_type_inference_unions()
        .expect("Failed to load typescript unions fixture");
    let orchestrator = run_index(fixture, vec!["ts".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );
}

// ==================== Java type inference tests ====================

#[tokio::test]
async fn test_java_generics_type_inference() {
    init_minimal_logging();

    let fixture =
        TestFixture::java_type_inference_generics().expect("Failed to load java generics fixture");
    let orchestrator = run_index(fixture, vec!["java".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );

    let function_ids = relation_index.function_index();
    assert!(
        !function_ids.is_empty(),
        "Should have extracted function entities from Java generics"
    );
}

#[tokio::test]
async fn test_java_control_flow_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::java_type_inference_control_flow()
        .expect("Failed to load java control flow fixture");
    let orchestrator = run_index(fixture, vec!["java".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );
}

// ==================== C# type inference tests ====================

#[tokio::test]
async fn test_csharp_generics_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::csharp_type_inference_generics()
        .expect("Failed to load csharp generics fixture");
    let orchestrator = run_index(fixture, vec!["cs".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );

    let function_ids = relation_index.function_index();
    assert!(
        !function_ids.is_empty(),
        "Should have extracted function entities from C# generics"
    );
}

#[tokio::test]
async fn test_csharp_control_flow_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::csharp_type_inference_control_flow()
        .expect("Failed to load csharp control flow fixture");
    let orchestrator = run_index(fixture, vec!["cs".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );
}

// ==================== Go type inference tests ====================

#[tokio::test]
async fn test_go_interfaces_type_inference() {
    init_minimal_logging();

    let fixture =
        TestFixture::go_type_inference_interfaces().expect("Failed to load go interfaces fixture");
    let orchestrator = run_index(fixture, vec!["go".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );

    let function_ids = relation_index.function_index();
    assert!(
        !function_ids.is_empty(),
        "Should have extracted function entities from Go interfaces"
    );
}

#[tokio::test]
async fn test_go_control_flow_type_inference() {
    init_minimal_logging();

    let fixture = TestFixture::go_type_inference_control_flow()
        .expect("Failed to load go control flow fixture");
    let orchestrator = run_index(fixture, vec!["go".to_string()]).await;

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 1,
        "Should have at least one file indexed"
    );
}
