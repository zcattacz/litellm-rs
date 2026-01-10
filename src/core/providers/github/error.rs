//! Error types for GitHub Models provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// GitHub Models error type (alias to unified ProviderError)
pub type GitHubError = ProviderError;
