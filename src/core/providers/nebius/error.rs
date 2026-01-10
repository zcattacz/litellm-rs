//! Nebius Error Handling
//!
//! Error mapping for Nebius AI cloud platform

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Nebius error mapper
#[derive(Debug)]
pub struct NebiusErrorMapper;

impl ErrorMapper<ProviderError> for NebiusErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            401 => ProviderError::authentication("nebius", "Invalid API key"),
            403 => ProviderError::authentication("nebius", "Permission denied"),
            404 => ProviderError::model_not_found("nebius", "Model not found"),
            429 => {
                let retry_after = parse_retry_after(response_body);
                ProviderError::rate_limit("nebius", retry_after)
            }
            500..=599 => ProviderError::api_error("nebius", status_code, response_body),
            _ => ProviderError::api_error("nebius", status_code, response_body),
        }
    }
}

/// Parse retry-after duration from response body
fn parse_retry_after(response_body: &str) -> Option<u64> {
    if response_body.contains("rate limit") || response_body.contains("too many requests") {
        Some(60) // Default 60 seconds
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nebius_error_mapper_401() {
        let mapper = NebiusErrorMapper;
        let err = mapper.map_http_error(401, "Unauthorized");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_nebius_error_mapper_403() {
        let mapper = NebiusErrorMapper;
        let err = mapper.map_http_error(403, "Forbidden");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_nebius_error_mapper_404() {
        let mapper = NebiusErrorMapper;
        let err = mapper.map_http_error(404, "Not found");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_nebius_error_mapper_429() {
        let mapper = NebiusErrorMapper;
        let err = mapper.map_http_error(429, "rate limit exceeded");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_nebius_error_mapper_500() {
        let mapper = NebiusErrorMapper;
        let err = mapper.map_http_error(500, "Internal error");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_nebius_error_mapper_503() {
        let mapper = NebiusErrorMapper;
        let err = mapper.map_http_error(503, "Service unavailable");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_nebius_error_mapper_unknown() {
        let mapper = NebiusErrorMapper;
        let err = mapper.map_http_error(418, "I'm a teapot");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_parse_retry_after_with_rate_limit() {
        let result = parse_retry_after("rate limit exceeded");
        assert_eq!(result, Some(60));
    }

    #[test]
    fn test_parse_retry_after_too_many_requests() {
        let result = parse_retry_after("too many requests");
        assert_eq!(result, Some(60));
    }

    #[test]
    fn test_parse_retry_after_without_rate_limit() {
        let result = parse_retry_after("other error");
        assert_eq!(result, None);
    }
}
