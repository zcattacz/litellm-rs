//! OCI-specific error types and error mapping
//!
//! Handles error conversion from OCI API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// OCI-specific error types
#[derive(Debug, Error)]
pub enum OciError {
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

    #[error("Compartment error: {0}")]
    CompartmentError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for OciError {
    fn error_type(&self) -> &'static str {
        match self {
            OciError::ApiError(_) => "api_error",
            OciError::AuthenticationError(_) => "authentication_error",
            OciError::RateLimitError(_) => "rate_limit_error",
            OciError::InvalidRequestError(_) => "invalid_request_error",
            OciError::ModelNotFoundError(_) => "model_not_found_error",
            OciError::ServiceUnavailableError(_) => "service_unavailable_error",
            OciError::StreamingError(_) => "streaming_error",
            OciError::ConfigurationError(_) => "configuration_error",
            OciError::NetworkError(_) => "network_error",
            OciError::CompartmentError(_) => "compartment_error",
            OciError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            OciError::RateLimitError(_)
                | OciError::ServiceUnavailableError(_)
                | OciError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            OciError::RateLimitError(_) => Some(60),
            OciError::ServiceUnavailableError(_) => Some(5),
            OciError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            OciError::AuthenticationError(_) => 401,
            OciError::RateLimitError(_) => 429,
            OciError::InvalidRequestError(_) => 400,
            OciError::ModelNotFoundError(_) => 404,
            OciError::ServiceUnavailableError(_) => 503,
            OciError::CompartmentError(_) => 400,
            OciError::ApiError(_) => 500,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        OciError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        OciError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => OciError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => OciError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        OciError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        OciError::ApiError(format!("Response parsing error: {}", details))
    }

    fn not_implemented(feature: &str) -> Self {
        OciError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<OciError> for ProviderError {
    fn from(error: OciError) -> Self {
        match error {
            OciError::ApiError(msg) => ProviderError::api_error("oci", 500, msg),
            OciError::AuthenticationError(msg) => ProviderError::authentication("oci", msg),
            OciError::RateLimitError(_) => ProviderError::rate_limit("oci", None),
            OciError::InvalidRequestError(msg) => ProviderError::invalid_request("oci", msg),
            OciError::ModelNotFoundError(msg) => ProviderError::model_not_found("oci", msg),
            OciError::ServiceUnavailableError(msg) => ProviderError::api_error("oci", 503, msg),
            OciError::StreamingError(msg) => {
                ProviderError::api_error("oci", 500, format!("Streaming error: {}", msg))
            }
            OciError::ConfigurationError(msg) => ProviderError::configuration("oci", msg),
            OciError::NetworkError(msg) => ProviderError::network("oci", msg),
            OciError::CompartmentError(msg) => ProviderError::configuration("oci", msg),
            OciError::UnknownError(msg) => ProviderError::api_error("oci", 500, msg),
        }
    }
}

/// Error mapper for OCI provider
pub struct OciErrorMapper;

impl ErrorMapper<OciError> for OciErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> OciError {
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
                            .and_then(|e| e.get("message"))
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| response_body.to_string())
            } else {
                response_body.to_string()
            }
        };

        match status_code {
            400 => OciError::InvalidRequestError(message),
            401 => OciError::AuthenticationError("Invalid authentication credentials".to_string()),
            403 => OciError::AuthenticationError("Access forbidden".to_string()),
            404 => OciError::ModelNotFoundError("Model or resource not found".to_string()),
            429 => OciError::RateLimitError("Rate limit exceeded".to_string()),
            500 => OciError::ApiError("Internal server error".to_string()),
            502 => OciError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => OciError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => OciError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oci_error_display() {
        let err = OciError::ApiError("test error".to_string());
        assert_eq!(err.to_string(), "API error: test error");

        let err = OciError::AuthenticationError("invalid key".to_string());
        assert_eq!(err.to_string(), "Authentication failed: invalid key");

        let err = OciError::CompartmentError("invalid compartment".to_string());
        assert_eq!(err.to_string(), "Compartment error: invalid compartment");
    }

    #[test]
    fn test_oci_error_type() {
        assert_eq!(OciError::ApiError("".to_string()).error_type(), "api_error");
        assert_eq!(
            OciError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            OciError::CompartmentError("".to_string()).error_type(),
            "compartment_error"
        );
    }

    #[test]
    fn test_oci_error_is_retryable() {
        assert!(OciError::RateLimitError("".to_string()).is_retryable());
        assert!(OciError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(OciError::NetworkError("".to_string()).is_retryable());
        assert!(!OciError::ApiError("".to_string()).is_retryable());
        assert!(!OciError::AuthenticationError("".to_string()).is_retryable());
    }

    #[test]
    fn test_oci_error_to_provider_error() {
        let err: ProviderError = OciError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = OciError::RateLimitError("limit".to_string()).into();
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err: ProviderError = OciError::ModelNotFoundError("model".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_oci_error_mapper_http_errors() {
        let mapper = OciErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, OciError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, OciError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, OciError::ModelNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, OciError::RateLimitError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, OciError::ServiceUnavailableError(_)));
    }
}
