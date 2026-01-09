//! Oobabooga-specific error types and error mapping
//!
//! Handles error conversion from Oobabooga API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Oobabooga-specific error types
#[derive(Debug, Error)]
pub enum OobaboogaError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

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

    #[error("Connection refused: {0}")]
    ConnectionRefusedError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Context length exceeded: max {max}, got {actual}")]
    ContextLengthExceeded { max: usize, actual: usize },

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for OobaboogaError {
    fn error_type(&self) -> &'static str {
        match self {
            OobaboogaError::ApiError(_) => "api_error",
            OobaboogaError::AuthenticationError(_) => "authentication_error",
            OobaboogaError::InvalidRequestError(_) => "invalid_request_error",
            OobaboogaError::ModelNotFoundError(_) => "model_not_found_error",
            OobaboogaError::ServiceUnavailableError(_) => "service_unavailable_error",
            OobaboogaError::StreamingError(_) => "streaming_error",
            OobaboogaError::ConfigurationError(_) => "configuration_error",
            OobaboogaError::NetworkError(_) => "network_error",
            OobaboogaError::ConnectionRefusedError(_) => "connection_refused_error",
            OobaboogaError::TimeoutError(_) => "timeout_error",
            OobaboogaError::ContextLengthExceeded { .. } => "context_length_exceeded",
            OobaboogaError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            OobaboogaError::ServiceUnavailableError(_)
                | OobaboogaError::NetworkError(_)
                | OobaboogaError::ConnectionRefusedError(_)
                | OobaboogaError::TimeoutError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            OobaboogaError::ServiceUnavailableError(_) => Some(5),
            OobaboogaError::NetworkError(_) => Some(2),
            OobaboogaError::ConnectionRefusedError(_) => Some(5),
            OobaboogaError::TimeoutError(_) => Some(10),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            OobaboogaError::AuthenticationError(_) => 401,
            OobaboogaError::InvalidRequestError(_) => 400,
            OobaboogaError::ModelNotFoundError(_) => 404,
            OobaboogaError::ServiceUnavailableError(_) => 503,
            OobaboogaError::ContextLengthExceeded { .. } => 400,
            OobaboogaError::ApiError(_) => 500,
            OobaboogaError::ConnectionRefusedError(_) => 503,
            OobaboogaError::TimeoutError(_) => 504,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        OobaboogaError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        OobaboogaError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => OobaboogaError::ServiceUnavailableError(format!(
                "Rate limited, retry after {} seconds",
                seconds
            )),
            None => OobaboogaError::ServiceUnavailableError("Rate limited".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        OobaboogaError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        OobaboogaError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        OobaboogaError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<OobaboogaError> for ProviderError {
    fn from(error: OobaboogaError) -> Self {
        match error {
            OobaboogaError::ApiError(msg) => ProviderError::api_error("oobabooga", 500, msg),
            OobaboogaError::AuthenticationError(msg) => {
                ProviderError::authentication("oobabooga", msg)
            }
            OobaboogaError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("oobabooga", msg)
            }
            OobaboogaError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("oobabooga", msg)
            }
            OobaboogaError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("oobabooga", 503, msg)
            }
            OobaboogaError::StreamingError(msg) => {
                ProviderError::streaming_error("oobabooga", "chat", None, None, msg)
            }
            OobaboogaError::ConfigurationError(msg) => {
                ProviderError::configuration("oobabooga", msg)
            }
            OobaboogaError::NetworkError(msg) => ProviderError::network("oobabooga", msg),
            OobaboogaError::ConnectionRefusedError(msg) => ProviderError::network(
                "oobabooga",
                format!(
                    "Connection refused: {}. Is text-generation-webui running?",
                    msg
                ),
            ),
            OobaboogaError::TimeoutError(msg) => ProviderError::Timeout {
                provider: "oobabooga",
                message: msg,
            },
            OobaboogaError::ContextLengthExceeded { max, actual } => {
                ProviderError::ContextLengthExceeded {
                    provider: "oobabooga",
                    max,
                    actual,
                }
            }
            OobaboogaError::UnknownError(msg) => ProviderError::api_error("oobabooga", 500, msg),
        }
    }
}

/// Error mapper for Oobabooga provider
pub struct OobaboogaErrorMapper;

impl ErrorMapper<OobaboogaError> for OobaboogaErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> OobaboogaError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            // Try to parse as JSON error
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
                // Oobabooga might return error in different formats
                json.get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .or_else(|| json.get("error").and_then(|e| e.as_str()))
                    .or_else(|| json.get("detail").and_then(|d| d.as_str()))
                    .unwrap_or(response_body)
                    .to_string()
            } else {
                response_body.to_string()
            }
        };

        // Check for specific error patterns
        let message_lower = message.to_lowercase();

        if message_lower.contains("model") && message_lower.contains("not found") {
            return OobaboogaError::ModelNotFoundError(message);
        }

        if message_lower.contains("context length") || message_lower.contains("too long") {
            return OobaboogaError::ContextLengthExceeded {
                max: 0,    // Unknown
                actual: 0, // Unknown
            };
        }

        match status_code {
            400 => OobaboogaError::InvalidRequestError(message),
            401 => OobaboogaError::AuthenticationError("Invalid API token".to_string()),
            403 => OobaboogaError::AuthenticationError("Access forbidden".to_string()),
            404 => OobaboogaError::ModelNotFoundError(message),
            408 | 504 => OobaboogaError::TimeoutError(message),
            500 => OobaboogaError::ApiError(message),
            502 | 503 => OobaboogaError::ServiceUnavailableError(message),
            _ => OobaboogaError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oobabooga_error_display() {
        let err = OobaboogaError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = OobaboogaError::AuthenticationError("invalid token".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid token");

        let err = OobaboogaError::ConnectionRefusedError("localhost:5000".to_string());
        assert_eq!(err.to_string(), "Connection refused: localhost:5000");
    }

    #[test]
    fn test_oobabooga_error_type() {
        assert_eq!(
            OobaboogaError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            OobaboogaError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            OobaboogaError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            OobaboogaError::ConnectionRefusedError("".to_string()).error_type(),
            "connection_refused_error"
        );
        assert_eq!(
            OobaboogaError::TimeoutError("".to_string()).error_type(),
            "timeout_error"
        );
        assert_eq!(
            OobaboogaError::ContextLengthExceeded { max: 0, actual: 0 }.error_type(),
            "context_length_exceeded"
        );
    }

    #[test]
    fn test_oobabooga_error_is_retryable() {
        assert!(OobaboogaError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(OobaboogaError::NetworkError("".to_string()).is_retryable());
        assert!(OobaboogaError::ConnectionRefusedError("".to_string()).is_retryable());
        assert!(OobaboogaError::TimeoutError("".to_string()).is_retryable());

        assert!(!OobaboogaError::ApiError("".to_string()).is_retryable());
        assert!(!OobaboogaError::AuthenticationError("".to_string()).is_retryable());
        assert!(!OobaboogaError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!OobaboogaError::ModelNotFoundError("".to_string()).is_retryable());
    }

    #[test]
    fn test_oobabooga_error_retry_delay() {
        assert_eq!(
            OobaboogaError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            OobaboogaError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(
            OobaboogaError::ConnectionRefusedError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            OobaboogaError::TimeoutError("".to_string()).retry_delay(),
            Some(10)
        );
        assert_eq!(OobaboogaError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_oobabooga_error_http_status() {
        assert_eq!(
            OobaboogaError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            OobaboogaError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            OobaboogaError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            OobaboogaError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(
            OobaboogaError::ConnectionRefusedError("".to_string()).http_status(),
            503
        );
        assert_eq!(
            OobaboogaError::TimeoutError("".to_string()).http_status(),
            504
        );
        assert_eq!(OobaboogaError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_oobabooga_error_factory_methods() {
        let err = OobaboogaError::not_supported("vision");
        assert!(matches!(err, OobaboogaError::InvalidRequestError(_)));

        let err = OobaboogaError::authentication_failed("bad token");
        assert!(matches!(err, OobaboogaError::AuthenticationError(_)));

        let err = OobaboogaError::rate_limited(Some(30));
        assert!(matches!(err, OobaboogaError::ServiceUnavailableError(_)));

        let err = OobaboogaError::network_error("connection failed");
        assert!(matches!(err, OobaboogaError::NetworkError(_)));

        let err = OobaboogaError::parsing_error("invalid json");
        assert!(matches!(err, OobaboogaError::ApiError(_)));

        let err = OobaboogaError::not_implemented("feature");
        assert!(matches!(err, OobaboogaError::InvalidRequestError(_)));
    }

    #[test]
    fn test_oobabooga_error_to_provider_error() {
        let err: ProviderError =
            OobaboogaError::AuthenticationError("bad token".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = OobaboogaError::ModelNotFoundError("gpt-4".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError =
            OobaboogaError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = OobaboogaError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));

        let err: ProviderError = OobaboogaError::TimeoutError("30s".to_string()).into();
        assert!(matches!(err, ProviderError::Timeout { .. }));

        let err: ProviderError =
            OobaboogaError::ContextLengthExceeded { max: 4096, actual: 5000 }.into();
        assert!(matches!(err, ProviderError::ContextLengthExceeded { .. }));
    }

    #[test]
    fn test_oobabooga_error_mapper_http_errors() {
        let mapper = OobaboogaErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, OobaboogaError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, OobaboogaError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "model not found");
        assert!(matches!(err, OobaboogaError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, OobaboogaError::ApiError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, OobaboogaError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(504, "gateway timeout");
        assert!(matches!(err, OobaboogaError::TimeoutError(_)));
    }

    #[test]
    fn test_oobabooga_error_mapper_pattern_matching() {
        let mapper = OobaboogaErrorMapper;

        // Model not found pattern
        let err = mapper.map_http_error(400, "model 'llama' not found");
        assert!(matches!(err, OobaboogaError::ModelNotFoundError(_)));

        // Context length pattern
        let err = mapper.map_http_error(400, "context length exceeded");
        assert!(matches!(err, OobaboogaError::ContextLengthExceeded { .. }));
    }

    #[test]
    fn test_oobabooga_error_mapper_json_error() {
        let mapper = OobaboogaErrorMapper;

        let json_body = r#"{"error": {"message": "model not found"}}"#;
        let err = mapper.map_http_error(404, json_body);
        assert!(matches!(err, OobaboogaError::ModelNotFoundError(_)));

        // Test with detail field (FastAPI style)
        let json_body = r#"{"detail": "Not authenticated"}"#;
        let err = mapper.map_http_error(401, json_body);
        assert!(matches!(err, OobaboogaError::AuthenticationError(_)));
    }

    #[test]
    fn test_oobabooga_error_mapper_empty_body() {
        let mapper = OobaboogaErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let OobaboogaError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
