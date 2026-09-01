//! Retrieval-method benchmark: dense / sparse / hybrid fusion comparison.
//!
//! Reuses `data/benchmark/{baseline}/{fixture}/bge-m3/bench_data.rkyv`
//! (no regeneration, no LLM calls) and compares the recall-method families
//! `emb`, `bm25`, `minmax-*` and `rrf-*`. Single-path rows measure the raw
//! chunk ranking; fused rows are measured per alignment key.
//!
//! Sub-modules:
//! - `fusion`: cross-path alignment key + minmax / RRF fusion
//! - `benchmark`: evaluation orchestration and metrics
//! - `report`: markdown report writers

pub mod benchmark;
pub mod fusion;
pub mod report;

pub use benchmark::{
    AlignmentStat, BenchmarkRun, MINMAX_WEIGHTS, MethodRelevance, MethodResult, RRF_K_VALUES,
    RetrievalMethodScore, collect_alignment_stat, evaluate_baseline,
    run_retrieval_method_benchmark,
};
pub use fusion::{
    FusedEntry, PreparedFusion, RankedPath, alignment_key, alignment_keys, dedup_ranked,
    fuse_minmax, fuse_rrf,
};
