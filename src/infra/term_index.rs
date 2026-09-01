use std::collections::HashMap;

use cce_storage_bm25::Bm25Document;
use cce_text::{Bm25TextCleaner, MixedTokenizer};

const FIELD_NAMES: [&str; 3] = ["title", "keywords", "content"];

/// BM25 document fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    Title = 0,
    Keywords = 1,
    Content = 2,
}

impl Field {
    pub const ALL: [Field; 3] = [Field::Title, Field::Keywords, Field::Content];

    pub fn from_index(idx: usize) -> Option<Field> {
        match idx {
            0 => Some(Field::Title),
            1 => Some(Field::Keywords),
            2 => Some(Field::Content),
            _ => None,
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }
}

/// In-memory term-document index built once and reused for all parameter sets.
///
/// All field texts are tokenized during construction. The resulting
/// postings and statistics are invariant under k1, b, and field-weight changes.
#[derive(Debug, Clone)]
pub struct InMemoryTermIndex {
    /// term → (doc_id, field_index, term_frequency)
    pub postings: HashMap<String, Vec<(u32, u8, u32)>>,
    /// term → [df_title, df_keywords, df_content]
    pub term_field_df: HashMap<String, [u32; 3]>,
    /// per-doc field lengths: [title_len, keywords_len, content_len]
    pub doc_lengths: Vec<[u32; 3]>,
    /// number of docs with non-empty content per field
    pub field_doc_count: [u32; 3],
    /// total tokens per field across all docs
    pub field_total_tokens: [u64; 3],
    pub n_docs: u32,
}

impl InMemoryTermIndex {
    /// Average raw token count of a field across ALL documents.
    ///
    /// Mirrors tantivy's `average_fieldnorm = total_num_tokens / num_docs`
    /// (the denominator is the full segment doc count, not the docs that
    /// carry the field — empty-field docs contribute a zero fieldnorm).
    pub fn avg_field_length(&self, field: Field) -> f64 {
        if self.n_docs == 0 {
            return 0.0;
        }
        self.field_total_tokens[field.index()] as f64 / self.n_docs as f64
    }
}

/// Build a term index from BM25 documents.
///
/// Tokenises each field (title, keywords, content) independently,
/// counts term frequencies per (doc, field), and records per-field
/// document frequencies.
pub fn build_term_index(documents: &[Bm25Document]) -> InMemoryTermIndex {
    let n_docs = documents.len();
    let mut postings: HashMap<String, Vec<(u32, u8, u32)>> = HashMap::new();
    let mut term_field_df: HashMap<String, [u32; 3]> = HashMap::new();
    let mut doc_lengths: Vec<[u32; 3]> = Vec::with_capacity(n_docs);
    let mut field_doc_count = [0u32; 3];
    let mut field_total_tokens = [0u64; 3];

    for (doc_id, doc) in documents.iter().enumerate() {
        let mut lengths = [0u32; 3];

        for (field_idx, field_name) in FIELD_NAMES.iter().enumerate() {
            let text = doc.get_field(field_name).map(|s| s.as_str()).unwrap_or("");
            let tokens = tokenize_text(text);
            lengths[field_idx] = tokens.len() as u32;

            if tokens.is_empty() {
                continue;
            }
            field_doc_count[field_idx] += 1;
            field_total_tokens[field_idx] += tokens.len() as u64;

            let mut tf_counter: HashMap<String, u32> = HashMap::new();
            for token in &tokens {
                *tf_counter.entry(token.clone()).or_insert(0) += 1;
            }

            for (term, tf) in tf_counter {
                postings.entry(term.clone()).or_default().push((
                    doc_id as u32,
                    field_idx as u8,
                    tf,
                ));

                let df_entry = term_field_df.entry(term).or_insert([0u32; 3]);
                df_entry[field_idx] += 1;
            }
        }

        doc_lengths.push(lengths);
    }

    InMemoryTermIndex {
        postings,
        term_field_df,
        doc_lengths,
        field_doc_count,
        field_total_tokens,
        n_docs: n_docs as u32,
    }
}

/// Tokenize text for BM25 scoring.
///
/// Uses the shared production `MixedTokenizer` so benchmark term statistics
/// mirror the real tantivy indexing behavior (identifier splitting, CJK
/// segmentation, dual-form tokens).
pub fn tokenize_text(text: &str) -> Vec<String> {
    MixedTokenizer::new().tokenize(text)
}

/// A single query token with its split/auxiliary flag.
///
/// Mirrors `MixedToken`: `is_split` is true when the token is a split form
/// (position_length == 0 in the production tokenizer), which production
/// down-weights by 0.5 when building the query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryTerm {
    pub text: String,
    pub is_split: bool,
}

/// The dual query forms produced by production `Bm25Retrieval::parse_query`:
/// a raw form (original query text) and a cleaned form (`Bm25TextCleaner`).
/// Both tokenize through the production `MixedTokenizer`.
#[derive(Debug, Clone, Default)]
pub struct QueryForms {
    pub raw: Vec<QueryTerm>,
    pub clean: Vec<QueryTerm>,
}

impl QueryForms {
    /// Returns whether any term exists across both forms.
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty() && self.clean.is_empty()
    }
}

/// Tokenize a single query form with the production tokenizer, marking split
/// (auxiliary) tokens so the scorer can apply the production 0.5 down-weight.
pub fn tokenize_query_form(text: &str) -> Vec<QueryTerm> {
    MixedTokenizer::new()
        .tokenize_offsets(text)
        .into_iter()
        .map(|t| QueryTerm {
            text: t.text,
            is_split: t.position_length == 0,
        })
        .collect()
}

/// Build the dual query forms exactly as production `parse_query` does:
/// the raw form from the original text, the clean form from the
/// `Bm25TextCleaner` output (only when cleaning actually changed the text).
pub fn build_query_forms(query_text: &str) -> QueryForms {
    let raw = tokenize_query_form(query_text);
    let cleaned = Bm25TextCleaner::new().clean(query_text);
    let clean = if !cleaned.is_empty() && cleaned != query_text {
        tokenize_query_form(&cleaned)
    } else {
        Vec::new()
    };
    QueryForms { raw, clean }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize_text("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = tokenize_text("foo.bar()");
        // MixedTokenizer emits the original form plus split forms.
        assert!(tokens.contains(&"foo.bar".to_string()));
        assert!(tokens.contains(&"foo".to_string()));
        assert!(tokens.contains(&"bar".to_string()));
    }

    #[test]
    fn test_tokenize_underscore() {
        let tokens = tokenize_text("foo_bar");
        // MixedTokenizer emits the original form plus split forms.
        assert!(tokens.contains(&"foo_bar".to_string()));
        assert!(tokens.contains(&"foo".to_string()));
        assert!(tokens.contains(&"bar".to_string()));
    }

    #[test]
    fn test_build_term_index_single_doc() {
        let doc = Bm25Document::new("test:1")
            .with_field("title", "Test Function")
            .with_field("content", "This is a test function for testing")
            .with_field("keywords", "test func");
        let index = build_term_index(&[doc]);

        assert_eq!(index.n_docs, 1);
        assert!(index.postings.contains_key("test"));
        assert!(index.postings.contains_key("function"));
        assert!(index.postings.contains_key("func"));
        assert_eq!(index.doc_lengths[0], [2, 2, 7]);
    }

    #[test]
    fn test_query_forms_identifier_marks_split_tokens() {
        let forms = build_query_forms("OnceCell::get_or_init");
        // Original token, not a split.
        assert!(
            forms
                .raw
                .iter()
                .any(|t| t.text == "oncecell::get_or_init" && !t.is_split)
        );
        // Split tokens are marked as auxiliary.
        for split in ["once", "cell", "get", "or", "init"] {
            assert!(
                forms.raw.iter().any(|t| t.text == split && t.is_split),
                "expected split token {split}"
            );
        }
    }

    #[test]
    fn test_query_forms_clean_form_uses_cleaner() {
        let forms = build_query_forms("get_or_init that returns in file x");
        assert!(forms.clean.iter().any(|t| t.text == "get_or_init"));
        // Redundant phrases are removed by the cleaner.
        assert!(!forms.clean.iter().any(|t| t.text == "that"));
        assert!(!forms.clean.iter().any(|t| t.text == "returns"));
    }

    #[test]
    fn test_query_forms_no_clean_form_when_unchanged() {
        let forms = build_query_forms("plain query");
        assert_eq!(
            forms
                .raw
                .iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>(),
            vec!["plain", "query"]
        );
        assert!(forms.clean.is_empty());
    }
}
