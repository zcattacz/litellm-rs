//! Deepgram-specific error types and error mapping
//!
//! Handles error conversion from Deepgram API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Deepgram-specific error types
#[derive(Debug, Error)]
pub enum DeepgramError {
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

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Audio processing error: {0}")]
    AudioProcessingError(String),

    #[error("Quota exceeded: {0}")]
    QuotaExceededError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for DeepgramError {
    fn error_type(&self) -> &'static str {
        match self {
            DeepgramError::ApiError(_) => "api_error",
            DeepgramError::AuthenticationError(_) => "authentication_error",
            DeepgramError::RateLimitError(_) => "rate_limit_error",
            DeepgramError::InvalidRequestError(_) => "invalid_request_error",
            DeepgramError::ModelNotFoundError(_) => "model_not_found_error",
            DeepgramError::ServiceUnavailableError(_) => "service_unavailable_error",
            DeepgramError::ConfigurationError(_) => "configuration_error",
            DeepgramError::NetworkError(_) => "network_error",
            DeepgramError::AudioProcessingError(_) => "audio_processing_error",
            DeepgramError::QuotaExceededError(_) => "quota_exceeded_error",
            DeepgramError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            DeepgramError::RateLimitError(_)
                | DeepgramError::ServiceUnavailableError(_)
                | DeepgramError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            DeepgramError::RateLimitError(_) => Some(60),
            DeepgramError::ServiceUnavailableError(_) => Some(5),
            DeepgramError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            DeepgramError::AuthenticationError(_) => 401,
            DeepgramError::RateLimitError(_) => 429,
            DeepgramError::InvalidRequestError(_) => 400,
            DeepgramError::ModelNotFoundError(_) => 404,
            DeepgramError::ServiceUnavailableError(_) => 503,
            DeepgramError::QuotaExceededError(_) => 402,
            DeepgramError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        DeepgramError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        DeepgramError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => DeepgramError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => DeepgramError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        DeepgramError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        DeepgramError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        DeepgramError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<DeepgramError> for ProviderError {
    fn from(error: DeepgramError) -> Self {
        match error {
            DeepgramError::ApiError(msg) => ProviderError::api_error("deepgram", 500, msg),
            DeepgramError::AuthenticationError(msg) => {
                ProviderError::authentication("deepgram", msg)
            }
            DeepgramError::RateLimitError(_) => ProviderError::rate_limit("deepgram", None),
            DeepgramError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("deepgram", msg)
            }
            DeepgramError::ModelNotFoundError(msg) => {
                ProviderError::model_not_found("deepgram", msg)
            }
            DeepgramError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("deepgram", 503, msg)
            }
            DeepgramError::ConfigurationError(msg) => ProviderError::configuration("deepgram", msg),
            DeepgramError::NetworkError(msg) => ProviderError::network("deepgram", msg),
            DeepgramError::AudioProcessingError(msg) => ProviderError::api_error(
                "deepgram",
                500,
                format!("Audio processing error: {}", msg),
            ),
            DeepgramError::QuotaExceededError(msg) => {
                ProviderError::quota_exceeded("deepgram", msg)
            }
            DeepgramError::UnknownError(msg) => ProviderError::api_error("deepgram", 500, msg),
        }
    }
}

/// Error mapper for Deepgram provider
pub struct DeepgramErrorMapper;

impl ErrorMapper<DeepgramError> for DeepgramErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> DeepgramError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => DeepgramError::InvalidRequestError(message),
            401 => DeepgramError::AuthenticationError("Invalid API key".to_string()),
            402 => DeepgramError::QuotaExceededError("Usage quota exceeded".to_string()),
            403 => DeepgramError::AuthenticationError("Access forbidden".to_string()),
            404 => DeepgramError::ModelNotFoundError("Model not found".to_string()),
            429 => DeepgramError::RateLimitError("Rate limit exceeded".to_string()),
            500 => DeepgramError::ApiError("Internal server error".to_string()),
            502 => DeepgramError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => DeepgramError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => DeepgramError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepgram_error_display() {
        let err = DeepgramError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = DeepgramError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = DeepgramError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");

        let err = DeepgramError::ModelNotFoundError("unknown model".to_string());
        assert_eq!(err.to_string(), "Model not found: unknown model");
    }

    #[test]
    fn test_deepgram_error_type() {
        assert_eq!(
            DeepgramError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            DeepgramError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            DeepgramError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            DeepgramError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            DeepgramError::ModelNotFoundError("".to_string()).error_type(),
            "model_not_found_error"
        );
        assert_eq!(
            DeepgramError::QuotaExceededError("".to_string()).error_type(),
            "quota_exceeded_error"
        );
    }

    #[test]
    fn test_deepgram_error_is_retryable() {
        assert!(DeepgramError::RateLimitError("".to_string()).is_retryable());
        assert!(DeepgramError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(DeepgramError::NetworkError("".to_string()).is_retryable());

        assert!(!DeepgramError::ApiError("".to_string()).is_retryable());
        assert!(!DeepgramError::AuthenticationError("".to_string()).is_retryable());
        assert!(!DeepgramError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_deepgram_error_retry_delay() {
        assert_eq!(
            DeepgramError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            DeepgramError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            DeepgramError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(DeepgramError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_deepgram_error_http_status() {
        assert_eq!(
            DeepgramError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            DeepgramError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            DeepgramError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            DeepgramError::ModelNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            DeepgramError::QuotaExceededError("".to_string()).http_status(),
            402
        );
    }

    #[test]
    fn test_deepgram_error_factory_methods() {
        let err = DeepgramError::not_supported("feature");
        assert!(matches!(err, DeepgramError::InvalidRequestError(_)));

        let err = DeepgramError::authentication_failed("bad key");
        assert!(matches!(err, DeepgramError::AuthenticationError(_)));

        let err = DeepgramError::rate_limited(Some(30));
        assert!(matches!(err, DeepgramError::RateLimitError(_)));

        let err = DeepgramError::network_error("connection failed");
        assert!(matches!(err, DeepgramError::NetworkError(_)));
    }

    #[test]
    fn test_deepgram_error_to_provider_error() {
        let err: ProviderError = DeepgramError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = DeepgramError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = DeepgramError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = DeepgramError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));
    }

    #[test]
    fn test_deepgram_error_mapper_http_errors() {
        let mapper = DeepgramErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, DeepgramError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, DeepgramError::AuthenticationError(_)));

        let err = mapper.map_http_error(402, "");
        assert!(matches!(err, DeepgramError::QuotaExceededError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, DeepgramError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, DeepgramError::RateLimitError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, DeepgramError::ServiceUnavailableError(_)));
    }
}
