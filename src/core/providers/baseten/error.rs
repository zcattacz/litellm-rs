//! Baseten-specific error types and error mapping
//!
//! Handles error conversion from Baseten API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Baseten-specific error types
#[derive(Debug, Error)]
pub enum BasetenError {
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

impl ProviderErrorTrait for BasetenError {
    fn error_type(&self) -> &'static str {
        match self {
            BasetenError::ApiError(_) => "api_error",
            BasetenError::AuthenticationError(_) => "authentication_error",
            BasetenError::RateLimitError(_) => "rate_limit_error",
            BasetenError::InvalidRequestError(_) => "invalid_request_error",
            BasetenError::ModelNotFoundError(_) => "model_not_found_error",
            BasetenError::ServiceUnavailableError(_) => "service_unavailable_error",
            BasetenError::StreamingError(_) => "streaming_error",
            BasetenError::ConfigurationError(_) => "configuration_error",
            BasetenError::NetworkError(_) => "network_error",
            BasetenError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            BasetenError::RateLimitError(_)
                | BasetenError::ServiceUnavailableError(_)
                | BasetenError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            BasetenError::RateLimitError(_) => Some(60),
            BasetenError::ServiceUnavailableError(_) => Some(5),
            BasetenError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            BasetenError::AuthenticationError(_) => 401,
            BasetenError::RateLimitError(_) => 429,
            BasetenError::InvalidRequestError(_) => 400,
            BasetenError::ModelNotFoundError(_) => 404,
            BasetenError::ServiceUnavailableError(_) => 503,
            BasetenError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        BasetenError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        BasetenError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => BasetenError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => BasetenError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        BasetenError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        BasetenError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        BasetenError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<BasetenError> for ProviderError {
    fn from(error: BasetenError) -> Self {
        match error {
            BasetenError::ApiError(msg) => ProviderError::api_error("baseten", 500, msg),
            BasetenError::AuthenticationError(msg) => ProviderError::authentication("baseten", msg),
            BasetenError::RateLimitError(_) => ProviderError::rate_limit("baseten", None),
            BasetenError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("baseten", msg)
            }
            BasetenError::ModelNotFoundError(msg) => ProviderError::model_not_found("baseten", msg),
            BasetenError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("baseten", 503, msg)
            }
            BasetenError::StreamingError(msg) => {
                ProviderError::api_error("baseten", 500, format!("Streaming error: {}", msg))
            }
            BasetenError::ConfigurationError(msg) => ProviderError::configuration("baseten", msg),
            BasetenError::NetworkError(msg) => ProviderError::network("baseten", msg),
            BasetenError::UnknownError(msg) => ProviderError::api_error("baseten", 500, msg),
        }
    }
}

/// Error mapper for Baseten provider
pub struct BasetenErrorMapper;

impl ErrorMapper<BasetenError> for BasetenErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> BasetenError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => BasetenError::InvalidRequestError(message),
            401 => BasetenError::AuthenticationError("Invalid API key".to_string()),
            403 => BasetenError::AuthenticationError("Access forbidden".to_string()),
            404 => BasetenError::ModelNotFoundError("Model not found".to_string()),
            429 => BasetenError::RateLimitError("Rate limit exceeded".to_string()),
            500 => BasetenError::ApiError("Internal server error".to_string()),
            502 => BasetenError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => BasetenError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => BasetenError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseten_error_display() {
        let err = BasetenError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = BasetenError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = BasetenError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_baseten_error_type() {
        assert_eq!(
            BasetenError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            BasetenError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            BasetenError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            BasetenError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            BasetenError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
    }

    #[test]
    fn test_baseten_error_is_retryable() {
        assert!(BasetenError::RateLimitError("".to_string()).is_retryable());
        assert!(BasetenError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(BasetenError::NetworkError("".to_string()).is_retryable());

        assert!(!BasetenError::ApiError("".to_string()).is_retryable());
        assert!(!BasetenError::AuthenticationError("".to_string()).is_retryable());
        assert!(!BasetenError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_baseten_error_retry_delay() {
        assert_eq!(
            BasetenError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            BasetenError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            BasetenError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(BasetenError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_baseten_error_http_status() {
        assert_eq!(
            BasetenError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            BasetenError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            BasetenError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            BasetenError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            BasetenError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(BasetenError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_baseten_error_factory_methods() {
        let err = BasetenError::not_supported("vision");
        assert!(matches!(err, BasetenError::InvalidRequestError(_)));

        let err = BasetenError::authentication_failed("bad key");
        assert!(matches!(err, BasetenError::AuthenticationError(_)));

        let err = BasetenError::rate_limited(Some(30));
        assert!(matches!(err, BasetenError::RateLimitError(_)));

        let err = BasetenError::network_error("connection failed");
        assert!(matches!(err, BasetenError::NetworkError(_)));

        let err = BasetenError::parsing_error("invalid json");
        assert!(matches!(err, BasetenError::ApiError(_)));
    }

    #[test]
    fn test_baseten_error_to_provider_error() {
        let err: ProviderError = BasetenError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = BasetenError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = BasetenError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = BasetenError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = BasetenError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_baseten_error_mapper_http_errors() {
        let mapper = BasetenErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, BasetenError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, BasetenError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, BasetenError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, BasetenError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, BasetenError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, BasetenError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, BasetenError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, BasetenError::ServiceUnavailableError(_)));
    }

    #[test]
    fn test_baseten_error_mapper_empty_body() {
        let mapper = BasetenErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let BasetenError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
