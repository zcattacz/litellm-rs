//! GitHub Models-specific error types and error mapping
//!
//! Handles error conversion from GitHub Models API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// GitHub Models-specific error types
#[derive(Debug, Error)]
pub enum GitHubError {
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

impl ProviderErrorTrait for GitHubError {
    fn error_type(&self) -> &'static str {
        match self {
            GitHubError::ApiError(_) => "api_error",
            GitHubError::AuthenticationError(_) => "authentication_error",
            GitHubError::RateLimitError(_) => "rate_limit_error",
            GitHubError::InvalidRequestError(_) => "invalid_request_error",
            GitHubError::ModelNotFoundError(_) => "model_not_found_error",
            GitHubError::ServiceUnavailableError(_) => "service_unavailable_error",
            GitHubError::StreamingError(_) => "streaming_error",
            GitHubError::ConfigurationError(_) => "configuration_error",
            GitHubError::NetworkError(_) => "network_error",
            GitHubError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            GitHubError::RateLimitError(_)
                | GitHubError::ServiceUnavailableError(_)
                | GitHubError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            GitHubError::RateLimitError(_) => Some(60), // Default 60 seconds for rate limit
            GitHubError::ServiceUnavailableError(_) => Some(5), // 5 seconds for service unavailable
            GitHubError::NetworkError(_) => Some(2),    // 2 seconds for network errors
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            GitHubError::AuthenticationError(_) => 401,
            GitHubError::RateLimitError(_) => 429,
            GitHubError::InvalidRequestError(_) => 400,
            GitHubError::ModelNotFoundError(_) => 404,
            GitHubError::ServiceUnavailableError(_) => 503,
            GitHubError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        GitHubError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        GitHubError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => GitHubError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => GitHubError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        GitHubError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        GitHubError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        GitHubError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<GitHubError> for ProviderError {
    fn from(error: GitHubError) -> Self {
        match error {
            GitHubError::ApiError(msg) => ProviderError::api_error("github", 500, msg),
            GitHubError::AuthenticationError(msg) => ProviderError::authentication("github", msg),
            GitHubError::RateLimitError(_) => ProviderError::rate_limit("github", None),
            GitHubError::InvalidRequestError(msg) => ProviderError::invalid_request("github", msg),
            GitHubError::ModelNotFoundError(msg) => ProviderError::model_not_found("github", msg),
            GitHubError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("github", 503, msg)
            }
            GitHubError::StreamingError(msg) => {
                ProviderError::api_error("github", 500, format!("Streaming error: {}", msg))
            }
            GitHubError::ConfigurationError(msg) => ProviderError::configuration("github", msg),
            GitHubError::NetworkError(msg) => ProviderError::network("github", msg),
            GitHubError::UnknownError(msg) => ProviderError::api_error("github", 500, msg),
        }
    }
}

/// Error mapper for GitHub Models provider
pub struct GitHubErrorMapper;

impl ErrorMapper<GitHubError> for GitHubErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> GitHubError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => GitHubError::InvalidRequestError(message),
            401 => GitHubError::AuthenticationError("Invalid GitHub token".to_string()),
            403 => GitHubError::AuthenticationError("Access forbidden".to_string()),
            404 => GitHubError::ModelNotFoundError("Model not found".to_string()),
            429 => GitHubError::RateLimitError("Rate limit exceeded".to_string()),
            500 => GitHubError::ApiError("Internal server error".to_string()),
            502 => GitHubError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => GitHubError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => GitHubError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_error_display() {
        let err = GitHubError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = GitHubError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = GitHubError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");
    }

    #[test]
    fn test_github_error_type() {
        assert_eq!(
            GitHubError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            GitHubError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            GitHubError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            GitHubError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            GitHubError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
    }

    #[test]
    fn test_github_error_is_retryable() {
        assert!(GitHubError::RateLimitError("".to_string()).is_retryable());
        assert!(GitHubError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(GitHubError::NetworkError("".to_string()).is_retryable());

        assert!(!GitHubError::ApiError("".to_string()).is_retryable());
        assert!(!GitHubError::AuthenticationError("".to_string()).is_retryable());
        assert!(!GitHubError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_github_error_retry_delay() {
        assert_eq!(
            GitHubError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            GitHubError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            GitHubError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(GitHubError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_github_error_http_status() {
        assert_eq!(
            GitHubError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            GitHubError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            GitHubError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            GitHubError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            GitHubError::ServiceUnavailableError("".to_string()).http_status(),
            503
        );
    }

    #[test]
    fn test_github_error_factory_methods() {
        let err = GitHubError::not_supported("vision");
        assert!(matches!(err, GitHubError::InvalidRequestError(_)));

        let err = GitHubError::authentication_failed("bad key");
        assert!(matches!(err, GitHubError::AuthenticationError(_)));

        let err = GitHubError::rate_limited(Some(30));
        assert!(matches!(err, GitHubError::RateLimitError(_)));

        let err = GitHubError::network_error("connection failed");
        assert!(matches!(err, GitHubError::NetworkError(_)));

        let err = GitHubError::parsing_error("invalid json");
        assert!(matches!(err, GitHubError::ApiError(_)));
    }

    #[test]
    fn test_github_error_to_provider_error() {
        let err: ProviderError = GitHubError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = GitHubError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = GitHubError::ModelNotFoundError("gpt-5".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_github_error_mapper_http_errors() {
        let mapper = GitHubErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, GitHubError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, GitHubError::AuthenticationError(_)));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, GitHubError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, GitHubError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, GitHubError::RateLimitError(_)));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, GitHubError::ApiError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, GitHubError::ServiceUnavailableError(_)));
    }
}
