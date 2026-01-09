//! Watsonx-specific error types and error mapping
//!
//! Handles error conversion from Watsonx API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Watsonx-specific error types
#[derive(Debug, Error)]
pub enum WatsonxError {
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

    #[error("Token generation error: {0}")]
    TokenError(String),

    #[error("Project or space ID required: {0}")]
    ProjectError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for WatsonxError {
    fn error_type(&self) -> &'static str {
        match self {
            WatsonxError::ApiError(_) => "api_error",
            WatsonxError::AuthenticationError(_) => "authentication_error",
            WatsonxError::RateLimitError(_) => "rate_limit_error",
            WatsonxError::InvalidRequestError(_) => "invalid_request_error",
            WatsonxError::ModelNotFoundError(_) => "model_not_found_error",
            WatsonxError::ServiceUnavailableError(_) => "service_unavailable_error",
            WatsonxError::StreamingError(_) => "streaming_error",
            WatsonxError::ConfigurationError(_) => "configuration_error",
            WatsonxError::NetworkError(_) => "network_error",
            WatsonxError::TokenError(_) => "token_error",
            WatsonxError::ProjectError(_) => "project_error",
            WatsonxError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            WatsonxError::RateLimitError(_)
                | WatsonxError::ServiceUnavailableError(_)
                | WatsonxError::NetworkError(_)
                | WatsonxError::TokenError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            WatsonxError::RateLimitError(_) => Some(60),
            WatsonxError::ServiceUnavailableError(_) => Some(5),
            WatsonxError::NetworkError(_) => Some(2),
            WatsonxError::TokenError(_) => Some(1),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            WatsonxError::AuthenticationError(_) => 401,
            WatsonxError::RateLimitError(_) => 429,
            WatsonxError::InvalidRequestError(_) => 400,
            WatsonxError::ModelNotFoundError(_) => 404,
            WatsonxError::ServiceUnavailableError(_) => 503,
            WatsonxError::ProjectError(_) => 400,
            WatsonxError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        WatsonxError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        WatsonxError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => WatsonxError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => WatsonxError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        WatsonxError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        WatsonxError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        WatsonxError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<WatsonxError> for ProviderError {
    fn from(error: WatsonxError) -> Self {
        match error {
            WatsonxError::ApiError(msg) => ProviderError::api_error("watsonx", 500, msg),
            WatsonxError::AuthenticationError(msg) => ProviderError::authentication("watsonx", msg),
            WatsonxError::RateLimitError(_) => ProviderError::rate_limit("watsonx", None),
            WatsonxError::InvalidRequestError(msg) => ProviderError::invalid_request("watsonx", msg),
            WatsonxError::ModelNotFoundError(msg) => ProviderError::model_not_found("watsonx", msg),
            WatsonxError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("watsonx", 503, msg)
            }
            WatsonxError::StreamingError(msg) => {
                ProviderError::api_error("watsonx", 500, format!("Streaming error: {}", msg))
            }
            WatsonxError::ConfigurationError(msg) => ProviderError::configuration("watsonx", msg),
            WatsonxError::NetworkError(msg) => ProviderError::network("watsonx", msg),
            WatsonxError::TokenError(msg) => {
                ProviderError::authentication("watsonx", format!("Token error: {}", msg))
            }
            WatsonxError::ProjectError(msg) => ProviderError::configuration("watsonx", msg),
            WatsonxError::UnknownError(msg) => ProviderError::api_error("watsonx", 500, msg),
        }
    }
}

/// Error mapper for Watsonx provider
pub struct WatsonxErrorMapper;

impl ErrorMapper<WatsonxError> for WatsonxErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> WatsonxError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            // Try to extract error message from JSON response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
                json.get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        json.get("errors")
                            .and_then(|e| e.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|e| e.get("message"))
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| response_body.to_string())
            } else {
                response_body.to_string()
            }
        };

        match status_code {
            400 => WatsonxError::InvalidRequestError(message),
            401 => WatsonxError::AuthenticationError("Invalid API key or token".to_string()),
            403 => WatsonxError::AuthenticationError("Access forbidden".to_string()),
            404 => WatsonxError::ModelNotFoundError("Model or resource not found".to_string()),
            429 => WatsonxError::RateLimitError("Rate limit exceeded".to_string()),
            500 => WatsonxError::ApiError("Internal server error".to_string()),
            502 => WatsonxError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => WatsonxError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => WatsonxError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watsonx_error_display() {
        let err = WatsonxError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = WatsonxError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = WatsonxError::TokenError("token expired".to_string());
        assert_eq!(err.to_string(), "Token generation error: token expired");
    }

    #[test]
    fn test_watsonx_error_type() {
        assert_eq!(
            WatsonxError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            WatsonxError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            WatsonxError::TokenError("".to_string()).error_type(),
            "token_error"
        );
        assert_eq!(
            WatsonxError::ProjectError("".to_string()).error_type(),
            "project_error"
        );
    }

    #[test]
    fn test_watsonx_error_is_retryable() {
        assert!(WatsonxError::RateLimitError("".to_string()).is_retryable());
        assert!(WatsonxError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(WatsonxError::NetworkError("".to_string()).is_retryable());
        assert!(WatsonxError::TokenError("".to_string()).is_retryable());

        assert!(!WatsonxError::ApiError("".to_string()).is_retryable());
        assert!(!WatsonxError::AuthenticationError("".to_string()).is_retryable());
        assert!(!WatsonxError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_watsonx_error_retry_delay() {
        assert_eq!(
            WatsonxError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            WatsonxError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            WatsonxError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(
            WatsonxError::TokenError("".to_string()).retry_delay(),
            Some(1)
        );
        assert_eq!(WatsonxError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_watsonx_error_http_status() {
        assert_eq!(
            WatsonxError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            WatsonxError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            WatsonxError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            WatsonxError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            WatsonxError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(WatsonxError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_watsonx_error_factory_methods() {
        let err = WatsonxError::not_supported("vision");
        assert!(matches!(err, WatsonxError::InvalidRequestError(_)));

        let err = WatsonxError::authentication_failed("bad key");
        assert!(matches!(err, WatsonxError::AuthenticationError(_)));

        let err = WatsonxError::rate_limited(Some(30));
        assert!(matches!(err, WatsonxError::RateLimitError(_)));

        let err = WatsonxError::network_error("connection failed");
        assert!(matches!(err, WatsonxError::NetworkError(_)));
    }

    #[test]
    fn test_watsonx_error_to_provider_error() {
        let err: ProviderError = WatsonxError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = WatsonxError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = WatsonxError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = WatsonxError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = WatsonxError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_watsonx_error_mapper_http_errors() {
        let mapper = WatsonxErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, WatsonxError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, WatsonxError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, WatsonxError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, WatsonxError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, WatsonxError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, WatsonxError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, WatsonxError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, WatsonxError::ServiceUnavailableError(_)));
    }

    #[test]
    fn test_watsonx_error_mapper_json_error() {
        let mapper = WatsonxErrorMapper;
        let json_body = r#"{"error": {"message": "Invalid model ID"}}"#;
        let err = mapper.map_http_error(400, json_body);
        if let WatsonxError::InvalidRequestError(msg) = err {
            assert!(msg.contains("Invalid model ID"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
