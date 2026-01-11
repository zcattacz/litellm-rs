//! Error types for NanoGPT provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// NanoGPT error type (alias to unified ProviderError)
pub type NanoGPTError = ProviderError;
