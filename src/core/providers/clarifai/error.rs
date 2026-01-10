//! Error types for Clarifai provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Clarifai error type (alias to unified ProviderError)
pub type ClarifaiError = ProviderError;
