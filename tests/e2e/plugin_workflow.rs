//! Plugin system workflow tests
//!
//! Tests for the complete plugin system workflow from loading to execution.
//! These tests verify that the flask_routes.lua plugin works correctly with plugins.json configuration.

use crate::helper::{
    EmptyFixture, ExpectedIndexResult, IndexWorkflowTest, assert_index_result, init_minimal_logging,
};
use cce_plugin::PluginRegistry;
use cce_plugin_runtime::{FilePluginSource, LuaPlugin};
use cce_types::grouper::EntityGroup;
use cce_types::{EntityKind, Language, ParsedFile};
use compact_str::CompactString;
use std::collections::HashMap;
use std::path::Path;

/// Basic plugin loading from plugins.json
#[tokio::test]
async fn test_plugin_loading_from_json() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Load plugins from .cce/plugins.json via FilePluginSource
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    let result = registry.load_source(&source);
    assert!(
        result.is_ok(),
        "Failed to load plugins from source: {:?}",
        result.err()
    );

    // Verify that the flask_route_plugin is loaded
    let bm25_generators = registry.get_bm25_generators(Some("app.py"), Some("python"));
    let emb_generators = registry.get_embedding_generators(Some("app.py"), Some("python"));
    assert!(
        !bm25_generators.is_empty() || !emb_generators.is_empty(),
        "Should have loaded NL generators (BM25 or embedding)"
    );
}

/// Plugin natural language generation
#[tokio::test]
async fn test_plugin_nl_generation() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    // Create an EntityGroup with Flask metadata
    let mut metadata = HashMap::new();
    metadata.insert("endpoint".to_string(), "/api/users".to_string());
    metadata.insert("methods".to_string(), "GET,POST".to_string());

    let entity_group = EntityGroup {
        group_id: CompactString::from("flask_route_group"),
        name: CompactString::from("get_users"),
        kind: EntityKind::Function,
        language: Language::Python,
        metadata,
        ..Default::default()
    };

    let bm25_generators = registry.get_bm25_generators(None, None);
    let emb_generators = registry.get_embedding_generators(None, None);

    // Test BM25 text generation
    if !bm25_generators.is_empty() {
        let bm25_result = bm25_generators[0].generate_bm25(&entity_group);
        assert!(bm25_result.is_ok(), "BM25 generation should succeed");

        if let Ok(Some(bm25_text)) = bm25_result {
            assert!(!bm25_text.is_empty(), "BM25 text should not be empty");

            // Verify that the generated text contains expected keywords
            assert!(bm25_text.contains("Flask") || bm25_text.contains("route"));
        }
    }

    // Test embedding text generation
    if !emb_generators.is_empty() {
        let embedding_result = emb_generators[0].generate_embedding(&entity_group);
        assert!(
            embedding_result.is_ok(),
            "Embedding generation should succeed"
        );

        if let Ok(Some(embedding_text)) = embedding_result {
            assert!(
                !embedding_text.is_empty(),
                "Embedding text should not be empty"
            );

            // Verify that the generated text contains expected keywords
            assert!(embedding_text.contains("endpoint") || embedding_text.contains("web"));
        }
    }
}

/// Plugin file filtering functionality
///
/// File-level filtering is handled at the registry level via file
/// patterns and language constraints.
#[tokio::test]
async fn test_plugin_file_filtering() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    // Verify generators are loaded from plugin configuration
    let generators = registry.get_bm25_generators(Some("app.py"), Some("python"));
    assert!(
        !generators.is_empty(),
        "Should have loaded generators from plugin config"
    );

    // Verify plugin metadata
    let plugin = &generators[0];
    let meta = plugin.metadata();
    assert_eq!(meta.id, "flask_route_plugin", "Plugin ID should match");
    assert!(
        meta.name.contains("Flask") || meta.name.contains("Route"),
        "Plugin name should reference Flask routes"
    );
}

/// Plugin integration with index workflow
///
/// The `flask_route_plugin` (TextGen + EntityExtract, priority 10, `*.py`)
/// is mounted into the real index pipeline. Its BM25 text — the
/// "Flask route handler function" marker — must land in the stored BM25
/// documents, proving the plugin → index → storage chain works end-to-end.
#[tokio::test]
async fn test_plugin_integration_with_index() {
    init_minimal_logging();

    let fixture = EmptyFixture::new().expect("Failed to create fixture");
    fixture
        .add_file(
            "app.py",
            r#"
from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/api/users', methods=['GET'])
def get_users():
    """Get all users from database"""
    users = [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]
    return jsonify(users)

@app.route('/api/users/<int:user_id>', methods=['GET'])
def get_user(user_id):
    """Get a specific user by ID"""
    # Simulate user lookup
    user = {"id": user_id, "name": f"User {user_id}"}
    return jsonify(user)
"#,
        )
        .expect("Failed to add Python file");

    // Load the real demo plugins (flask_route_plugin TextGen/EntityExtract).
    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins from source");
    assert!(
        !registry
            .get_plugins(
                cce_plugin::PluginCapability::TextGen,
                Some("app.py"),
                Some("python")
            )
            .is_empty(),
        "flask_route_plugin must match app.py"
    );

    // External BM25 client so the stored documents can be inspected after
    // indexing.
    let bm25_temp = tempfile::TempDir::new().expect("Failed to create BM25 temp dir");
    let bm25_path = bm25_temp
        .path()
        .to_str()
        .expect("BM25 path is not valid UTF-8")
        .to_string();
    let bm25_config = cce_storage_bm25::Bm25Config::default()
        .enabled()
        .with_index_name("default")
        .with_index_path(&bm25_path);
    let mut bm25 = cce_storage_bm25::Bm25Client::new(bm25_config);
    bm25.connect().await.expect("Failed to connect BM25 client");
    let bm25 = std::sync::Arc::new(tokio::sync::Mutex::new(bm25));

    let mut index_test = IndexWorkflowTest::new(fixture.into_test_fixture())
        .with_extensions(vec!["py".to_string()])
        .with_relations(false)
        .with_vectors(false)
        .with_bm25(true)
        .with_plugin_registry(std::sync::Arc::new(registry))
        .with_bm25_client(bm25.clone());

    let result = index_test.execute().await.expect("Index failed");

    assert_index_result(
        &result,
        ExpectedIndexResult {
            min_files: Some(1),
            no_errors: true,
            ..Default::default()
        },
    );

    // The plugin-generated BM25 text must be present in the stored index.
    // Content is an index-only field (read back from SQLite at query time),
    // so we assert via a keyword search over the plugin marker phrase.
    let guard = bm25.lock().await;
    let count = guard
        .document_count_by_project(1)
        .await
        .expect("Failed to count BM25 documents");
    assert!(count > 0, "BM25 index must contain documents");

    let manager = guard
        .index_manager()
        .cloned()
        .expect("BM25 index manager must be available");
    let manager_guard = manager.read().await;
    let search_results = cce_storage_bm25::Bm25Retrieval::new()
        .search(
            &manager_guard,
            guard.schema(),
            "route handler function",
            &cce_storage_bm25::Bm25SearchOptions {
                limit: 10,
                offset: 0,
                field_weights: HashMap::new(),
                highlight: false,
                project_id: 1,
                epochs: Vec::new(),
                excluded_files: None,
                exclude_test: false,
                include_categories: Vec::new(),
                exclude_categories: Vec::new(),
                term_operator: Default::default(),
            },
        )
        .expect("BM25 search must succeed");

    assert!(
        search_results
            .iter()
            .any(|r| r.document_id.contains("bm25")),
        "the plugin marker phrase must be searchable in the BM25 index, got: {:?}",
        search_results
            .iter()
            .map(|r| r.document_id.as_str())
            .collect::<Vec<_>>()
    );
}

/// Index-level fault tolerance against a broken plugin.
///
/// A `TextGen` Lua plugin that raises at runtime must not take down the whole
/// index operation: the affected file falls back to the built-in text
/// generation and the run completes with no errors.
#[tokio::test]
async fn test_plugin_runtime_error_only_fails_its_file() {
    init_minimal_logging();

    let fixture = EmptyFixture::new().expect("Failed to create fixture");
    fixture
        .add_file(
            "app.py",
            r#"
@app.route('/api/users', methods=['GET'])
def get_users():
    return "ok"
"#,
        )
        .expect("Failed to add Python file");
    fixture
        .add_file(
            "src/lib.rs",
            r#"
pub fn compute() -> i32 { 42 }
"#,
        )
        .expect("Failed to add Rust file");

    // A syntactically valid Lua plugin that always raises at runtime.
    let broken_script = r#"
plugin = {
    id = "broken_generator",
    name = "Broken Generator",
    version = "0.1.0",
    capabilities = { "text_gen" }
}
function plugin.generate_bm25(group)
    error("intentional boom")
end
function plugin.generate_embedding(group)
    error("intentional boom")
end
"#;
    let broken = cce_plugin_runtime::LuaPlugin::from_script(broken_script)
        .expect("broken plugin script must load");
    let mut registry = PluginRegistry::new();
    registry.register(std::sync::Arc::new(broken));

    let mut index_test = IndexWorkflowTest::new(fixture.into_test_fixture())
        .with_extensions(vec!["py".to_string(), "rs".to_string()])
        .with_relations(false)
        .with_vectors(false)
        .with_bm25(true)
        .with_plugin_registry(std::sync::Arc::new(registry));

    let result = index_test.execute().await.expect("Index failed");

    assert_index_result(
        &result,
        ExpectedIndexResult {
            min_files: Some(2),
            no_errors: true,
            ..Default::default()
        },
    );
}

/// Plugin error handling and recovery
#[tokio::test]
async fn test_plugin_error_handling() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // Test with invalid plugin path
    let source = FilePluginSource::from_project(project_root, Some("nonexistent_plugins.json"));
    let result = registry.load_source(&source);
    assert!(
        result.is_ok(),
        "Should handle missing plugin registry gracefully"
    );

    // Verify that no plugins are loaded when registry is missing
    let generators = registry.get_bm25_generators(None, None);
    assert_eq!(
        generators.len(),
        0,
        "No generators should be loaded from missing registry"
    );
}

/// Plugin disable functionality
#[tokio::test]
async fn test_plugin_disable_functionality() {
    init_minimal_logging();

    let registry = PluginRegistry::new();

    // Test with plugin system disabled — the caller is expected to
    // skip loading entirely when disabled, so we verify an empty registry.
    let bm25_generators = registry.get_bm25_generators(None, None);
    assert_eq!(
        bm25_generators.len(),
        0,
        "No BM25 generators should be loaded when plugin system is disabled"
    );

    let emb_generators = registry.get_embedding_generators(None, None);
    assert_eq!(
        emb_generators.len(),
        0,
        "No embedding generators should be loaded when plugin system is disabled"
    );
}

/// EntityExtract — Flask routes injected as entities
///
/// The flask_routes.lua plugin declares regex patterns; the host compiles
/// them and injects each match (a route) as a standalone group that flows
/// through the grouper → NL → chunker pipeline.
#[tokio::test]
async fn test_plugin_entity_extract_routes() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let pipeline = cce_parser::grouper::PreprocessingPipeline::new()
        .with_plugin_registry(std::sync::Arc::new(registry));
    let content = "@app.route('/users')\ndef users():\n    pass\n";
    let parsed = cce_types::ParsedFile::new(Language::Python, "app.py".to_string(), content);
    let result = pipeline.process(&parsed);

    let names: Vec<String> = result
        .groups
        .iter()
        .flat_map(|g| {
            let mut names = Vec::new();
            if let Some(ref h) = g.header {
                names.push(h.name.clone());
            }
            names.extend(g.members.iter().map(|m| m.name.clone()));
            names
        })
        .collect();
    assert!(
        names.iter().any(|n| n == "/users"),
        "Flask route /users should be extracted as an entity"
    );
}

/// FormatParse — proto document format
///
/// The proto_format.lua plugin parses `.proto` files into document chunks.
#[tokio::test]
async fn test_plugin_format_parse_proto() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let router = cce_parser::document::PipelineRouter::new();
    let config = cce_config::modules::ChunkingConfig::default();
    let (chunks, _) = router
        .process_with_plugins(
            "message User {}\nservice UserService {}\n",
            "api.proto",
            &config,
            cce_types::ast_to_nl::options::OutputMode::Both,
            &registry,
        )
        .expect("proto document should be processed by the plugin");

    assert!(!chunks.is_empty());
    assert!(chunks.iter().all(|c| c.metadata.is_document()));
    let combined: String = chunks
        .iter()
        .map(|c| c.text.clone())
        .collect::<Vec<_>>()
        .join("");
    assert!(combined.contains("User"));
}

/// Rerank — plugin reranking of candidates
#[tokio::test]
async fn test_plugin_rerank() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let rerankers = registry.get_plugins(cce_plugin::PluginCapability::Rerank, None, None);
    assert!(
        !rerankers.is_empty(),
        "rerank_plugin should be loaded from plugins.json"
    );

    let candidates = vec![cce_types::RerankCandidate {
        id: "c1".to_string(),
        content: "function handles user login".to_string(),
        file_path: "app.py".to_string(),
        initial_score: 0.5,
        entity_type: Some("function".to_string()),
        metadata: HashMap::new(),
    }];
    let result = rerankers[0]
        .rerank("login", candidates)
        .expect("rerank should succeed");
    let result = result.expect("rerank should produce a result");
    assert_eq!(result.reranked_candidates.len(), 1);
    assert_eq!(result.reranked_candidates[0].id, "c1");
}

/// GroupOverride — full grouping override replaces built-in.
///
/// group_override.lua applies to `*.module` files and returns per-module
/// groups; every produced group must originate from the plugin.
#[tokio::test]
async fn test_plugin_group_override() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let overriders = registry.get_plugins(
        cce_plugin::PluginCapability::GroupOverride,
        Some("app.module"),
        None,
    );
    assert!(
        !overriders.is_empty(),
        "group_override_plugin should match *.module files"
    );

    let context = cce_types::GroupPluginContext {
        file_path: "app.module".to_string(),
        language: "python".to_string(),
        source: "def a(): pass".to_string(),
        entities: vec![cce_types::PluginEntity::new("1", "function", "a")],
        relations: Vec::new(),
    };
    let groups = overriders[0]
        .group(context)
        .expect("group override should succeed");
    assert!(
        groups.is_some(),
        "group override must produce groups for .module files"
    );
    let groups = groups.unwrap();
    assert!(!groups.is_empty());
    assert!(
        groups.iter().all(|g| g.group_id.starts_with("override_")),
        "all groups must originate from the override plugin"
    );
}

/// RelationExtract — Spring symbols/relations via Lua plugin.
#[tokio::test]
async fn test_plugin_relation_extract() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let extractors = registry.get_plugins(
        cce_plugin::PluginCapability::RelationExtract,
        Some("app/UserService.java"),
        Some("java"),
    );
    assert!(
        !extractors.is_empty(),
        "spring_relations_plugin should match *.java files"
    );

    let content =
        "@Service\nclass UserService {\n  @Autowired\n  private UserRepository repo;\n}\n";
    let symbols = extractors[0]
        .extract_symbols(content, "app/UserService.java", "java")
        .expect("extract_symbols should succeed");
    assert!(symbols.is_some());
    let symbols = symbols.unwrap();
    assert!(
        symbols.iter().any(|s| s.name == "UserService"),
        "UserService bean symbol must be extracted"
    );

    let relations = extractors[0]
        .extract_relations(content, "app/UserService.java", "java")
        .expect("extract_relations should succeed");
    assert!(relations.is_some());
}

/// QueryRewrite — query hooks plugin rewrites the query.
#[tokio::test]
async fn test_plugin_query_rewrite() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let rewritters = registry.get_plugins(cce_plugin::PluginCapability::QueryRewrite, None, None);
    assert!(
        !rewritters.is_empty(),
        "query_hooks_plugin should be loaded from plugins.json"
    );

    let result = rewritters[0]
        .rewrite_query("tf model")
        .expect("rewrite_query should succeed");
    let result = result.expect("rewrite_query should produce a result");
    assert!(
        result.rewritten_query.to_lowercase().contains("tensorflow"),
        "tf should be expanded to tensorflow"
    );
}

/// Fusion — query hooks plugin overrides fusion weights.
#[tokio::test]
async fn test_plugin_fusion_weights() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let fusion = registry.get_plugins(cce_plugin::PluginCapability::Fusion, None, None);
    assert!(
        !fusion.is_empty(),
        "query_hooks_plugin should provide Fusion"
    );

    let weights = fusion[0]
        .fusion_weights("abc", 10, 5)
        .expect("fusion_weights should succeed");
    let weights = weights.expect("fusion_weights should produce a result");
    assert_eq!(weights.vector_weight, Some(0.3));
    assert_eq!(weights.bm25_weight, Some(0.7));
}

/// ResultFilter — query hooks plugin removes noise candidates.
#[tokio::test]
async fn test_plugin_result_filter() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let filters = registry.get_plugins(cce_plugin::PluginCapability::ResultFilter, None, None);
    assert!(
        !filters.is_empty(),
        "query_hooks_plugin should provide ResultFilter"
    );

    let candidates = vec![
        cce_types::RerankCandidate {
            id: "generated1".to_string(),
            content: "code".to_string(),
            file_path: "src/generated-code/x.rs".to_string(),
            initial_score: 0.9,
            entity_type: None,
            metadata: HashMap::new(),
        },
        cce_types::RerankCandidate {
            id: "real1".to_string(),
            content: "real fn".to_string(),
            file_path: "src/app.rs".to_string(),
            initial_score: 0.8,
            entity_type: None,
            metadata: HashMap::new(),
        },
    ];
    let entries = filters[0]
        .filter_results("fn", candidates)
        .expect("filter_results should succeed");
    assert!(entries.is_some());
    let entries = entries.unwrap();
    let removed = entries
        .iter()
        .find(|e| e.id == "generated1")
        .map(|e| e.remove)
        .unwrap_or(false);
    assert!(removed, "generated-code candidate should be removed");
}

/// FileFilter — file_filter.lua excludes scratch paths.
#[tokio::test]
async fn test_plugin_file_filter() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let filters = registry.get_plugins(
        cce_plugin::PluginCapability::FileFilter,
        Some("scratch/tmp.txt"),
        None,
    );
    assert!(
        !filters.is_empty(),
        "file_filter_plugin should be loaded from plugins.json"
    );

    let decision = filters[0]
        .filter_file("scratch/tmp.txt", false, 10)
        .expect("filter_file should succeed");
    assert_eq!(
        decision,
        Some(cce_types::FileFilterDecision::Exclude),
        "scratch paths must be excluded"
    );

    let decision = filters[0]
        .filter_file("src/app.cconf", false, 10)
        .expect("filter_file should succeed");
    assert_eq!(
        decision,
        Some(cce_types::FileFilterDecision::Include),
        "custom .cconf extension must be included"
    );
}

/// SymbolExtract — zig_symbol_extract.lua extracts Zig imports
/// and exports from custom-language source.
#[tokio::test]
async fn test_plugin_symbol_extract() {
    init_minimal_logging();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    let extractors = registry.get_plugins(
        cce_plugin::PluginCapability::SymbolExtract,
        Some("src/lib.zig"),
        Some("zig"),
    );
    assert!(
        !extractors.is_empty(),
        "zig_symbol_extract_plugin should match *.zig files"
    );

    let content =
        "const std = @import(\"std\");\nconst mem = @import(\"mem.zig\");\npub fn main() void {}";
    let imports = extractors[0]
        .extract_imports(content, "src/lib.zig", "zig")
        .expect("extract_imports should succeed");
    let imports = imports.expect("imports must be extracted");
    assert_eq!(imports.len(), 2);
    assert!(
        imports.iter().any(|i| i.path == "std"),
        "std import must be extracted"
    );
    assert!(
        imports.iter().any(|i| i.path == "mem.zig"),
        "mem.zig import must be extracted"
    );

    let exports = extractors[0]
        .extract_exports(content, "src/lib.zig", "zig")
        .expect("extract_exports should succeed");
    let exports = exports.expect("exports must be extracted");
    assert_eq!(exports.len(), 1);
    assert_eq!(exports[0].name, "main");
    assert_eq!(exports[0].kind, "function");
}

/// Full JS-like parse chain (AstParser → EntityExtractor) for a language
/// whose grammar and query schemes are JavaScript (plain `JavaScript` or a
/// `LanguageRemap` custom language backed by it).
fn parse_js_like(language: &Language, path: &str, content: &str) -> ParsedFile {
    let mut parsed = ParsedFile::new(*language, path.to_string(), content);
    let (tree, _) = cce_parser::parser::ast_parser::AstParser::new()
        .parse_with_tree(content, language)
        .expect("grammar must parse the file");
    let entities = cce_parser::parser::extractor::EntityExtractor::new()
        .extract(&tree, content, language)
        .expect("entity extraction must succeed");
    for e in entities {
        parsed.add_entity(e);
    }
    parsed
}

/// LanguageRemap — a Lua plugin remaps a template DSL onto
/// the host JavaScript grammar; `.tmpl` files parse with the same entity
/// extraction as equivalent JavaScript files.
#[tokio::test]
async fn test_plugin_language_remap_lua() {
    init_minimal_logging();

    let script = r#"
        plugin = {
            id = "tmpl_remap",
            name = "Template Remap",
            language_name = "tmpl",
            language_extensions = { "tmpl" },
            remap_grammar_language = "JavaScript"
        }
    "#;
    let mut registry = PluginRegistry::new();
    registry.register(std::sync::Arc::new(
        LuaPlugin::from_script(script).expect("valid lua"),
    ));
    let count = cce_parser::tree_sitter_init::register_ast_language_plugins(
        &registry,
        cce_config::project::LanguageExtensionConflictPolicy::Allow,
        cce_config::project::GrammarAbiPolicy::Deny,
    );
    assert_eq!(count, 1, "remap plugin must register");

    let index = cce_types::language::plugin_language_for_extension("tmpl")
        .expect("extension routed to a custom language");
    let custom = Language::Custom(index);

    let content = "function greet(name) { return \"hi \" + name; }\nconst PI = 3.14;\nclass Greeter extends Base { constructor() {} }";
    let parsed = parse_js_like(&custom, "page.tmpl", content);
    assert!(
        !parsed.entities.is_empty(),
        "the remapped grammar must yield AST entities"
    );

    let pipeline = cce_parser::grouper::PreprocessingPipeline::new();
    let result = pipeline.process(&parsed);
    let names: Vec<String> = result
        .groups
        .iter()
        .flat_map(|g| {
            let mut names = Vec::new();
            if let Some(ref h) = g.header {
                names.push(h.name.to_string());
            }
            names.extend(g.members.iter().map(|m| m.name.to_string()));
            names
        })
        .collect();
    assert!(
        names.iter().any(|n| n == "greet"),
        "function entity must be extracted via the remapped JS grammar: {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "Greeter"),
        "class entity must be extracted via the remapped JS grammar: {names:?}"
    );

    // Baseline: the same content parsed as plain JavaScript must expose the
    // same entity names (the remap falls back to the JS query schemes).
    let baseline = pipeline.process(&parse_js_like(&Language::JavaScript, "page.js", content));
    let baseline_names: Vec<String> = baseline
        .groups
        .iter()
        .flat_map(|g| {
            let mut names = Vec::new();
            if let Some(ref h) = g.header {
                names.push(h.name.to_string());
            }
            names.extend(g.members.iter().map(|m| m.name.to_string()));
            names
        })
        .collect();
    assert!(
        baseline_names.iter().any(|n| n == "greet"),
        "baseline JS must also expose greet"
    );
    assert!(
        baseline_names.iter().any(|n| n == "Greeter"),
        "baseline JS must also expose Greeter"
    );
}

/// SymbolExtract — full relation-index flow for a custom
/// language.
///
/// Indexes a Zig project through the orchestrator. The `zig_symbol_extract.lua`
/// plugin supplies imports for the custom language (`Language::Custom`) on raw
/// source text, so no built-in grammar/entity extractor is needed; the imports
/// land in the relation index's per-file import tables and form the cross-file
/// dependency graph (main → math → io).
#[tokio::test]
async fn test_plugin_symbol_extract_relation_index() {
    use cce_config::RelationConfig;
    use cce_orchestrator::{IndexOptions, IndexOrchestrator};
    use cce_plugin::PluginCapability;
    use cce_relation::index::ImportIndexOps;
    use std::sync::Arc;

    init_minimal_logging();

    // Route `.zig` → `Language::Custom` using the built-in Rust grammar as a
    // stand-in so the AST pipeline parses. Query schemes are empty (zero
    // entities); `SymbolExtract` plugins operate on raw source text.
    cce_parser::tree_sitter_init::register_plugin_language_with_builtin_grammar(
        "zig",
        &["zig".to_string()],
        Language::Rust,
    );

    let fixture = EmptyFixture::new().expect("Failed to create fixture");
    fixture
        .add_file(
            "src/main.zig",
            "const std = @import(\"std\");\nconst math = @import(\"math.zig\");\npub fn main() void {}",
        )
        .expect("add main.zig");
    fixture
        .add_file(
            "src/math.zig",
            "const io = @import(\"io.zig\");\npub fn add(a: i32, b: i32) i32 { return a + b; }",
        )
        .expect("add math.zig");
    fixture
        .add_file("src/io.zig", "pub fn write(s: []const u8) void {}")
        .expect("add io.zig");
    let fixture = fixture.into_test_fixture();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");
    assert!(
        !registry
            .get_plugins(
                PluginCapability::SymbolExtract,
                Some("src/main.zig"),
                Some("zig")
            )
            .is_empty(),
        "zig_symbol_extract_plugin should match *.zig files for the custom language"
    );

    let relation_config = RelationConfig {
        plugin_symbol_extract_enabled: true,
        analyze_imports: true,
        track_cross_file_deps: true,
        ..Default::default()
    };
    let mut orchestrator = IndexOrchestrator::new(1)
        .expect("failed to create IndexOrchestrator")
        .with_relation_config(relation_config)
        .with_plugin_registry(Arc::new(registry));

    let options = IndexOptions {
        root_dir: fixture.root_path().to_path_buf(),
        extensions: vec!["zig".to_string()],
        build_relations: true,
        store_vectors: false,
        store_bm25: false,
        store_summaries: false,
        respect_gitignore: false,
        ..IndexOptions::new(fixture.root_path())
    };
    let result = orchestrator
        .execute(options)
        .await
        .expect("Indexing Zig project should succeed");
    assert!(
        result.total_files >= 3,
        "all zig files should be scanned, got {}",
        result.total_files
    );

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available after indexing");

    // main.zig imports std + math.zig through the plugin extractor.
    let main_imports = relation_index
        .get_import_table("src/main.zig")
        .expect("main.zig must have an import table");
    assert_eq!(
        main_imports.import_count(),
        2,
        "plugin imports for main.zig must land in the import table"
    );
    assert!(
        main_imports
            .standardized_imports
            .iter()
            .any(|i| i.source == "std")
    );
    assert!(
        main_imports
            .standardized_imports
            .iter()
            .any(|i| i.source == "math.zig")
    );

    // math.zig imports io.zig.
    let math_imports = relation_index
        .get_import_table("src/math.zig")
        .expect("math.zig must have an import table");
    assert_eq!(math_imports.import_count(), 1);
    assert!(
        math_imports
            .standardized_imports
            .iter()
            .any(|i| i.source == "io.zig")
    );

    // Cross-file dependency graph: main → math → io.
    let builder = orchestrator
        .get_relation_builder()
        .expect("relation builder available");
    let deps = builder.dependency_graph();
    assert!(
        deps.has_dependency("src/main.zig", "math.zig"),
        "main.zig must depend on math.zig"
    );
    assert!(
        deps.has_dependency("src/math.zig", "io.zig"),
        "math.zig must depend on io.zig"
    );
}

/// the `plugin_symbol_extract_enabled` gate — with it off,
/// custom-language files get no plugin imports in the relation index.
#[tokio::test]
async fn test_plugin_symbol_extract_gated_off() {
    use cce_orchestrator::{IndexOptions, IndexOrchestrator};
    use cce_relation::index::ImportIndexOps;
    use std::sync::Arc;

    init_minimal_logging();

    // Reuse the same `zig` custom language + stand-in grammar as the positive
    // test (same index, idempotent registration). The plugin still matches
    // `*.zig` files; the gate is the only thing turned off.
    cce_parser::tree_sitter_init::register_plugin_language_with_builtin_grammar(
        "zig",
        &["zig".to_string()],
        Language::Rust,
    );

    let fixture = EmptyFixture::new().expect("Failed to create fixture");
    fixture
        .add_file("src/lib.zig", "const std = @import(\"std\");")
        .expect("add lib.zig");
    let fixture = fixture.into_test_fixture();

    let mut registry = PluginRegistry::new();
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let source = FilePluginSource::from_project(project_root, Some(".cce/plugins.json"));
    registry
        .load_source(&source)
        .expect("Failed to load plugins");

    // symbol_extract_enabled defaults to false.
    let mut orchestrator = IndexOrchestrator::new(1)
        .expect("failed to create IndexOrchestrator")
        .with_plugin_registry(Arc::new(registry));

    let options = IndexOptions {
        root_dir: fixture.root_path().to_path_buf(),
        extensions: vec!["zig".to_string()],
        build_relations: true,
        store_vectors: false,
        store_bm25: false,
        store_summaries: false,
        respect_gitignore: false,
        ..IndexOptions::new(fixture.root_path())
    };
    let result = orchestrator
        .execute(options)
        .await
        .expect("Indexing should succeed");
    assert!(result.total_files >= 1, "zig file should be scanned");

    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");
    let imports = relation_index
        .get_import_table("src/lib.zig")
        .expect("file must be indexed");
    assert_eq!(
        imports.import_count(),
        0,
        "plugin imports must be gated behind plugin_symbol_extract_enabled"
    );
}
