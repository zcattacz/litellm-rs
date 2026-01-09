//! GitHub Copilot-specific error types and error mapping
//!
//! Handles error conversion from GitHub Copilot API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// GitHub Copilot-specific error types
#[derive(Debug, Error)]
pub enum GitHubCopilotError {
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

    #[error("OAuth device code error: {0}")]
    DeviceCodeError(String),

    #[error("OAuth access token error: {0}")]
    AccessTokenError(String),

    #[error("API key expired: {0}")]
    ApiKeyExpiredError(String),

    #[error("API key refresh error: {0}")]
    RefreshApiKeyError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for GitHubCopilotError {
    fn error_type(&self) -> &'static str {
        match self {
            GitHubCopilotError::ApiError(_) => "api_error",
            GitHubCopilotError::AuthenticationError(_) => "authentication_error",
            GitHubCopilotError::RateLimitError(_) => "rate_limit_error",
            GitHubCopilotError::InvalidRequestError(_) => "invalid_request_error",
            GitHubCopilotError::ModelNotFoundError(_) => "model_not_found_error",
            GitHubCopilotError::ServiceUnavailableError(_) => "service_unavailable_error",
            GitHubCopilotError::StreamingError(_) => "streaming_error",
            GitHubCopilotError::ConfigurationError(_) => "configuration_error",
            GitHubCopilotError::NetworkError(_) => "network_error",
            GitHubCopilotError::DeviceCodeError(_) => "device_code_error",
            GitHubCopilotError::AccessTokenError(_) => "access_token_error",
            GitHubCopilotError::ApiKeyExpiredError(_) => "api_key_expired_error",
            GitHubCopilotError::RefreshApiKeyError(_) => "refresh_api_key_error",
            GitHubCopilotError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            GitHubCopilotError::RateLimitError(_)
                | GitHubCopilotError::ServiceUnavailableError(_)
                | GitHubCopilotError::NetworkError(_)
                | GitHubCopilotError::ApiKeyExpiredError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            GitHubCopilotError::RateLimitError(_) => Some(60),
            GitHubCopilotError::ServiceUnavailableError(_) => Some(5),
            GitHubCopilotError::NetworkError(_) => Some(2),
            GitHubCopilotError::ApiKeyExpiredError(_) => Some(1), // Quick retry after refresh
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            GitHubCopilotError::AuthenticationError(_) => 401,
            GitHubCopilotError::RateLimitError(_) => 429,
            GitHubCopilotError::InvalidRequestError(_) => 400,
            GitHubCopilotError::ModelNotFoundError(_) => 404,
            GitHubCopilotError::ServiceUnavailableError(_) => 503,
            GitHubCopilotError::ApiError(_) => 500,
            GitHubCopilotError::ApiKeyExpiredError(_) => 401,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        GitHubCopilotError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        GitHubCopilotError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => GitHubCopilotError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => GitHubCopilotError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        GitHubCopilotError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        GitHubCopilotError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        GitHubCopilotError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<GitHubCopilotError> for ProviderError {
    fn from(error: GitHubCopilotError) -> Self {
        match error {
            GitHubCopilotError::ApiError(msg) => {
                ProviderError::api_error("github_copilot", 500, msg)
            }
            GitHubCopilotError::AuthenticationError(msg) => {
                ProviderError::authentication("github_copilot", msg)
            }
            GitHubCopilotError::RateLimitError(_) => {
                ProviderError::rate_limit("github_copilot", None)
            }
            GitHubCopilotError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("github_copilot", msg)
            }
            GitHubCopilotError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("github_copilot", msg)
            }
            GitHubCopilotError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("github_copilot", 503, msg)
            }
            GitHubCopilotError::StreamingError(msg) => {
                ProviderError::api_error("github_copilot", 500, format!("Streaming error: {}", msg))
            }
            GitHubCopilotError::ConfigurationError(msg) => {
                ProviderError::configuration("github_copilot", msg)
            }
            GitHubCopilotError::NetworkError(msg) => ProviderError::network("github_copilot", msg),
            GitHubCopilotError::DeviceCodeError(msg) => ProviderError::authentication(
                "github_copilot",
                format!("Device code error: {}", msg),
            ),
            GitHubCopilotError::AccessTokenError(msg) => ProviderError::authentication(
                "github_copilot",
                format!("Access token error: {}", msg),
            ),
            GitHubCopilotError::ApiKeyExpiredError(msg) => {
                ProviderError::authentication("github_copilot", format!("API key expired: {}", msg))
            }
            GitHubCopilotError::RefreshApiKeyError(msg) => {
                ProviderError::authentication("github_copilot", format!("Refresh error: {}", msg))
            }
            GitHubCopilotError::UnknownError(msg) => {
                ProviderError::api_error("github_copilot", 500, msg)
            }
        }
    }
}

/// Error mapper for GitHub Copilot provider
pub struct GitHubCopilotErrorMapper;

impl ErrorMapper<GitHubCopilotError> for GitHubCopilotErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> GitHubCopilotError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => GitHubCopilotError::InvalidRequestError(message),
            401 => {
                GitHubCopilotError::AuthenticationError("Invalid or expired API key".to_string())
            }
            403 => GitHubCopilotError::AuthenticationError("Access forbidden".to_string()),
            404 => GitHubCopilotError::ModelNotFoundError("Model not found".to_string()),
            429 => GitHubCopilotError::RateLimitError("Rate limit exceeded".to_string()),
            500 => GitHubCopilotError::ApiError("Internal server error".to_string()),
            502 => GitHubCopilotError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => GitHubCopilotError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => GitHubCopilotError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_copilot_error_display() {
        let err = GitHubCopilotError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = GitHubCopilotError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = GitHubCopilotError::DeviceCodeError("code error".to_string());
        assert_eq!(err.to_string(), "OAuth device code error: code error");
    }

    #[test]
    fn test_github_copilot_error_type() {
        assert_eq!(
            GitHubCopilotError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            GitHubCopilotError::DeviceCodeError("".to_string()).error_type(),
            "device_code_error"
        );
        assert_eq!(
            GitHubCopilotError::ApiKeyExpiredError("".to_string()).error_type(),
            "api_key_expired_error"
        );
    }

    #[test]
    fn test_github_copilot_error_is_retryable() {
        assert!(GitHubCopilotError::RateLimitError("".to_string()).is_retryable());
        assert!(GitHubCopilotError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(GitHubCopilotError::NetworkError("".to_string()).is_retryable());
        assert!(GitHubCopilotError::ApiKeyExpiredError("".to_string()).is_retryable());

        assert!(!GitHubCopilotError::ApiError("".to_string()).is_retryable());
        assert!(!GitHubCopilotError::DeviceCodeError("".to_string()).is_retryable());
    }

    #[test]
    fn test_github_copilot_error_retry_delay() {
        assert_eq!(
            GitHubCopilotError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            GitHubCopilotError::ApiKeyExpiredError("".to_string()).retry_delay(),
            Some(1)
        );
        assert_eq!(
            GitHubCopilotError::DeviceCodeError("".to_string()).retry_delay(),
            None
        );
    }

    #[test]
    fn test_github_copilot_error_http_status() {
        assert_eq!(
            GitHubCopilotError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            GitHubCopilotError::ApiKeyExpiredError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            GitHubCopilotError::RateLimitError("".to_string()).http_status(),
            429
        );
    }

    #[test]
    fn test_github_copilot_error_to_provider_error() {
        let err: ProviderError =
            GitHubCopilotError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = GitHubCopilotError::DeviceCodeError("error".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = GitHubCopilotError::RefreshApiKeyError("error".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_github_copilot_error_mapper_http_errors() {
        let mapper = GitHubCopilotErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, GitHubCopilotError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, GitHubCopilotError::AuthenticationError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, GitHubCopilotError::RateLimitError(_)));
    }
}
