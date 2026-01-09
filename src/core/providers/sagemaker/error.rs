//! Sagemaker-specific error types and error mapping
//!
//! Handles error conversion from AWS Sagemaker API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;
use crate::core::types::errors::ProviderErrorTrait;
use thiserror::Error;

/// Sagemaker-specific error types
#[derive(Debug, Error)]
pub enum SagemakerError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Invalid request: {0}")]
    InvalidRequestError(String),

    #[error("Model/Endpoint not found: {0}")]
    EndpointNotFoundError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailableError(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("Response parsing error: {0}")]
    ParsingError(String),

    #[error("Unknown error: {0}")]
    UnknownError(String),
}

impl ProviderErrorTrait for SagemakerError {
    fn error_type(&self) -> &'static str {
        match self {
            SagemakerError::ApiError(_) => "api_error",
            SagemakerError::AuthenticationError(_) => "authentication_error",
            SagemakerError::RateLimitError(_) => "rate_limit_error",
            SagemakerError::InvalidRequestError(_) => "invalid_request_error",
            SagemakerError::EndpointNotFoundError(_) => "endpoint_not_found_error",
            SagemakerError::ServiceUnavailableError(_) => "service_unavailable_error",
            SagemakerError::StreamingError(_) => "streaming_error",
            SagemakerError::ConfigurationError(_) => "configuration_error",
            SagemakerError::NetworkError(_) => "network_error",
            SagemakerError::SigningError(_) => "signing_error",
            SagemakerError::ParsingError(_) => "parsing_error",
            SagemakerError::UnknownError(_) => "unknown_error",
        }
    }

    fn is_retryable(&self) -> bool {
        matches!(
            self,
            SagemakerError::RateLimitError(_)
                | SagemakerError::ServiceUnavailableError(_)
                | SagemakerError::NetworkError(_)
        )
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            SagemakerError::RateLimitError(_) => Some(60),
            SagemakerError::ServiceUnavailableError(_) => Some(5),
            SagemakerError::NetworkError(_) => Some(2),
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            SagemakerError::AuthenticationError(_) => 401,
            SagemakerError::RateLimitError(_) => 429,
            SagemakerError::InvalidRequestError(_) => 400,
            SagemakerError::EndpointNotFoundError(_) => 404,
            SagemakerError::ServiceUnavailableError(_) => 503,
            SagemakerError::ApiError(_) => 500,
            SagemakerError::SigningError(_) => 401,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        SagemakerError::InvalidRequestError(format!("Feature not supported: {}", feature))
    }

    fn authentication_failed(reason: &str) -> Self {
        SagemakerError::AuthenticationError(reason.to_string())
    }

    fn rate_limited(retry_after: Option<u64>) -> Self {
        match retry_after {
            Some(seconds) => SagemakerError::RateLimitError(format!(
                "Rate limit exceeded, retry after {} seconds",
                seconds
            )),
            None => SagemakerError::RateLimitError("Rate limit exceeded".to_string()),
        }
    }

    fn network_error(details: &str) -> Self {
        SagemakerError::NetworkError(details.to_string())
    }

    fn parsing_error(details: &str) -> Self {
        SagemakerError::ParsingError(details.to_string())
    }

    fn not_implemented(feature: &str) -> Self {
        SagemakerError::InvalidRequestError(format!("Feature not implemented: {}", feature))
    }
}

impl From<SagemakerError> for ProviderError {
    fn from(error: SagemakerError) -> Self {
        match error {
            SagemakerError::ApiError(msg) => ProviderError::api_error("sagemaker", 500, msg),
            SagemakerError::AuthenticationError(msg) => {
                ProviderError::authentication("sagemaker", msg)
            }
            SagemakerError::RateLimitError(_) => ProviderError::rate_limit("sagemaker", None),
            SagemakerError::InvalidRequestError(msg) => {
                ProviderError::invalid_request("sagemaker", msg)
            }
            SagemakerError::EndpointNotFoundError(msg) => {
                ProviderError::model_not_found("sagemaker", msg)
            }
            SagemakerError::ServiceUnavailableError(msg) => {
                ProviderError::api_error("sagemaker", 503, msg)
            }
            SagemakerError::StreamingError(msg) => {
                ProviderError::api_error("sagemaker", 500, format!("Streaming error: {}", msg))
            }
            SagemakerError::ConfigurationError(msg) => {
                ProviderError::configuration("sagemaker", msg)
            }
            SagemakerError::NetworkError(msg) => ProviderError::network("sagemaker", msg),
            SagemakerError::SigningError(msg) => {
                ProviderError::authentication("sagemaker", format!("Signing error: {}", msg))
            }
            SagemakerError::ParsingError(msg) => {
                ProviderError::response_parsing("sagemaker", msg)
            }
            SagemakerError::UnknownError(msg) => ProviderError::api_error("sagemaker", 500, msg),
        }
    }
}

/// Error mapper for Sagemaker provider
pub struct SagemakerErrorMapper;

impl ErrorMapper<SagemakerError> for SagemakerErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> SagemakerError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => {
                // Check for specific Sagemaker/HuggingFace TGI errors
                if message.contains("temperature") {
                    SagemakerError::InvalidRequestError(
                        "Temperature must be strictly positive for HuggingFace TGI".to_string(),
                    )
                } else if message.contains("max_new_tokens") {
                    SagemakerError::InvalidRequestError(
                        "max_new_tokens must be strictly positive".to_string(),
                    )
                } else {
                    SagemakerError::InvalidRequestError(message)
                }
            }
            401 => SagemakerError::AuthenticationError(
                "Invalid AWS credentials or signature".to_string(),
            ),
            403 => SagemakerError::AuthenticationError("Access forbidden".to_string()),
            404 => SagemakerError::EndpointNotFoundError(
                "Sagemaker endpoint not found".to_string(),
            ),
            424 => SagemakerError::EndpointNotFoundError(
                "Sagemaker endpoint not ready or failed".to_string(),
            ),
            429 => SagemakerError::RateLimitError("Rate limit exceeded".to_string()),
            500 => SagemakerError::ApiError("Internal server error".to_string()),
            502 => SagemakerError::ServiceUnavailableError("Bad gateway".to_string()),
            503 => SagemakerError::ServiceUnavailableError("Service unavailable".to_string()),
            _ => SagemakerError::ApiError(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sagemaker_error_display() {
        let err = SagemakerError::ApiError("API failure".to_string());
        assert_eq!(err.to_string(), "API error: API failure");

        let err = SagemakerError::AuthenticationError("Bad credentials".to_string());
        assert_eq!(err.to_string(), "Authentication failed: Bad credentials");

        let err = SagemakerError::EndpointNotFoundError("my-endpoint".to_string());
        assert_eq!(err.to_string(), "Model/Endpoint not found: my-endpoint");

        let err = SagemakerError::SigningError("Invalid signature".to_string());
        assert_eq!(err.to_string(), "Signing error: Invalid signature");

        let err = SagemakerError::ParsingError("Invalid JSON".to_string());
        assert_eq!(err.to_string(), "Response parsing error: Invalid JSON");
    }

    #[test]
    fn test_sagemaker_error_type() {
        assert_eq!(SagemakerError::ApiError("".to_string()).error_type(), "api_error");
        assert_eq!(
            SagemakerError::AuthenticationError("".to_string()).error_type(),
            "authentication_error"
        );
        assert_eq!(
            SagemakerError::EndpointNotFoundError("".to_string()).error_type(),
            "endpoint_not_found_error"
        );
        assert_eq!(
            SagemakerError::SigningError("".to_string()).error_type(),
            "signing_error"
        );
        assert_eq!(
            SagemakerError::ParsingError("".to_string()).error_type(),
            "parsing_error"
        );
    }

    #[test]
    fn test_sagemaker_error_is_retryable() {
        assert!(SagemakerError::RateLimitError("".to_string()).is_retryable());
        assert!(SagemakerError::ServiceUnavailableError("".to_string()).is_retryable());
        assert!(SagemakerError::NetworkError("".to_string()).is_retryable());

        assert!(!SagemakerError::ApiError("".to_string()).is_retryable());
        assert!(!SagemakerError::AuthenticationError("".to_string()).is_retryable());
        assert!(!SagemakerError::SigningError("".to_string()).is_retryable());
    }

    #[test]
    fn test_sagemaker_error_retry_delay() {
        assert_eq!(
            SagemakerError::RateLimitError("".to_string()).retry_delay(),
            Some(60)
        );
        assert_eq!(
            SagemakerError::ServiceUnavailableError("".to_string()).retry_delay(),
            Some(5)
        );
        assert_eq!(
            SagemakerError::NetworkError("".to_string()).retry_delay(),
            Some(2)
        );
        assert_eq!(SagemakerError::ApiError("".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_sagemaker_error_http_status() {
        assert_eq!(
            SagemakerError::AuthenticationError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            SagemakerError::SigningError("".to_string()).http_status(),
            401
        );
        assert_eq!(
            SagemakerError::RateLimitError("".to_string()).http_status(),
            429
        );
        assert_eq!(
            SagemakerError::EndpointNotFoundError("".to_string()).http_status(),
            404
        );
    }

    #[test]
    fn test_sagemaker_error_factory_methods() {
        let err = SagemakerError::not_supported("embeddings");
        assert!(matches!(err, SagemakerError::InvalidRequestError(_)));

        let err = SagemakerError::authentication_failed("bad key");
        assert!(matches!(err, SagemakerError::AuthenticationError(_)));

        let err = SagemakerError::rate_limited(Some(30));
        assert!(matches!(err, SagemakerError::RateLimitError(_)));

        let err = SagemakerError::network_error("timeout");
        assert!(matches!(err, SagemakerError::NetworkError(_)));

        let err = SagemakerError::parsing_error("invalid json");
        assert!(matches!(err, SagemakerError::ParsingError(_)));
    }

    #[test]
    fn test_sagemaker_error_to_provider_error() {
        let err: ProviderError = SagemakerError::AuthenticationError("bad key".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = SagemakerError::EndpointNotFoundError("ep".to_string()).into();
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err: ProviderError = SagemakerError::SigningError("sig".to_string()).into();
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err: ProviderError = SagemakerError::ParsingError("json".to_string()).into();
        assert!(matches!(err, ProviderError::ResponseParsing { .. }));
    }

    #[test]
    fn test_sagemaker_error_mapper_http_errors() {
        let mapper = SagemakerErrorMapper;

        let err = mapper.map_http_error(400, "temperature must be strictly positive");
        assert!(matches!(err, SagemakerError::InvalidRequestError(_)));
        if let SagemakerError::InvalidRequestError(msg) = err {
            assert!(msg.contains("temperature"));
        }

        let err = mapper.map_http_error(400, "max_new_tokens must be positive");
        assert!(matches!(err, SagemakerError::InvalidRequestError(_)));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, SagemakerError::AuthenticationError(_)));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, SagemakerError::EndpointNotFoundError(_)));

        let err = mapper.map_http_error(424, "");
        assert!(matches!(err, SagemakerError::EndpointNotFoundError(_)));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, SagemakerError::RateLimitError(_)));

        let err = mapper.map_http_error(502, "");
        assert!(matches!(err, SagemakerError::ServiceUnavailableError(_)));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, SagemakerError::ServiceUnavailableError(_)));
    }
}
