//! NVIDIA NIM-specific error types and error mapping
//!
//! Handles error conversion from NVIDIA NIM API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// NVIDIA NIM-specific error types
#[derive(Debug, Error)]
pub enum NvidiaNimError {
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

impl ProviderErrorTrait for NvidiaNimError {
    fn error_type(&self) -> &'static str {
        match self {
            NvidiaNimError::ApiError(_) => "api_error",
            NvidiaNimError::AuthenticationError(_) => "authentication_error",
            NvidiaNimError::RateLimitError(_) => "rate_limit_error",
            NvidiaNimError::InvalidRequestError(_) => "invalid_request_error",
            NvidiaNimError::ModelNotFoundError(_) => "model_not_found_error",
            NvidiaNimError::ServiceUnavailableError(_) => "service_unavailable_error",
            NvidiaNimError::StreamingError(_) => "streaming_error",
            NvidiaNimError::ConfigurationError(_) => "configuration_error",
            NvidiaNimError::NetworkError(_) => "network_error",
            NvidiaNimError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            NvidiaNimError::RateLimitError(_)
                | NvidiaNimError::ServiceUnavailableError(_)
                | NvidiaNimError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            NvidiaNimError::RateLimitError(_) => Some(60),
            NvidiaNimError::ServiceUnavailableError(_) => Some(5),
            NvidiaNimError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            NvidiaNimError::AuthenticationError(_) => 401,
            NvidiaNimError::RateLimitError(_) => 429,
            NvidiaNimError::InvalidRequestError(_) => 400,
            NvidiaNimError::ModelNotFoundError(_) => 404,
            NvidiaNimError::ServiceUnavailableError(_) => 503,
            NvidiaNimError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        NvidiaNimError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        NvidiaNimError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => NvidiaNimError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => NvidiaNimError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        NvidiaNimError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        NvidiaNimError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        NvidiaNimError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<NvidiaNimError> for ProviderError {
    fn from(error: NvidiaNimError) -> Self {
        match error {
            NvidiaNimError::ApiError(msg) => ProviderError::api_error("nvidia_nim", 500, msg),
            NvidiaNimError::AuthenticationError(msg) => {
                ProviderError::authentication("nvidia_nim", msg)
            }
            NvidiaNimError::RateLimitError(_) => ProviderError::rate_limit("nvidia_nim", None),
            NvidiaNimError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("nvidia_nim", msg)
            }
            NvidiaNimError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("nvidia_nim", msg)
            }
            NvidiaNimError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("nvidia_nim", 503, msg)
            }
            NvidiaNimError::StreamingError(msg) => {
                ProviderError::api_error("nvidia_nim", 500, format!("Streaming error: {}", msg))
            }
            NvidiaNimError::ConfigurationError(msg) => {
                ProviderError::configuration("nvidia_nim", msg)
            }
            NvidiaNimError::NetworkError(msg) => ProviderError::network("nvidia_nim", msg),
            NvidiaNimError::UnknownError(msg) => ProviderError::api_error("nvidia_nim", 500, msg),
        }
    }
}

/// Error mapper for NVIDIA NIM provider
pub struct NvidiaNimErrorMapper;

impl ErrorMapper<NvidiaNimError> for NvidiaNimErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> NvidiaNimError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => NvidiaNimError::InvalidRequestError(message),
            401 => NvidiaNimError::AuthenticationError("Invalid API key".to_string()),
            403 => NvidiaNimError::AuthenticationError("Access forbidden".to_string()),
            404 => NvidiaNimError::ModelNotFoundError("Model not found".to_string()),
            429 => NvidiaNimError::RateLimitError("Rate limit exceeded".to_string()),
            500 => NvidiaNimError::ApiError("Internal server error".to_string()),
            502 => NvidiaNimError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => NvidiaNimError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => NvidiaNimError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvidia_nim_error_display() {
        let err = NvidiaNimError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = NvidiaNimError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = NvidiaNimError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_nvidia_nim_error_type() {
        assert_eq!(
            NvidiaNimError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            NvidiaNimError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            NvidiaNimError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            NvidiaNimError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            NvidiaNimError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
    }

    #[test]
    fn test_nvidia_nim_error_is_retryable() {
        assert!(NvidiaNimError::RateLimitError("".to_string()).is_retryable());
        assert!(NvidiaNimError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(NvidiaNimError::NetworkError("".to_string()).is_retryable());

        assert!(!NvidiaNimError::ApiError("".to_string()).is_retryable());
        assert!(!NvidiaNimError::AuthenticationError("".to_string()).is_retryable());
        assert!(!NvidiaNimError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_nvidia_nim_error_retry_delay() {
        assert_eq!(
            NvidiaNimError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            NvidiaNimError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            NvidiaNimError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(NvidiaNimError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_nvidia_nim_error_http_status() {
        assert_eq!(
            NvidiaNimError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            NvidiaNimError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            NvidiaNimError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            NvidiaNimError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            NvidiaNimError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
    }

    #[test]
    fn test_nvidia_nim_error_factory_methods() {
        let err = NvidiaNimError::not_supported("vision");
        assert!(matches!(err, NvidiaNimError::InvalidRequestError(_)));

        let err = NvidiaNimError::authentication_failed("bad key");
        assert!(matches!(err, NvidiaNimError::AuthenticationError(_)));

        let err = NvidiaNimError::rate_limited(Some(30));
        assert!(matches!(err, NvidiaNimError::RateLimitError(_)));

        let err = NvidiaNimError::network_error("connection failed");
        assert!(matches!(err, NvidiaNimError::NetworkError(_)));
    }

    #[test]
    fn test_nvidia_nim_error_to_provider_error() {
        let err: ProviderError =
            NvidiaNimError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = NvidiaNimError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = NvidiaNimError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError =
            NvidiaNimError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));
    }

    #[test]
    fn test_nvidia_nim_error_mapper_http_errors() {
        let mapper = NvidiaNimErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, NvidiaNimError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, NvidiaNimError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, NvidiaNimError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, NvidiaNimError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, NvidiaNimError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, NvidiaNimError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, NvidiaNimError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, NvidiaNimError::ServiceUnavailableError(_)));
    }
}
