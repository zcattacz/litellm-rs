//! Error types for Novita provider.
//!
//! Uses the unified ProviderError type for consistent error handling
//! across all providers.

pub use crate::core::providers::unified_provider::ProviderError;

/// Novita error type (alias to unified ProviderError)
pub type NovitaError = ProviderError;
