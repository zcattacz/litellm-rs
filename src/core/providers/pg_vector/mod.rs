//! PostgreSQL pgvector Provider
//!
//! Vector storage provider using PostgreSQL with the pgvector extension.
//! Provides efficient vector storage and similarity search capabilities.
//!
//! # Features
//!
//! - Vector storage with configurable dimensions (up to 16000)
//! - Multiple index types: IVFFlat, HNSW, or no index
//! - Distance metrics: L2 (Euclidean), Cosine, Inner Product
//! - Batch operations for efficient bulk inserts
//! - Metadata storage with JSONB for filtering
//! - Connection pooling support
//!
//! # Example
//!
//! ```rust,ignore
//! use litellm_rs::core::providers::pg_vector::{
//!     PgVectorConfig, PgVectorProvider, IndexType, DistanceMetric
//! };
//!
//! // Create configuration
//! let config = PgVectorConfig::new("postgresql://user:pass@localhost:5432/db")
//!     .with_table_name("embeddings")
//!     .with_dimension(1536)
//!     .with_index_type(IndexType::Hnsw)
//!     .with_distance_metric(DistanceMetric::Cosine);
//!
//! // Create provider
//! let provider = PgVectorProvider::new(config).await?;
//!
//! // Generate SQL for table creation
//! let create_table_sql = provider.create_table_sql();
//! let create_index_sql = provider.create_index_sql();
//! ```
//!
//! # SQL Generation
//!
//! The provider generates SQL statements that can be executed against PostgreSQL:
//!
//! - `create_extension_sql()` - Creates the pgvector extension
//! - `create_table_sql()` - Creates the embeddings table
//! - `create_index_sql()` - Creates the vector index (if configured)
//! - `upsert_sql()` - Inserts or updates a vector
//! - `search_sql()` - Similarity search query
//!
//! # Distance Metrics
//!
//! | Metric | Operator | Use Case |
//! |--------|----------|----------|
//! | L2 (Euclidean) | `<->` | General purpose, image embeddings |
//! | Cosine | `<=>` | Text embeddings, normalized vectors |
//! | Inner Product | `<#>` | When vectors are normalized, dot product similarity |
//!
//! # Index Types
//!
//! | Type | Pros | Cons |
//! |------|------|------|
//! | IVFFlat | Good balance of speed/accuracy | Requires training data |
//! | HNSW | Fast queries, no training needed | Higher memory usage |
//! | None | Exact results | Slow for large datasets |

mod config;
mod models;
mod provider;

// Re-export main types
pub use config::{DistanceMetric, IndexType, PROVIDER_NAME, PgVectorConfig, PgVectorConfigBuilder};
pub use models::{EmbeddingModel, SearchOptions, SearchResult, TableStats, VectorPoint};
pub use provider::{
    PgVectorExecutor, PgVectorProvider, PreparedStatement, QueryRow, StatementParam,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify all main types are accessible
        let _ = PROVIDER_NAME;
        let _ = IndexType::default();
        let _ = DistanceMetric::default();
        let _ = EmbeddingModel::default();
    }

    #[tokio::test]
    async fn test_provider_integration() {
        // Create a provider with test configuration
        let config = PgVectorConfig::new("postgresql://localhost:5432/test")
            .with_table_name("test_vectors")
            .with_dimension(768)
            .with_index_type(IndexType::Hnsw)
            .with_distance_metric(DistanceMetric::Cosine);

        let provider = PgVectorProvider::new(config).await.unwrap();

        // Verify SQL generation
        let create_ext = provider.create_extension_sql();
        assert!(create_ext.contains("vector"));

        let create_table = provider.create_table_sql();
        assert!(create_table.contains("test_vectors"));
        assert!(create_table.contains("vector(768)"));

        let create_index = provider.create_index_sql();
        assert!(create_index.is_some());
        assert!(create_index.unwrap().contains("hnsw"));
    }

    #[test]
    fn test_embedding_models() {
        // Test common embedding model dimensions
        assert_eq!(EmbeddingModel::OpenAISmall.dimension(), 1536);
        assert_eq!(EmbeddingModel::OpenAILarge.dimension(), 3072);
        assert_eq!(EmbeddingModel::CohereEnglishV3.dimension(), 1024);
        assert_eq!(EmbeddingModel::MiniLMv2.dimension(), 384);
    }

    #[test]
    fn test_search_options() {
        let options = SearchOptions::new(10)
            .with_threshold(0.8)
            .with_vector()
            .with_filter_eq("type", "document");

        assert_eq!(options.limit, 10);
        assert_eq!(options.threshold, Some(0.8));
        assert!(options.include_vector);
        assert!(options.metadata_filters.is_some());
    }
}
