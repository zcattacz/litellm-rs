//! PostgreSQL pgvector Models
//!
//! Embedding dimension configurations and model-specific settings.

use serde::{Deserialize, Serialize};

/// Common embedding model dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EmbeddingModel {
    /// OpenAI text-embedding-3-small (1536 dimensions)
    #[default]
    OpenAISmall,
    /// OpenAI text-embedding-3-large (3072 dimensions)
    OpenAILarge,
    /// OpenAI text-embedding-ada-002 (1536 dimensions)
    OpenAIAda002,
    /// Cohere embed-english-v3.0 (1024 dimensions)
    CohereEnglishV3,
    /// Cohere embed-multilingual-v3.0 (1024 dimensions)
    CohereMultilingualV3,
    /// Voyage AI voyage-2 (1024 dimensions)
    VoyageAI2,
    /// Voyage AI voyage-large-2 (1536 dimensions)
    VoyageAILarge2,
    /// Google Gecko (768 dimensions)
    GoogleGecko,
    /// Google Gecko-latest (768 dimensions)
    GoogleGeckoLatest,
    /// Sentence Transformers all-MiniLM-L6-v2 (384 dimensions)
    MiniLMv2,
    /// Sentence Transformers all-mpnet-base-v2 (768 dimensions)
    MPNetBase,
    /// BGE small-en-v1.5 (384 dimensions)
    BGESmall,
    /// BGE base-en-v1.5 (768 dimensions)
    BGEBase,
    /// BGE large-en-v1.5 (1024 dimensions)
    BGELarge,
    /// Custom dimension
    Custom(usize),
}

impl EmbeddingModel {
    /// Get the dimension for this embedding model
    pub fn dimension(&self) -> usize {
        match self {
            EmbeddingModel::OpenAISmall => 1536,
            EmbeddingModel::OpenAILarge => 3072,
            EmbeddingModel::OpenAIAda002 => 1536,
            EmbeddingModel::CohereEnglishV3 => 1024,
            EmbeddingModel::CohereMultilingualV3 => 1024,
            EmbeddingModel::VoyageAI2 => 1024,
            EmbeddingModel::VoyageAILarge2 => 1536,
            EmbeddingModel::GoogleGecko => 768,
            EmbeddingModel::GoogleGeckoLatest => 768,
            EmbeddingModel::MiniLMv2 => 384,
            EmbeddingModel::MPNetBase => 768,
            EmbeddingModel::BGESmall => 384,
            EmbeddingModel::BGEBase => 768,
            EmbeddingModel::BGELarge => 1024,
            EmbeddingModel::Custom(dim) => *dim,
        }
    }

    /// Get the model name
    pub fn name(&self) -> &str {
        match self {
            EmbeddingModel::OpenAISmall => "text-embedding-3-small",
            EmbeddingModel::OpenAILarge => "text-embedding-3-large",
            EmbeddingModel::OpenAIAda002 => "text-embedding-ada-002",
            EmbeddingModel::CohereEnglishV3 => "embed-english-v3.0",
            EmbeddingModel::CohereMultilingualV3 => "embed-multilingual-v3.0",
            EmbeddingModel::VoyageAI2 => "voyage-2",
            EmbeddingModel::VoyageAILarge2 => "voyage-large-2",
            EmbeddingModel::GoogleGecko => "textembedding-gecko",
            EmbeddingModel::GoogleGeckoLatest => "textembedding-gecko@latest",
            EmbeddingModel::MiniLMv2 => "all-MiniLM-L6-v2",
            EmbeddingModel::MPNetBase => "all-mpnet-base-v2",
            EmbeddingModel::BGESmall => "bge-small-en-v1.5",
            EmbeddingModel::BGEBase => "bge-base-en-v1.5",
            EmbeddingModel::BGELarge => "bge-large-en-v1.5",
            EmbeddingModel::Custom(_) => "custom",
        }
    }

    /// Get the provider for this model
    pub fn provider(&self) -> &str {
        match self {
            EmbeddingModel::OpenAISmall
            | EmbeddingModel::OpenAILarge
            | EmbeddingModel::OpenAIAda002 => "openai",
            EmbeddingModel::CohereEnglishV3 | EmbeddingModel::CohereMultilingualV3 => "cohere",
            EmbeddingModel::VoyageAI2 | EmbeddingModel::VoyageAILarge2 => "voyage",
            EmbeddingModel::GoogleGecko | EmbeddingModel::GoogleGeckoLatest => "google",
            EmbeddingModel::MiniLMv2 | EmbeddingModel::MPNetBase => "sentence-transformers",
            EmbeddingModel::BGESmall | EmbeddingModel::BGEBase | EmbeddingModel::BGELarge => "bge",
            EmbeddingModel::Custom(_) => "custom",
        }
    }

    /// Try to parse a model from a string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "text-embedding-3-small" | "openai-small" => Some(EmbeddingModel::OpenAISmall),
            "text-embedding-3-large" | "openai-large" => Some(EmbeddingModel::OpenAILarge),
            "text-embedding-ada-002" | "ada-002" | "ada002" => Some(EmbeddingModel::OpenAIAda002),
            "embed-english-v3.0" | "cohere-english-v3" => Some(EmbeddingModel::CohereEnglishV3),
            "embed-multilingual-v3.0" | "cohere-multilingual-v3" => {
                Some(EmbeddingModel::CohereMultilingualV3)
            }
            "voyage-2" | "voyage2" => Some(EmbeddingModel::VoyageAI2),
            "voyage-large-2" | "voyage-large2" => Some(EmbeddingModel::VoyageAILarge2),
            "textembedding-gecko" | "gecko" => Some(EmbeddingModel::GoogleGecko),
            "textembedding-gecko@latest" | "gecko-latest" => {
                Some(EmbeddingModel::GoogleGeckoLatest)
            }
            "all-minilm-l6-v2" | "minilm" | "minilm-v2" => Some(EmbeddingModel::MiniLMv2),
            "all-mpnet-base-v2" | "mpnet" => Some(EmbeddingModel::MPNetBase),
            "bge-small-en-v1.5" | "bge-small" => Some(EmbeddingModel::BGESmall),
            "bge-base-en-v1.5" | "bge-base" => Some(EmbeddingModel::BGEBase),
            "bge-large-en-v1.5" | "bge-large" => Some(EmbeddingModel::BGELarge),
            _ => None,
        }
    }
}

/// Vector point for storage in pgvector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    /// Unique identifier for the vector
    pub id: String,
    /// The vector embedding data
    pub vector: Vec<f32>,
    /// Optional metadata as JSON
    pub metadata: Option<serde_json::Value>,
    /// Optional text content associated with the vector
    pub content: Option<String>,
}

impl VectorPoint {
    /// Create a new vector point
    pub fn new(id: impl Into<String>, vector: Vec<f32>) -> Self {
        Self {
            id: id.into(),
            vector,
            metadata: None,
            content: None,
        }
    }

    /// Add metadata to the vector point
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add content to the vector point
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Get the dimension of the vector
    pub fn dimension(&self) -> usize {
        self.vector.len()
    }
}

/// Search result from pgvector similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The ID of the matching vector
    pub id: String,
    /// Similarity/distance score
    pub score: f32,
    /// Optional metadata from the stored vector
    pub metadata: Option<serde_json::Value>,
    /// Optional content from the stored vector
    pub content: Option<String>,
    /// Optional vector data (if requested)
    pub vector: Option<Vec<f32>>,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(id: impl Into<String>, score: f32) -> Self {
        Self {
            id: id.into(),
            score,
            metadata: None,
            content: None,
            vector: None,
        }
    }
}

/// Statistics about the vector table
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableStats {
    /// Total number of vectors stored
    pub total_vectors: u64,
    /// Dimension of vectors in the table
    pub dimension: usize,
    /// Index type if any
    pub index_type: Option<String>,
    /// Table size in bytes
    pub table_size_bytes: Option<u64>,
    /// Index size in bytes
    pub index_size_bytes: Option<u64>,
}

/// Options for similarity search
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub limit: usize,
    /// Minimum similarity threshold (0.0 - 1.0 for cosine, varies for others)
    pub threshold: Option<f32>,
    /// Whether to include the vector in results
    pub include_vector: bool,
    /// Whether to include metadata in results
    pub include_metadata: bool,
    /// Whether to include content in results
    pub include_content: bool,
    /// Optional filter on metadata (SQL WHERE clause fragment)
    pub metadata_filter: Option<String>,
}

impl SearchOptions {
    /// Create new search options with a limit
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            include_metadata: true,
            include_content: true,
            ..Default::default()
        }
    }

    /// Set the similarity threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Include vector data in results
    pub fn with_vector(mut self) -> Self {
        self.include_vector = true;
        self
    }

    /// Set metadata filter
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.metadata_filter = Some(filter.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_model_dimensions() {
        assert_eq!(EmbeddingModel::OpenAISmall.dimension(), 1536);
        assert_eq!(EmbeddingModel::OpenAILarge.dimension(), 3072);
        assert_eq!(EmbeddingModel::CohereEnglishV3.dimension(), 1024);
        assert_eq!(EmbeddingModel::MiniLMv2.dimension(), 384);
        assert_eq!(EmbeddingModel::Custom(512).dimension(), 512);
    }

    #[test]
    fn test_embedding_model_from_name() {
        assert_eq!(
            EmbeddingModel::from_name("text-embedding-3-small"),
            Some(EmbeddingModel::OpenAISmall)
        );
        assert_eq!(
            EmbeddingModel::from_name("ada-002"),
            Some(EmbeddingModel::OpenAIAda002)
        );
        assert_eq!(EmbeddingModel::from_name("unknown-model"), None);
    }

    #[test]
    fn test_vector_point_creation() {
        let point = VectorPoint::new("test-id", vec![0.1, 0.2, 0.3])
            .with_metadata(serde_json::json!({"key": "value"}))
            .with_content("test content");

        assert_eq!(point.id, "test-id");
        assert_eq!(point.dimension(), 3);
        assert!(point.metadata.is_some());
        assert!(point.content.is_some());
    }

    #[test]
    fn test_search_options() {
        let options = SearchOptions::new(10)
            .with_threshold(0.8)
            .with_vector()
            .with_filter("metadata->>'type' = 'document'");

        assert_eq!(options.limit, 10);
        assert_eq!(options.threshold, Some(0.8));
        assert!(options.include_vector);
        assert!(options.metadata_filter.is_some());
    }

    #[test]
    fn test_embedding_model_provider() {
        assert_eq!(EmbeddingModel::OpenAISmall.provider(), "openai");
        assert_eq!(EmbeddingModel::CohereEnglishV3.provider(), "cohere");
        assert_eq!(EmbeddingModel::VoyageAI2.provider(), "voyage");
        assert_eq!(EmbeddingModel::Custom(512).provider(), "custom");
    }
}
