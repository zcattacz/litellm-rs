//! Error types for xAI provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// xAI error type (alias to unified ProviderError)
pub type XAIError = ProviderError;
