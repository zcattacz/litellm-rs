//! LM Studio-specific error types and error mapping
//!
//! Handles error conversion from LM Studio API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// LM Studio-specific error types
#[derive(Debug, Error)]
pub enum LMStudioError {
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

impl ProviderErrorTrait for LMStudioError {
    fn error_type(&self) -> &'static str {
        match self {
            LMStudioError::ApiError(_) => "api_error",
            LMStudioError::AuthenticationError(_) => "authentication_error",
            LMStudioError::InvalidRequestError(_) => "invalid_request_error",
            LMStudioError::ModelNotFoundError(_) => "model_not_found_error",
            LMStudioError::ServiceUnavailableError(_) => "service_unavailable_error",
            LMStudioError::StreamingError(_) => "streaming_error",
            LMStudioError::ConfigurationError(_) => "configuration_error",
            LMStudioError::NetworkError(_) => "network_error",
            LMStudioError::ConnectionRefusedError(_) => "connection_refused_error",
            LMStudioError::TimeoutError(_) => "timeout_error",
            LMStudioError::ContextLengthExceeded { .. } => "context_length_exceeded",
            LMStudioError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            LMStudioError::ServiceUnavailableError(_)
                | LMStudioError::NetworkError(_)
                | LMStudioError::ConnectionRefusedError(_)
                | LMStudioError::TimeoutError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            LMStudioError::ServiceUnavailableError(_) => Some(5),
            LMStudioError::NetworkError(_) => Some(2),
            LMStudioError::ConnectionRefusedError(_) => Some(5),
            LMStudioError::TimeoutError(_) => Some(10),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            LMStudioError::AuthenticationError(_) => 401,
            LMStudioError::InvalidRequestError(_) => 400,
            LMStudioError::ModelNotFoundError(_) => 404,
            LMStudioError::ServiceUnavailableError(_) => 503,
            LMStudioError::ContextLengthExceeded { .. } => 400,
            LMStudioError::ApiError(_) => 500,
            LMStudioError::ConnectionRefusedError(_) => 503,
            LMStudioError::TimeoutError(_) => 504,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        LMStudioError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        LMStudioError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => LMStudioError::ServiceUnavailableError(format!(
                "Rate limited, retry after {} seconds",
                seconds
            )),
            None => LMStudioError::ServiceUnavailableError("Rate limited".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        LMStudioError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        LMStudioError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        LMStudioError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<LMStudioError> for ProviderError {
    fn from(error: LMStudioError) -> Self {
        match error {
            LMStudioError::ApiError(msg) => ProviderError::api_error("lm_studio", 500, msg),
            LMStudioError::AuthenticationError(msg) => {
                ProviderError::authentication("lm_studio", msg)
            }
            LMStudioError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("lm_studio", msg)
            }
            LMStudioError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("lm_studio", msg)
            }
            LMStudioError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("lm_studio", 503, msg)
            }
            LMStudioError::StreamingError(msg) => {
                ProviderError::streaming_error("lm_studio", "chat", None, None, msg)
            }
            LMStudioError::ConfigurationError(msg) => {
                ProviderError::configuration("lm_studio", msg)
            }
            LMStudioError::NetworkError(msg) => ProviderError::network("lm_studio", msg),
            LMStudioError::ConnectionRefusedError(msg) => ProviderError::network(
                "lm_studio",
                format!("Connection refused: {}. Is LM Studio running?", msg),
            ),
            LMStudioError::TimeoutError(msg) => ProviderError::Timeout {
                provider: "lm_studio",
                message: msg,
            },
            LMStudioError::ContextLengthExceeded { max, actual } => {
                ProviderError::ContextLengthExceeded {
                    provider: "lm_studio",
                    max,
                    actual,
                }
            }
            LMStudioError::UnknownError(msg) => ProviderError::api_error("lm_studio", 500, msg),
        }
    }
}

/// Error mapper for LM Studio provider
pub struct LMStudioErrorMapper;

impl ErrorMapper<LMStudioError> for LMStudioErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> LMStudioError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            // Try to parse as JSON error (OpenAI format)
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
                json.get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .or_else(|| json.get("error").and_then(|e| e.as_str()))
                    .unwrap_or(response_body)
                    .to_string()
            } else {
                response_body.to_string()
            }
        };

        // Check for specific LM Studio error patterns
        let message_lower = message.to_lowercase();

        if message_lower.contains("model") && message_lower.contains("not found") {
            return LMStudioError::ModelNotFoundError(message);
        }

        if message_lower.contains("context length") || message_lower.contains("too long") {
            return LMStudioError::ContextLengthExceeded {
                max: 0,    // Unknown
                actual: 0, // Unknown
            };
        }

        match status_code {
            400 => LMStudioError::InvalidRequestError(message),
            401 => LMStudioError::AuthenticationError("Invalid API key".to_string()),
            403 => LMStudioError::AuthenticationError("Access forbidden".to_string()),
            404 => LMStudioError::ModelNotFoundError(message),
            408 | 504 => LMStudioError::TimeoutError(message),
            500 => LMStudioError::ApiError(message),
            502 | 503 => LMStudioError::ServiceUnavailableError(message),
            _ => LMStudioError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lm_studio_error_display() {
        let err = LMStudioError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = LMStudioError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = LMStudioError::ConnectionRefusedError("localhost:1234".to_string());
        assert_eq!(err.to_string(), "Connection refused: localhost:1234");
    }

    #[test]
    fn test_lm_studio_error_type() {
        assert_eq!(
            LMStudioError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            LMStudioError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            LMStudioError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            LMStudioError::ConnectionRefusedError("".to_string()).error_type(),
            "connection_refused_error"
        );
        assert_eq!(
            LMStudioError::TimeoutError("".to_string()).error_type(),
            "timeout_error"
        );
        assert_eq!(
            LMStudioError::ContextLengthExceeded { max: 0, actual: 0 }.error_type(),
            "context_length_exceeded"
        );
    }

    #[test]
    fn test_lm_studio_error_is_retryable() {
        assert!(LMStudioError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(LMStudioError::NetworkError("".to_string()).is_retryable());
        assert!(LMStudioError::ConnectionRefusedError("".to_string()).is_retryable());
        assert!(LMStudioError::TimeoutError("".to_string()).is_retryable());

        assert!(!LMStudioError::ApiError("".to_string()).is_retryable());
        assert!(!LMStudioError::AuthenticationError("".to_string()).is_retryable());
        assert!(!LMStudioError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!LMStudioError::ModelNotFoundError("".to_string()).is_retryable());
    }

    #[test]
    fn test_lm_studio_error_retry_delay() {
        assert_eq!(
            LMStudioError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            LMStudioError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(
            LMStudioError::ConnectionRefusedError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            LMStudioError::TimeoutError("".to_string()).retry_delay(),
            Some(10)
        );
        assert_eq!(LMStudioError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_lm_studio_error_http_status() {
        assert_eq!(
            LMStudioError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            LMStudioError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            LMStudioError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            LMStudioError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(
            LMStudioError::ConnectionRefusedError("".to_string()).http_status(),
            503
        );
        assert_eq!(
            LMStudioError::TimeoutError("".to_string()).http_status(),
            504
        );
        assert_eq!(LMStudioError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_lm_studio_error_factory_methods() {
        let err = LMStudioError::not_supported("vision");
        assert!(matches!(err, LMStudioError::InvalidRequestError(_)));

        let err = LMStudioError::authentication_failed("bad key");
        assert!(matches!(err, LMStudioError::AuthenticationError(_)));

        let err = LMStudioError::rate_limited(Some(30));
        assert!(matches!(err, LMStudioError::ServiceUnavailableError(_)));

        let err = LMStudioError::network_error("connection failed");
        assert!(matches!(err, LMStudioError::NetworkError(_)));

        let err = LMStudioError::parsing_error("invalid json");
        assert!(matches!(err, LMStudioError::ApiError(_)));

        let err = LMStudioError::not_implemented("feature");
        assert!(matches!(err, LMStudioError::InvalidRequestError(_)));
    }

    #[test]
    fn test_lm_studio_error_to_provider_error() {
        let err: ProviderError = LMStudioError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = LMStudioError::ModelNotFoundError("gpt-4".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = LMStudioError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = LMStudioError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));

        let err: ProviderError = LMStudioError::TimeoutError("30s".to_string()).into();
        assert!(matches!(err, ProviderError::Timeout { .. }));

        let err: ProviderError =
            LMStudioError::ContextLengthExceeded { max: 4096, actual: 5000 }.into();
        assert!(matches!(err, ProviderError::ContextLengthExceeded { .. }));
    }

    #[test]
    fn test_lm_studio_error_mapper_http_errors() {
        let mapper = LMStudioErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, LMStudioError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, LMStudioError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "model not found");
        assert!(matches!(err, LMStudioError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, LMStudioError::ApiError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, LMStudioError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(504, "gateway timeout");
        assert!(matches!(err, LMStudioError::TimeoutError(_)));
    }

    #[test]
    fn test_lm_studio_error_mapper_pattern_matching() {
        let mapper = LMStudioErrorMapper;

        // Model not found pattern
        let err = mapper.map_http_error(400, "model 'llama3' not found");
        assert!(matches!(err, LMStudioError::ModelNotFoundError(_)));

        // Context length pattern
        let err = mapper.map_http_error(400, "context length exceeded");
        assert!(matches!(err, LMStudioError::ContextLengthExceeded { .. }));
    }

    #[test]
    fn test_lm_studio_error_mapper_json_error() {
        let mapper = LMStudioErrorMapper;

        let json_body = r#"{"error": {"message": "model not found"}}"#;
        let err = mapper.map_http_error(404, json_body);
        assert!(matches!(err, LMStudioError::ModelNotFoundError(_)));
    }

    #[test]
    fn test_lm_studio_error_mapper_empty_body() {
        let mapper = LMStudioErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let LMStudioError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
