//! Hyperbolic-specific error types and error mapping
//!
//! Handles error conversion from Hyperbolic API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Hyperbolic-specific error types
#[derive(Debug, Error)]
pub enum HyperbolicError {
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

impl ProviderErrorTrait for HyperbolicError {
    fn error_type(&self) -> &'static str {
        match self {
            HyperbolicError::ApiError(_) => "api_error",
            HyperbolicError::AuthenticationError(_) => "authentication_error",
            HyperbolicError::RateLimitError(_) => "rate_limit_error",
            HyperbolicError::InvalidRequestError(_) => "invalid_request_error",
            HyperbolicError::ModelNotFoundError(_) => "model_not_found_error",
            HyperbolicError::ServiceUnavailableError(_) => "service_unavailable_error",
            HyperbolicError::StreamingError(_) => "streaming_error",
            HyperbolicError::ConfigurationError(_) => "configuration_error",
            HyperbolicError::NetworkError(_) => "network_error",
            HyperbolicError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            HyperbolicError::RateLimitError(_)
                | HyperbolicError::ServiceUnavailableError(_)
                | HyperbolicError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            HyperbolicError::RateLimitError(_) => Some(60), // Default 60 seconds for rate limit
            HyperbolicError::ServiceUnavailableError(_) => Some(5), // 5 seconds for service unavailable
            HyperbolicError::NetworkError(_) => Some(2),            // 2 seconds for network errors
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            HyperbolicError::AuthenticationError(_) => 401,
            HyperbolicError::RateLimitError(_) => 429,
            HyperbolicError::InvalidRequestError(_) => 400,
            HyperbolicError::ModelNotFoundError(_) => 404,
            HyperbolicError::ServiceUnavailableError(_) => 503,
            HyperbolicError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        HyperbolicError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        HyperbolicError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => HyperbolicError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => HyperbolicError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        HyperbolicError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        HyperbolicError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        HyperbolicError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<HyperbolicError> for ProviderError {
    fn from(error: HyperbolicError) -> Self {
        match error {
            HyperbolicError::ApiError(msg) => ProviderError::api_error("hyperbolic", 500, msg),
            HyperbolicError::AuthenticationError(msg) => {
                ProviderError::authentication("hyperbolic", msg)
            }
            HyperbolicError::RateLimitError(_) => ProviderError::rate_limit("hyperbolic", None),
            HyperbolicError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("hyperbolic", msg)
            }
            HyperbolicError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("hyperbolic", msg)
            }
            HyperbolicError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("hyperbolic", 503, msg)
            }
            HyperbolicError::StreamingError(msg) => {
                ProviderError::api_error("hyperbolic", 500, format!("Streaming error: {}", msg))
            }
            HyperbolicError::ConfigurationError(msg) => {
                ProviderError::configuration("hyperbolic", msg)
            }
            HyperbolicError::NetworkError(msg) => ProviderError::network("hyperbolic", msg),
            HyperbolicError::UnknownError(msg) => ProviderError::api_error("hyperbolic", 500, msg),
        }
    }
}

/// Error mapper for Hyperbolic provider
pub struct HyperbolicErrorMapper;

impl ErrorMapper<HyperbolicError> for HyperbolicErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> HyperbolicError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => HyperbolicError::InvalidRequestError(message),
            401 => HyperbolicError::AuthenticationError("Invalid API key".to_string()),
            403 => HyperbolicError::AuthenticationError("Access forbidden".to_string()),
            404 => HyperbolicError::ModelNotFoundError("Model not found".to_string()),
            429 => HyperbolicError::RateLimitError("Rate limit exceeded".to_string()),
            500 => HyperbolicError::ApiError("Internal server error".to_string()),
            502 => HyperbolicError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => HyperbolicError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => HyperbolicError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperbolic_error_display() {
        let err = HyperbolicError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = HyperbolicError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = HyperbolicError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_hyperbolic_error_type() {
        assert_eq!(
            HyperbolicError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            HyperbolicError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            HyperbolicError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            HyperbolicError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            HyperbolicError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            HyperbolicError::ServiceUnavailableError("".to_string()).error_type(),
            "service_unavailable_error"
        );
        assert_eq!(
            HyperbolicError::StreamingError("".to_string()).error_type(),
            "streaming_error"
        );
        assert_eq!(
            HyperbolicError::ConfigurationError("".to_string()).error_type(),
            "configuration_error"
        );
        assert_eq!(
            HyperbolicError::NetworkError("".to_string()).error_type(),
            "network_error"
        );
        assert_eq!(
            HyperbolicError::UnknownError("".to_string()).error_type(),
            "unknown_error"
        );
    }

    #[test]
    fn test_hyperbolic_error_is_retryable() {
        assert!(HyperbolicError::RateLimitError("".to_string()).is_retryable());
        assert!(HyperbolicError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(HyperbolicError::NetworkError("".to_string()).is_retryable());

        assert!(!HyperbolicError::ApiError("".to_string()).is_retryable());
        assert!(!HyperbolicError::AuthenticationError("".to_string()).is_retryable());
        assert!(!HyperbolicError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!HyperbolicError::ModelNotFoundError("".to_string()).is_retryable());
    }

    #[test]
    fn test_hyperbolic_error_retry_delay() {
        assert_eq!(
            HyperbolicError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            HyperbolicError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            HyperbolicError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(
            HyperbolicError::ApiError("".to_string()).retry_delay(),
            None
        );
    }

    #[test]
    fn test_hyperbolic_error_http_status() {
        assert_eq!(
            HyperbolicError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            HyperbolicError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            HyperbolicError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            HyperbolicError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            HyperbolicError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(HyperbolicError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_hyperbolic_error_factory_methods() {
        let err = HyperbolicError::not_supported("vision");
        assert!(matches!(err, HyperbolicError::InvalidRequestError(_)));

        let err = HyperbolicError::authentication_failed("bad key");
        assert!(matches!(err, HyperbolicError::AuthenticationError(_)));

        let err = HyperbolicError::rate_limited(Some(30));
        assert!(matches!(err, HyperbolicError::RateLimitError(_)));

        let err = HyperbolicError::rate_limited(None);
        assert!(matches!(err, HyperbolicError::RateLimitError(_)));

        let err = HyperbolicError::network_error("connection failed");
        assert!(matches!(err, HyperbolicError::NetworkError(_)));

        let err = HyperbolicError::parsing_error("invalid json");
        assert!(matches!(err, HyperbolicError::ApiError(_)));

        let err = HyperbolicError::not_implemented("feature");
        assert!(matches!(err, HyperbolicError::InvalidRequestError(_)));
    }

    #[test]
    fn test_hyperbolic_error_to_provider_error() {
        let err: ProviderError = HyperbolicError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = HyperbolicError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = HyperbolicError::ModelNotFoundError("gpt-5".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError =
            HyperbolicError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = HyperbolicError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_hyperbolic_error_mapper_http_errors() {
        let mapper = HyperbolicErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, HyperbolicError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, HyperbolicError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, HyperbolicError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, HyperbolicError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, HyperbolicError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, HyperbolicError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, HyperbolicError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, HyperbolicError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(418, "teapot");
        assert!(matches!(err, HyperbolicError::ApiError(_)));
    }

    #[test]
    fn test_hyperbolic_error_mapper_empty_body() {
        let mapper = HyperbolicErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let HyperbolicError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
