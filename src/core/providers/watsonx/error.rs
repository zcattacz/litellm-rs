//! Error types for Watsonx provider.
//!
//! This module provides Watsonx-specific error handling using the unified ProviderError type.

use crate::core::providers::base_provider::HttpErrorMapper;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use serde_json::Value;

/// Provider name constant
const PROVIDER_NAME: &str = "watsonx";

/// Watsonx error type (alias to unified ProviderError)
pub type WatsonxError = ProviderError;

/// Error mapper for Watsonx provider
pub struct WatsonxErrorMapper;

impl ErrorMapper<WatsonxError> for WatsonxErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> WatsonxError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            // Try to extract error message from JSON response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
                json.get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        json.get("errors")
                            .and_then(|e| e.as_array())
                            .and_then(|arr| arr.first())
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
            400 => ProviderError::invalid_request(PROVIDER_NAME, message),
            401 => ProviderError::authentication(PROVIDER_NAME, "Invalid API key or token"),
            403 => ProviderError::authentication(PROVIDER_NAME, "Access forbidden"),
            404 => ProviderError::model_not_found(PROVIDER_NAME, "Model or resource not found"),
            429 => ProviderError::rate_limit(PROVIDER_NAME, None),
            500 => ProviderError::api_error(PROVIDER_NAME, 500, "Internal server error"),
            502 => ProviderError::provider_unavailable(PROVIDER_NAME, "Bad gateway"),
            503 => ProviderError::provider_unavailable(PROVIDER_NAME, "Service unavailable"),
            _ => ProviderError::api_error(PROVIDER_NAME, status_code, message),
        }
    }

    fn map_json_error(&self, error_response: &Value) -> WatsonxError {
        HttpErrorMapper::parse_json_error(PROVIDER_NAME, error_response)
    }

    fn map_network_error(&self, error: &dyn std::error::Error) -> WatsonxError {
        ProviderError::network(PROVIDER_NAME, error.to_string())
    }

    fn map_parsing_error(&self, error: &dyn std::error::Error) -> WatsonxError {
        ProviderError::response_parsing(PROVIDER_NAME, error.to_string())
    }

    fn map_timeout_error(&self, timeout_duration: std::time::Duration) -> WatsonxError {
        ProviderError::timeout(
            PROVIDER_NAME,
            format!("Request timed out after {:?}", timeout_duration),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watsonx_error_mapper_http_errors() {
        let mapper = WatsonxErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, ProviderError::ApiError { .. }));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }

    #[test]
    fn test_watsonx_error_mapper_json_error() {
        let mapper = WatsonxErrorMapper;
        let json_body = r#"{"error": {"message": "Invalid model ID"}}"#;
        let err = mapper.map_http_error(400, json_body);
        if let ProviderError::InvalidRequest { message, .. } = err {
            assert!(message.contains("Invalid model ID"));
        } else {
            panic!("Expected InvalidRequest error");
        }
    }

    #[test]
    fn test_error_display() {
        let err = ProviderError::api_error(PROVIDER_NAME, 500, "test error");
        assert!(err.to_string().contains("watsonx"));
        assert!(err.to_string().contains("test error"));

        let err = ProviderError::authentication(PROVIDER_NAME, "invalid key");
        assert!(err.to_string().contains("watsonx"));
        assert!(err.to_string().contains("invalid key"));
    }

    #[test]
    fn test_error_retryability() {
        // Rate limit errors should be retryable
        let rate_error = ProviderError::rate_limit(PROVIDER_NAME, Some(60));
        assert!(rate_error.is_retryable());
        assert!(rate_error.retry_delay().is_some());

        // Rate limit without retry_after is still retryable but no delay
        let rate_error_no_delay = ProviderError::rate_limit(PROVIDER_NAME, None);
        assert!(rate_error_no_delay.is_retryable());

        // Service unavailable should be retryable
        let service_error = ProviderError::provider_unavailable(PROVIDER_NAME, "Service down");
        assert!(service_error.is_retryable());
        assert!(service_error.retry_delay().is_some());

        // Network errors should be retryable
        let network_error = ProviderError::network(PROVIDER_NAME, "Connection failed");
        assert!(network_error.is_retryable());
        assert!(network_error.retry_delay().is_some());

        // Authentication errors should not be retryable
        let auth_error = ProviderError::authentication(PROVIDER_NAME, "Bad key");
        assert!(!auth_error.is_retryable());
        assert!(auth_error.retry_delay().is_none());
    }

    #[test]
    fn test_error_http_status() {
        assert_eq!(
            ProviderError::authentication(PROVIDER_NAME, "").http_status(),
            401
        );
        assert_eq!(
            ProviderError::rate_limit(PROVIDER_NAME, None).http_status(),
            429
        );
        assert_eq!(
            ProviderError::invalid_request(PROVIDER_NAME, "").http_status(),
            400
        );
        assert_eq!(
            ProviderError::model_not_found(PROVIDER_NAME, "model").http_status(),
            404
        );
        assert_eq!(
            ProviderError::provider_unavailable(PROVIDER_NAME, "").http_status(),
            503
        );
        assert_eq!(
            ProviderError::api_error(PROVIDER_NAME, 500, "").http_status(),
            500
        );
    }
}
