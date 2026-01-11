//! OCI-specific error types and error mapping
//!
//! Uses unified ProviderError with OCI-specific error mapper.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// OCI error type (alias to unified ProviderError)
pub type OciError = ProviderError;

/// Error mapper for OCI provider
pub struct OciErrorMapper;

impl ErrorMapper<ProviderError> for OciErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
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
            400 => ProviderError::invalid_request("oci", message),
            401 => ProviderError::authentication("oci", "Invalid authentication credentials"),
            403 => ProviderError::authentication("oci", "Access forbidden"),
            404 => ProviderError::model_not_found("oci", "Model or resource not found"),
            429 => ProviderError::rate_limit_simple("oci", "Rate limit exceeded"),
            500 => ProviderError::api_error("oci", 500, "Internal server error"),
            502 => ProviderError::provider_unavailable("oci", "Bad gateway"),
            503 => ProviderError::provider_unavailable("oci", "Service unavailable"),
            _ => ProviderError::api_error("oci", status_code, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oci_error_mapper_http_errors() {
        let mapper = OciErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }
}
