//! Clarifai-specific error types and error mapping
//!
//! Handles error conversion from Clarifai API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Clarifai-specific error types
#[derive(Debug, Error)]
pub enum ClarifaiError {
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

impl ProviderErrorTrait for ClarifaiError {
    fn error_type(&self) -> &'static str {
        match self {
            ClarifaiError::ApiError(_) => "api_error",
            ClarifaiError::AuthenticationError(_) => "authentication_error",
            ClarifaiError::RateLimitError(_) => "rate_limit_error",
            ClarifaiError::InvalidRequestError(_) => "invalid_request_error",
            ClarifaiError::ModelNotFoundError(_) => "model_not_found_error",
            ClarifaiError::ServiceUnavailableError(_) => "service_unavailable_error",
            ClarifaiError::StreamingError(_) => "streaming_error",
            ClarifaiError::ConfigurationError(_) => "configuration_error",
            ClarifaiError::NetworkError(_) => "network_error",
            ClarifaiError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            ClarifaiError::RateLimitError(_)
                | ClarifaiError::ServiceUnavailableError(_)
                | ClarifaiError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            ClarifaiError::RateLimitError(_) => Some(60),
            ClarifaiError::ServiceUnavailableError(_) => Some(5),
            ClarifaiError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            ClarifaiError::AuthenticationError(_) => 401,
            ClarifaiError::RateLimitError(_) => 429,
            ClarifaiError::InvalidRequestError(_) => 400,
            ClarifaiError::ModelNotFoundError(_) => 404,
            ClarifaiError::ServiceUnavailableError(_) => 503,
            ClarifaiError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        ClarifaiError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        ClarifaiError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => ClarifaiError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => ClarifaiError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        ClarifaiError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        ClarifaiError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        ClarifaiError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<ClarifaiError> for ProviderError {
    fn from(error: ClarifaiError) -> Self {
        match error {
            ClarifaiError::ApiError(msg) => ProviderError::api_error("clarifai", 500, msg),
            ClarifaiError::AuthenticationError(msg) => {
                ProviderError::authentication("clarifai", msg)
            }
            ClarifaiError::RateLimitError(_) => ProviderError::rate_limit("clarifai", None),
            ClarifaiError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("clarifai", msg)
            }
            ClarifaiError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("clarifai", msg)
            }
            ClarifaiError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("clarifai", 503, msg)
            }
            ClarifaiError::StreamingError(msg) => {
                ProviderError::api_error("clarifai", 500, format!("Streaming error: {}", msg))
            }
            ClarifaiError::ConfigurationError(msg) => ProviderError::configuration("clarifai", msg),
            ClarifaiError::NetworkError(msg) => ProviderError::network("clarifai", msg),
            ClarifaiError::UnknownError(msg) => ProviderError::api_error("clarifai", 500, msg),
        }
    }
}

/// Error mapper for Clarifai provider
pub struct ClarifaiErrorMapper;

impl ErrorMapper<ClarifaiError> for ClarifaiErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ClarifaiError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => ClarifaiError::InvalidRequestError(message),
            401 => ClarifaiError::AuthenticationError("Invalid API key".to_string()),
            403 => ClarifaiError::AuthenticationError("Access forbidden".to_string()),
            404 => ClarifaiError::ModelNotFoundError("Model not found".to_string()),
            429 => ClarifaiError::RateLimitError("Rate limit exceeded".to_string()),
            500 => ClarifaiError::ApiError("Internal server error".to_string()),
            502 => ClarifaiError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => ClarifaiError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => ClarifaiError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarifai_error_display() {
        let err = ClarifaiError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = ClarifaiError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = ClarifaiError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_clarifai_error_type() {
        assert_eq!(
            ClarifaiError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            ClarifaiError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            ClarifaiError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            ClarifaiError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            ClarifaiError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
    }

    #[test]
    fn test_clarifai_error_is_retryable() {
        assert!(ClarifaiError::RateLimitError("".to_string()).is_retryable());
        assert!(ClarifaiError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(ClarifaiError::NetworkError("".to_string()).is_retryable());

        assert!(!ClarifaiError::ApiError("".to_string()).is_retryable());
        assert!(!ClarifaiError::AuthenticationError("".to_string()).is_retryable());
        assert!(!ClarifaiError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_clarifai_error_retry_delay() {
        assert_eq!(
            ClarifaiError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            ClarifaiError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            ClarifaiError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(ClarifaiError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_clarifai_error_http_status() {
        assert_eq!(
            ClarifaiError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            ClarifaiError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            ClarifaiError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            ClarifaiError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            ClarifaiError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(ClarifaiError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_clarifai_error_factory_methods() {
        let err = ClarifaiError::not_supported("vision");
        assert!(matches!(err, ClarifaiError::InvalidRequestError(_)));

        let err = ClarifaiError::authentication_failed("bad key");
        assert!(matches!(err, ClarifaiError::AuthenticationError(_)));

        let err = ClarifaiError::rate_limited(Some(30));
        assert!(matches!(err, ClarifaiError::RateLimitError(_)));

        let err = ClarifaiError::network_error("connection failed");
        assert!(matches!(err, ClarifaiError::NetworkError(_)));

        let err = ClarifaiError::parsing_error("invalid json");
        assert!(matches!(err, ClarifaiError::ApiError(_)));
    }

    #[test]
    fn test_clarifai_error_to_provider_error() {
        let err: ProviderError = ClarifaiError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = ClarifaiError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = ClarifaiError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = ClarifaiError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = ClarifaiError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_clarifai_error_mapper_http_errors() {
        let mapper = ClarifaiErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, ClarifaiError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, ClarifaiError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, ClarifaiError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, ClarifaiError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, ClarifaiError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, ClarifaiError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, ClarifaiError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, ClarifaiError::ServiceUnavailableError(_)));
    }

    #[test]
    fn test_clarifai_error_mapper_empty_body() {
        let mapper = ClarifaiErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let ClarifaiError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
