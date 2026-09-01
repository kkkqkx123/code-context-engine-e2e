use cce_storage_bm25::TermOperator;

use crate::infra::term_index::{Field, InMemoryTermIndex, QueryForms};

/// BM25 scoring parameters.
#[derive(Debug, Clone)]
pub struct Bm25Config {
    pub k1: f64,
    pub b: f64,
    pub title_weight: f64,
    pub keywords_weight: f64,
    pub content_weight: f64,
}

impl Bm25Config {
    pub fn field_weight(&self, field: Field) -> f64 {
        match field {
            Field::Title => self.title_weight,
            Field::Keywords => self.keywords_weight,
            Field::Content => self.content_weight,
        }
    }
}

/// Production raw-form boost multipliers (`Bm25Retrieval::parse_query`):
/// the raw form boosts title/keywords by 1.5x and halves the content weight,
/// while the cleaned form uses the standard weights.
const RAW_TITLE_BOOST: f64 = 1.5;
const RAW_CONTENT_BOOST: f64 = 0.5;
const RAW_KEYWORDS_BOOST: f64 = 1.5;
/// Production down-weight for split (auxiliary) query tokens.
const SPLIT_TOKEN_WEIGHT: f64 = 0.5;

/// tantivy fieldnorm quantization, mirroring the vendored fork's
/// `FIELD_NORMS_TABLE` (crates/tantivy/src/fieldnorm/code.rs): the per-doc
/// length in the BM25 length normalization is the decoded table value at the
/// largest id whose entry does not exceed the raw token count.
fn tantivy_fieldnorm(raw_len: u32) -> u32 {
    const fn decode(id: u32) -> u32 {
        if id < 24 {
            id
        } else {
            let x = id - 24;
            let bits = x & 0b111;
            let shift = x >> 3;
            24 + if shift == 0 {
                bits
            } else {
                (bits | 8) << (shift - 1)
            }
        }
    }

    let mut lo = 0u32;
    let mut hi = 255u32;
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if decode(mid) <= raw_len {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    decode(lo)
}

/// Per-field weights for one query form.
#[derive(Debug, Clone, Copy)]
struct FormWeights {
    title: f64,
    content: f64,
    keywords: f64,
}

impl FormWeights {
    fn field(&self, field: Field) -> f64 {
        match field {
            Field::Title => self.title,
            Field::Keywords => self.keywords,
            Field::Content => self.content,
        }
    }
}

/// Score all documents against each query with production retrieval
/// semantics.
///
/// Mirrors `Bm25Retrieval::parse_query` / `build_query`:
/// - Each query contributes a raw form (boosts 1.5x/0.5x/1.5x) and, when
///   cleaning changed the text, a clean form (standard weights).
/// - Split tokens (position_length == 0) carry a 0.5 down-weight.
/// - Per-token field clauses are summed across title/content/keywords.
/// - `operator` maps to the form-level clause occurrence: `Or` (Should)
///   requires a doc to match any token; `And` (Must) requires every token.
/// - Both forms merge with OR; tantivy sums Should-clause scores, so the
///   final score is `raw_score + clean_score` (not max).
///
/// Documents that match no query form receive a zero score and rank at the
/// tail (production never returns them; the evaluator keeps them so every
/// chunk participates in the corpus ranking).
///
/// Returns `all_ranked[query_idx]` = sorted `Vec<(doc_idx, score)>`
/// (descending, truncated to top_k).
pub fn score_all(
    term_index: &InMemoryTermIndex,
    queries: &[QueryForms],
    config: &Bm25Config,
    operator: TermOperator,
    top_k: usize,
) -> Vec<Vec<(usize, f64)>> {
    let n_docs = term_index.n_docs as usize;
    let k1 = config.k1;
    let b = config.b;

    let avg_len: [f64; 3] = [
        term_index.avg_field_length(Field::Title),
        term_index.avg_field_length(Field::Keywords),
        term_index.avg_field_length(Field::Content),
    ];

    let raw_weights = FormWeights {
        title: config.title_weight * RAW_TITLE_BOOST,
        content: config.content_weight * RAW_CONTENT_BOOST,
        keywords: config.keywords_weight * RAW_KEYWORDS_BOOST,
    };
    let clean_weights = FormWeights {
        title: config.title_weight,
        content: config.content_weight,
        keywords: config.keywords_weight,
    };

    let mut all_results = Vec::with_capacity(queries.len());

    for query in queries {
        let mut forms: Vec<(Vec<crate::infra::term_index::QueryTerm>, FormWeights)> = Vec::new();
        if !query.raw.is_empty() {
            forms.push((query.raw.clone(), raw_weights));
        }
        if !query.clean.is_empty() {
            forms.push((query.clean.clone(), clean_weights));
        }

        // Per-form doc matching: Or starts empty and unions; And starts full
        // and intersects (a form with zero terms never matches in And mode).
        // Only active forms are allocated, so unused slots cannot leak a
        // default `true` into the merge below.
        let mut form_matches = vec![vec![operator == TermOperator::And; n_docs]; forms.len()];
        // Per-form doc scores. In And mode a doc failing the form's Must
        // constraint is excluded from that form entirely (production
        // BooleanQuery semantics), so its partial score must not leak into
        // the merged total; contributions are zeroed below per form.
        let mut form_scores = vec![vec![0.0_f64; n_docs]; forms.len()];

        for (form_idx, (terms, weights)) in forms.iter().enumerate() {
            for term in terms {
                let Some(postings) = term_index.postings.get(&term.text) else {
                    // No postings: the term cannot match anything.
                    if operator == TermOperator::And {
                        form_matches[form_idx].fill(false);
                    }
                    continue;
                };
                let scale = if term.is_split {
                    SPLIT_TOKEN_WEIGHT
                } else {
                    1.0
                };

                let idf: [f64; 3] = {
                    let mut arr = [0.0_f64; 3];
                    if let Some(dfs) = term_index.term_field_df.get(&term.text) {
                        // idf uses the GLOBAL segment doc count as N and the
                        // per-field document frequency as n (tantivy
                        // `Bm25Weight::for_terms` semantics).
                        let field_n = term_index.n_docs as f64;
                        for field_idx in 0..3 {
                            let df = dfs[field_idx];
                            if df > 0 {
                                arr[field_idx] =
                                    ((field_n - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();
                            }
                        }
                    }
                    arr
                };

                let mut term_hit_docs: Vec<usize> = Vec::new();
                for &(doc_id, field_idx, tf) in postings {
                    let idx = field_idx as usize;
                    let idf_val = idf[idx];
                    if idf_val == 0.0 {
                        continue;
                    }
                    let Some(field) = Field::from_index(idx) else {
                        continue;
                    };
                    let weight = weights.field(field) * scale;
                    if weight == 0.0 {
                        continue;
                    }

                    let raw_len = term_index.doc_lengths[doc_id as usize][idx];
                    let doc_len = tantivy_fieldnorm(raw_len) as f64;
                    let avg = avg_len[idx];
                    if avg <= 0.0 {
                        continue;
                    }

                    let tf_f = tf as f64;
                    let norm = 1.0 - b + b * doc_len / avg;
                    let bm25_score = weight * idf_val * tf_f * (k1 + 1.0) / (tf_f + k1 * norm);

                    form_scores[form_idx][doc_id as usize] += bm25_score;
                    term_hit_docs.push(doc_id as usize);
                }

                // Or mode: a doc is a candidate once any term matches.
                // And mode: the form matches a doc only when every term hits.
                match operator {
                    TermOperator::Or => {
                        for doc in term_hit_docs {
                            form_matches[form_idx][doc] = true;
                        }
                    }
                    TermOperator::And => {
                        let mut hit: Vec<bool> = vec![false; n_docs];
                        for doc in term_hit_docs {
                            hit[doc] = true;
                        }
                        for (i, matched) in form_matches[form_idx].iter_mut().enumerate() {
                            *matched &= hit[i];
                        }
                    }
                }
            }
        }

        // Merge: OR across forms (Should). A form contributes only for docs
        // that satisfy its occurrence constraint; docs matching nothing score
        // zero.
        let mut doc_scores = vec![0.0_f64; n_docs];
        for (form_idx, form_match) in form_matches.iter().enumerate() {
            for (i, matched) in form_match.iter().enumerate() {
                if *matched {
                    doc_scores[i] += form_scores[form_idx][i];
                }
            }
        }

        let mut ranked: Vec<(usize, f64)> = doc_scores.into_iter().enumerate().collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
        ranked.truncate(top_k);
        all_results.push(ranked);
    }

    all_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::term_index::build_query_forms;
    use crate::infra::term_index::build_term_index;
    use cce_storage_bm25::Bm25Document;

    fn make_docs() -> Vec<Bm25Document> {
        vec![
            Bm25Document::new("d:1")
                .with_field("title", "Alpha")
                .with_field("content", "alpha beta gamma delta")
                .with_field("keywords", "alpha"),
            Bm25Document::new("d:2")
                .with_field("title", "Beta")
                .with_field("content", "beta gamma epsilon")
                .with_field("keywords", "beta"),
            Bm25Document::new("d:3")
                .with_field("title", "Gamma")
                .with_field("content", "gamma delta zeta")
                .with_field("keywords", "gamma"),
        ]
    }

    fn default_config() -> Bm25Config {
        Bm25Config {
            k1: 1.8,
            b: 0.6,
            title_weight: 2.0,
            keywords_weight: 2.0,
            content_weight: 1.0,
        }
    }

    fn forms(text: &str) -> Vec<crate::infra::term_index::QueryForms> {
        vec![build_query_forms(text)]
    }

    #[test]
    fn test_score_all_returns_ranked_results() {
        let docs = make_docs();
        let index = build_term_index(&docs);
        let results = score_all(
            &index,
            &forms("alpha"),
            &default_config(),
            TermOperator::Or,
            10,
        );
        assert_eq!(results.len(), 1);
        // doc 0 should rank highest for "alpha"
        assert_eq!(results[0][0].0, 0);
    }

    #[test]
    fn test_score_all_top_k_truncation() {
        let docs = make_docs();
        let index = build_term_index(&docs);
        let config = Bm25Config {
            k1: 1.5,
            b: 0.75,
            title_weight: 1.0,
            keywords_weight: 1.0,
            content_weight: 1.0,
        };
        let results = score_all(&index, &forms("gamma"), &config, TermOperator::Or, 2);
        assert_eq!(results[0].len(), 2);
    }

    #[test]
    fn test_zero_weight_skips_field() {
        let docs = make_docs();
        let index = build_term_index(&docs);
        let config = Bm25Config {
            k1: 1.5,
            b: 0.75,
            title_weight: 0.0,
            keywords_weight: 0.0,
            content_weight: 0.0,
        };
        let results = score_all(&index, &forms("alpha"), &config, TermOperator::Or, 10);
        assert_eq!(results[0].iter().map(|(_, s)| *s as i64).sum::<i64>(), 0);
    }

    #[test]
    fn test_or_mode_matches_docs_with_any_term() {
        // "alpha" in doc 0, "gamma" in doc 2; doc 1 contains neither.
        // "or" must match docs 0 and 2.
        let docs = vec![
            Bm25Document::new("d:1")
                .with_field("title", "Alpha")
                .with_field("content", "alpha beta gamma delta")
                .with_field("keywords", "alpha"),
            Bm25Document::new("d:2")
                .with_field("title", "Beta")
                .with_field("content", "beta epsilon")
                .with_field("keywords", "beta"),
            Bm25Document::new("d:3")
                .with_field("title", "Gamma")
                .with_field("content", "gamma delta zeta")
                .with_field("keywords", "gamma"),
        ];
        let index = build_term_index(&docs);
        let results = score_all(
            &index,
            &forms("alpha gamma"),
            &default_config(),
            TermOperator::Or,
            10,
        );
        let ranked: Vec<usize> = results[0].iter().map(|(d, _)| *d).collect();
        assert!(ranked.contains(&0) && ranked.contains(&2));
        // Doc 1 contains neither term -> zero score at the tail.
        let zero = results[0]
            .iter()
            .find(|(d, s)| *d == 1 && *s == 0.0)
            .expect("doc 1 ranked at the tail");
        assert_eq!(zero.1, 0.0);
    }

    #[test]
    fn test_and_mode_requires_every_term() {
        // doc 0 has "beta" but not "gamma"; doc 2 has "gamma" but not "beta";
        // only doc 1 contains both.
        let docs = vec![
            Bm25Document::new("d:1")
                .with_field("title", "Alpha")
                .with_field("content", "alpha beta delta")
                .with_field("keywords", "alpha"),
            Bm25Document::new("d:2")
                .with_field("title", "Beta")
                .with_field("content", "beta gamma epsilon")
                .with_field("keywords", "beta"),
            Bm25Document::new("d:3")
                .with_field("title", "Gamma")
                .with_field("content", "gamma delta zeta")
                .with_field("keywords", "gamma"),
        ];
        let index = build_term_index(&docs);
        // "beta gamma": only doc 1 contains both.
        let results = score_all(
            &index,
            &forms("beta gamma"),
            &default_config(),
            TermOperator::And,
            10,
        );
        let ranked: Vec<usize> = results[0].iter().map(|(d, _)| *d).collect();
        assert!(ranked.contains(&1));
        let zero_docs: Vec<usize> = results[0]
            .iter()
            .filter(|(_, s)| *s == 0.0)
            .map(|(d, _)| *d)
            .collect();
        assert!(zero_docs.contains(&0) && zero_docs.contains(&2));
    }

    #[test]
    fn test_dual_form_scores_sum_not_max() {
        // The clean form is an independent tantivy Should-clause whose score
        // is added on top of the raw form (production semantics), so a query
        // with both forms must outscore the identical raw form alone.
        let docs = vec![
            Bm25Document::new("d:0")
                .with_field("title", "Alpha")
                .with_field("content", "alpha that returns beta")
                .with_field("keywords", "alpha"),
        ];
        let index = build_term_index(&docs);

        let alpha = crate::infra::term_index::QueryTerm {
            text: "alpha".to_string(),
            is_split: false,
        };
        let raw_only: Vec<QueryForms> = vec![QueryForms {
            raw: vec![alpha.clone()],
            clean: vec![],
        }];
        let dual: Vec<QueryForms> = vec![QueryForms {
            raw: vec![alpha.clone()],
            clean: vec![alpha],
        }];
        let raw_only = score_all(&index, &raw_only, &default_config(), TermOperator::Or, 10);
        let dual = score_all(&index, &dual, &default_config(), TermOperator::Or, 10);
        // Adding the clean form adds the clean-form clause score (sum, not
        // max): the merged score must be strictly higher.
        assert!(
            dual[0][0].1 > raw_only[0][0].1,
            "dual-form score must exceed raw-only: {} vs {}",
            dual[0][0].1,
            raw_only[0][0].1
        );
    }

    #[test]
    fn test_split_tokens_are_down_weighted() {
        // Query "get_or_init" matches doc 0 via the original term (weight 1.0)
        // and doc 1 via split tokens only (weight 0.5 each).
        let docs = vec![
            Bm25Document::new("d:0")
                .with_field("title", "get_or_init")
                .with_field("content", "function get_or_init")
                .with_field("keywords", "get_or_init"),
            Bm25Document::new("d:1")
                .with_field("title", "Other")
                .with_field("content", "the init get or run")
                .with_field("keywords", "other"),
        ];
        let index = build_term_index(&docs);
        let results = score_all(
            &index,
            &forms("get_or_init"),
            &default_config(),
            TermOperator::Or,
            10,
        );
        let (_, score0) = results[0][0];
        let doc1_score = results[0].iter().find(|(d, _)| *d == 1).map(|(_, s)| *s);
        let doc1_score = doc1_score.expect("doc 1 ranked");
        // Exact match must rank above the split-token-only match.
        assert!(score0 > doc1_score);
    }

    #[test]
    fn test_keywords_duplicate_split_forms_inflate_score() {
        // 45b6 removed split forms from the keywords field. Indexing still
        // re-splits identifiers, so the old form "get_or_init get or init"
        // yields tf=2 for the split terms in keywords while the new form
        // yields tf=1. The query "get" must therefore score the old doc
        // strictly higher, ceteris paribus.
        let old_style = Bm25Document::new("d:0")
            .with_field("title", "same")
            .with_field("content", "identical content")
            .with_field("keywords", "get_or_init get or init");
        let new_style = Bm25Document::new("d:1")
            .with_field("title", "same")
            .with_field("content", "identical content")
            .with_field("keywords", "get_or_init");
        let index = build_term_index(&[old_style, new_style]);

        let results = score_all(
            &index,
            &forms("get"),
            &default_config(),
            TermOperator::Or,
            10,
        );
        let score_old = results[0]
            .iter()
            .find(|(d, _)| *d == 0)
            .map(|(_, s)| *s)
            .unwrap();
        let score_new = results[0]
            .iter()
            .find(|(d, _)| *d == 1)
            .map(|(_, s)| *s)
            .unwrap();
        assert!(
            score_old > score_new,
            "old keywords inflate split-term tf: {score_old} vs {score_new}"
        );
    }
}
