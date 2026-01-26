//! Error types for Gradient AI provider.

pub use crate::core::providers::unified_provider::ProviderError;

/// Gradient AI error type (alias to unified ProviderError)
pub type GradientAIError = ProviderError;

/// GradientAI-specific error constructors
impl ProviderError {
    /// Create GradientAI streaming error
    pub fn gradient_ai_streaming(message: impl Into<String>) -> Self {
        Self::streaming_error("gradient_ai", "chat", None, None, message)
    }
}

/// Error mapper for Gradient AI provider
pub struct GradientAIErrorMapper;

impl crate::core::traits::error_mapper::trait_def::ErrorMapper<GradientAIError>
    for GradientAIErrorMapper
{
    fn map_http_error(&self, status_code: u16, response_body: &str) -> GradientAIError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => ProviderError::invalid_request("gradient_ai", message),
            401 => ProviderError::authentication("gradient_ai", "Invalid API key"),
            403 => ProviderError::authentication("gradient_ai", "Access forbidden"),
            404 => ProviderError::model_not_found("gradient_ai", "Model not found"),
            429 => ProviderError::rate_limit("gradient_ai", None),
            500 => ProviderError::api_error("gradient_ai", 500, "Internal server error"),
            502 => ProviderError::provider_unavailable("gradient_ai", "Bad gateway"),
            503 => ProviderError::provider_unavailable("gradient_ai", "Service unavailable"),
            _ => ProviderError::api_error("gradient_ai", status_code, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_ai_error_types() {
        let err = ProviderError::authentication("gradient_ai", "Invalid API key");
        match err {
            ProviderError::Authentication { provider, message } => {
                assert_eq!(provider, "gradient_ai");
                assert!(message.contains("Invalid API key"));
            }
            _ => panic!("Expected Authentication error"),
        }
    }

    #[test]
    fn test_gradient_ai_rate_limit() {
        let err = ProviderError::rate_limit("gradient_ai", Some(60));
        match err {
            ProviderError::RateLimit {
                provider,
                retry_after,
                ..
            } => {
                assert_eq!(provider, "gradient_ai");
                assert_eq!(retry_after, Some(60));
            }
            _ => panic!("Expected RateLimit error"),
        }
    }

    #[test]
    fn test_gradient_ai_network_error() {
        let err = ProviderError::network("gradient_ai", "Connection timeout");
        match err {
            ProviderError::Network { provider, message } => {
                assert_eq!(provider, "gradient_ai");
                assert!(message.contains("Connection timeout"));
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn test_gradient_ai_configuration_error() {
        let err = ProviderError::configuration("gradient_ai", "Invalid config");
        match err {
            ProviderError::Configuration { provider, message } => {
                assert_eq!(provider, "gradient_ai");
                assert!(message.contains("Invalid config"));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_gradient_ai_model_not_found() {
        let err = ProviderError::model_not_found("gradient_ai", "unknown-model");
        match err {
            ProviderError::ModelNotFound { provider, model } => {
                assert_eq!(provider, "gradient_ai");
                assert_eq!(model, "unknown-model");
            }
            _ => panic!("Expected ModelNotFound error"),
        }
    }

    #[test]
    fn test_gradient_ai_streaming_error() {
        let err = ProviderError::gradient_ai_streaming("Stream failed");
        match err {
            ProviderError::Streaming {
                provider, message, ..
            } => {
                assert_eq!(provider, "gradient_ai");
                assert!(message.contains("Stream failed"));
            }
            _ => panic!("Expected Streaming error"),
        }
    }

    #[test]
    fn test_error_mapper_http_errors() {
        let mapper = GradientAIErrorMapper;

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper,
            400,
            "bad request",
        );
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper, 401, "",
        );
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper, 403, "",
        );
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper, 404, "",
        );
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper, 429, "",
        );
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper, 500, "",
        );
        assert!(matches!(err, ProviderError::ApiError { .. }));

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper, 502, "",
        );
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));

        let err = crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
            &mapper, 503, "",
        );
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }

    #[test]
    fn test_error_is_retryable() {
        assert!(ProviderError::rate_limit("gradient_ai", None).is_retryable());
        assert!(ProviderError::network("gradient_ai", "timeout").is_retryable());
        assert!(ProviderError::provider_unavailable("gradient_ai", "down").is_retryable());

        assert!(!ProviderError::authentication("gradient_ai", "bad key").is_retryable());
        assert!(!ProviderError::invalid_request("gradient_ai", "bad req").is_retryable());
        assert!(!ProviderError::model_not_found("gradient_ai", "model").is_retryable());
    }

    #[test]
    fn test_error_retry_delay() {
        assert_eq!(
            ProviderError::rate_limit("gradient_ai", Some(30)).retry_delay(),
            Some(30)
        );
        assert_eq!(
            ProviderError::network("gradient_ai", "timeout").retry_delay(),
            Some(1)
        );
        assert_eq!(
            ProviderError::provider_unavailable("gradient_ai", "down").retry_delay(),
            Some(5)
        );
        assert_eq!(
            ProviderError::authentication("gradient_ai", "bad").retry_delay(),
            None
        );
    }

    #[test]
    fn test_error_http_status() {
        assert_eq!(
            ProviderError::authentication("gradient_ai", "bad").http_status(),
            401
        );
        assert_eq!(
            ProviderError::rate_limit("gradient_ai", None).http_status(),
            429
        );
        assert_eq!(
            ProviderError::invalid_request("gradient_ai", "bad").http_status(),
            400
        );
        assert_eq!(
            ProviderError::model_not_found("gradient_ai", "model").http_status(),
            404
        );
        assert_eq!(
            ProviderError::provider_unavailable("gradient_ai", "down").http_status(),
            503
        );
    }
}
