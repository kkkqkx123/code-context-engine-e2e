use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::FixtureSpec;
use crate::baselines::{entity_based, full_pipeline, full_pipeline_raw};
use crate::bench_data::{BenchmarkData, Bm25DocRecord, ChunkData, QueryData, RetrieverDataset};
use crate::embedding::{EmbeddingConfig, EmbeddingProviderType};
use cce_config::{
    AppConfig,
    modules::{EmbeddingModelConfig, ProviderConfig},
};
use cce_llm_client::OpenAICompatibleProvider;
use cce_scanner::{FSScanner, ScanOptions};

pub fn data_dir(baseline: &str, project: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("benchmark")
        .join(baseline)
        .join(project)
        .join("bge-m3")
}

pub struct GeneratedChunks {
    pub embedding_chunks: Vec<ChunkData>,
    pub embedding_texts: Vec<String>,
    pub bm25_chunks: Vec<ChunkData>,
    pub bm25_texts: Vec<String>,
    /// BM25 document fields (title/content/keywords), index-aligned with
    /// `bm25_chunks` and `bm25_texts`.
    pub bm25_documents: Vec<Bm25DocRecord>,
}

/// Build the BM25 document records for a chunk list.
///
/// Mirrors the production field rules in
/// `cce_orchestrator::index::storage_coordinator::mapping::build_bm25_documents`
/// (title = `bm25_title`, keywords = `bm25_keywords` joined, content = text)
/// so the offline scorer consumes exactly the fields production indexes.
pub fn bm25_doc_records(
    chunks: &[cce_parser::ast_to_nl::chunker::ChunkedResult],
) -> Vec<Bm25DocRecord> {
    chunks
        .iter()
        .map(|chunk| Bm25DocRecord {
            title: chunk.bm25_title.as_deref().unwrap_or("").to_string(),
            keywords: chunk.bm25_keywords.join(" "),
            content: chunk.text.clone(),
        })
        .collect()
}

pub fn fixture_source_path(spec: FixtureSpec) -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let mut path = base.join(spec.category.dir_name());
    if !spec.subdirectory.is_empty() {
        path = path.join(&spec.subdirectory);
    }
    path
}

pub fn collect_files_by_ext(
    root_path: &Path,
    dir: &Path,
    files: &mut Vec<(String, String)>,
    extension: &str,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files_by_ext(root_path, &path, files, extension)?;
            } else if path.extension().is_some_and(|e| e == extension) {
                let content = std::fs::read_to_string(&path)?;
                let rel = path
                    .strip_prefix(root_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();
                files.push((rel, content));
            }
        }
    }
    Ok(())
}

pub fn load_fixture_files(spec: FixtureSpec) -> io::Result<Vec<(String, String)>> {
    let extension = match spec.category {
        crate::FixtureCategory::Rust => "rs",
        crate::FixtureCategory::Python => "py",
        crate::FixtureCategory::Java => "java",
        crate::FixtureCategory::TypeScript => "ts",
        crate::FixtureCategory::CSharp => "cs",
        crate::FixtureCategory::Cpp => "cpp",
        crate::FixtureCategory::Go => "go",
        crate::FixtureCategory::Kotlin => "kt",
        crate::FixtureCategory::Scala => "scala",
        crate::FixtureCategory::Php => "php",
        crate::FixtureCategory::Ruby => "rb",
        crate::FixtureCategory::Dart => "dart",
        crate::FixtureCategory::JavaScript => "js",
        crate::FixtureCategory::MultiLanguage => "*",
        crate::FixtureCategory::Documents => "*",
    };
    let source_path = fixture_source_path(spec);
    let mut files = Vec::new();
    collect_files_by_ext(&source_path, &source_path, &mut files, extension)?;
    Ok(files)
}

pub fn scan_fixture(spec: FixtureSpec) -> anyhow::Result<Vec<cce_scanner::FileEntry>> {
    let include_pattern = match spec.category {
        crate::FixtureCategory::Rust => "*.rs",
        crate::FixtureCategory::Python => "*.py",
        crate::FixtureCategory::Java => "*.java",
        crate::FixtureCategory::TypeScript => "*.ts",
        crate::FixtureCategory::CSharp => "*.cs",
        crate::FixtureCategory::Cpp => "*.cpp",
        crate::FixtureCategory::Go => "*.go",
        crate::FixtureCategory::Kotlin => "*.kt",
        crate::FixtureCategory::Scala => "*.scala",
        crate::FixtureCategory::Php => "*.php",
        crate::FixtureCategory::Ruby => "*.rb",
        crate::FixtureCategory::Dart => "*.dart",
        crate::FixtureCategory::JavaScript => "*.js",
        crate::FixtureCategory::MultiLanguage => "*",
        crate::FixtureCategory::Documents => "*",
    };
    let root = fixture_source_path(spec);
    let mut scanner = FSScanner::new();
    let opts = ScanOptions {
        root_path: root.to_string_lossy().to_string(),
        include_patterns: vec![include_pattern.to_string()],
        ..Default::default()
    };
    Ok(scanner.scan(&opts)?)
}

pub async fn embed_texts(
    embedder: &OpenAICompatibleProvider,
    texts: &[String],
) -> anyhow::Result<(Vec<f32>, usize)> {
    const MAX_BATCH_CHUNKS: usize = 16;
    let mut all_vectors = Vec::new();
    let mut dim = 0;

    for (batch_idx, chunk_batch) in texts.chunks(MAX_BATCH_CHUNKS).enumerate() {
        let refs: Vec<&str> = chunk_batch.iter().map(|s| s.as_str()).collect();
        eprintln!(
            "  Embedding batch {}: {} texts, max_len={}",
            batch_idx,
            refs.len(),
            refs.iter().map(|s| s.len()).max().unwrap_or(0)
        );
        let result = embedder.embed(&refs).await?;
        if dim == 0 {
            dim = result.embeddings.first().map(|v| v.len()).unwrap_or(0);
        }
        for emb in &result.embeddings {
            all_vectors.extend_from_slice(emb);
        }
    }

    Ok((all_vectors, dim))
}

pub fn build_embedder() -> (EmbeddingConfig, AppConfig) {
    let api_key = std::env::var("CCE_EMB_API_KEY_SILICONFLOW").unwrap_or_default();
    if api_key.is_empty() {
        tracing::warn!("CCE_EMB_API_KEY_SILICONFLOW not set; embedding will fail");
    }

    let emb_config = EmbeddingConfig {
        provider_type: EmbeddingProviderType::LlamaCppServer,
        endpoint: "https://api.siliconflow.cn/v1".to_string(),
        model: "BAAI/bge-m3".to_string(),
        dimension: 1024,
        api_key,
    };

    let app_config = {
        let mut providers = HashMap::new();
        providers.insert(
            "test-provider".into(),
            ProviderConfig {
                id: "test-provider".into(),
                name: "Test Provider".into(),
                base_url: emb_config.endpoint.clone(),
                api_keys: vec![emb_config.api_key.clone()],
                ..Default::default()
            },
        );
        let mut models = HashMap::new();
        models.insert(
            emb_config.model.clone(),
            EmbeddingModelConfig {
                provider_id: "test-provider".into(),
                model: emb_config.model.clone(),
                vector_dimension: 0,
                max_item_tokens: 8192,
                max_batch_tokens: 16384,
                ..Default::default()
            },
        );
        let mut ac = AppConfig::default();
        ac.llm.providers = providers;
        ac.llm.embedding_models = models;
        ac.embedder.default_model = emb_config.model.clone();
        ac.embedder.use_base64 = false;
        ac
    };
    (emb_config, app_config)
}

/// Assert every chunk has a unique (file_path, chunk_id) pair.
///
/// A violation indicates a bug upstream in the chunking pipeline.
/// Panics immediately — do not silently hide defects.
pub fn validate_unique_chunks(chunks: &[ChunkData], label: &str) {
    let mut seen = std::collections::HashSet::new();
    for chunk in chunks {
        let key = (chunk.file_path.as_str(), chunk.chunk_id.as_str());
        if !seen.insert(key) {
            panic!(
                "DUPLICATE CHUNK in {label}: file={} chunk_id={} entity={}",
                chunk.file_path, chunk.chunk_id, chunk.entity_name,
            );
        }
    }
}

pub fn persist(dir: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(dir).unwrap_or_else(|_| panic!("Failed to create dir: {:?}", dir));
    let path = dir.join("bench_data.rkyv");
    std::fs::write(&path, bytes).unwrap_or_else(|_| panic!("Failed to write {}", path.display()));
    tracing::info!("Saved {} bytes to {}", bytes.len(), path.display());
}

pub fn normalize_file_path(abs_path: &str) -> String {
    crate::bench_data::normalize_path(abs_path)
}

pub fn chunk_data_from_result(chunk: &cce_parser::ast_to_nl::chunker::ChunkedResult) -> ChunkData {
    let (start_line, end_line) = chunk
        .metadata
        .source_span
        .line_range_opt()
        .unwrap_or((0, 0));
    let source_ranges = chunk
        .metadata
        .source_ranges()
        .iter()
        .map(|span| {
            let (start_line, end_line) = span.line_range_opt().unwrap_or((0, 0));
            crate::bench_data::ChunkSourceRange {
                start_line,
                end_line,
            }
        })
        .collect();
    ChunkData {
        chunk_id: chunk.chunk_id.clone(),
        entity_name: get_chunk_entity_name(chunk).to_string(),
        file_path: normalize_file_path(&chunk.metadata.file_path),
        start_line,
        end_line,
        source_ranges,
        source_span_kind: chunk.metadata.source_span_kind.to_string(),
        test_info: chunk.metadata.test_info,
        language: chunk.metadata.content_type.language(),
        entity_ids: chunk
            .metadata
            .content_entity_ids()
            .iter()
            .map(|id| id.0 as i64)
            .collect(),
        segment_id: chunk.metadata.segment_id.clone(),
    }
}

/// Project-level configuration for benchmark data generation.
pub struct ProjectConfig {
    pub name: &'static str,
    pub target_spec: FixtureSpec,
    pub distractor_spec: FixtureSpec,
    pub queries: Vec<QueryData>,
}

/// Run the full 2-baseline benchmark data generation for a project.
///
/// Handles embedder setup, both baselines (full_pipeline, direct_chunking),
/// and persistence to `data/benchmark/{baseline}/{name}/bge-m3/`.
pub async fn run_bench_gen(config: ProjectConfig) -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    crate::init_minimal_logging();

    let (emb_config, app_config) = build_embedder();
    let embedder =
        cce_llm_client::OpenAICompatibleProvider::from_model(&app_config, &emb_config.model)?;

    let queries = &config.queries;
    eprintln!("=== QUERIES: {} ===", queries.len());

    let target_spec = config.target_spec;
    let distractor_spec = config.distractor_spec;

    for baseline in [
        "full_pipeline",
        "full_pipeline_raw_source",
        "direct_chunking",
    ] {
        eprintln!("\n{}", "=".repeat(60));
        eprintln!("=== Baseline: {baseline} ===");
        eprintln!("{}\n", "=".repeat(60));

        eprintln!("=== Phase 1: generating chunks ===");
        let chunks = if baseline == "full_pipeline" {
            full_pipeline::gen_full_pipeline_chunks(target_spec.clone(), distractor_spec.clone())
                .await?
        } else if baseline == "full_pipeline_raw_source" {
            full_pipeline_raw::gen_full_pipeline_raw_source_chunks(
                target_spec.clone(),
                distractor_spec.clone(),
            )
            .await?
        } else {
            entity_based::gen_entity_based_chunks(target_spec.clone(), distractor_spec.clone())
                .await?
        };

        eprintln!("\n=== Phase 2: embedding chunks ===");
        let (chunk_vectors, dimension) = if !chunks.embedding_texts.is_empty() {
            eprintln!(
                "  Embedding {} code chunks...",
                chunks.embedding_texts.len()
            );
            match embed_texts(&embedder, &chunks.embedding_texts).await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!(
                        "  WARNING: code chunk embedding failed: {e}; saving without vectors"
                    );
                    (vec![], 0)
                }
            }
        } else {
            (vec![], 0)
        };

        eprintln!("\n=== Phase 3: assembling and persisting ===");
        for (variant_queries, suffix) in [(queries, "")] {
            eprintln!("\n--- {baseline}{suffix} ---");
            let query_texts: Vec<String> = variant_queries.iter().map(|q| q.text.clone()).collect();

            let (query_vectors, _) = match embed_texts(&embedder, &query_texts).await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("  WARNING: query embedding failed: {e}; saving without vectors");
                    (vec![], 0)
                }
            };

            if chunks.bm25_documents.len() != chunks.bm25_chunks.len() {
                anyhow::bail!(
                    "BM25 document/chunk misalignment: {} documents vs {} chunks",
                    chunks.bm25_documents.len(),
                    chunks.bm25_chunks.len()
                );
            }

            let code_bench = BenchmarkData {
                queries: variant_queries.to_vec(),
                query_texts,
                embedding: RetrieverDataset {
                    chunks: chunks.embedding_chunks.clone(),
                    texts: chunks.embedding_texts.clone(),
                    vectors: chunk_vectors.clone(),
                    query_vectors,
                    dimension: dimension as u32,
                },
                bm25: RetrieverDataset {
                    chunks: chunks.bm25_chunks.clone(),
                    texts: chunks.bm25_texts.clone(),
                    vectors: vec![],
                    query_vectors: vec![],
                    dimension: 0,
                },
                bm25_documents: chunks.bm25_documents.clone(),
            };
            let code_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&code_bench)?.to_vec();

            let baseline_name = format!("{baseline}{suffix}");
            persist(&data_dir(&baseline_name, config.name), &code_bytes);
        }
    }

    eprintln!("\n{}", "=".repeat(60));
    eprintln!("=== All baselines generated ===");
    eprintln!("{}", "=".repeat(60));
    Ok(())
}

fn get_chunk_entity_name(chunk: &cce_parser::ast_to_nl::chunker::ChunkedResult) -> &str {
    chunk.bm25_title.as_deref().unwrap_or("unknown")
}
