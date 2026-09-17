//! Temporary diagnostic: dump exact rerank inputs for one query.
use cce_e2e_tests::bench_data::BenchmarkData;
use cce_e2e_tests::rerank_benchmark::{decode_sidecar, RERANK_CANDIDATE_DEPTH, RERANK_MODEL_KEY};

fn main() {
    let baseline = std::env::args().nth(1).unwrap_or("full_pipeline".into());
    let method = std::env::args().nth(2).unwrap_or("bm25".into());
    let qid = std::env::args().nth(3).unwrap_or("FZ-G1Q1-naming_case".into());
    let base = format!(
        "crates/app/cce-e2e-tests/data/benchmark/{baseline}/once_cell/bge-m3"
    );
    let bench: BenchmarkData = rkyv::from_bytes::<BenchmarkData, rkyv::rancor::Error>(
        &std::fs::read(format!("{base}/bench_data.rkyv")).expect("bench"),
    )
    .expect("decode bench");
    let sidecar = decode_sidecar(
        &std::fs::read(format!(
            "{base}/rerank_{RERANK_MODEL_KEY}_{method}_depth{RERANK_CANDIDATE_DEPTH}.rkyv"
        ))
        .expect("sidecar"),
    )
    .expect("decode sidecar");
    let qi = bench.queries.iter().position(|q| q.id == qid).expect("query");
    println!(
        "QUERY [{qid}]: {}",
        bench.query_texts.get(qi).map(String::as_str).unwrap_or("?")
    );
    let outcome = sidecar.queries.iter().find(|o| o.query_id == qid).expect("outcome");
    let judgment = cce_e2e_tests::judgments::once_cell::once_cell_relevance_judgments()
        .into_iter()
        .find(|j| j.id == qid)
        .expect("judgment");
    println!("JUDGMENT: {:?}", judgment.relevant_ranges);
    for c in outcome.candidates.iter().take(10) {
        let text = bench
            .embedding
            .chunks
            .iter()
            .zip(bench.embedding.texts.iter())
            .find(|(ch2, _)| ch2.chunk_id == c.chunk_id)
            .map(|(_, t)| t.as_str())
            .or_else(|| {
                bench
                    .bm25
                    .chunks
                    .iter()
                    .zip(bench.bm25_documents.iter())
                    .find(|(ch2, _)| ch2.chunk_id == c.chunk_id)
                    .map(|(_, d)| d.content.as_str())
            })
            .unwrap_or("?");
        let loc = bench
            .embedding
            .chunks
            .iter()
            .chain(bench.bm25.chunks.iter())
            .find(|ch| ch.chunk_id == c.chunk_id)
            .map(|ch| format!("{}:{}-{}", ch.file_path, ch.start_line, ch.end_line))
            .unwrap_or("?".into());
        let truncated: String = if text.chars().count() > 500 {
            let end = text.char_indices().nth(500).map(|(i, _)| i).unwrap_or(text.len());
            format!("{}...", &text[..end])
        } else {
            text.to_string()
        };
        println!(
            "rank={:>2} init={:.4} rerank={:.4} {loc} (full_len={} sent_len={})\n  SENT: {}",
            c.recall_rank,
            c.initial_score,
            c.rerank_score,
            text.chars().count(),
            truncated.chars().count(),
            truncated.replace('\n', " "),
        );
    }
}
