//! Llamafile-specific error types and error mapping
//!
//! Handles error conversion from Llamafile API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Llamafile-specific error types
#[derive(Debug, Error)]
pub enum LlamafileError {
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

impl ProviderErrorTrait for LlamafileError {
    fn error_type(&self) -> &'static str {
        match self {
            LlamafileError::ApiError(_) => "api_error",
            LlamafileError::AuthenticationError(_) => "authentication_error",
            LlamafileError::InvalidRequestError(_) => "invalid_request_error",
            LlamafileError::ModelNotFoundError(_) => "model_not_found_error",
            LlamafileError::ServiceUnavailableError(_) => "service_unavailable_error",
            LlamafileError::StreamingError(_) => "streaming_error",
            LlamafileError::ConfigurationError(_) => "configuration_error",
            LlamafileError::NetworkError(_) => "network_error",
            LlamafileError::ConnectionRefusedError(_) => "connection_refused_error",
            LlamafileError::TimeoutError(_) => "timeout_error",
            LlamafileError::ContextLengthExceeded { .. } => "context_length_exceeded",
            LlamafileError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            LlamafileError::ServiceUnavailableError(_)
                | LlamafileError::NetworkError(_)
                | LlamafileError::ConnectionRefusedError(_)
                | LlamafileError::TimeoutError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            LlamafileError::ServiceUnavailableError(_) => Some(5),
            LlamafileError::NetworkError(_) => Some(2),
            LlamafileError::ConnectionRefusedError(_) => Some(5),
            LlamafileError::TimeoutError(_) => Some(10),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            LlamafileError::AuthenticationError(_) => 401,
            LlamafileError::InvalidRequestError(_) => 400,
            LlamafileError::ModelNotFoundError(_) => 404,
            LlamafileError::ServiceUnavailableError(_) => 503,
            LlamafileError::ContextLengthExceeded { .. } => 400,
            LlamafileError::ApiError(_) => 500,
            LlamafileError::ConnectionRefusedError(_) => 503,
            LlamafileError::TimeoutError(_) => 504,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        LlamafileError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        LlamafileError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => LlamafileError::ServiceUnavailableError(format!(
                "Rate limited, retry after {} seconds",
                seconds
            )),
            None => LlamafileError::ServiceUnavailableError("Rate limited".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        LlamafileError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        LlamafileError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        LlamafileError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<LlamafileError> for ProviderError {
    fn from(error: LlamafileError) -> Self {
        match error {
            LlamafileError::ApiError(msg) => ProviderError::api_error("llamafile", 500, msg),
            LlamafileError::AuthenticationError(msg) => {
                ProviderError::authentication("llamafile", msg)
            }
            LlamafileError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("llamafile", msg)
            }
            LlamafileError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("llamafile", msg)
            }
            LlamafileError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("llamafile", 503, msg)
            }
            LlamafileError::StreamingError(msg) => {
                ProviderError::streaming_error("llamafile", "chat", None, None, msg)
            }
            LlamafileError::ConfigurationError(msg) => {
                ProviderError::configuration("llamafile", msg)
            }
            LlamafileError::NetworkError(msg) => ProviderError::network("llamafile", msg),
            LlamafileError::ConnectionRefusedError(msg) => ProviderError::network(
                "llamafile",
                format!("Connection refused: {}. Is llamafile running?", msg),
            ),
            LlamafileError::TimeoutError(msg) => ProviderError::Timeout {
                provider: "llamafile",
                message: msg,
            },
            LlamafileError::ContextLengthExceeded { max, actual } => {
                ProviderError::ContextLengthExceeded {
                    provider: "llamafile",
                    max,
                    actual,
                }
            }
            LlamafileError::UnknownError(msg) => ProviderError::api_error("llamafile", 500, msg),
        }
    }
}

/// Error mapper for Llamafile provider
pub struct LlamafileErrorMapper;

impl ErrorMapper<LlamafileError> for LlamafileErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> LlamafileError {
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

        // Check for specific Llamafile error patterns
        let message_lower = message.to_lowercase();

        if message_lower.contains("model") && message_lower.contains("not found") {
            return LlamafileError::ModelNotFoundError(message);
        }

        if message_lower.contains("context length") || message_lower.contains("too long") {
            return LlamafileError::ContextLengthExceeded {
                max: 0,    // Unknown
                actual: 0, // Unknown
            };
        }

        match status_code {
            400 => LlamafileError::InvalidRequestError(message),
            401 => LlamafileError::AuthenticationError("Invalid API key".to_string()),
            403 => LlamafileError::AuthenticationError("Access forbidden".to_string()),
            404 => LlamafileError::ModelNotFoundError(message),
            408 | 504 => LlamafileError::TimeoutError(message),
            500 => LlamafileError::ApiError(message),
            502 | 503 => LlamafileError::ServiceUnavailableError(message),
            _ => LlamafileError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llamafile_error_display() {
        let err = LlamafileError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = LlamafileError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = LlamafileError::ConnectionRefusedError("localhost:8080".to_string());
        assert_eq!(err.to_string(), "Connection refused: localhost:8080");
    }

    #[test]
    fn test_llamafile_error_type() {
        assert_eq!(
            LlamafileError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            LlamafileError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            LlamafileError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            LlamafileError::ConnectionRefusedError("".to_string()).error_type(),
            "connection_refused_error"
        );
        assert_eq!(
            LlamafileError::TimeoutError("".to_string()).error_type(),
            "timeout_error"
        );
        assert_eq!(
            LlamafileError::ContextLengthExceeded { max: 0, actual: 0 }.error_type(),
            "context_length_exceeded"
        );
    }

    #[test]
    fn test_llamafile_error_is_retryable() {
        assert!(LlamafileError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(LlamafileError::NetworkError("".to_string()).is_retryable());
        assert!(LlamafileError::ConnectionRefusedError("".to_string()).is_retryable());
        assert!(LlamafileError::TimeoutError("".to_string()).is_retryable());

        assert!(!LlamafileError::ApiError("".to_string()).is_retryable());
        assert!(!LlamafileError::AuthenticationError("".to_string()).is_retryable());
        assert!(!LlamafileError::InvalidRequestError("".to_string()).is_retryable());
        assert!(!LlamafileError::ModelNotFoundError("".to_string()).is_retryable());
    }

    #[test]
    fn test_llamafile_error_retry_delay() {
        assert_eq!(
            LlamafileError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            LlamafileError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(
            LlamafileError::ConnectionRefusedError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            LlamafileError::TimeoutError("".to_string()).retry_delay(),
            Some(10)
        );
        assert_eq!(LlamafileError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_llamafile_error_http_status() {
        assert_eq!(
            LlamafileError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            LlamafileError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            LlamafileError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            LlamafileError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(
            LlamafileError::ConnectionRefusedError("".to_string()).http_status(),
            503
        );
        assert_eq!(
            LlamafileError::TimeoutError("".to_string()).http_status(),
            504
        );
        assert_eq!(LlamafileError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_llamafile_error_factory_methods() {
        let err = LlamafileError::not_supported("vision");
        assert!(matches!(err, LlamafileError::InvalidRequestError(_)));

        let err = LlamafileError::authentication_failed("bad key");
        assert!(matches!(err, LlamafileError::AuthenticationError(_)));

        let err = LlamafileError::rate_limited(Some(30));
        assert!(matches!(err, LlamafileError::ServiceUnavailableError(_)));

        let err = LlamafileError::network_error("connection failed");
        assert!(matches!(err, LlamafileError::NetworkError(_)));

        let err = LlamafileError::parsing_error("invalid json");
        assert!(matches!(err, LlamafileError::ApiError(_)));

        let err = LlamafileError::not_implemented("feature");
        assert!(matches!(err, LlamafileError::InvalidRequestError(_)));
    }

    #[test]
    fn test_llamafile_error_to_provider_error() {
        let err: ProviderError =
            LlamafileError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = LlamafileError::ModelNotFoundError("gpt-4".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError =
            LlamafileError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = LlamafileError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));

        let err: ProviderError = LlamafileError::TimeoutError("30s".to_string()).into();
        assert!(matches!(err, ProviderError::Timeout { .. }));

        let err: ProviderError =
            LlamafileError::ContextLengthExceeded { max: 4096, actual: 5000 }.into();
        assert!(matches!(err, ProviderError::ContextLengthExceeded { .. }));
    }

    #[test]
    fn test_llamafile_error_mapper_http_errors() {
        let mapper = LlamafileErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, LlamafileError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, LlamafileError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "model not found");
        assert!(matches!(err, LlamafileError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, LlamafileError::ApiError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, LlamafileError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(504, "gateway timeout");
        assert!(matches!(err, LlamafileError::TimeoutError(_)));
    }

    #[test]
    fn test_llamafile_error_mapper_pattern_matching() {
        let mapper = LlamafileErrorMapper;

        // Model not found pattern
        let err = mapper.map_http_error(400, "model 'llama' not found");
        assert!(matches!(err, LlamafileError::ModelNotFoundError(_)));

        // Context length pattern
        let err = mapper.map_http_error(400, "context length exceeded");
        assert!(matches!(err, LlamafileError::ContextLengthExceeded { .. }));
    }

    #[test]
    fn test_llamafile_error_mapper_json_error() {
        let mapper = LlamafileErrorMapper;

        let json_body = r#"{"error": {"message": "model not found"}}"#;
        let err = mapper.map_http_error(404, json_body);
        assert!(matches!(err, LlamafileError::ModelNotFoundError(_)));
    }

    #[test]
    fn test_llamafile_error_mapper_empty_body() {
        let mapper = LlamafileErrorMapper;
        let err = mapper.map_http_error(400, "");
        if let LlamafileError::InvalidRequestError(msg) = err {
            assert!(msg.contains("HTTP error 400"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
