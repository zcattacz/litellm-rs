//! LangGraph Error Handling
//!
//! Error mapping for LangGraph Cloud API responses

use crate::core::providers::base::HttpErrorMapper;
use crate::core::providers::shared::parse_retry_after_from_body;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Provider name constant for error messages
pub const PROVIDER_NAME: &str = "langgraph";

/// Error mapper for LangGraph API responses
#[derive(Debug, Clone, Copy)]
pub struct LangGraphErrorMapper;

impl ErrorMapper<ProviderError> for LangGraphErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            400 => {
                // Parse error message from response body
                let message =
                    parse_error_message(response_body).unwrap_or_else(|| "Bad request".to_string());
                ProviderError::invalid_request(PROVIDER_NAME, message)
            }
            401 => ProviderError::authentication(
                PROVIDER_NAME,
                "Invalid API key. Check LANGGRAPH_API_KEY or LANGSMITH_API_KEY",
            ),
            403 => ProviderError::authentication(PROVIDER_NAME, "Permission denied"),
            404 => {
                // Could be graph not found, thread not found, or run not found
                let message = parse_error_message(response_body)
                    .unwrap_or_else(|| "Resource not found".to_string());
                if message.contains("graph") {
                    ProviderError::model_not_found(PROVIDER_NAME, message)
                } else {
                    ProviderError::invalid_request(PROVIDER_NAME, message)
                }
            }
            409 => {
                // Conflict - usually concurrent modification
                ProviderError::invalid_request(
                    PROVIDER_NAME,
                    parse_error_message(response_body)
                        .unwrap_or_else(|| "Conflict - resource was modified".to_string()),
                )
            }
            422 => {
                // Validation error
                let message = parse_error_message(response_body)
                    .unwrap_or_else(|| "Validation error".to_string());
                ProviderError::invalid_request(PROVIDER_NAME, message)
            }
            429 => {
                // Rate limiting
                let retry_after = parse_retry_after_from_body(response_body);
                ProviderError::rate_limit(PROVIDER_NAME, retry_after)
            }
            500..=599 => ProviderError::provider_unavailable(
                PROVIDER_NAME,
                parse_error_message(response_body)
                    .unwrap_or_else(|| format!("Server error (status {})", status_code)),
            ),
            _ => HttpErrorMapper::map_status_code(PROVIDER_NAME, status_code, response_body),
        }
    }
}

/// Parse error message from LangGraph API response
fn parse_error_message(response_body: &str) -> Option<String> {
    // Try to parse as JSON and extract error message
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(response_body) {
        // LangGraph error format: {"detail": "error message"} or {"error": {"message": "..."}}
        if let Some(detail) = json.get("detail").and_then(|v| v.as_str()) {
            return Some(detail.to_string());
        }
        if let Some(message) = json
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            return Some(message.to_string());
        }
        if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
            return Some(message.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_mapper_401() {
        let mapper = LangGraphErrorMapper;
        let err = mapper.map_http_error(401, "Unauthorized");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_error_mapper_403() {
        let mapper = LangGraphErrorMapper;
        let err = mapper.map_http_error(403, "Forbidden");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_error_mapper_404_graph() {
        let mapper = LangGraphErrorMapper;
        let err = mapper.map_http_error(404, r#"{"detail": "graph not found"}"#);
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_error_mapper_404_other() {
        let mapper = LangGraphErrorMapper;
        let err = mapper.map_http_error(404, r#"{"detail": "thread not found"}"#);
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));
    }

    #[test]
    fn test_error_mapper_429() {
        let mapper = LangGraphErrorMapper;
        let err = mapper.map_http_error(429, "Rate limit exceeded");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_error_mapper_500() {
        let mapper = LangGraphErrorMapper;
        let err = mapper.map_http_error(500, "Internal server error");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }

    #[test]
    fn test_error_mapper_422() {
        let mapper = LangGraphErrorMapper;
        let err = mapper.map_http_error(422, r#"{"detail": "Invalid input format"}"#);
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));
    }

    #[test]
    fn test_parse_error_message_detail() {
        let msg = parse_error_message(r#"{"detail": "Test error"}"#);
        assert_eq!(msg, Some("Test error".to_string()));
    }

    #[test]
    fn test_parse_error_message_nested() {
        let msg = parse_error_message(r#"{"error": {"message": "Nested error"}}"#);
        assert_eq!(msg, Some("Nested error".to_string()));
    }

    #[test]
    fn test_parse_error_message_plain() {
        let msg = parse_error_message(r#"{"message": "Plain message"}"#);
        assert_eq!(msg, Some("Plain message".to_string()));
    }

    #[test]
    fn test_parse_error_message_invalid_json() {
        let msg = parse_error_message("not json");
        assert_eq!(msg, None);
    }

    #[test]
    fn test_parse_retry_after() {
        let retry = parse_retry_after_from_body(r#"{"retry_after": 30}"#);
        assert_eq!(retry, Some(30));
    }

    #[test]
    fn test_parse_retry_after_default() {
        let retry = parse_retry_after_from_body("rate limited");
        assert_eq!(retry, Some(60));
    }
}
