//! Main LiteLLM error types

use super::config::ConfigError;
use crate::core::router::error::RouterError;

/// Top-level error type for the LiteLLM gateway
#[derive(Debug, thiserror::Error)]
pub enum LiteLLMError {
    /// Provider-specific error
    #[error("Provider error ({provider}): {message}")]
    Provider {
        provider: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Request routing error
    #[error("Routing error: {0}")]
    Routing(RouterError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(ConfigError),

    /// Authentication failure
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization/permission denied
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Request validation error
    #[error("Validation error: {field}: {message}")]
    Validation { field: String, message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after: Option<u64>,
    },

    /// Network connectivity error
    #[error("Network error: {0}")]
    Network(String),

    /// Operation timeout
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// Response parsing error
    #[error("Parsing error: {0}")]
    Parsing(String),

    /// JSON serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Cache operation error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Internal system error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Resource not found
    #[error("Not found: {resource}")]
    NotFound { resource: String },

    /// Unsupported operation
    #[error("Unsupported operation: {operation}")]
    UnsupportedOperation { operation: String },
}

impl From<RouterError> for LiteLLMError {
    fn from(err: RouterError) -> Self {
        LiteLLMError::Routing(err)
    }
}

impl From<ConfigError> for LiteLLMError {
    fn from(err: ConfigError) -> Self {
        LiteLLMError::Configuration(err)
    }
}

impl LiteLLMError {
    pub fn provider_error(provider: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn provider_error_with_source(
        provider: impl Into<String>,
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::Provider {
            provider: provider.into(),
            message: message.into(),
            source: Some(source),
        }
    }

    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication(message.into())
    }

    pub fn authorization(message: impl Into<String>) -> Self {
        Self::Authorization(message.into())
    }

    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn rate_limit(message: impl Into<String>, retry_after: Option<u64>) -> Self {
        Self::RateLimit {
            message: message.into(),
            retry_after,
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    pub fn parsing(message: impl Into<String>) -> Self {
        Self::Parsing(message.into())
    }

    pub fn cache(message: impl Into<String>) -> Self {
        Self::Cache(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable(message.into())
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn unsupported_operation(operation: impl Into<String>) -> Self {
        Self::UnsupportedOperation {
            operation: operation.into(),
        }
    }
}

/// HTTP status code mapping
impl LiteLLMError {
    pub fn to_http_status(&self) -> u16 {
        match self {
            Self::Authentication(_) => 401,
            Self::Authorization(_) => 403,
            Self::NotFound { .. } => 404,
            Self::UnsupportedOperation { .. } => 405,
            Self::RateLimit { .. } => 429,
            Self::Validation { .. } => 400,
            Self::Configuration(_) => 400,
            Self::Network(_) | Self::ServiceUnavailable(_) => 503,
            Self::Timeout { .. } => 504,
            Self::Provider { .. }
            | Self::Routing(_)
            | Self::Internal(_)
            | Self::Parsing(_)
            | Self::Serialization(_)
            | Self::Cache(_) => 500,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_)
                | Self::Timeout { .. }
                | Self::ServiceUnavailable(_)
                | Self::RateLimit { .. }
                | Self::Provider { .. }
                | Self::Internal(_)
        )
    }

    /// Get retry delay
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimit { retry_after, .. } => *retry_after,
            Self::Network(_) | Self::Timeout { .. } => Some(1),
            Self::ServiceUnavailable(_) => Some(5),
            Self::Internal(_) => Some(1),
            _ => None,
        }
    }
}

/// Result type alias
pub type LiteLLMResult<T> = Result<T, LiteLLMError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Constructor Tests ====================

    #[test]
    fn test_provider_error() {
        let err = LiteLLMError::provider_error("openai", "API key invalid");
        assert!(matches!(err, LiteLLMError::Provider { .. }));
        assert!(err.to_string().contains("openai"));
        assert!(err.to_string().contains("API key invalid"));
    }

    #[test]
    fn test_provider_error_with_source() {
        let source = std::io::Error::other("source error");
        let err = LiteLLMError::provider_error_with_source(
            "anthropic",
            "Connection failed",
            Box::new(source),
        );

        if let LiteLLMError::Provider {
            provider,
            message,
            source,
        } = err
        {
            assert_eq!(provider, "anthropic");
            assert_eq!(message, "Connection failed");
            assert!(source.is_some());
        } else {
            panic!("Expected Provider error");
        }
    }

    #[test]
    fn test_authentication_error() {
        let err = LiteLLMError::authentication("Invalid API key");
        assert!(matches!(err, LiteLLMError::Authentication(_)));
        assert!(err.to_string().contains("Invalid API key"));
    }

    #[test]
    fn test_authorization_error() {
        let err = LiteLLMError::authorization("Insufficient permissions");
        assert!(matches!(err, LiteLLMError::Authorization(_)));
        assert!(err.to_string().contains("Insufficient permissions"));
    }

    #[test]
    fn test_validation_error() {
        let err = LiteLLMError::validation("model", "Model name required");
        if let LiteLLMError::Validation { field, message } = err {
            assert_eq!(field, "model");
            assert_eq!(message, "Model name required");
        } else {
            panic!("Expected Validation error");
        }
    }

    #[test]
    fn test_rate_limit_error_with_retry() {
        let err = LiteLLMError::rate_limit("Too many requests", Some(60));
        if let LiteLLMError::RateLimit {
            message,
            retry_after,
        } = err
        {
            assert_eq!(message, "Too many requests");
            assert_eq!(retry_after, Some(60));
        } else {
            panic!("Expected RateLimit error");
        }
    }

    #[test]
    fn test_rate_limit_error_without_retry() {
        let err = LiteLLMError::rate_limit("Slow down", None);
        if let LiteLLMError::RateLimit { retry_after, .. } = err {
            assert!(retry_after.is_none());
        } else {
            panic!("Expected RateLimit error");
        }
    }

    #[test]
    fn test_network_error() {
        let err = LiteLLMError::network("Connection refused");
        assert!(matches!(err, LiteLLMError::Network(_)));
        assert!(err.to_string().contains("Connection refused"));
    }

    #[test]
    fn test_timeout_error() {
        let err = LiteLLMError::timeout("completion request");
        if let LiteLLMError::Timeout { operation } = err {
            assert_eq!(operation, "completion request");
        } else {
            panic!("Expected Timeout error");
        }
    }

    #[test]
    fn test_parsing_error() {
        let err = LiteLLMError::parsing("Invalid JSON response");
        assert!(matches!(err, LiteLLMError::Parsing(_)));
    }

    #[test]
    fn test_cache_error() {
        let err = LiteLLMError::cache("Redis connection lost");
        assert!(matches!(err, LiteLLMError::Cache(_)));
    }

    #[test]
    fn test_internal_error() {
        let err = LiteLLMError::internal("Unexpected state");
        assert!(matches!(err, LiteLLMError::Internal(_)));
    }

    #[test]
    fn test_service_unavailable_error() {
        let err = LiteLLMError::service_unavailable("Backend overloaded");
        assert!(matches!(err, LiteLLMError::ServiceUnavailable(_)));
    }

    #[test]
    fn test_not_found_error() {
        let err = LiteLLMError::not_found("model/gpt-5");
        if let LiteLLMError::NotFound { resource } = err {
            assert_eq!(resource, "model/gpt-5");
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[test]
    fn test_unsupported_operation_error() {
        let err = LiteLLMError::unsupported_operation("image generation");
        if let LiteLLMError::UnsupportedOperation { operation } = err {
            assert_eq!(operation, "image generation");
        } else {
            panic!("Expected UnsupportedOperation error");
        }
    }

    // ==================== HTTP Status Mapping Tests ====================

    #[test]
    fn test_http_status_authentication() {
        let err = LiteLLMError::authentication("Invalid");
        assert_eq!(err.to_http_status(), 401);
    }

    #[test]
    fn test_http_status_authorization() {
        let err = LiteLLMError::authorization("Denied");
        assert_eq!(err.to_http_status(), 403);
    }

    #[test]
    fn test_http_status_not_found() {
        let err = LiteLLMError::not_found("resource");
        assert_eq!(err.to_http_status(), 404);
    }

    #[test]
    fn test_http_status_unsupported_operation() {
        let err = LiteLLMError::unsupported_operation("op");
        assert_eq!(err.to_http_status(), 405);
    }

    #[test]
    fn test_http_status_rate_limit() {
        let err = LiteLLMError::rate_limit("limited", None);
        assert_eq!(err.to_http_status(), 429);
    }

    #[test]
    fn test_http_status_validation() {
        let err = LiteLLMError::validation("field", "message");
        assert_eq!(err.to_http_status(), 400);
    }

    #[test]
    fn test_http_status_network() {
        let err = LiteLLMError::network("failed");
        assert_eq!(err.to_http_status(), 503);
    }

    #[test]
    fn test_http_status_service_unavailable() {
        let err = LiteLLMError::service_unavailable("down");
        assert_eq!(err.to_http_status(), 503);
    }

    #[test]
    fn test_http_status_timeout() {
        let err = LiteLLMError::timeout("request");
        assert_eq!(err.to_http_status(), 504);
    }

    #[test]
    fn test_http_status_internal() {
        let err = LiteLLMError::internal("error");
        assert_eq!(err.to_http_status(), 500);
    }

    #[test]
    fn test_http_status_provider() {
        let err = LiteLLMError::provider_error("test", "error");
        assert_eq!(err.to_http_status(), 500);
    }

    // ==================== Retryable Tests ====================

    #[test]
    fn test_is_retryable_network() {
        let err = LiteLLMError::network("failed");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_timeout() {
        let err = LiteLLMError::timeout("request");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_service_unavailable() {
        let err = LiteLLMError::service_unavailable("down");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_rate_limit() {
        let err = LiteLLMError::rate_limit("limited", None);
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_provider() {
        let err = LiteLLMError::provider_error("test", "error");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_retryable_internal() {
        let err = LiteLLMError::internal("error");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_authentication() {
        let err = LiteLLMError::authentication("invalid");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_authorization() {
        let err = LiteLLMError::authorization("denied");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_validation() {
        let err = LiteLLMError::validation("field", "message");
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_not_found() {
        let err = LiteLLMError::not_found("resource");
        assert!(!err.is_retryable());
    }

    // ==================== Retry Delay Tests ====================

    #[test]
    fn test_retry_delay_rate_limit_with_retry_after() {
        let err = LiteLLMError::rate_limit("limited", Some(30));
        assert_eq!(err.retry_delay(), Some(30));
    }

    #[test]
    fn test_retry_delay_rate_limit_without_retry_after() {
        let err = LiteLLMError::rate_limit("limited", None);
        assert_eq!(err.retry_delay(), None);
    }

    #[test]
    fn test_retry_delay_network() {
        let err = LiteLLMError::network("failed");
        assert_eq!(err.retry_delay(), Some(1));
    }

    #[test]
    fn test_retry_delay_timeout() {
        let err = LiteLLMError::timeout("request");
        assert_eq!(err.retry_delay(), Some(1));
    }

    #[test]
    fn test_retry_delay_service_unavailable() {
        let err = LiteLLMError::service_unavailable("down");
        assert_eq!(err.retry_delay(), Some(5));
    }

    #[test]
    fn test_retry_delay_internal() {
        let err = LiteLLMError::internal("error");
        assert_eq!(err.retry_delay(), Some(1));
    }

    #[test]
    fn test_retry_delay_authentication() {
        let err = LiteLLMError::authentication("invalid");
        assert_eq!(err.retry_delay(), None);
    }

    // ==================== Display/Debug Tests ====================

    #[test]
    fn test_error_display_provider() {
        let err = LiteLLMError::provider_error("openai", "API error");
        let display = err.to_string();
        assert!(display.contains("Provider error"));
        assert!(display.contains("openai"));
        assert!(display.contains("API error"));
    }

    #[test]
    fn test_error_display_validation() {
        let err = LiteLLMError::validation("temperature", "must be between 0 and 2");
        let display = err.to_string();
        assert!(display.contains("temperature"));
        assert!(display.contains("must be between 0 and 2"));
    }

    #[test]
    fn test_error_debug() {
        let err = LiteLLMError::internal("test error");
        let debug = format!("{:?}", err);
        assert!(debug.contains("Internal"));
    }

    // ==================== From Implementations ====================

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err: LiteLLMError = json_err.into();
        assert!(matches!(err, LiteLLMError::Serialization(_)));
    }
}
