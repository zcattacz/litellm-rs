//! Gradient AI-specific error types and error mapping
//!
//! Handles error conversion from Gradient AI API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Gradient AI-specific error types
#[derive(Debug, Error)]
pub enum GradientAIError {
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

    #[error("Unsupported parameter: {0}")]
    UnsupportedParamsError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for GradientAIError {
    fn error_type(&self) -> &'static str {
        match self {
            GradientAIError::ApiError(_) => "api_error",
            GradientAIError::AuthenticationError(_) => "authentication_error",
            GradientAIError::RateLimitError(_) => "rate_limit_error",
            GradientAIError::InvalidRequestError(_) => "invalid_request_error",
            GradientAIError::ModelNotFoundError(_) => "model_not_found_error",
            GradientAIError::ServiceUnavailableError(_) => "service_unavailable_error",
            GradientAIError::StreamingError(_) => "streaming_error",
            GradientAIError::ConfigurationError(_) => "configuration_error",
            GradientAIError::NetworkError(_) => "network_error",
            GradientAIError::UnsupportedParamsError(_) => "unsupported_params_error",
            GradientAIError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            GradientAIError::RateLimitError(_)
                | GradientAIError::ServiceUnavailableError(_)
                | GradientAIError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            GradientAIError::RateLimitError(_) => Some(60),
            GradientAIError::ServiceUnavailableError(_) => Some(5),
            GradientAIError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            GradientAIError::AuthenticationError(_) => 401,
            GradientAIError::RateLimitError(_) => 429,
            GradientAIError::InvalidRequestError(_) => 400,
            GradientAIError::UnsupportedParamsError(_) => 400,
            GradientAIError::ModelNotFoundError(_) => 404,
            GradientAIError::ServiceUnavailableError(_) => 503,
            GradientAIError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        GradientAIError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        GradientAIError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => GradientAIError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => GradientAIError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        GradientAIError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        GradientAIError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        GradientAIError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<GradientAIError> for ProviderError {
    fn from(error: GradientAIError) -> Self {
        match error {
            GradientAIError::ApiError(msg) => ProviderError::api_error("gradient_ai", 500, msg),
            GradientAIError::AuthenticationError(msg) => {
                ProviderError::authentication("gradient_ai", msg)
            }
            GradientAIError::RateLimitError(_) => ProviderError::rate_limit("gradient_ai", None),
            GradientAIError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("gradient_ai", msg)
            }
            GradientAIError::UnsupportedParamsError(msg) => {
                ProviderError::invalid_request("gradient_ai", msg)
            }
            GradientAIError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("gradient_ai", msg)
            }
            GradientAIError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("gradient_ai", 503, msg)
            }
            GradientAIError::StreamingError(msg) => {
                ProviderError::api_error("gradient_ai", 500, format!("Streaming error: {}", msg))
            }
            GradientAIError::ConfigurationError(msg) => {
                ProviderError::configuration("gradient_ai", msg)
            }
            GradientAIError::NetworkError(msg) => ProviderError::network("gradient_ai", msg),
            GradientAIError::UnknownError(msg) => ProviderError::api_error("gradient_ai", 500, msg),
        }
    }
}

/// Error mapper for Gradient AI provider
pub struct GradientAIErrorMapper;

impl ErrorMapper<GradientAIError> for GradientAIErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> GradientAIError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => GradientAIError::InvalidRequestError(message),
            401 => GradientAIError::AuthenticationError("Invalid API key".to_string()),
            403 => GradientAIError::AuthenticationError("Access forbidden".to_string()),
            404 => GradientAIError::ModelNotFoundError("Model not found".to_string()),
            429 => GradientAIError::RateLimitError("Rate limit exceeded".to_string()),
            500 => GradientAIError::ApiError("Internal server error".to_string()),
            502 => GradientAIError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => GradientAIError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => GradientAIError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_ai_error_display() {
        let err = GradientAIError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = GradientAIError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = GradientAIError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");

        let err = GradientAIError::UnsupportedParamsError("param".to_string());
        assert_eq!(err.to_string(), "Unsupported parameter: param");
    }

    #[test]
    fn test_gradient_ai_error_type() {
        assert_eq!(
            GradientAIError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            GradientAIError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            GradientAIError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            GradientAIError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            GradientAIError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            GradientAIError::UnsupportedParamsError("".to_string()).error_type(),
            "unsupported_params_error"
        );
    }

    #[test]
    fn test_gradient_ai_error_is_retryable() {
        assert!(GradientAIError::RateLimitError("".to_string()).is_retryable());
        assert!(GradientAIError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(GradientAIError::NetworkError("".to_string()).is_retryable());

        assert!(!GradientAIError::ApiError("".to_string()).is_retryable());
        assert!(!GradientAIError::AuthenticationError("".to_string()).is_retryable());
        assert!(!GradientAIError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!GradientAIError::UnsupportedParamsError("".to_string()).is_retryable());
    }

    #[test]
    fn test_gradient_ai_error_retry_delay() {
        assert_eq!(
            GradientAIError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            GradientAIError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            GradientAIError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(
            GradientAIError::ApiError("".to_string()).retry_delay(),
            None
        );
    }

    #[test]
    fn test_gradient_ai_error_http_status() {
        assert_eq!(
            GradientAIError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            GradientAIError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            GradientAIError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            GradientAIError::UnsupportedParamsError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            GradientAIError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            GradientAIError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(GradientAIError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_gradient_ai_error_factory_methods() {
        let err = GradientAIError::not_supported("vision");
        assert!(matches!(err, GradientAIError::InvalidRequestError(_)));

        let err = GradientAIError::authentication_failed("bad key");
        assert!(matches!(err, GradientAIError::AuthenticationError(_)));

        let err = GradientAIError::rate_limited(Some(30));
        assert!(matches!(err, GradientAIError::RateLimitError(_)));

        let err = GradientAIError::network_error("connection failed");
        assert!(matches!(err, GradientAIError::NetworkError(_)));

        let err = GradientAIError::parsing_error("invalid json");
        assert!(matches!(err, GradientAIError::ApiError(_)));
    }

    #[test]
    fn test_gradient_ai_error_to_provider_error() {
        let err: ProviderError = GradientAIError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = GradientAIError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = GradientAIError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError =
            GradientAIError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = GradientAIError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));

        let err: ProviderError =
            GradientAIError::UnsupportedParamsError("param".to_string()).into();
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));
    }

    #[test]
    fn test_gradient_ai_error_mapper_http_errors() {
        let mapper = GradientAIErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, GradientAIError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, GradientAIError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, GradientAIError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, GradientAIError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, GradientAIError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, GradientAIError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, GradientAIError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, GradientAIError::ServiceUnavailableError(_)));
    }

    #[test]
    fn test_gradient_ai_error_mapper_empty_body() {
        let mapper = GradientAIErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let GradientAIError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
