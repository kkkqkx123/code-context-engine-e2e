//! Report writers for the retrieval-method benchmark.
//!
//! Output layout (under `outputs/benchmark/{fixture}/retrieval_method/`):
//! `run_manifest.txt`, `alignment_coverage.md`, `aggregate_top{k}.md`,
//! `aggregate_by_query_type_top{k}.md`, `per_query_top{k}.md`,
//! `fusion_gain_by_query_type_top{k}.md`, `weight_sensitivity.md`,
//! `rrf_k_sensitivity.md`, `relevance_top5.md`.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::bench_data::RelevanceLevel;
use crate::judgments::evaluate::{DEFAULT_TOP_K, TOP_K_VALUES};
use crate::retrieval_method::benchmark::{
    AlignmentStat, MINMAX_WEIGHTS, MethodRelevance, MethodResult, RRF_K_VALUES,
};

/// Write the full report set for a benchmark run.
pub fn write_all(
    out: &Path,
    project: &str,
    judgment_count: usize,
    methods: &[String],
    alignment: &[AlignmentStat],
    results: &[MethodResult],
    relevance: &[MethodRelevance],
) -> Result<(), Box<dyn std::error::Error>> {
    write_run_manifest(out, project, judgment_count, methods, alignment)?;
    write_alignment_coverage(out, alignment)?;
    for &top_k in TOP_K_VALUES {
        write_aggregate(out, results, top_k)?;
        write_aggregate_by_query_type(out, results, top_k)?;
        write_per_query(out, results, top_k)?;
        write_fusion_gain(out, results, top_k)?;
    }
    write_weight_sensitivity(out, results)?;
    write_rrf_sensitivity(out, results)?;
    write_relevance(out, relevance)?;
    Ok(())
}

/// Per-(baseline, query_type) sensitivity group: unique query ids plus
/// (method, F1) observation pairs.
type SensitivityGroup =
    BTreeMap<(String, String), (std::collections::HashSet<String>, Vec<(String, f64)>)>;

/// Write `run_manifest.txt` describing data sources and the method matrix.
fn write_run_manifest(
    out: &Path,
    project: &str,
    judgment_count: usize,
    methods: &[String],
    alignment: &[AlignmentStat],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join("run_manifest.txt");
    let mut f = File::create(&path)?;
    writeln!(f, "=== Retrieval-Method Benchmark Run Manifest ===")?;
    writeln!(f)?;
    writeln!(f, "project: {project}")?;
    writeln!(f, "judgments: {judgment_count}")?;
    writeln!(f, "methods: {}", methods.join(", "))?;
    writeln!(f, "minmax weights (vector/bm25):")?;
    for (alpha, beta) in MINMAX_WEIGHTS {
        writeln!(f, "  minmax-{alpha:.1}: {alpha} / {beta}")?;
    }
    writeln!(f, "rrf k values:")?;
    for k in RRF_K_VALUES {
        writeln!(f, "  rrf-{k:.0}: k={k}")?;
    }
    writeln!(
        f,
        "alignment key: entity_id (multi-entity expanded) -> segment_id -> chunk_id"
    )?;
    writeln!(
        f,
        "measurement semantics: single paths on raw chunk ranking; fused methods per alignment key"
    )?;
    writeln!(f, "test chunks: included (All variant)")?;
    writeln!(f)?;
    for stat in alignment {
        writeln!(
            f,
            "baseline {}: {} emb chunks ({} keys), {} bm25 chunks ({} keys), {} common keys (emb-only {}, bm25-only {})",
            stat.baseline,
            stat.emb_chunks,
            stat.emb_keys,
            stat.bm25_chunks,
            stat.bm25_keys,
            stat.common_keys,
            stat.emb_only_keys,
            stat.bm25_only_keys,
        )?;
    }
    Ok(())
}

/// Write `alignment_coverage.md` with cross-path key coverage per baseline.
fn write_alignment_coverage(
    out: &Path,
    stats: &[AlignmentStat],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join("alignment_coverage.md");
    let mut f = File::create(&path)?;

    let headers = &[
        "baseline",
        "emb_chunks",
        "bm25_chunks",
        "emb_keys",
        "bm25_keys",
        "common",
        "emb_only",
        "bm25_only",
        "coverage",
    ];
    let rows: Vec<Vec<String>> = stats
        .iter()
        .map(|s| {
            let total = s.emb_keys.max(s.bm25_keys);
            let coverage = if total == 0 {
                0.0
            } else {
                s.common_keys as f64 / total as f64
            };
            vec![
                s.baseline.clone(),
                s.emb_chunks.to_string(),
                s.bm25_chunks.to_string(),
                s.emb_keys.to_string(),
                s.bm25_keys.to_string(),
                s.common_keys.to_string(),
                s.emb_only_keys.to_string(),
                s.bm25_only_keys.to_string(),
                format!("{:.1}%", coverage * 100.0),
            ]
        })
        .collect();
    write_md_table(&mut f, headers, &rows)?;

    writeln!(f)?;
    writeln!(f, "Coverage = common keys / max(emb_keys, bm25_keys).")?;
    writeln!(
        f,
        "Keys present in only one path receive partial credit in hybrid fusion (include_single_path=true)."
    )?;
    Ok(())
}

/// Write `aggregate_top{k}.md` grouped by (baseline, method).
fn write_aggregate(
    out: &Path,
    results: &[MethodResult],
    top_k: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join(format!("aggregate_top{top_k}.md"));
    let mut f = File::create(&path)?;

    let mut map: BTreeMap<(String, String), Vec<&MethodResult>> = BTreeMap::new();
    for r in results.iter().filter(|r| r.top_k == top_k) {
        map.entry((r.baseline.clone(), r.method.clone()))
            .or_default()
            .push(r);
    }

    let headers = &[
        "k", "baseline", "method", "n", "R_s", "R_r", "R_a", "P_s", "P_r", "P_a", "F1_s", "F1_r",
        "F1_a", "1st_hit", "redund",
    ];
    let rows: Vec<Vec<String>> = map
        .iter()
        .map(|((baseline, method), group)| {
            vec![
                format!("{top_k}"),
                baseline.clone(),
                method.clone(),
                format!("{}", group.len()),
                avg_str(group.iter().map(|r| r.score.range.recall_strong)),
                avg_str(group.iter().map(|r| r.score.range.recall_related)),
                avg_str(group.iter().map(|r| r.score.range.recall_any)),
                avg_str(group.iter().map(|r| r.score.range.precision_strong)),
                avg_str(group.iter().map(|r| r.score.range.precision_related)),
                avg_str(group.iter().map(|r| r.score.range.precision_any)),
                avg_str(group.iter().map(|r| r.score.range.f1_strong)),
                avg_str(group.iter().map(|r| r.score.range.f1_related)),
                avg_str(group.iter().map(|r| r.score.range.f1_any)),
                first_hit_str(group),
                avg_str(group.iter().map(|r| r.score.redundant_coverage as f64)),
            ]
        })
        .collect();
    write_md_table(&mut f, headers, &rows)?;
    Ok(())
}

/// Write `aggregate_by_query_type_top{k}.md` grouped by (baseline, method,
/// query_type).
fn write_aggregate_by_query_type(
    out: &Path,
    results: &[MethodResult],
    top_k: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join(format!("aggregate_by_query_type_top{top_k}.md"));
    let mut f = File::create(&path)?;

    let mut map: BTreeMap<(String, String, String), Vec<&MethodResult>> = BTreeMap::new();
    for r in results.iter().filter(|r| r.top_k == top_k) {
        map.entry((
            r.baseline.clone(),
            r.method.clone(),
            r.query_type.to_string(),
        ))
        .or_default()
        .push(r);
    }

    let headers = &[
        "k",
        "baseline",
        "method",
        "query_type",
        "n",
        "R_s",
        "R_r",
        "R_a",
        "P_s",
        "P_r",
        "P_a",
        "F1_s",
        "F1_r",
        "F1_a",
        "1st_hit",
    ];
    let rows: Vec<Vec<String>> = map
        .iter()
        .map(|((baseline, method, query_type), group)| {
            vec![
                format!("{top_k}"),
                baseline.clone(),
                method.clone(),
                query_type.clone(),
                format!("{}", group.len()),
                avg_str(group.iter().map(|r| r.score.range.recall_strong)),
                avg_str(group.iter().map(|r| r.score.range.recall_related)),
                avg_str(group.iter().map(|r| r.score.range.recall_any)),
                avg_str(group.iter().map(|r| r.score.range.precision_strong)),
                avg_str(group.iter().map(|r| r.score.range.precision_related)),
                avg_str(group.iter().map(|r| r.score.range.precision_any)),
                avg_str(group.iter().map(|r| r.score.range.f1_strong)),
                avg_str(group.iter().map(|r| r.score.range.f1_related)),
                avg_str(group.iter().map(|r| r.score.range.f1_any)),
                first_hit_str(group),
            ]
        })
        .collect();
    write_md_table(&mut f, headers, &rows)?;
    Ok(())
}

/// Write `per_query_top{k}.md` with per-query × per-method detail rows.
fn write_per_query(
    out: &Path,
    results: &[MethodResult],
    top_k: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join(format!("per_query_top{top_k}.md"));
    let mut f = File::create(&path)?;

    let headers = &[
        "k",
        "baseline",
        "method",
        "query_id",
        "query_type",
        "s_m",
        "r_m",
        "a_m",
        "rel",
        "R_s",
        "R_r",
        "R_a",
        "P_s",
        "P_r",
        "P_a",
        "F1_s",
        "F1_r",
        "F1_a",
        "1st_hit",
        "redund",
    ];
    let rows: Vec<Vec<String>> = results
        .iter()
        .filter(|r| r.top_k == top_k)
        .map(|r| {
            let s = &r.score;
            vec![
                format!("{}", r.top_k),
                r.baseline.clone(),
                r.method.clone(),
                r.query_id.clone(),
                r.query_type.to_string(),
                format!("{}", s.range.strong_matches),
                format!("{}", s.range.related_matches),
                format!("{}", s.range.any_matches),
                format!("{}", s.range.total_relevant),
                format!("{:.4}", s.range.recall_strong),
                format!("{:.4}", s.range.recall_related),
                format!("{:.4}", s.range.recall_any),
                format!("{:.4}", s.range.precision_strong),
                format!("{:.4}", s.range.precision_related),
                format!("{:.4}", s.range.precision_any),
                format!("{:.4}", s.range.f1_strong),
                format!("{:.4}", s.range.f1_related),
                format!("{:.4}", s.range.f1_any),
                first_hit_cell(s.avg_first_hit_rank),
                format!("{}", s.redundant_coverage),
            ]
        })
        .collect();
    write_md_table(&mut f, headers, &rows)?;
    Ok(())
}

/// Write `fusion_gain_by_query_type_top{k}.md`.
///
/// Per-query fusion gain is `(F1_hybrid - max(F1_emb, F1_bm25)) / max(...)`,
/// averaged per (baseline, query_type) for each hybrid method.
fn write_fusion_gain(
    out: &Path,
    results: &[MethodResult],
    top_k: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join(format!("fusion_gain_by_query_type_top{top_k}.md"));
    let mut f = File::create(&path)?;

    // Baseline per-query F1 for the two single-path methods.
    let mut single: BTreeMap<(String, String), (f64, f64)> = BTreeMap::new();
    for r in results.iter().filter(|r| r.top_k == top_k) {
        let entry = single
            .entry((r.baseline.clone(), r.query_id.clone()))
            .or_insert((0.0, 0.0));
        match r.method.as_str() {
            "emb" => entry.0 = r.score.range.f1_any,
            "bm25" => entry.1 = r.score.range.f1_any,
            _ => {}
        }
    }

    let mut map: BTreeMap<(String, String, String), Vec<f64>> = BTreeMap::new();
    for r in results.iter().filter(|r| r.top_k == top_k) {
        if r.method == "emb" || r.method == "bm25" {
            continue;
        }
        let Some(&(f1_emb, f1_bm25)) = single.get(&(r.baseline.clone(), r.query_id.clone())) else {
            continue;
        };
        let best = f1_emb.max(f1_bm25);
        let gain = if best > 0.0 {
            (r.score.range.f1_any - best) / best
        } else {
            0.0
        };
        map.entry((
            r.baseline.clone(),
            r.query_type.to_string(),
            r.method.clone(),
        ))
        .or_default()
        .push(gain);
    }

    let headers = &["k", "baseline", "query_type", "method", "n", "fusion_gain"];
    let rows: Vec<Vec<String>> = map
        .iter()
        .map(|((baseline, query_type, method), gains)| {
            vec![
                format!("{top_k}"),
                baseline.clone(),
                query_type.clone(),
                method.clone(),
                format!("{}", gains.len()),
                avg_str(gains.iter().copied()),
            ]
        })
        .collect();
    write_md_table(&mut f, headers, &rows)?;
    Ok(())
}

/// Write `weight_sensitivity.md`: F1@5 spread across `minmax-*` weights per
/// (baseline, query_type), plus the std across weights.
fn write_weight_sensitivity(
    out: &Path,
    results: &[MethodResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join("weight_sensitivity.md");
    let mut f = File::create(&path)?;

    let mut per_query: BTreeMap<(String, String, String), f64> = BTreeMap::new();
    for r in results.iter().filter(|r| r.top_k == DEFAULT_TOP_K) {
        per_query.insert(
            (r.baseline.clone(), r.query_id.clone(), r.method.clone()),
            r.score.range.f1_any,
        );
    }

    let mut grouped: SensitivityGroup = BTreeMap::new();
    for (key, f1) in &per_query {
        if !key.2.starts_with("minmax-") {
            continue;
        }
        let (query_ids, entries) = grouped
            .entry((key.0.clone(), query_type_of(results, &key.1, &key.0)))
            .or_default();
        query_ids.insert(key.1.clone());
        entries.push((key.2.clone(), *f1));
    }

    let headers = &[
        "baseline",
        "query_type",
        "n",
        "minmax-0.9",
        "minmax-0.7",
        "minmax-0.6",
        "minmax-0.5",
        "minmax-0.4",
        "minmax-0.3",
        "minmax-0.1",
        "std",
    ];
    let mut rows = Vec::new();
    for ((baseline, query_type), (query_ids, entries)) in &grouped {
        let mut by_weight: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        for (method, f1) in entries {
            by_weight.entry(method.clone()).or_default().push(*f1);
        }
        let mut row = vec![
            baseline.clone(),
            query_type.clone(),
            format!("{}", query_ids.len()),
        ];
        let mut means = Vec::new();
        for (alpha, _) in MINMAX_WEIGHTS {
            let alpha_key = format!("minmax-{alpha:.1}");
            let vals = by_weight.get(&alpha_key).cloned().unwrap_or_default();
            let mean = mean_of(&vals);
            row.push(format!("{mean:.4}"));
            means.push(mean);
        }
        row.push(format!("{:.4}", std_dev(&means)));
        rows.push(row);
    }
    write_md_table(&mut f, headers, &rows)?;
    Ok(())
}

/// Write `rrf_k_sensitivity.md`: F1@5 spread across `rrf-*` k values per
/// (baseline, query_type), plus the std across k values.
fn write_rrf_sensitivity(
    out: &Path,
    results: &[MethodResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join("rrf_k_sensitivity.md");
    let mut f = File::create(&path)?;

    let mut per_query: BTreeMap<(String, String, String), f64> = BTreeMap::new();
    for r in results.iter().filter(|r| r.top_k == DEFAULT_TOP_K) {
        per_query.insert(
            (r.baseline.clone(), r.query_id.clone(), r.method.clone()),
            r.score.range.f1_any,
        );
    }

    let mut grouped: SensitivityGroup = BTreeMap::new();
    for (key, f1) in &per_query {
        if !key.2.starts_with("rrf-") {
            continue;
        }
        let (query_ids, entries) = grouped
            .entry((key.0.clone(), query_type_of(results, &key.1, &key.0)))
            .or_default();
        query_ids.insert(key.1.clone());
        entries.push((key.2.clone(), *f1));
    }

    let headers = &[
        "baseline",
        "query_type",
        "n",
        "rrf-30",
        "rrf-60",
        "rrf-100",
        "std",
    ];
    let mut rows = Vec::new();
    for ((baseline, query_type), (query_ids, entries)) in &grouped {
        let mut by_k: BTreeMap<String, Vec<f64>> = BTreeMap::new();
        for (method, f1) in entries {
            by_k.entry(method.clone()).or_default().push(*f1);
        }
        let mut row = vec![
            baseline.clone(),
            query_type.clone(),
            format!("{}", query_ids.len()),
        ];
        let mut means = Vec::new();
        for k in RRF_K_VALUES {
            let key = format!("rrf-{k:.0}");
            let vals = by_k.get(&key).cloned().unwrap_or_default();
            let mean = mean_of(&vals);
            row.push(format!("{mean:.4}"));
            means.push(mean);
        }
        row.push(format!("{:.4}", std_dev(&means)));
        rows.push(row);
    }
    write_md_table(&mut f, headers, &rows)?;
    Ok(())
}

/// Write `relevance_top5.md` with top-5 strong/related hits per method.
fn write_relevance(
    out: &Path,
    relevance: &[MethodRelevance],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out.join("relevance_top5.md");
    let mut f = File::create(&path)?;

    let headers = &[
        "baseline", "method", "query_id", "rank", "level", "location",
    ];
    let mut rows = Vec::new();
    for info in relevance {
        let mut collect = |chunks: &[(usize, String, RelevanceLevel)]| {
            for (rank, loc, level) in chunks {
                rows.push(vec![
                    info.baseline.clone(),
                    info.method.clone(),
                    info.query_id.clone(),
                    rank.to_string(),
                    format!("{:?}", level),
                    loc.clone(),
                ]);
            }
        };
        collect(&info.strong_chunks);
        collect(&info.related_chunks);
    }
    write_md_table(&mut f, headers, &rows)?;
    Ok(())
}

/// Resolve the query type for a query id inside a baseline.
fn query_type_of(results: &[MethodResult], query_id: &str, baseline: &str) -> String {
    results
        .iter()
        .find(|r| r.baseline == baseline && r.query_id == query_id)
        .map(|r| r.query_type.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Format an average of a float iterator.
fn avg_str(iter: impl Iterator<Item = f64>) -> String {
    let vals: Vec<f64> = iter.collect();
    format!("{:.4}", mean_of(&vals))
}

/// Format an optional average first-hit rank.
fn first_hit_str(results: &[&MethodResult]) -> String {
    let hits: Vec<f64> = results
        .iter()
        .filter_map(|r| r.score.avg_first_hit_rank)
        .collect();
    if hits.is_empty() {
        "n/a".to_string()
    } else {
        format!("{:.1}", hits.iter().sum::<f64>() / hits.len() as f64)
    }
}

/// Format an optional average first-hit rank for a per-query cell.
fn first_hit_cell(value: Option<f64>) -> String {
    value.map_or_else(|| "none".to_string(), |v| format!("{v:.1}"))
}

fn mean_of(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = mean_of(values);
    let variance =
        values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / values.len() as f64;
    variance.sqrt()
}

/// Write a formatted markdown table with auto-sized columns.
fn write_md_table(
    f: &mut File,
    headers: &[&str],
    rows: &[Vec<String>],
) -> Result<(), Box<dyn std::error::Error>> {
    let ncols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, val) in row.iter().enumerate() {
            widths[i] = widths[i].max(val.len());
        }
    }

    let hdr: String = (0..ncols)
        .map(|i| format!("| {:<w$} ", headers[i], w = widths[i]))
        .collect::<Vec<_>>()
        .join("")
        + "|";
    writeln!(f, "{hdr}")?;

    let sep: String = (0..ncols)
        .map(|i| format!("|{:-<w$}", "", w = widths[i] + 2))
        .collect::<Vec<_>>()
        .join("")
        + "|";
    writeln!(f, "{sep}")?;

    for row in rows {
        let line: String = (0..ncols)
            .map(|i| format!("| {:<w$} ", row[i], w = widths[i]))
            .collect::<Vec<_>>()
            .join("")
            + "|";
        writeln!(f, "{line}")?;
    }
    Ok(())
}
