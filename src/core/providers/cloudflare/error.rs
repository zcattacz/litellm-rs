//! Error types for Cloudflare provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Cloudflare error type (alias to unified ProviderError)
pub type CloudflareError = ProviderError;
