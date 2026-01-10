//! Error types for Snowflake provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Snowflake error type (alias to unified ProviderError)
pub type SnowflakeError = ProviderError;
