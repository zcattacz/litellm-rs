//! Novita-specific error types and error mapping
//!
//! Handles error conversion from Novita API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Novita-specific error types
#[derive(Debug, Error)]
pub enum NovitaError {
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

impl ProviderErrorTrait for NovitaError {
    fn error_type(&self) -> &'static str {
        match self {
            NovitaError::ApiError(_) => "api_error",
            NovitaError::AuthenticationError(_) => "authentication_error",
            NovitaError::RateLimitError(_) => "rate_limit_error",
            NovitaError::InvalidRequestError(_) => "invalid_request_error",
            NovitaError::ModelNotFoundError(_) => "model_not_found_error",
            NovitaError::ServiceUnavailableError(_) => "service_unavailable_error",
            NovitaError::StreamingError(_) => "streaming_error",
            NovitaError::ConfigurationError(_) => "configuration_error",
            NovitaError::NetworkError(_) => "network_error",
            NovitaError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            NovitaError::RateLimitError(_)
                | NovitaError::ServiceUnavailableError(_)
                | NovitaError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            NovitaError::RateLimitError(_) => Some(60), // Default 60 seconds for rate limit
            NovitaError::ServiceUnavailableError(_) => Some(5), // 5 seconds for service unavailable
            NovitaError::NetworkError(_) => Some(2),    // 2 seconds for network errors
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            NovitaError::AuthenticationError(_) => 401,
            NovitaError::RateLimitError(_) => 429,
            NovitaError::InvalidRequestError(_) => 400,
            NovitaError::ModelNotFoundError(_) => 404,
            NovitaError::ServiceUnavailableError(_) => 503,
            NovitaError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        NovitaError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        NovitaError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => NovitaError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => NovitaError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        NovitaError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        NovitaError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        NovitaError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<NovitaError> for ProviderError {
    fn from(error: NovitaError) -> Self {
        match error {
            NovitaError::ApiError(msg) => ProviderError::api_error("novita", 500, msg),
            NovitaError::AuthenticationError(msg) => ProviderError::authentication("novita", msg),
            NovitaError::RateLimitError(_) => ProviderError::rate_limit("novita", None),
            NovitaError::InvalidRequestError(msg) => ProviderError::invalid_request("novita", msg),
            NovitaError::ModelNotFoundError(msg) => ProviderError::model_not_found("novita", msg),
            NovitaError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("novita", 503, msg)
            }
            NovitaError::StreamingError(msg) => {
                ProviderError::api_error("novita", 500, format!("Streaming error: {}", msg))
            }
            NovitaError::ConfigurationError(msg) => ProviderError::configuration("novita", msg),
            NovitaError::NetworkError(msg) => ProviderError::network("novita", msg),
            NovitaError::UnknownError(msg) => ProviderError::api_error("novita", 500, msg),
        }
    }
}

/// Error mapper for Novita provider
pub struct NovitaErrorMapper;

impl ErrorMapper<NovitaError> for NovitaErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> NovitaError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => NovitaError::InvalidRequestError(message),
            401 => NovitaError::AuthenticationError("Invalid API key".to_string()),
            403 => NovitaError::AuthenticationError("Access forbidden".to_string()),
            404 => NovitaError::ModelNotFoundError("Model not found".to_string()),
            429 => NovitaError::RateLimitError("Rate limit exceeded".to_string()),
            500 => NovitaError::ApiError("Internal server error".to_string()),
            502 => NovitaError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => NovitaError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => NovitaError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novita_error_display() {
        let err = NovitaError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = NovitaError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = NovitaError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_novita_error_type() {
        assert_eq!(
            NovitaError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            NovitaError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            NovitaError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            NovitaError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            NovitaError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            NovitaError::ServiceUnavailableError("".to_string()).error_type(),
            "service_unavailable_error"
        );
        assert_eq!(
            NovitaError::StreamingError("".to_string()).error_type(),
            "streaming_error"
        );
        assert_eq!(
            NovitaError::ConfigurationError("".to_string()).error_type(),
            "configuration_error"
        );
        assert_eq!(
            NovitaError::NetworkError("".to_string()).error_type(),
            "network_error"
        );
        assert_eq!(
            NovitaError::UnknownError("".to_string()).error_type(),
            "unknown_error"
        );
    }

    #[test]
    fn test_novita_error_is_retryable() {
        assert!(NovitaError::RateLimitError("".to_string()).is_retryable());
        assert!(NovitaError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(NovitaError::NetworkError("".to_string()).is_retryable());

        assert!(!NovitaError::ApiError("".to_string()).is_retryable());
        assert!(!NovitaError::AuthenticationError("".to_string()).is_retryable());
        assert!(!NovitaError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!NovitaError::ModelNotFoundError("".to_string()).is_retryable());
    }

    #[test]
    fn test_novita_error_retry_delay() {
        assert_eq!(
            NovitaError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            NovitaError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            NovitaError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(NovitaError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_novita_error_http_status() {
        assert_eq!(
            NovitaError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            NovitaError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            NovitaError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            NovitaError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            NovitaError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(NovitaError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_novita_error_factory_methods() {
        let err = NovitaError::not_supported("vision");
        assert!(matches!(err, NovitaError::InvalidRequestError(_)));

        let err = NovitaError::authentication_failed("bad key");
        assert!(matches!(err, NovitaError::AuthenticationError(_)));

        let err = NovitaError::rate_limited(Some(30));
        assert!(matches!(err, NovitaError::RateLimitError(_)));

        let err = NovitaError::rate_limited(None);
        assert!(matches!(err, NovitaError::RateLimitError(_)));

        let err = NovitaError::network_error("connection failed");
        assert!(matches!(err, NovitaError::NetworkError(_)));

        let err = NovitaError::parsing_error("invalid json");
        assert!(matches!(err, NovitaError::ApiError(_)));

        let err = NovitaError::not_implemented("feature");
        assert!(matches!(err, NovitaError::InvalidRequestError(_)));
    }

    #[test]
    fn test_novita_error_to_provider_error() {
        let err: ProviderError = NovitaError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = NovitaError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = NovitaError::ModelNotFoundError("gpt-5".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = NovitaError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = NovitaError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_novita_error_mapper_http_errors() {
        let mapper = NovitaErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, NovitaError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, NovitaError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, NovitaError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, NovitaError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, NovitaError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, NovitaError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, NovitaError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, NovitaError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(418, "teapot");
        assert!(matches!(err, NovitaError::ApiError(_)));
    }

    #[test]
    fn test_novita_error_mapper_empty_body() {
        let mapper = NovitaErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let NovitaError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
