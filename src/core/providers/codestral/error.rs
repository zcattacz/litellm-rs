//! Codestral provider error types

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodestralError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("Model not found: {0}")]
    ModelNotFoundError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailableError(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for CodestralError {
    fn error_type(&self) -> &'static str {
        match self {
            CodestralError::ApiError(_) => "api_error",
            CodestralError::AuthenticationError(_) => "authentication_error",
            CodestralError::RateLimitError(_) => "rate_limit_error",
            CodestralError::InvalidRequestError(_) => "invalid_request_error",
            CodestralError::ModelNotFoundError(_) => "model_not_found_error",
            CodestralError::ServiceUnavailableError(_) => "service_unavailable_error",
            CodestralError::StreamingError(_) => "streaming_error",
            CodestralError::ConfigurationError(_) => "configuration_error",
            CodestralError::NetworkError(_) => "network_error",
            CodestralError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            CodestralError::RateLimitError(_)
                | CodestralError::ServiceUnavailableError(_)
                | CodestralError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            CodestralError::RateLimitError(_) => Some(60),
            CodestralError::ServiceUnavailableError(_) => Some(5),
            CodestralError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            CodestralError::AuthenticationError(_) => 401,
            CodestralError::RateLimitError(_) => 429,
            CodestralError::InvalidRequestError(_) => 400,
            CodestralError::ModelNotFoundError(_) => 404,
            CodestralError::ServiceUnavailableError(_) => 503,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        CodestralError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        CodestralError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(s) => CodestralError::RateLimitError(format!("Retry after {} seconds", s)),
            None => CodestralError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        CodestralError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        CodestralError::ApiError(format!("Parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        CodestralError::InvalidRequestError(format!("Not implemented: {}", feature))
    }
}

impl From<CodestralError> for ProviderError {
    fn from(error: CodestralError) -> Self {
        match error {
            CodestralError::ApiError(msg) => ProviderError::api_error("codestral", 500, msg),
            CodestralError::AuthenticationError(msg) => {
                ProviderError::authentication("codestral", msg)
            }
            CodestralError::RateLimitError(_) => ProviderError::rate_limit("codestral", None),
            CodestralError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("codestral", msg)
            }
            CodestralError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("codestral", msg)
            }
            CodestralError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("codestral", 503, msg)
            }
            CodestralError::StreamingError(msg) => {
                ProviderError::api_error("codestral", 500, format!("Streaming: {}", msg))
            }
            CodestralError::ConfigurationError(msg) => {
                ProviderError::configuration("codestral", msg)
            }
            CodestralError::NetworkError(msg) => ProviderError::network("codestral", msg),
            CodestralError::UnknownError(msg) => ProviderError::api_error("codestral", 500, msg),
        }
    }
}

pub struct CodestralErrorMapper;

impl ErrorMapper<CodestralError> for CodestralErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> CodestralError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => CodestralError::InvalidRequestError(message),
            401 => CodestralError::AuthenticationError("Invalid API key".to_string()),
            403 => CodestralError::AuthenticationError("Access forbidden".to_string()),
            404 => CodestralError::ModelNotFoundError("Model not found".to_string()),
            429 => CodestralError::RateLimitError("Rate limit exceeded".to_string()),
            500 => CodestralError::ApiError("Internal server error".to_string()),
            502 => CodestralError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => CodestralError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => CodestralError::ApiError(message),
        }
    }
}
