//! Error types for Sagemaker provider.

pub use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Sagemaker error type (alias to unified ProviderError)
pub type SagemakerError = ProviderError;

/// Sagemaker error mapper
#[derive(Debug)]
pub struct SagemakerErrorMapper;

impl ErrorMapper<ProviderError> for SagemakerErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            400 => ProviderError::invalid_request("sagemaker", response_body),
            401 | 403 => ProviderError::authentication("sagemaker", response_body),
            404 | 424 => ProviderError::model_not_found("sagemaker", response_body),
            429 => ProviderError::rate_limit("sagemaker", None),
            502 | 503 => ProviderError::provider_unavailable("sagemaker", response_body),
            _ => ProviderError::api_error("sagemaker", status_code, response_body),
        }
    }
}
