//! Snowflake-specific error types and error mapping
//!
//! Handles error conversion from Snowflake Cortex API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Snowflake-specific error types
#[derive(Debug, Error)]
pub enum SnowflakeError {
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

    #[error("Account ID required: {0}")]
    AccountError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for SnowflakeError {
    fn error_type(&self) -> &'static str {
        match self {
            SnowflakeError::ApiError(_) => "api_error",
            SnowflakeError::AuthenticationError(_) => "authentication_error",
            SnowflakeError::RateLimitError(_) => "rate_limit_error",
            SnowflakeError::InvalidRequestError(_) => "invalid_request_error",
            SnowflakeError::ModelNotFoundError(_) => "model_not_found_error",
            SnowflakeError::ServiceUnavailableError(_) => "service_unavailable_error",
            SnowflakeError::StreamingError(_) => "streaming_error",
            SnowflakeError::ConfigurationError(_) => "configuration_error",
            SnowflakeError::NetworkError(_) => "network_error",
            SnowflakeError::AccountError(_) => "account_error",
            SnowflakeError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            SnowflakeError::RateLimitError(_)
                | SnowflakeError::ServiceUnavailableError(_)
                | SnowflakeError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            SnowflakeError::RateLimitError(_) => Some(60),
            SnowflakeError::ServiceUnavailableError(_) => Some(5),
            SnowflakeError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            SnowflakeError::AuthenticationError(_) => 401,
            SnowflakeError::RateLimitError(_) => 429,
            SnowflakeError::InvalidRequestError(_) => 400,
            SnowflakeError::ModelNotFoundError(_) => 404,
            SnowflakeError::ServiceUnavailableError(_) => 503,
            SnowflakeError::AccountError(_) => 400,
            SnowflakeError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        SnowflakeError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        SnowflakeError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => SnowflakeError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => SnowflakeError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        SnowflakeError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        SnowflakeError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        SnowflakeError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<SnowflakeError> for ProviderError {
    fn from(error: SnowflakeError) -> Self {
        match error {
            SnowflakeError::ApiError(msg) => ProviderError::api_error("snowflake", 500, msg),
            SnowflakeError::AuthenticationError(msg) => {
                ProviderError::authentication("snowflake", msg)
            }
            SnowflakeError::RateLimitError(_) => ProviderError::rate_limit("snowflake", None),
            SnowflakeError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("snowflake", msg)
            }
            SnowflakeError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("snowflake", msg)
            }
            SnowflakeError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("snowflake", 503, msg)
            }
            SnowflakeError::StreamingError(msg) => {
                ProviderError::api_error("snowflake", 500, format!("Streaming error: {}", msg))
            }
            SnowflakeError::ConfigurationError(msg) => {
                ProviderError::configuration("snowflake", msg)
            }
            SnowflakeError::NetworkError(msg) => ProviderError::network("snowflake", msg),
            SnowflakeError::AccountError(msg) => ProviderError::configuration("snowflake", msg),
            SnowflakeError::UnknownError(msg) => ProviderError::api_error("snowflake", 500, msg),
        }
    }
}

/// Error mapper for Snowflake provider
pub struct SnowflakeErrorMapper;

impl ErrorMapper<SnowflakeError> for SnowflakeErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> SnowflakeError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            // Try to extract error message from JSON response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
                json.get("message")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        json.get("error")
                            .and_then(|e| e.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| response_body.to_string())
            } else {
                response_body.to_string()
            }
        };

        match status_code {
            400 => SnowflakeError::InvalidRequestError(message),
            401 => SnowflakeError::AuthenticationError("Invalid JWT or PAT token".to_string()),
            403 => SnowflakeError::AuthenticationError("Access forbidden".to_string()),
            404 => SnowflakeError::ModelNotFoundError("Model or endpoint not found".to_string()),
            429 => SnowflakeError::RateLimitError("Rate limit exceeded".to_string()),
            500 => SnowflakeError::ApiError("Internal server error".to_string()),
            502 => SnowflakeError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => SnowflakeError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => SnowflakeError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake_error_display() {
        let err = SnowflakeError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = SnowflakeError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = SnowflakeError::AccountError("account not found".to_string());
        assert_eq!(err.to_string(), "Account ID required: account not found");
    }

    #[test]
    fn test_snowflake_error_type() {
        assert_eq!(
            SnowflakeError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            SnowflakeError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            SnowflakeError::AccountError("".to_string()).error_type(),
            "account_error"
        );
    }

    #[test]
    fn test_snowflake_error_is_retryable() {
        assert!(SnowflakeError::RateLimitError("".to_string()).is_retryable());
        assert!(SnowflakeError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(SnowflakeError::NetworkError("".to_string()).is_retryable());

        assert!(!SnowflakeError::ApiError("".to_string()).is_retryable());
        assert!(!SnowflakeError::AuthenticationError("".to_string()).is_retryable());
        assert!(!SnowflakeError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_snowflake_error_retry_delay() {
        assert_eq!(
            SnowflakeError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            SnowflakeError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            SnowflakeError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(SnowflakeError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_snowflake_error_http_status() {
        assert_eq!(
            SnowflakeError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            SnowflakeError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            SnowflakeError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            SnowflakeError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            SnowflakeError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
        assert_eq!(SnowflakeError::ApiError("".to_string()).http_status(), 500);
    }

    #[test]
    fn test_snowflake_error_factory_methods() {
        let err = SnowflakeError::not_supported("vision");
        assert!(matches!(err, SnowflakeError::InvalidRequestError(_)));

        let err = SnowflakeError::authentication_failed("bad key");
        assert!(matches!(err, SnowflakeError::AuthenticationError(_)));

        let err = SnowflakeError::rate_limited(Some(30));
        assert!(matches!(err, SnowflakeError::RateLimitError(_)));

        let err = SnowflakeError::network_error("connection failed");
        assert!(matches!(err, SnowflakeError::NetworkError(_)));
    }

    #[test]
    fn test_snowflake_error_to_provider_error() {
        let err: ProviderError = SnowflakeError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = SnowflakeError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = SnowflakeError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError =
            SnowflakeError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));

        let err: ProviderError = SnowflakeError::NetworkError("timeout".to_string()).into();
        assert!(matches!(err, ProviderError::Network { .. }));
    }

    #[test]
    fn test_snowflake_error_mapper_http_errors() {
        let mapper = SnowflakeErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, SnowflakeError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, SnowflakeError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, SnowflakeError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, SnowflakeError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, SnowflakeError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, SnowflakeError::ApiError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, SnowflakeError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, SnowflakeError::ServiceUnavailableError(_)));
    }

    #[test]
    fn test_snowflake_error_mapper_json_error() {
        let mapper = SnowflakeErrorMapper;
        let json_body = r#"{"message": "Invalid model specified"}"#;
        let err = mapper.map_http_error(400, json_body);
        if let SnowflakeError::InvalidRequestError(msg) = err {
            assert!(msg.contains("Invalid model specified"));
        } else {
            panic!("Expected InvalidRequestError");
        }
    }
}
