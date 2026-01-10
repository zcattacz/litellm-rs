//! Error types for Vertex AI provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Vertex AI error type (alias to unified ProviderError)
pub type VertexAIError = ProviderError;