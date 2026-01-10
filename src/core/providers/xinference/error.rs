//! Xinference provider error types

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Xinference-specific error types
#[derive(Debug, Error)]
pub enum XinferenceError {
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

impl ProviderErrorTrait for XinferenceError {
    fn error_type(&self) -> &'static str {
        match self {
            XinferenceError::ApiError(_) => "api_error",
            XinferenceError::AuthenticationError(_) => "authentication_error",
            XinferenceError::RateLimitError(_) => "rate_limit_error",
            XinferenceError::InvalidRequestError(_) => "invalid_request_error",
            XinferenceError::ModelNotFoundError(_) => "model_not_found_error",
            XinferenceError::ServiceUnavailableError(_) => "service_unavailable_error",
            XinferenceError::StreamingError(_) => "streaming_error",
            XinferenceError::ConfigurationError(_) => "configuration_error",
            XinferenceError::NetworkError(_) => "network_error",
            XinferenceError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            XinferenceError::RateLimitError(_)
                | XinferenceError::ServiceUnavailableError(_)
                | XinferenceError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            XinferenceError::RateLimitError(_) => Some(30),
            XinferenceError::ServiceUnavailableError(_) => Some(5),
            XinferenceError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            XinferenceError::AuthenticationError(_) => 401,
            XinferenceError::RateLimitError(_) => 429,
            XinferenceError::InvalidRequestError(_) => 400,
            XinferenceError::ModelNotFoundError(_) => 404,
            XinferenceError::ServiceUnavailableError(_) => 503,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        XinferenceError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        XinferenceError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => XinferenceError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => XinferenceError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        XinferenceError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        XinferenceError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        XinferenceError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<XinferenceError> for ProviderError {
    fn from(error: XinferenceError) -> Self {
        match error {
            XinferenceError::ApiError(msg) => ProviderError::api_error("xinference", 500, msg),
            XinferenceError::AuthenticationError(msg) => {
                ProviderError::authentication("xinference", msg)
            }
            XinferenceError::RateLimitError(_) => ProviderError::rate_limit("xinference", None),
            XinferenceError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("xinference", msg)
            }
            XinferenceError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("xinference", msg)
            }
            XinferenceError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("xinference", 503, msg)
            }
            XinferenceError::StreamingError(msg) => {
                ProviderError::api_error("xinference", 500, format!("Streaming error: {}", msg))
            }
            XinferenceError::ConfigurationError(msg) => {
                ProviderError::configuration("xinference", msg)
            }
            XinferenceError::NetworkError(msg) => ProviderError::network("xinference", msg),
            XinferenceError::UnknownError(msg) => ProviderError::api_error("xinference", 500, msg),
        }
    }
}

/// Error mapper for Xinference provider
pub struct XinferenceErrorMapper;

impl ErrorMapper<XinferenceError> for XinferenceErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> XinferenceError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => XinferenceError::InvalidRequestError(message),
            401 => XinferenceError::AuthenticationError("Invalid API key".to_string()),
            403 => XinferenceError::AuthenticationError("Access forbidden".to_string()),
            404 => XinferenceError::ModelNotFoundError("Model not found".to_string()),
            429 => XinferenceError::RateLimitError("Rate limit exceeded".to_string()),
            500 => XinferenceError::ApiError("Internal server error".to_string()),
            502 => XinferenceError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => XinferenceError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => XinferenceError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = XinferenceError::ApiError("test".to_string());
        assert_eq!(err.to_string(), "API error: test");
    }

    #[test]
    fn test_error_is_retryable() {
        assert!(XinferenceError::RateLimitError("".to_string()).is_retryable());
        assert!(XinferenceError::NetworkError("".to_string()).is_retryable());
        assert!(!XinferenceError::ApiError("".to_string()).is_retryable());
    }

    #[test]
    fn test_error_mapper() {
        let mapper = XinferenceErrorMapper;
        let err = mapper.map_http_error(404, "not found");
        assert!(matches!(err, XinferenceError::ModelNotFoundError(_)));
    }
}
