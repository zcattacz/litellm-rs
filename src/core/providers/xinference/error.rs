//! Error types for Xinference provider.

pub use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Xinference error type (alias to unified ProviderError)
pub type XinferenceError = ProviderError;

/// Xinference error mapper
#[derive(Debug)]
pub struct XinferenceErrorMapper;

impl ErrorMapper<ProviderError> for XinferenceErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            400 => ProviderError::invalid_request("xinference", response_body),
            401 | 403 => ProviderError::authentication("xinference", response_body),
            404 => ProviderError::model_not_found("xinference", response_body),
            429 => ProviderError::rate_limit("xinference", None),
            500..=599 => ProviderError::provider_unavailable("xinference", response_body),
            _ => ProviderError::api_error("xinference", status_code, response_body),
        }
    }
}
