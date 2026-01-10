//! Error types for vLLM provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// vLLM error type (alias to unified ProviderError)
pub type VLLMError = ProviderError;
