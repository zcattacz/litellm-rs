//! Together AI Rerank API
//!
//! Provides document reranking functionality using Together AI's rerank models.
//! Docs: <https://docs.together.ai/reference/rerank>

use serde::{Deserialize, Serialize};

/// Rerank request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    /// The model to use for reranking
    pub model: String,

    /// The query to rank documents against
    pub query: String,

    /// The documents to rerank
    pub documents: Vec<RerankDocument>,

    /// Number of top results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<u32>,

    /// Whether to return the documents in the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_documents: Option<bool>,

    /// Fields to rank on for structured documents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank_fields: Option<Vec<String>>,
}

/// A document for reranking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RerankDocument {
    /// Simple text document
    Text(String),

    /// Structured document with fields
    Structured(std::collections::HashMap<String, String>),
}

/// Rerank response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    /// Unique identifier for this request
    #[serde(default)]
    pub id: String,

    /// The model used for reranking
    #[serde(default)]
    pub model: String,

    /// The reranked results
    pub results: Vec<RerankResult>,

    /// Usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RerankUsage>,
}

/// A single rerank result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    /// The index of the document in the original list
    pub index: u32,

    /// The relevance score (higher is more relevant)
    pub relevance_score: f64,

    /// The document text (if return_documents was true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<RerankResultDocument>,
}

/// Document content in rerank result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResultDocument {
    /// The document text
    pub text: String,
}

/// Usage information for reranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankUsage {
    /// Number of search units used
    #[serde(default)]
    pub search_units: u32,

    /// Total tokens processed
    #[serde(default)]
    pub total_tokens: u32,
}

impl RerankRequest {
    /// Create a new rerank request with text documents
    pub fn new(model: impl Into<String>, query: impl Into<String>, documents: Vec<String>) -> Self {
        Self {
            model: model.into(),
            query: query.into(),
            documents: documents.into_iter().map(RerankDocument::Text).collect(),
            top_n: None,
            return_documents: Some(true),
            rank_fields: None,
        }
    }

    /// Set the number of top results to return
    pub fn with_top_n(mut self, top_n: u32) -> Self {
        self.top_n = Some(top_n);
        self
    }

    /// Set whether to return documents in the response
    pub fn with_return_documents(mut self, return_documents: bool) -> Self {
        self.return_documents = Some(return_documents);
        self
    }
}

impl RerankResponse {
    /// Get the top N results sorted by relevance score
    pub fn top_results(&self, n: usize) -> Vec<&RerankResult> {
        let mut results: Vec<_> = self.results.iter().collect();
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.into_iter().take(n).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rerank_request_creation() {
        let request = RerankRequest::new(
            "Salesforce/Llama-Rank-V1",
            "What is machine learning?",
            vec![
                "Machine learning is a subset of AI".to_string(),
                "Deep learning uses neural networks".to_string(),
            ],
        );

        assert_eq!(request.model, "Salesforce/Llama-Rank-V1");
        assert_eq!(request.query, "What is machine learning?");
        assert_eq!(request.documents.len(), 2);
        assert_eq!(request.return_documents, Some(true));
    }

    #[test]
    fn test_rerank_request_with_options() {
        let request = RerankRequest::new(
            "Salesforce/Llama-Rank-V1",
            "test query",
            vec!["doc1".to_string()],
        )
        .with_top_n(5)
        .with_return_documents(false);

        assert_eq!(request.top_n, Some(5));
        assert_eq!(request.return_documents, Some(false));
    }

    #[test]
    fn test_rerank_response_top_results() {
        let response = RerankResponse {
            id: "test-id".to_string(),
            model: "test-model".to_string(),
            results: vec![
                RerankResult {
                    index: 0,
                    relevance_score: 0.5,
                    document: None,
                },
                RerankResult {
                    index: 1,
                    relevance_score: 0.9,
                    document: None,
                },
                RerankResult {
                    index: 2,
                    relevance_score: 0.7,
                    document: None,
                },
            ],
            usage: None,
        };

        let top = response.top_results(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].index, 1); // Highest score
        assert_eq!(top[1].index, 2); // Second highest
    }

    #[test]
    fn test_rerank_request_serialization() {
        let request = RerankRequest::new(
            "model",
            "query",
            vec!["doc1".to_string(), "doc2".to_string()],
        )
        .with_top_n(3);

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "model");
        assert_eq!(json["query"], "query");
        assert_eq!(json["top_n"], 3);
        assert_eq!(json["return_documents"], true);
    }

    #[test]
    fn test_rerank_response_deserialization() {
        let json = r#"{
            "id": "rerank-123",
            "model": "Salesforce/Llama-Rank-V1",
            "results": [
                {
                    "index": 0,
                    "relevance_score": 0.95,
                    "document": {"text": "test document"}
                }
            ],
            "usage": {
                "search_units": 1,
                "total_tokens": 100
            }
        }"#;

        let response: RerankResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.id, "rerank-123");
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].index, 0);
        assert!((response.results[0].relevance_score - 0.95).abs() < f64::EPSILON);
        assert!(response.usage.is_some());
    }

    #[test]
    fn test_rerank_document_text() {
        let doc = RerankDocument::Text("simple text".to_string());
        let json = serde_json::to_string(&doc).unwrap();
        assert_eq!(json, "\"simple text\"");
    }

    #[test]
    fn test_rerank_document_structured() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("title".to_string(), "Test Title".to_string());
        fields.insert("content".to_string(), "Test Content".to_string());

        let doc = RerankDocument::Structured(fields);
        let json = serde_json::to_value(&doc).unwrap();
        assert_eq!(json["title"], "Test Title");
        assert_eq!(json["content"], "Test Content");
    }
}
