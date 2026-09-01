//! Export NL documents for C# fixtures (basic, generics, control_flow, MediatR).
//!
//! Output:
//!   outputs/scenarios/csharp/summary/{fixture}/    — markdown NL docs
//!   outputs/scenarios/csharp/chunks/{fixture}/emb/ — chunk segmentation (Embedding)
//!   outputs/scenarios/csharp/chunks/{fixture}/bm25/— chunk segmentation (BM25)

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use cce_orchestrator::export::{DirectExporter, ExportConfig};
use cce_orchestrator::index::FileProcessor;
use cce_parser::ast_to_nl::ConversionRequest;
use cce_parser::ast_to_nl::chunker::ChunkedResult;
use cce_scanner::{FSScanner, ScanOptions};
use cce_types::OutputMode;

use cce_e2e_tests::{
    FixtureSpec, OutputCategory, OutputManager, TestFixture, init_minimal_logging,
};

#[tokio::main]
async fn main() {
    init_minimal_logging();

    let specs: Vec<(&str, FixtureSpec)> = vec![
        ("basic", FixtureSpec::csharp_basic()),
        ("generics", FixtureSpec::csharp_type_inference_generics()),
        (
            "control_flow",
            FixtureSpec::csharp_type_inference_control_flow(),
        ),
        ("MediatR", FixtureSpec::csharp_mediatr()),
    ];

    for (name, spec) in &specs {
        let fixture = TestFixture::load(spec.clone())
            .unwrap_or_else(|e| panic!("Failed to load {}: {}", name, e));
        export_fixture(name, fixture).await;
    }

    println!("\n=== All C# fixtures exported ===");
}

async fn export_fixture(fixture_name: &str, fixture: TestFixture) {
    println!("\n=== Exporting fixture: {} ===", fixture_name);

    let project_root = fixture.root_path().to_path_buf();
    let project_root_str = project_root.to_string_lossy().replace('\\', "/");

    let export_config = ExportConfig::new(project_root.clone(), 1);
    let direct_exporter = Arc::new(DirectExporter::new(export_config));

    let mut scanner = FSScanner::new();
    let scan_opts = ScanOptions {
        root_path: project_root.to_string_lossy().to_string(),
        include_patterns: vec!["*.cs".to_string()],
        ..Default::default()
    };
    let file_entries = scanner
        .scan(&scan_opts)
        .expect("Failed to scan fixture directory");

    let mut file_processor = FileProcessor::new();
    let mut indexed_files = 0;
    let mut emb_by_file: BTreeMap<String, Vec<ChunkedResult>> = BTreeMap::new();
    let mut bm25_by_file: BTreeMap<String, Vec<ChunkedResult>> = BTreeMap::new();

    for entry in &file_entries {
        let file_path = entry.path.to_string_lossy().to_string();
        let relative_path = normalize_file_path(&file_path, &project_root_str);

        match file_processor
            .process_file_complete(entry, OutputMode::Embedding)
            .await
        {
            Ok(result) => {
                if let Some(processing_result) = result.processing_result {
                    if !processing_result.groups.is_empty() {
                        let converter = file_processor.converter();
                        let source = &*result.parsed_file.source;
                        let request = ConversionRequest {
                            force_mode: Some(OutputMode::Embedding),
                        };
                        let conversions = converter.convert_entity_groups(
                            &processing_result.groups,
                            &file_path,
                            Some(&request),
                            Some(&processing_result),
                            Some(source),
                        );
                        match direct_exporter
                            .export_groups(&conversions, &file_path)
                            .await
                        {
                            Ok(_) => indexed_files += 1,
                            Err(e) => eprintln!("  Export error: {}", e),
                        }
                    }
                }
                if !result.chunks.is_empty() {
                    emb_by_file
                        .entry(relative_path.clone())
                        .or_default()
                        .extend(result.chunks);
                }
            }
            Err(e) => eprintln!("  Process error: {}", e),
        }

        if let Ok(result) = file_processor
            .process_file_complete(entry, OutputMode::Bm25)
            .await
        {
            if !result.chunks.is_empty() {
                bm25_by_file
                    .entry(relative_path)
                    .or_default()
                    .extend(result.chunks);
            }
        }
    }

    for chunks in emb_by_file.values_mut() {
        chunks.sort_by(|a, b| {
            a.metadata
                .source_span
                .line_range_opt()
                .map(|range| range.0)
                .unwrap_or(usize::MAX)
                .cmp(
                    &b.metadata
                        .source_span
                        .line_range_opt()
                        .map(|range| range.0)
                        .unwrap_or(usize::MAX),
                )
        });
    }
    for chunks in bm25_by_file.values_mut() {
        chunks.sort_by(|a, b| {
            a.metadata
                .source_span
                .line_range_opt()
                .map(|range| range.0)
                .unwrap_or(usize::MAX)
                .cmp(
                    &b.metadata
                        .source_span
                        .line_range_opt()
                        .map(|range| range.0)
                        .unwrap_or(usize::MAX),
                )
        });
    }

    write_summary_output(fixture_name, &project_root, indexed_files);
    let total_emb = write_chunk_outputs(&emb_by_file, &format!("chunks/{fixture_name}/emb"));
    let total_bm25 = write_chunk_outputs(&bm25_by_file, &format!("chunks/{fixture_name}/bm25"));
    println!("  {fixture_name} chunks: emb={total_emb} bm25={total_bm25}");
}

fn write_summary_output(fixture_name: &str, project_root: &Path, indexed_files: usize) {
    let cce_dir = project_root.join(".cce").join("nl_docs");
    let exported_files = collect_md_files(&cce_dir);

    let output_mgr = OutputManager::builder()
        .category(OutputCategory::Scenarios)
        .language("csharp")
        .scenario(format!("summary/{fixture_name}"))
        .build();
    let output_dir = output_mgr
        .ensure_output_dir()
        .expect("Failed to create output dir");

    let mut copied_files = Vec::new();
    for file in &exported_files {
        copy_to_output(&cce_dir, &output_dir, file).ok();
        let relative = file.strip_prefix(&cce_dir).unwrap_or(file);
        copied_files.push(relative.to_path_buf());
    }

    let mut summary = String::new();
    summary.push_str(&format!("# Export Output for {fixture_name} Fixture\n\n"));
    summary.push_str(&format!("**Indexed files:** {}\n", indexed_files));
    summary.push_str(&format!("**Exported files:** {}\n\n", copied_files.len()));
    summary.push_str("## Files\n\n");
    copied_files.sort();
    for rel_path in &copied_files {
        summary.push_str(&format!(
            "- {}\n",
            rel_path.to_string_lossy().replace('\\', "/")
        ));
    }
    output_mgr.write("SUMMARY.md", &summary).ok();

    println!("  {fixture_name} summary: {} files", copied_files.len());
}

fn write_chunk_outputs(
    chunks_by_file: &BTreeMap<String, Vec<ChunkedResult>>,
    scenario: &str,
) -> usize {
    let chunking_mgr = OutputManager::builder()
        .category(OutputCategory::Scenarios)
        .language("csharp")
        .scenario(scenario)
        .build();
    let output_dir = chunking_mgr
        .ensure_output_dir()
        .expect("Failed to create chunking output dir");

    for (file_path, chunks) in chunks_by_file {
        let path = Path::new(file_path);
        let content = render_chunking_file(chunks, file_path);
        let file_name = path.file_name().unwrap().to_string_lossy();
        let file_name = format!("{file_name}.txt");
        let file_dir = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let target_dir = output_dir.join(&file_dir);
        std::fs::create_dir_all(&target_dir).expect("Failed to create target dir");
        let target_path = target_dir.join(&file_name);
        std::fs::write(&target_path, &content)
            .unwrap_or_else(|e| panic!("Failed to write {file_name}: {e}"));
    }

    chunks_by_file.values().map(Vec::len).sum()
}

fn collect_files_by_extension(dir: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if !dir.exists() {
        return files;
    }
    collect_files_recursive(dir, extension, &mut files);
    files
}

fn collect_md_files(dir: &Path) -> Vec<std::path::PathBuf> {
    collect_files_by_extension(dir, "md")
}

fn collect_files_recursive(dir: &Path, extension: &str, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_recursive(&path, extension, files);
            } else if path.extension().is_some_and(|e| e == extension) {
                files.push(path);
            }
        }
    }
}

fn copy_to_output(cce_dir: &Path, output_dir: &Path, file: &Path) -> std::io::Result<()> {
    let relative = file
        .strip_prefix(cce_dir)
        .unwrap_or_else(|_| file.file_name().map(Path::new).unwrap_or(file));
    let target = output_dir.join(relative);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(file, &target)?;
    Ok(())
}

fn normalize_file_path(abs_path: &str, manifest_dir: &str) -> String {
    let abs = abs_path.replace('\\', "/");
    let manifest_dir = manifest_dir.replace('\\', "/");
    if let Some(stripped) = abs.strip_prefix(&manifest_dir) {
        let stripped = stripped.trim_start_matches('/');
        if !stripped.is_empty() {
            return stripped.to_string();
        }
    }
    abs
}

fn render_chunking_file(
    chunks: &[cce_parser::ast_to_nl::chunker::ChunkedResult],
    file_path: &str,
) -> String {
    let mut out = String::new();
    let total = chunks.len();
    let path = Path::new(file_path);
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| file_path.to_string());

    out.push_str(&format!("{file_name}\n"));
    out.push_str(&format!("total chunks: {total}\n"));
    out.push_str("---\n");

    for (i, chunk) in chunks.iter().enumerate() {
        let (start_line, end_line) = chunk
            .metadata
            .source_span
            .line_range_opt()
            .unwrap_or((0, 0));
        let code_meta = chunk.metadata.code_metadata.as_ref();

        out.push_str(&format!(
            "\n=== CHUNK {}/{} (lines {start_line}-{end_line}) ===\n",
            i + 1,
            total,
        ));
        out.push_str(&format!(
            "chunk_id: {} | group: {} | kind: {}\n",
            chunk.chunk_id,
            chunk.source_group_id,
            code_meta
                .map(|m| format!("{:?}", m.entity_kind))
                .unwrap_or_else(|| "n/a".to_string())
        ));

        let is_fragment = code_meta.is_some_and(|m| m.is_fragment);
        if is_fragment {
            let frag_idx = code_meta.and_then(|m| m.fragment_index).unwrap_or(0);
            let total_frags = code_meta.and_then(|m| m.total_fragments).unwrap_or(0);
            out.push_str(&format!(
                "fragment: {}/{total_frags} | split_reason: {}\n",
                frag_idx + 1,
                code_meta
                    .map(|m| format!("{:?}", m.split_reason))
                    .unwrap_or_else(|| "n/a".to_string())
            ));
        } else {
            out.push_str(&format!(
                "split_reason: {}\n",
                code_meta
                    .map(|m| format!("{:?}", m.split_reason))
                    .unwrap_or_else(|| "n/a".to_string())
            ));
        }

        if chunk.path == cce_parser::ast_to_nl::chunker::ChunkPath::Bm25 {
            out.push_str(&format!(
                "title: {} | tokens: {}\n",
                chunk.bm25_title.as_deref().unwrap_or("n/a"),
                chunk.token_count
            ));
            out.push_str(&format!("keywords: [{}]\n", chunk.bm25_keywords.join(", ")));
        } else {
            out.push_str(&format!("tokens: {}\n", chunk.token_count));
        }

        if !chunk.related_groups.is_empty() {
            let rel_str: Vec<String> = chunk
                .related_groups
                .iter()
                .map(|rel| format!("{} ({})", rel.group_id, rel.relation_type))
                .collect();
            out.push_str(&format!("related: {}\n", rel_str.join(", ")));
        }

        if let Some(o) = &chunk.prev_overlap {
            out.push_str(&format!(
                "prev_overlap: {} tokens from {}\n",
                o.token_count, o.source_chunk_id
            ));
        }
        if let Some(o) = &chunk.next_overlap {
            out.push_str(&format!(
                "next_overlap: {} tokens from {}\n",
                o.token_count, o.source_chunk_id
            ));
        }

        out.push('\n');
        out.push_str(&chunk.text);
        out.push('\n');
    }

    out
}
