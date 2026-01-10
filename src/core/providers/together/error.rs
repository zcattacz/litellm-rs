//! Error types for Together AI provider.

pub use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Together AI error type (alias to unified ProviderError)
pub type TogetherError = ProviderError;

/// Together AI error mapper
#[derive(Debug)]
pub struct TogetherErrorMapper;

impl ErrorMapper<ProviderError> for TogetherErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            400 => ProviderError::invalid_request("together", response_body),
            401 => ProviderError::authentication("together", "Invalid API key"),
            403 => ProviderError::authentication("together", "Access forbidden"),
            404 => ProviderError::model_not_found("together", response_body),
            429 => ProviderError::rate_limit("together", None),
            500..=599 => ProviderError::provider_unavailable("together", response_body),
            _ => ProviderError::api_error("together", status_code, response_body),
        }
    }
}
