//! ElevenLabs-specific error types and error mapping
//!
//! Handles error conversion from ElevenLabs API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// ElevenLabs error type alias (uses unified ProviderError)
pub type ElevenLabsError = ProviderError;

/// Error mapper for ElevenLabs provider
pub struct ElevenLabsErrorMapper;

impl ErrorMapper<ProviderError> for ElevenLabsErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => ProviderError::invalid_request("elevenlabs", message),
            401 => ProviderError::authentication("elevenlabs", "Invalid API key"),
            402 => ProviderError::quota_exceeded("elevenlabs", "Character quota exceeded"),
            403 => ProviderError::authentication("elevenlabs", "Access forbidden"),
            404 => ProviderError::model_not_found("elevenlabs", "Voice not found"),
            429 => ProviderError::rate_limit("elevenlabs", Some(60)),
            500 => ProviderError::api_error("elevenlabs", 500, "Internal server error"),
            502 => ProviderError::api_error("elevenlabs", 502, "Bad gateway"),
            503 => ProviderError::api_error("elevenlabs", 503, "Service unavailable"),
            _ => ProviderError::api_error("elevenlabs", status_code, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elevenlabs_error_mapper_http_errors() {
        let mapper = ElevenLabsErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = mapper.map_http_error(402, "");
        assert!(matches!(err, ProviderError::QuotaExceeded { .. }));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, ProviderError::ApiError { .. }));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_elevenlabs_error_mapper_unknown_status() {
        let mapper = ElevenLabsErrorMapper;
        let err = mapper.map_http_error(418, "I'm a teapot");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }
}
