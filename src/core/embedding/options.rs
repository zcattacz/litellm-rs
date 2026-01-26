//! Embedding options - Python LiteLLM compatible
//!
//! This module provides embedding configuration options with a builder pattern.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Embedding options - Python LiteLLM compatible
///
/// These options control how embeddings are generated and allow customization
/// of API behavior, authentication, and encoding format.
///
/// # Example
///
/// ```rust
/// use litellm_rs::core::embedding::EmbeddingOptions;
///
/// let options = EmbeddingOptions::new()
///     .with_api_key("sk-...")
///     .with_dimensions(1536)
///     .with_encoding_format("float");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbeddingOptions {
    /// User identifier for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Encoding format for the embeddings (e.g., "float", "base64")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,

    /// Number of dimensions for the embedding output
    /// Only supported by models that allow dimension reduction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,

    /// API key to use for this request
    /// Overrides environment variable configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// Custom API base URL
    /// Allows using custom endpoints or proxies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,

    /// Request timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// Custom headers to include in the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Task type for specialized embeddings (e.g., for Vertex AI)
    /// Values: "RETRIEVAL_QUERY", "RETRIEVAL_DOCUMENT", "SEMANTIC_SIMILARITY", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,

    /// Extra provider-specific parameters
    #[serde(default)]
    pub extra_params: HashMap<String, serde_json::Value>,
}

impl EmbeddingOptions {
    /// Create new empty options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set user identifier
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set encoding format
    pub fn with_encoding_format(mut self, format: impl Into<String>) -> Self {
        self.encoding_format = Some(format.into());
        self
    }

    /// Set output dimensions
    pub fn with_dimensions(mut self, dimensions: u32) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set API base URL
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = Some(api_base.into());
        self
    }

    /// Set request timeout in seconds
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set custom headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Add a single custom header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Set task type for specialized embeddings
    pub fn with_task_type(mut self, task_type: impl Into<String>) -> Self {
        self.task_type = Some(task_type.into());
        self
    }

    /// Add an extra parameter
    pub fn with_extra_param(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.extra_params.insert(key.into(), value.into());
        self
    }

    /// Set multiple extra parameters
    pub fn with_extra_params(mut self, params: HashMap<String, serde_json::Value>) -> Self {
        self.extra_params.extend(params);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_options_default() {
        let opts = EmbeddingOptions::default();
        assert!(opts.user.is_none());
        assert!(opts.encoding_format.is_none());
        assert!(opts.dimensions.is_none());
        assert!(opts.api_key.is_none());
        assert!(opts.api_base.is_none());
        assert!(opts.timeout.is_none());
        assert!(opts.headers.is_none());
        assert!(opts.task_type.is_none());
        assert!(opts.extra_params.is_empty());
    }

    #[test]
    fn test_embedding_options_builder() {
        let opts = EmbeddingOptions::new()
            .with_user("user-123")
            .with_encoding_format("float")
            .with_dimensions(1536)
            .with_api_key("sk-test")
            .with_api_base("https://api.example.com")
            .with_timeout(30)
            .with_task_type("RETRIEVAL_QUERY");

        assert_eq!(opts.user, Some("user-123".to_string()));
        assert_eq!(opts.encoding_format, Some("float".to_string()));
        assert_eq!(opts.dimensions, Some(1536));
        assert_eq!(opts.api_key, Some("sk-test".to_string()));
        assert_eq!(opts.api_base, Some("https://api.example.com".to_string()));
        assert_eq!(opts.timeout, Some(30));
        assert_eq!(opts.task_type, Some("RETRIEVAL_QUERY".to_string()));
    }

    #[test]
    fn test_embedding_options_headers() {
        let opts = EmbeddingOptions::new()
            .with_header("X-Custom-Header", "value1")
            .with_header("X-Another-Header", "value2");

        let headers = opts.headers.unwrap();
        assert_eq!(headers.get("X-Custom-Header"), Some(&"value1".to_string()));
        assert_eq!(headers.get("X-Another-Header"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_embedding_options_bulk_headers() {
        let mut headers = HashMap::new();
        headers.insert("Header1".to_string(), "Value1".to_string());
        headers.insert("Header2".to_string(), "Value2".to_string());

        let opts = EmbeddingOptions::new().with_headers(headers.clone());
        assert_eq!(opts.headers, Some(headers));
    }

    #[test]
    fn test_embedding_options_extra_params() {
        let opts = EmbeddingOptions::new()
            .with_extra_param("custom_field", serde_json::json!("value"))
            .with_extra_param("numeric_field", serde_json::json!(42));

        assert_eq!(
            opts.extra_params.get("custom_field"),
            Some(&serde_json::json!("value"))
        );
        assert_eq!(
            opts.extra_params.get("numeric_field"),
            Some(&serde_json::json!(42))
        );
    }

    #[test]
    fn test_embedding_options_serialization() {
        let opts = EmbeddingOptions::new()
            .with_dimensions(256)
            .with_encoding_format("base64");

        let json = serde_json::to_value(&opts).unwrap();
        assert_eq!(json["dimensions"], 256);
        assert_eq!(json["encoding_format"], "base64");
        // None values should be skipped
        assert!(!json.as_object().unwrap().contains_key("user"));
        assert!(!json.as_object().unwrap().contains_key("api_key"));
    }

    #[test]
    fn test_embedding_options_deserialization() {
        let json = r#"{
            "user": "test-user",
            "dimensions": 512,
            "encoding_format": "float"
        }"#;

        let opts: EmbeddingOptions = serde_json::from_str(json).unwrap();
        assert_eq!(opts.user, Some("test-user".to_string()));
        assert_eq!(opts.dimensions, Some(512));
        assert_eq!(opts.encoding_format, Some("float".to_string()));
    }

    #[test]
    fn test_embedding_options_clone() {
        let opts = EmbeddingOptions::new()
            .with_api_key("key")
            .with_dimensions(768);

        let cloned = opts.clone();
        assert_eq!(opts.api_key, cloned.api_key);
        assert_eq!(opts.dimensions, cloned.dimensions);
    }
}
