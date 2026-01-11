//! Embedding types - Input and response types for the embedding API
//!
//! This module provides the input types for the embedding function.

use serde::{Deserialize, Serialize};

/// Input type for embeddings
///
/// Supports single text, multiple texts, or token arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// Single text string
    Text(String),
    /// Array of text strings
    TextArray(Vec<String>),
}

impl EmbeddingInput {
    /// Create from a single text
    pub fn text(text: impl Into<String>) -> Self {
        EmbeddingInput::Text(text.into())
    }

    /// Create from multiple texts
    pub fn texts(texts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        EmbeddingInput::TextArray(texts.into_iter().map(|t| t.into()).collect())
    }

    /// Get the number of inputs
    pub fn len(&self) -> usize {
        match self {
            EmbeddingInput::Text(_) => 1,
            EmbeddingInput::TextArray(arr) => arr.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        match self {
            EmbeddingInput::Text(t) => t.is_empty(),
            EmbeddingInput::TextArray(arr) => arr.is_empty(),
        }
    }

    /// Convert to a vector of strings
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            EmbeddingInput::Text(text) => vec![text.clone()],
            EmbeddingInput::TextArray(texts) => texts.clone(),
        }
    }

    /// Iterate over the texts
    pub fn iter(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        match self {
            EmbeddingInput::Text(text) => Box::new(std::iter::once(text)),
            EmbeddingInput::TextArray(texts) => Box::new(texts.iter()),
        }
    }
}

impl From<String> for EmbeddingInput {
    fn from(text: String) -> Self {
        EmbeddingInput::Text(text)
    }
}

impl From<&str> for EmbeddingInput {
    fn from(text: &str) -> Self {
        EmbeddingInput::Text(text.to_string())
    }
}

impl From<Vec<String>> for EmbeddingInput {
    fn from(texts: Vec<String>) -> Self {
        EmbeddingInput::TextArray(texts)
    }
}

impl From<Vec<&str>> for EmbeddingInput {
    fn from(texts: Vec<&str>) -> Self {
        EmbeddingInput::TextArray(texts.into_iter().map(|s| s.to_string()).collect())
    }
}

impl<const N: usize> From<[&str; N]> for EmbeddingInput {
    fn from(texts: [&str; N]) -> Self {
        EmbeddingInput::TextArray(texts.into_iter().map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_input_from_string() {
        let input: EmbeddingInput = "hello".into();
        match input {
            EmbeddingInput::Text(t) => assert_eq!(t, "hello"),
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_embedding_input_from_vec() {
        let input: EmbeddingInput = vec!["a", "b", "c"].into();
        match input {
            EmbeddingInput::TextArray(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], "a");
            }
            _ => panic!("Expected TextArray variant"),
        }
    }

    #[test]
    fn test_embedding_input_from_array() {
        let input: EmbeddingInput = ["one", "two"].into();
        assert_eq!(input.len(), 2);
    }

    #[test]
    fn test_embedding_input_text_constructor() {
        let input = EmbeddingInput::text("hello world");
        assert_eq!(input.len(), 1);
    }

    #[test]
    fn test_embedding_input_texts_constructor() {
        let input = EmbeddingInput::texts(["a", "b", "c"]);
        assert_eq!(input.len(), 3);
    }

    #[test]
    fn test_embedding_input_to_vec_single() {
        let input = EmbeddingInput::text("test");
        let vec = input.to_vec();
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], "test");
    }

    #[test]
    fn test_embedding_input_to_vec_multiple() {
        let input = EmbeddingInput::texts(["a", "b"]);
        let vec = input.to_vec();
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_embedding_input_iter() {
        let input = EmbeddingInput::texts(["x", "y", "z"]);
        let items: Vec<_> = input.iter().collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "x");
    }

    #[test]
    fn test_embedding_input_is_empty() {
        let empty = EmbeddingInput::text("");
        assert!(empty.is_empty());

        let empty_arr = EmbeddingInput::texts(Vec::<String>::new());
        assert!(empty_arr.is_empty());

        let non_empty = EmbeddingInput::text("hello");
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_embedding_input_serialization() {
        let single = EmbeddingInput::text("hello");
        let json = serde_json::to_value(&single).unwrap();
        assert_eq!(json, "hello");

        let multiple = EmbeddingInput::texts(["a", "b"]);
        let json = serde_json::to_value(&multiple).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0], "a");
    }

    #[test]
    fn test_embedding_input_deserialization() {
        let single: EmbeddingInput = serde_json::from_str("\"test\"").unwrap();
        assert_eq!(single.len(), 1);

        let multiple: EmbeddingInput = serde_json::from_str("[\"a\", \"b\", \"c\"]").unwrap();
        assert_eq!(multiple.len(), 3);
    }

    #[test]
    fn test_embedding_input_clone() {
        let input = EmbeddingInput::texts(["x", "y"]);
        let cloned = input.clone();
        assert_eq!(input.len(), cloned.len());
    }
}
