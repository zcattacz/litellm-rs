//! ElevenLabs-specific error types and error mapping
//!
//! Handles error conversion from ElevenLabs API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// ElevenLabs-specific error types
#[derive(Debug, Error)]
pub enum ElevenLabsError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("Voice not found: {0}")]
    VoiceNotFoundError(String),

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

impl ProviderErrorTrait for ElevenLabsError {
    fn error_type(&self) -> &'static str {
        match self {
            ElevenLabsError::ApiError(_) => "api_error",
            ElevenLabsError::AuthenticationError(_) => "authentication_error",
            ElevenLabsError::RateLimitError(_) => "rate_limit_error",
            ElevenLabsError::InvalidRequestError(_) => "invalid_request_error",
            ElevenLabsError::VoiceNotFoundError(_) => "voice_not_found_error",
            ElevenLabsError::ServiceUnavailableError(_) => "service_unavailable_error",
            ElevenLabsError::ConfigurationError(_) => "configuration_error",
            ElevenLabsError::NetworkError(_) => "network_error",
            ElevenLabsError::AudioProcessingError(_) => "audio_processing_error",
            ElevenLabsError::QuotaExceededError(_) => "quota_exceeded_error",
            ElevenLabsError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            ElevenLabsError::RateLimitError(_)
                | ElevenLabsError::ServiceUnavailableError(_)
                | ElevenLabsError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            ElevenLabsError::RateLimitError(_) => Some(60),
            ElevenLabsError::ServiceUnavailableError(_) => Some(5),
            ElevenLabsError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            ElevenLabsError::AuthenticationError(_) => 401,
            ElevenLabsError::RateLimitError(_) => 429,
            ElevenLabsError::InvalidRequestError(_) => 400,
            ElevenLabsError::VoiceNotFoundError(_) => 404,
            ElevenLabsError::ServiceUnavailableError(_) => 503,
            ElevenLabsError::QuotaExceededError(_) => 402,
            ElevenLabsError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        ElevenLabsError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        ElevenLabsError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => ElevenLabsError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => ElevenLabsError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        ElevenLabsError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        ElevenLabsError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        ElevenLabsError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<ElevenLabsError> for ProviderError {
    fn from(error: ElevenLabsError) -> Self {
        match error {
            ElevenLabsError::ApiError(msg) => ProviderError::api_error("elevenlabs", 500, msg),
            ElevenLabsError::AuthenticationError(msg) => {
                ProviderError::authentication("elevenlabs", msg)
            }
            ElevenLabsError::RateLimitError(_) => ProviderError::rate_limit("elevenlabs", None),
            ElevenLabsError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("elevenlabs", msg)
            }
            ElevenLabsError::VoiceNotFoundError(msg) => {
                ProviderError::model_not_found("elevenlabs", msg)
            }
            ElevenLabsError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("elevenlabs", 503, msg)
            }
            ElevenLabsError::ConfigurationError(msg) => {
                ProviderError::configuration("elevenlabs", msg)
            }
            ElevenLabsError::NetworkError(msg) => ProviderError::network("elevenlabs", msg),
            ElevenLabsError::AudioProcessingError(msg) => ProviderError::api_error(
                "elevenlabs",
                500,
                format!("Audio processing error: {}", msg),
            ),
            ElevenLabsError::QuotaExceededError(msg) => {
                ProviderError::quota_exceeded("elevenlabs", msg)
            }
            ElevenLabsError::UnknownError(msg) => ProviderError::api_error("elevenlabs", 500, msg),
        }
    }
}

/// Error mapper for ElevenLabs provider
pub struct ElevenLabsErrorMapper;

impl ErrorMapper<ElevenLabsError> for ElevenLabsErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ElevenLabsError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => ElevenLabsError::InvalidRequestError(message),
            401 => ElevenLabsError::AuthenticationError("Invalid API key".to_string()),
            402 => ElevenLabsError::QuotaExceededError("Character quota exceeded".to_string()),
            403 => ElevenLabsError::AuthenticationError("Access forbidden".to_string()),
            404 => ElevenLabsError::VoiceNotFoundError("Voice not found".to_string()),
            429 => ElevenLabsError::RateLimitError("Rate limit exceeded".to_string()),
            500 => ElevenLabsError::ApiError("Internal server error".to_string()),
            502 => ElevenLabsError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => ElevenLabsError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => ElevenLabsError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevenlabs_error_display() {
        let err = ElevenLabsError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = ElevenLabsError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = ElevenLabsError::RateLimitError("limit exceeded".to_string());
        assert_eq!(err.to_string(), "Rate limit exceeded: limit exceeded");

        let err = ElevenLabsError::VoiceNotFoundError("unknown voice".to_string());
        assert_eq!(err.to_string(), "Voice not found: unknown voice");
    }

    #[test]
    fn test_elevenlabs_error_type() {
        assert_eq!(
            ElevenLabsError::ApiError("".to_string()).error_type(),
            "api_error"
        );
        assert_eq!(
            ElevenLabsError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            ElevenLabsError::RateLimitError("".to_string()).error_type(),
            "rate_limit_error"
        );
        assert_eq!(
            ElevenLabsError::InvalidRequestError("".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(
            ElevenLabsError::VoiceNotFoundError("".to_string()).error_type(),
            "voice_not_found_error"
        );
        assert_eq!(
            ElevenLabsError::QuotaExceededError("".to_string()).error_type(),
            "quota_exceeded_error"
        );
    }

    #[test]
    fn test_elevenlabs_error_is_retryable() {
        assert!(ElevenLabsError::RateLimitError("".to_string()).is_retryable());
        assert!(ElevenLabsError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(ElevenLabsError::NetworkError("".to_string()).is_retryable());

        assert!(!ElevenLabsError::ApiError("".to_string()).is_retryable());
        assert!(!ElevenLabsError::AuthenticationError("".to_string()).is_retryable());
        assert!(!ElevenLabsError::InvalidRequestError("".to_string()).is_retryable());
    }

    #[test]
    fn test_elevenlabs_error_retry_delay() {
        assert_eq!(
            ElevenLabsError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            ElevenLabsError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            ElevenLabsError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(
            ElevenLabsError::ApiError("".to_string()).retry_delay(),
            None
        );
    }

    #[test]
    fn test_elevenlabs_error_http_status() {
        assert_eq!(
            ElevenLabsError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            ElevenLabsError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            ElevenLabsError::InvalidRequestError("".to_string()).http_status(),
            400
        );
        assert_eq!(
            ElevenLabsError::VoiceNotFoundError("".to_string()).http_status(),
            404
        );
        assert_eq!(
            ElevenLabsError::QuotaExceededError("".to_string()).http_status(),
            402
        );
    }

    #[test]
    fn test_elevenlabs_error_factory_methods() {
        let err = ElevenLabsError::not_supported("feature");
        assert!(matches!(err, ElevenLabsError::InvalidRequestError(_)));

        let err = ElevenLabsError::authentication_failed("bad key");
        assert!(matches!(err, ElevenLabsError::AuthenticationError(_)));

        let err = ElevenLabsError::rate_limited(Some(30));
        assert!(matches!(err, ElevenLabsError::RateLimitError(_)));

        let err = ElevenLabsError::network_error("connection failed");
        assert!(matches!(err, ElevenLabsError::NetworkError(_)));
    }

    #[test]
    fn test_elevenlabs_error_to_provider_error() {
        let err: ProviderError = ElevenLabsError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = ElevenLabsError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = ElevenLabsError::VoiceNotFoundError("unknown".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError =
            ElevenLabsError::ConfigurationError("bad config".to_string()).into();
        assert!(matches!(err, ProviderError::Configuration { .. }));
    }

    #[test]
    fn test_elevenlabs_error_mapper_http_errors() {
        let mapper = ElevenLabsErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, ElevenLabsError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, ElevenLabsError::AuthenticationError(_)));

        let err = mapper.map_http_error(402, "");
        assert!(matches!(err, ElevenLabsError::QuotaExceededError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, ElevenLabsError::VoiceNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, ElevenLabsError::RateLimitError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, ElevenLabsError::ServiceUnavailableError(_)));
    }
}
