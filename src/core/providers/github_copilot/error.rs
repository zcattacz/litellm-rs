//! Error types for GitHub Copilot provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// GitHub Copilot error type (alias to unified ProviderError)
pub type GitHubCopilotError = ProviderError;
