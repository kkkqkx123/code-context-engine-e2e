//! Type inference integration tests.
//!
//! Tests the full pipeline: source code parsing -> type inference -> symbol table construction.
//! Each language fixture contains realistic code snippets exercising type inference patterns.
//!
//! Indexing smoke tests use the orchestrator; precise type assertions parse the
//! fixture with [`ParseCoordinator`](cce_parser::parser::ParseCoordinator) and
//! check the canonical snapshot from
//! [`collect_type_bindings`](cce_e2e_tests::type_inference_assert::collect_type_bindings).
//! Run `cargo run -p cce-e2e-tests --example export_type_inference` to regenerate
//! the human-readable `TYPE_INFERENCE.md` reports for visual inspection.

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

// ==================== Snapshot assertion helpers ====================

/// Parse every file in a loaded fixture with the same tree-sitter pipeline
/// used by the indexer and return the canonical type snapshot.
fn snapshot_for(
    fixture: &TestFixture,
) -> Vec<cce_e2e_tests::type_inference_assert::CanonicalTypeBinding> {
    use cce_parser::parser::ParseCoordinator;

    let mut files = Vec::new();
    let mut stack = vec![fixture.root_path().to_path_buf()];
    let mut coordinator = ParseCoordinator::new();
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).expect("fixture dir should be readable");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(content) = std::fs::read_to_string(&path) {
                let rel = path
                    .strip_prefix(fixture.root_path())
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if let Ok(parsed) = coordinator.parse(&rel, &content) {
                    files.push(parsed);
                }
            }
        }
    }
    assert!(!files.is_empty(), "Fixture should yield parsed files");
    cce_e2e_tests::type_inference_assert::collect_type_bindings(&files)
}

// ==================== New language coverage ====================

#[test]
fn test_c_declarations_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_return_has_type;

    init_minimal_logging();
    let fixture = TestFixture::c_type_inference_declarations().expect("Failed to load c fixture");
    let bindings = snapshot_for(&fixture);

    // Locals inside `main` are not extracted as entities for C; the
    // observable surface is function return types (including typedef
    // aliases), which verifies C inferer dispatch end to end.
    assert_return_has_type(&bindings, "add", "int");
    assert_return_has_type(&bindings, "distance", "double");
    assert_return_has_type(&bindings, "count_items", "size_alias");
    assert_return_has_type(&bindings, "main", "int");
}

#[test]
fn test_cpp_declarations_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_return_has_type;

    init_minimal_logging();
    let fixture =
        TestFixture::cpp_type_inference_declarations().expect("Failed to load cpp fixture");
    let bindings = snapshot_for(&fixture);

    // Locals inside `main` are not extracted as entities for C++; the
    // observable surface is function return types (including templates).
    assert_return_has_type(&bindings, "process_items", "int");
    assert_return_has_type(&bindings, "to_map", "std::map");
    assert_return_has_type(&bindings, "identity", "T");
    assert_return_has_type(&bindings, "main", "int");
}

#[test]
fn test_kotlin_control_flow_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_narrowed_has_type;

    init_minimal_logging();
    // NB: expression-body functions and `val` metadata are not captured by
    // the Kotlin extractor yet, so `generics.kt` is index-smoke only (see
    // `test_new_language_fixtures_index`). Narrowing via `is`/`when` works.
    let fixture = TestFixture::kotlin_type_inference_control_flow()
        .expect("Failed to load kotlin control flow fixture");
    let bindings = snapshot_for(&fixture);
    assert_narrowed_has_type(&bindings, "value", "String");
    assert_narrowed_has_type(&bindings, "result", "Result.Success");
}

#[test]
fn test_scala_control_flow_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_narrowed_has_type;

    init_minimal_logging();
    // NB: `val`/return metadata is not captured by the Scala extractor yet,
    // so `declarations.scala` is index-smoke only. `match`/`isInstanceOf`
    // narrowing works.
    let fixture = TestFixture::scala_type_inference_control_flow()
        .expect("Failed to load scala control flow fixture");
    let bindings = snapshot_for(&fixture);
    assert_narrowed_has_type(&bindings, "c", "Circle");
}

#[test]
fn test_ruby_constructors_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_variable_has_type;

    init_minimal_logging();
    let fixture =
        TestFixture::ruby_type_inference_constructors().expect("Failed to load ruby fixture");
    let bindings = snapshot_for(&fixture);

    assert_variable_has_type(&bindings, "user", "User");
    assert_variable_has_type(&bindings, "calc", "Calculator");
}

#[test]
fn test_php_phpdoc_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_variable_has_type;

    init_minimal_logging();
    let fixture = TestFixture::php_type_inference_phpdoc().expect("Failed to load php fixture");
    let bindings = snapshot_for(&fixture);

    // Observable today: property annotations and literal types.
    // `new User(...)` constructor metadata is not captured yet.
    assert_variable_has_type(&bindings, "age", "int");
    assert_variable_has_type(&bindings, "greeting", "string");
}

#[test]
fn test_javascript_narrowing_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_return_has_type;

    init_minimal_logging();
    let fixture =
        TestFixture::javascript_type_inference_narrowing().expect("Failed to load js fixture");
    let bindings = snapshot_for(&fixture);

    // Return-type harvesting works; `typeof`/`instanceof` narrowing does not
    // produce bindings for this fixture yet.
    assert_return_has_type(&bindings, "handleResult", "unknown");
    assert_return_has_type(&bindings, "process", "falsy");
    assert_return_has_type(&bindings, "createUser", "kind");
}

#[test]
fn test_python_cross_file_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_return_has_type;

    init_minimal_logging();
    let fixture =
        TestFixture::python_type_inference_cross_file().expect("Failed to load cross-file fixture");
    let bindings = snapshot_for(&fixture);

    assert_return_has_type(&bindings, "load_user", "User");
    assert_return_has_type(&bindings, "render_greeting", "str");
}

#[tokio::test]
async fn test_new_language_fixtures_index() {
    init_minimal_logging();

    for (fixture, extensions) in [
        (
            TestFixture::c_type_inference_declarations().expect("Failed to load c fixture"),
            vec!["c".to_string(), "h".to_string()],
        ),
        (
            TestFixture::c_basic().expect("Failed to load c basic fixture"),
            vec!["c".to_string(), "h".to_string()],
        ),
        (
            TestFixture::bash_basic().expect("Failed to load bash fixture"),
            vec!["sh".to_string()],
        ),
        (
            TestFixture::lua_basic().expect("Failed to load lua fixture"),
            vec!["lua".to_string()],
        ),
        (
            TestFixture::cpp_type_inference_declarations().expect("Failed to load cpp fixture"),
            vec!["cpp".to_string()],
        ),
        (
            TestFixture::kotlin_type_inference_generics().expect("Failed to load kotlin fixture"),
            vec!["kt".to_string()],
        ),
        (
            TestFixture::scala_type_inference_declarations().expect("Failed to load scala fixture"),
            vec!["scala".to_string()],
        ),
        (
            TestFixture::ruby_type_inference_constructors().expect("Failed to load ruby fixture"),
            vec!["rb".to_string()],
        ),
        (
            TestFixture::php_type_inference_phpdoc().expect("Failed to load php fixture"),
            vec!["php".to_string()],
        ),
        (
            TestFixture::dart_type_inference_declarations().expect("Failed to load dart fixture"),
            vec!["dart".to_string()],
        ),
        (
            TestFixture::javascript_type_inference_narrowing().expect("Failed to load js fixture"),
            vec!["js".to_string()],
        ),
        (
            TestFixture::typescript_basic().expect("Failed to load typescript basic fixture"),
            vec!["ts".to_string()],
        ),
        (
            TestFixture::javascript_basic().expect("Failed to load javascript basic fixture"),
            vec!["js".to_string()],
        ),
        (
            TestFixture::cpp_basic().expect("Failed to load cpp basic fixture"),
            vec!["cpp".to_string(), "h".to_string()],
        ),
    ] {
        let orchestrator = run_index(fixture, extensions).await;
        let relation_index = orchestrator
            .get_relation_index()
            .expect("Relation index should be available");
        assert!(
            relation_index.file_count() >= 1,
            "Should have at least one file indexed"
        );
    }
}

#[tokio::test]
async fn test_typescript_basic_resolves_cross_file_calls() {
    init_minimal_logging();

    let fixture = TestFixture::typescript_basic().expect("Failed to load typescript basic fixture");
    let orchestrator = run_index(fixture, vec!["ts".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 3,
        "typescript basic should index calculator, string_utils and main"
    );
    assert!(
        !relation_index.function_index().is_empty(),
        "typescript basic should extract function entities"
    );
}

#[tokio::test]
async fn test_javascript_basic_resolves_cross_file_calls() {
    init_minimal_logging();

    let fixture = TestFixture::javascript_basic().expect("Failed to load javascript basic fixture");
    let orchestrator = run_index(fixture, vec!["js".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 3,
        "javascript basic should index calculator, stringUtils and main"
    );
    assert!(
        !relation_index.function_index().is_empty(),
        "javascript basic should extract function entities"
    );
}

#[tokio::test]
async fn test_cpp_basic_resolves_header_source_calls() {
    init_minimal_logging();

    let fixture = TestFixture::cpp_basic().expect("Failed to load cpp basic fixture");
    let orchestrator = run_index(fixture, vec!["cpp".to_string(), "h".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 3,
        "cpp basic should index header, implementation and main"
    );
    assert!(
        !relation_index.function_index().is_empty(),
        "cpp basic should extract function entities"
    );
}

#[test]
fn test_typescript_cross_file_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_return_has_type;

    init_minimal_logging();
    let fixture = TestFixture::typescript_type_inference_cross_file()
        .expect("Failed to load typescript cross-file fixture");
    let bindings = snapshot_for(&fixture);

    assert_return_has_type(&bindings, "loadUser", "User");
    assert_return_has_type(&bindings, "renderGreeting", "string");
}

#[tokio::test]
async fn test_typescript_cross_file_resolves_calls() {
    init_minimal_logging();

    let fixture = TestFixture::typescript_type_inference_cross_file()
        .expect("Failed to load typescript cross-file fixture");
    let orchestrator = run_index(fixture, vec!["ts".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 2,
        "typescript cross-file should index models and service"
    );
    assert!(
        !relation_index.function_index().is_empty(),
        "typescript cross-file should extract function entities"
    );
    assert!(
        relation_index.resolved_relation_count() >= 1,
        "typescript cross-file should resolve at least one call edge"
    );
}

#[tokio::test]
async fn test_javascript_cross_file_resolves_calls() {
    init_minimal_logging();

    let fixture = TestFixture::javascript_type_inference_cross_file()
        .expect("Failed to load javascript cross-file fixture");
    let orchestrator = run_index(fixture, vec!["js".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 2,
        "javascript cross-file should index models and service"
    );
    assert!(
        !relation_index.function_index().is_empty(),
        "javascript cross-file should extract function entities"
    );
    assert!(
        relation_index.resolved_relation_count() >= 1,
        "javascript cross-file should resolve at least one call edge"
    );
}

#[tokio::test]
async fn test_scala_basic_resolves_calls() {
    init_minimal_logging();

    let fixture = TestFixture::scala_basic().expect("Failed to load scala basic fixture");
    let orchestrator = run_index(fixture, vec!["scala".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 2,
        "scala basic should index Models and Main"
    );
    assert!(
        !relation_index.function_index().is_empty(),
        "scala basic should extract function entities"
    );
}

#[tokio::test]
async fn test_dart_basic_resolves_calls() {
    init_minimal_logging();

    let fixture = TestFixture::dart_basic().expect("Failed to load dart basic fixture");
    let orchestrator = run_index(fixture, vec!["dart".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");

    assert!(
        relation_index.file_count() >= 2,
        "dart basic should index lib and bin entrypoint"
    );
    assert!(
        !relation_index.function_index().is_empty(),
        "dart basic should extract function entities"
    );
}

#[tokio::test]
async fn test_review_fixtures_index() {
    init_minimal_logging();

    for (fixture, extensions) in [
        (
            TestFixture::kotlin_review_coroutines().expect("Failed to load kotlin review fixture"),
            vec!["kt".to_string()],
        ),
        (
            TestFixture::scala_review_case_class().expect("Failed to load scala review fixture"),
            vec!["scala".to_string()],
        ),
        (
            TestFixture::ruby_review_mixin().expect("Failed to load ruby review fixture"),
            vec!["rb".to_string()],
        ),
        (
            TestFixture::php_review_namespace_trait().expect("Failed to load php review fixture"),
            vec!["php".to_string()],
        ),
        (
            TestFixture::typescript_review_re_export()
                .expect("Failed to load typescript re-export fixture"),
            vec!["ts".to_string()],
        ),
        (
            TestFixture::typescript_review_wildcard()
                .expect("Failed to load typescript wildcard fixture"),
            vec!["ts".to_string()],
        ),
        (
            TestFixture::python_review_re_export()
                .expect("Failed to load python re-export fixture"),
            vec!["py".to_string()],
        ),
        (
            TestFixture::python_review_wildcard().expect("Failed to load python wildcard fixture"),
            vec!["py".to_string()],
        ),
    ] {
        let orchestrator = run_index(fixture, extensions).await;
        let relation_index = orchestrator
            .get_relation_index()
            .expect("Relation index should be available");
        assert!(
            relation_index.file_count() >= 2,
            "review fixture should index at least two files"
        );
        assert!(
            !relation_index.function_index().is_empty(),
            "review fixture should extract function entities"
        );
    }
}

#[tokio::test]
async fn test_re_export_chain_resolves_calls() {
    init_minimal_logging();

    for (fixture, extensions) in [
        (
            TestFixture::typescript_review_re_export()
                .expect("Failed to load typescript re-export fixture"),
            vec!["ts".to_string()],
        ),
        (
            TestFixture::python_review_re_export()
                .expect("Failed to load python re-export fixture"),
            vec!["py".to_string()],
        ),
    ] {
        let orchestrator = run_index(fixture, extensions).await;
        let relation_index = orchestrator
            .get_relation_index()
            .expect("Relation index should be available");
        assert!(
            relation_index.file_count() >= 3,
            "re-export fixture should index origin, middle and consumer"
        );
        assert!(
            relation_index.resolved_relation_count() >= 1,
            "re-export chain should resolve at least one call edge"
        );
    }
}

#[tokio::test]
async fn test_wildcard_import_resolves_calls() {
    init_minimal_logging();

    for (fixture, extensions) in [
        (
            TestFixture::typescript_review_wildcard()
                .expect("Failed to load typescript wildcard fixture"),
            vec!["ts".to_string()],
        ),
        (
            TestFixture::python_review_wildcard().expect("Failed to load python wildcard fixture"),
            vec!["py".to_string()],
        ),
    ] {
        let orchestrator = run_index(fixture, extensions).await;
        let relation_index = orchestrator
            .get_relation_index()
            .expect("Relation index should be available");
        assert!(
            relation_index.file_count() >= 2,
            "wildcard fixture should index utils and consumer"
        );
        assert!(
            relation_index.resolved_relation_count() >= 1,
            "wildcard import should resolve at least one call edge"
        );
    }
}

#[tokio::test]
async fn test_shell_and_c_review_fixtures_index() {
    init_minimal_logging();

    for (fixture, extensions) in [
        (
            TestFixture::bash_type_inference_variables()
                .expect("Failed to load bash type inference fixture"),
            vec!["sh".to_string()],
        ),
        (
            TestFixture::lua_type_inference_variables()
                .expect("Failed to load lua type inference fixture"),
            vec!["lua".to_string()],
        ),
        (
            TestFixture::c_review_macros().expect("Failed to load c review fixture"),
            vec!["c".to_string(), "h".to_string()],
        ),
        (
            TestFixture::cpp_review_templates().expect("Failed to load cpp review fixture"),
            vec!["cpp".to_string(), "h".to_string()],
        ),
        (
            TestFixture::bash_review_pipeline().expect("Failed to load bash review fixture"),
            vec!["sh".to_string()],
        ),
        (
            TestFixture::lua_review_closure().expect("Failed to load lua review fixture"),
            vec!["lua".to_string()],
        ),
        (
            TestFixture::dart_review_mixin_async().expect("Failed to load dart review fixture"),
            vec!["dart".to_string()],
        ),
    ] {
        let orchestrator = run_index(fixture, extensions).await;
        let relation_index = orchestrator
            .get_relation_index()
            .expect("Relation index should be available");
        assert!(
            relation_index.file_count() >= 1,
            "fixture should index at least one file"
        );
        assert!(
            !relation_index.function_index().is_empty(),
            "fixture should extract function entities"
        );
    }
}

#[tokio::test]
async fn test_visibility_and_overload_fixtures_index() {
    init_minimal_logging();

    for (fixture, extensions) in [
        (
            TestFixture::java_type_inference_visibility()
                .expect("Failed to load java visibility fixture"),
            vec!["java".to_string()],
        ),
        (
            TestFixture::java_type_inference_overloads()
                .expect("Failed to load java overloads fixture"),
            vec!["java".to_string()],
        ),
        (
            TestFixture::csharp_type_inference_overloads()
                .expect("Failed to load csharp overloads fixture"),
            vec!["cs".to_string()],
        ),
        (
            TestFixture::typescript_type_inference_overloads()
                .expect("Failed to load typescript overloads fixture"),
            vec!["ts".to_string()],
        ),
    ] {
        let orchestrator = run_index(fixture, extensions).await;
        let relation_index = orchestrator
            .get_relation_index()
            .expect("Relation index should be available");
        assert!(
            relation_index.file_count() >= 1,
            "visibility/overload fixture should index at least one file"
        );
        assert!(
            !relation_index.function_index().is_empty(),
            "visibility/overload fixture should extract function entities"
        );
        assert!(
            relation_index.resolved_relation_count() >= 1,
            "visibility/overload fixture should resolve at least one call edge"
        );
    }
}
