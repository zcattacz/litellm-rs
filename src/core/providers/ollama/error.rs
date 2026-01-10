//! Error types for Ollama provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Ollama error type (alias to unified ProviderError)
pub type OllamaError = ProviderError;
