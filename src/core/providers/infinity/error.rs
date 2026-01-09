//! Infinity-specific error types and error mapping
//!
//! Handles error conversion from Infinity API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Infinity-specific error types
#[derive(Debug, Error)]
pub enum InfinityError {
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

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for InfinityError {
    fn error_type(&self) -> &'static str {
        match self {
            InfinityError::ApiError(_) => "api_error",
            InfinityError::AuthenticationError(_) => "authentication_error",
            InfinityError::RateLimitError(_) => "rate_limit_error",
            InfinityError::InvalidRequestError(_) => "invalid_request_error",
            InfinityError::ModelNotFoundError(_) => "model_not_found_error",
            InfinityError::ServiceUnavailableError(_) => "service_unavailable_error",
            InfinityError::ConfigurationError(_) => "configuration_error",
            InfinityError::NetworkError(_) => "network_error",
            InfinityError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            InfinityError::RateLimitError(_)
                | InfinityError::ServiceUnavailableError(_)
                | InfinityError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            InfinityError::RateLimitError(_) => Some(60), // Default 60 seconds for rate limit
            InfinityError::ServiceUnavailableError(_) => Some(5), // 5 seconds for service unavailable
            InfinityError::NetworkError(_) => Some(2),            // 2 seconds for network errors
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            InfinityError::AuthenticationError(_) => 401,
            InfinityError::RateLimitError(_) => 429,
            InfinityError::InvalidRequestError(_) => 400,
            InfinityError::ModelNotFoundError(_) => 404,
            InfinityError::ServiceUnavailableError(_) => 503,
            InfinityError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        InfinityError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        InfinityError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => InfinityError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => InfinityError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        InfinityError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        InfinityError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        InfinityError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<InfinityError> for ProviderError {
    fn from(error: InfinityError) -> Self {
        match error {
            InfinityError::ApiError(msg) => ProviderError::api_error("infinity", 500, msg),
            InfinityError::AuthenticationError(msg) => {
                ProviderError::authentication("infinity", msg)
            }
            InfinityError::RateLimitError(_) => ProviderError::rate_limit("infinity", None),
            InfinityError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("infinity", msg)
            }
            InfinityError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("infinity", msg)
            }
            InfinityError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("infinity", 503, msg)
            }
            InfinityError::ConfigurationError(msg) => ProviderError::configuration("infinity", msg),
            InfinityError::NetworkError(msg) => ProviderError::network("infinity", msg),
            InfinityError::UnknownError(msg) => ProviderError::api_error("infinity", 500, msg),
        }
    }
}

/// Error mapper for Infinity provider
pub struct InfinityErrorMapper;

impl ErrorMapper<InfinityError> for InfinityErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> InfinityError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => InfinityError::InvalidRequestError(message),
            401 => InfinityError::AuthenticationError("Invalid API key".to_string()),
            403 => InfinityError::AuthenticationError("Access forbidden".to_string()),
            404 => InfinityError::ModelNotFoundError("Model not found".to_string()),
            429 => InfinityError::RateLimitError("Rate limit exceeded".to_string()),
            500 => InfinityError::ApiError("Internal server error".to_string()),
            502 => InfinityError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => InfinityError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => InfinityError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinity_error_display() {
        let err = InfinityError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = InfinityError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = InfinityError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_infinity_error_type() {
        assert_eq!(
            InfinityError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            InfinityError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            InfinityError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            InfinityError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            InfinityError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            InfinityError::ServiceUnavailableError("".to_string()).error_type(),
            "service_unavailable_error"
        );
        assert_eq!(
            InfinityError::ConfigurationError("".to_string()).error_type(),
            "configuration_error"
        );
        assert_eq!(
            InfinityError::NetworkError("".to_string()).error_type(),
            "network_error"
        );
        assert_eq!(
            InfinityError::UnknownError("".to_string()).error_type(),
            "unknown_error"
        );
    }

    #[test]
    fn test_infinity_error_is_retryable() {
        assert!(InfinityError::RateLimitError("".to_string()).is_retryable());
        assert!(InfinityError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(InfinityError::NetworkError("".to_string()).is_retryable());

        assert!(!InfinityError::ApiError("".to_string()).is_retryable());
        assert!(!InfinityError::AuthenticationError("".to_string()).is_retryable());
        assert!(!InfinityError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!InfinityError::ModelNotFoundError("".to_string()).is_retryable());
    }

    #[test]
    fn test_infinity_error_retry_delay() {
        assert_eq!(
            InfinityError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            InfinityError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            InfinityError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(InfinityError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_infinity_error_http_status() {
        assert_eq!(
            InfinityError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            InfinityError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            InfinityError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            InfinityError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            InfinityError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(InfinityError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_infinity_error_factory_methods() {
        let err = InfinityError::not_supported("vision");
        assert!(matches!(err, InfinityError::InvalidRequestError(_)));

        let err = InfinityError::authentication_failed("bad key");
        assert!(matches!(err, InfinityError::AuthenticationError(_)));

        let err = InfinityError::rate_limited(Some(30));
        assert!(matches!(err, InfinityError::RateLimitError(_)));

        let err = InfinityError::rate_limited(None);
        assert!(matches!(err, InfinityError::RateLimitError(_)));

        let err = InfinityError::network_error("connection failed");
        assert!(matches!(err, InfinityError::NetworkError(_)));

        let err = InfinityError::parsing_error("invalid json");
        assert!(matches!(err, InfinityError::ApiError(_)));

        let err = InfinityError::not_implemented("feature");
        assert!(matches!(err, InfinityError::InvalidRequestError(_)));
    }

    #[test]
    fn test_infinity_error_to_provider_error() {
        let err: ProviderError = InfinityError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = InfinityError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = InfinityError::ModelNotFoundError("gpt-5".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = InfinityError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = InfinityError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_infinity_error_mapper_http_errors() {
        let mapper = InfinityErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, InfinityError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, InfinityError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, InfinityError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, InfinityError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, InfinityError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, InfinityError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, InfinityError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, InfinityError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(418, "teapot");
        assert!(matches!(err, InfinityError::ApiError(_)));
    }

    #[test]
    fn test_infinity_error_mapper_empty_body() {
        let mapper = InfinityErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let InfinityError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
