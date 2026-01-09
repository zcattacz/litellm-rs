//! V0 AI Provider Module
//!
//! V0 is an OpenAI-compatible AI platform for developers
//! https://v0.dev/

pub mod chat;

use crate::core::{
    traits::{
        error_mapper::{trait_def::ErrorMapper, types::GenericErrorMapper},
        provider::{LLMProvider, ProviderConfig},
    },
    types::{
        common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
        errors::ProviderErrorTrait,
        requests::ChatRequest,
        responses::ChatResponse,
    },
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

/// V0 Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V0Config {
    /// API base URL for V0
    pub api_base: String,
    /// API key for authentication
    pub api_key: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for V0Config {
    fn default() -> Self {
        Self {
            api_base: "https://api.v0.dev/v1".to_string(),
            api_key: String::new(),
            timeout_seconds: 60,
            max_retries: 3,
        }
    }
}

impl V0Config {
    /// Configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("V0 API key is required".to_string());
        }
        if self.api_base.is_empty() {
            return Err("V0 API base URL is required".to_string());
        }
        Ok(())
    }
}

/// implementation ProviderConfig trait
impl ProviderConfig for V0Config {
    /// Configuration
    fn validate(&self) -> Result<(), String> {
        self.validate()
    }

    /// Get
    fn api_key(&self) -> Option<&str> {
        if self.api_key.is_empty() {
            None
        } else {
            Some(&self.api_key)
        }
    }

    /// Get
    fn api_base(&self) -> Option<&str> {
        Some(&self.api_base)
    }

    /// Get
    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }

    /// Get
    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// V0 supported models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum V0Model {
    /// V0 Default Model
    V0Default,
    /// Custom model
    Custom(String),
}

impl V0Model {
    /// Get model identifier for API calls
    pub fn model_id(&self) -> String {
        match self {
            Self::V0Default => "v0-default".to_string(),
            Self::Custom(id) => id.clone(),
        }
    }

    /// Check if model supports function calling
    pub fn supports_function_calling(&self) -> bool {
        matches!(self, Self::V0Default | Self::Custom(_))
    }

    /// Check if model supports streaming
    pub fn supports_streaming(&self) -> bool {
        true
    }

    /// Get maximum context window size
    pub fn max_context_tokens(&self) -> usize {
        match self {
            Self::V0Default => 32768,
            Self::Custom(_) => 32768, // Default assumption
        }
    }
}

/// Parse model string to V0Model enum
pub fn parse_v0_model(model: &str) -> V0Model {
    match model {
        "v0" | "v0-default" => V0Model::V0Default,
        _ => V0Model::Custom(model.to_string()),
    }
}

/// V0 Provider implementation
#[derive(Debug, Clone)]
pub struct V0Provider {
    config: V0Config,
    client: reqwest::Client,
}

impl V0Provider {
    /// Create a new V0 provider
    ///
    /// # Errors
    /// Returns error if HTTP client cannot be created
    pub fn new(
        config: V0Config,
    ) -> Result<Self, crate::core::providers::unified_provider::ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                crate::core::providers::unified_provider::ProviderError::Configuration {
                    provider: "v0",
                    message: format!("Failed to create HTTP client: {}", e),
                }
            })?;

        Ok(Self { config, client })
    }

    /// Create a new V0 provider with default client on error
    pub fn new_or_default(config: V0Config) -> Self {
        Self::new(config.clone()).unwrap_or_else(|e| {
            tracing::error!("Failed to create V0 provider: {}, using default client", e);
            Self {
                config,
                client: reqwest::Client::new(),
            }
        })
    }

    /// Get API endpoint URL
    fn get_endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// Create request headers
    fn create_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(auth_value) = format!("Bearer {}", self.config.api_key).parse() {
            headers.insert(reqwest::header::AUTHORIZATION, auth_value);
        }
        if let Ok(content_type) = "application/json".parse() {
            headers.insert(reqwest::header::CONTENT_TYPE, content_type);
        }
        headers
    }

    /// Internal health check method
    async fn check_health(&self) -> Result<(), V0Error> {
        let url = self.get_endpoint("models");
        let response = self
            .client
            .get(&url)
            .headers(self.create_headers())
            .send()
            .await
            .map_err(V0Error::HttpError)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(V0Error::ApiError(format!(
                "Health check failed with status: {}",
                response.status()
            )))
        }
    }
}

/// V0 specific errors
#[derive(Debug, thiserror::Error)]
pub enum V0Error {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Model not found: {0}")]
    ModelNotFound(String),
}

/// Error
pub struct V0ErrorMapper;

impl ErrorMapper<V0Error> for V0ErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> V0Error {
        match status_code {
            401 => V0Error::AuthenticationFailed,
            429 => V0Error::RateLimitExceeded,
            404 => V0Error::ModelNotFound("Model not found".to_string()),
            _ => GenericErrorMapper.map_http_error(status_code, response_body),
        }
    }
}

/// Implementation of unified LLMProvider trait
///
/// V0 is an OpenAI-compatible AI platform
#[async_trait]
impl LLMProvider for V0Provider {
    type Config = V0Config;
    type Error = V0Error;
    type ErrorMapper = V0ErrorMapper;

    /// Get
    fn name(&self) -> &'static str {
        "v0"
    }

    /// Get
    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::ToolCalling,
            ProviderCapability::FunctionCalling,
        ]
    }

    /// Model
    fn models(&self) -> &[ModelInfo] {
        // Use LazyLock for lazy initialization of static data
        static MODELS: LazyLock<Vec<ModelInfo>> = LazyLock::new(|| {
            vec![ModelInfo {
                id: "v0-default".to_string(),
                name: "V0 Default Model".to_string(),
                provider: "v0".to_string(),
                max_context_length: 32768,
                max_output_length: Some(8192),
                supports_streaming: true,
                supports_tools: true,
                supports_multimodal: false,
                input_cost_per_1k_tokens: Some(0.1),
                output_cost_per_1k_tokens: Some(0.2),
                currency: "USD".to_string(),
                capabilities: vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                    ProviderCapability::ToolCalling,
                ],
                created_at: None,
                updated_at: None,
                metadata: HashMap::new(),
            }]
        });
        &MODELS
    }

    // ==================== Python LiteLLM compatible interface ====================

    /// Get
    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "messages",
            "model",
            "temperature",
            "max_tokens",
            "top_p",
            "stream",
            "tools",
            "tool_choice",
            "user",
            "seed",
        ]
    }

    /// Map OpenAI parameters to V0 parameters
    async fn map_openai_params(
        &self,
        mut params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // V0 uses OpenAI-compatible parameters, so most parameters are passed through directly

        // Can add specific parameter mapping logic here
        // For example: rename certain parameters or convert formats

        // Ensure stream parameter is boolean value, not Option<bool>
        if let Some(stream_val) = params.get("stream") {
            if let Some(stream_bool) = stream_val.as_bool() {
                params.insert("stream".to_string(), Value::Bool(stream_bool));
            }
        }

        Ok(params)
    }

    /// Request
    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Request
        if request.messages.is_empty() {
            return Err(V0Error::InvalidRequest(
                "Messages cannot be empty".to_string(),
            ));
        }

        if request.model.is_empty() {
            return Err(V0Error::InvalidRequest("Model cannot be empty".to_string()));
        }

        // Convert to V0 API format (OpenAI compatible)
        let v0_request = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "top_p": request.top_p,
            "stream": request.stream,
            "tools": request.tools,
            "tool_choice": request.tool_choice,
        });

        Ok(v0_request)
    }

    /// Response
    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        // Response
        let response_json: Value =
            serde_json::from_slice(raw_response).map_err(V0Error::JsonError)?;

        // Convert to standard ChatResponse format
        // Response
        // Create

        let choices = response_json
            .get("choices")
            .and_then(|c| c.as_array())
            .ok_or_else(|| V0Error::ApiError("Invalid response format".to_string()))?;

        let usage = response_json
            .get("usage")
            .map(|u| serde_json::from_value(u.clone()))
            .transpose()
            .map_err(V0Error::JsonError)?;

        let chat_response = ChatResponse {
            id: request_id.to_string(),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            model: model.to_string(),
            choices: serde_json::from_value(serde_json::Value::Array(choices.clone()))
                .map_err(V0Error::JsonError)?,
            usage,
            system_fingerprint: None,
        };

        Ok(chat_response)
    }

    /// Error
    fn get_error_mapper(&self) -> Self::ErrorMapper {
        V0ErrorMapper
    }

    // ==================== Core functionality: chat completion ====================

    /// Request
    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        // Use new transformation flow
        let _transformed_request = self
            .transform_request(request.clone(), context.clone())
            .await?;

        // Should call actual API here, using original handler for demonstration
        chat::V0ChatHandler::handle_chat_completion(self, request).await
    }

    /// Check
    async fn health_check(&self) -> HealthStatus {
        match self.check_health().await {
            Ok(_) => HealthStatus::Healthy,
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    /// Request
    async fn calculate_cost(
        &self,
        _model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // V0 pricing: input $0.1/1K tokens, output $0.2/1K tokens
        let input_cost = (input_tokens as f64 / 1000.0) * 0.1;
        let output_cost = (output_tokens as f64 / 1000.0) * 0.2;
        Ok(input_cost + output_cost)
    }
}

impl ProviderErrorTrait for V0Error {
    fn error_type(&self) -> &'static str {
        match self {
            Self::HttpError(_) => "network_error",
            Self::JsonError(_) => "parsing_error",
            Self::ApiError(_) => "api_error",
            Self::InvalidRequest(_) => "invalid_request_error",
            Self::RateLimitExceeded => "rate_limit_error",
            Self::AuthenticationFailed => "authentication_error",
            Self::ModelNotFound(_) => "model_not_found",
        }
    }

    fn is_retryable(&self) -> bool {
        match self {
            Self::HttpError(_) | Self::RateLimitExceeded => true,
            Self::ApiError(_) => true, // Depends on the specific API error
            _ => false,
        }
    }

    fn retry_delay(&self) -> Option<u64> {
        match self {
            Self::RateLimitExceeded => Some(60), // 1 minute
            Self::HttpError(_) => Some(1),       // 1 second
            Self::ApiError(_) => Some(5),        // 5 seconds
            _ => None,
        }
    }

    fn http_status(&self) -> u16 {
        match self {
            Self::AuthenticationFailed => 401,
            Self::RateLimitExceeded => 429,
            Self::ModelNotFound(_) => 404,
            Self::InvalidRequest(_) => 400,
            Self::HttpError(_) => 503,
            _ => 500,
        }
    }

    fn not_supported(feature: &str) -> Self {
        Self::InvalidRequest(format!("Feature '{}' is not supported by V0", feature))
    }

    fn authentication_failed(_reason: &str) -> Self {
        Self::AuthenticationFailed
    }

    fn rate_limited(_retry_after: Option<u64>) -> Self {
        Self::RateLimitExceeded
    }

    fn network_error(details: &str) -> Self {
        Self::InvalidRequest(format!("Network error: {}", details))
    }

    fn parsing_error(details: &str) -> Self {
        // Create a JSON parsing error by attempting to parse invalid JSON
        let invalid_json = format!("{{\"error\": \"{}\"}}", details.replace('"', "\\\""));
        match serde_json::from_str::<serde_json::Value>(&invalid_json) {
            Err(e) => Self::JsonError(e),
            Ok(_) => {
                // If that somehow succeeded, create a different error
                Self::InvalidRequest(format!("JSON parsing error: {}", details))
            }
        }
    }

    fn not_implemented(feature: &str) -> Self {
        Self::InvalidRequest(format!("Feature '{}' not implemented for V0", feature))
    }
}

// Provider trait implementation removed - V0Provider is now included through the Provider enum variants

// ==================== Unit Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== V0Config Tests ====================

    #[test]
    fn test_v0_config_default() {
        let config = V0Config::default();
        assert_eq!(config.api_base, "https://api.v0.dev/v1");
        assert!(config.api_key.is_empty());
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_v0_config_clone() {
        let config = V0Config {
            api_base: "https://custom.api.v0.dev".to_string(),
            api_key: "test-key".to_string(),
            timeout_seconds: 120,
            max_retries: 5,
        };
        let cloned = config.clone();
        assert_eq!(config.api_base, cloned.api_base);
        assert_eq!(config.api_key, cloned.api_key);
        assert_eq!(config.timeout_seconds, cloned.timeout_seconds);
    }

    #[test]
    fn test_v0_config_validate_success() {
        let config = V0Config {
            api_base: "https://api.v0.dev/v1".to_string(),
            api_key: "valid-api-key".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_v0_config_validate_empty_api_key() {
        let config = V0Config {
            api_base: "https://api.v0.dev/v1".to_string(),
            api_key: String::new(),
            timeout_seconds: 60,
            max_retries: 3,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_v0_config_validate_empty_api_base() {
        let config = V0Config {
            api_base: String::new(),
            api_key: "valid-key".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API base"));
    }

    #[test]
    fn test_v0_config_serialization() {
        let config = V0Config {
            api_base: "https://api.v0.dev/v1".to_string(),
            api_key: "test-key".to_string(),
            timeout_seconds: 60,
            max_retries: 3,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"api_base\""));
        assert!(json.contains("\"api_key\""));
        assert!(json.contains("\"timeout_seconds\":60"));
    }

    #[test]
    fn test_v0_config_deserialization() {
        let json = r#"{
            "api_base": "https://api.v0.dev/v1",
            "api_key": "test-key",
            "timeout_seconds": 90,
            "max_retries": 5
        }"#;
        let config: V0Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_base, "https://api.v0.dev/v1");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.timeout_seconds, 90);
        assert_eq!(config.max_retries, 5);
    }

    // ==================== ProviderConfig Trait Tests ====================

    #[test]
    fn test_provider_config_api_key() {
        let config = V0Config {
            api_key: "my-key".to_string(),
            ..Default::default()
        };
        assert_eq!(config.api_key(), Some("my-key"));
    }

    #[test]
    fn test_provider_config_api_key_empty() {
        let config = V0Config::default();
        assert_eq!(config.api_key(), None);
    }

    #[test]
    fn test_provider_config_api_base() {
        let config = V0Config {
            api_base: "https://custom.api.com".to_string(),
            ..Default::default()
        };
        assert_eq!(config.api_base(), Some("https://custom.api.com"));
    }

    #[test]
    fn test_provider_config_timeout() {
        let config = V0Config {
            timeout_seconds: 120,
            ..Default::default()
        };
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }

    #[test]
    fn test_provider_config_max_retries() {
        let config = V0Config {
            max_retries: 5,
            ..Default::default()
        };
        assert_eq!(config.max_retries(), 5);
    }

    // ==================== V0Model Tests ====================

    #[test]
    fn test_v0_model_default_id() {
        let model = V0Model::V0Default;
        assert_eq!(model.model_id(), "v0-default");
    }

    #[test]
    fn test_v0_model_custom_id() {
        let model = V0Model::Custom("my-custom-model".to_string());
        assert_eq!(model.model_id(), "my-custom-model");
    }

    #[test]
    fn test_v0_model_supports_function_calling() {
        assert!(V0Model::V0Default.supports_function_calling());
        assert!(V0Model::Custom("test".to_string()).supports_function_calling());
    }

    #[test]
    fn test_v0_model_supports_streaming() {
        assert!(V0Model::V0Default.supports_streaming());
        assert!(V0Model::Custom("test".to_string()).supports_streaming());
    }

    #[test]
    fn test_v0_model_max_context_tokens() {
        assert_eq!(V0Model::V0Default.max_context_tokens(), 32768);
        assert_eq!(
            V0Model::Custom("test".to_string()).max_context_tokens(),
            32768
        );
    }

    #[test]
    fn test_v0_model_clone() {
        let model = V0Model::V0Default;
        let cloned = model.clone();
        assert!(matches!(cloned, V0Model::V0Default));

        let custom = V0Model::Custom("test".to_string());
        let custom_cloned = custom.clone();
        assert!(matches!(custom_cloned, V0Model::Custom(s) if s == "test"));
    }

    #[test]
    fn test_v0_model_serialization() {
        let model = V0Model::V0Default;
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, "\"V0Default\"");

        let custom = V0Model::Custom("my-model".to_string());
        let json = serde_json::to_string(&custom).unwrap();
        assert!(json.contains("Custom"));
        assert!(json.contains("my-model"));
    }

    // ==================== parse_v0_model Tests ====================

    #[test]
    fn test_parse_v0_model_default() {
        let model = parse_v0_model("v0");
        assert!(matches!(model, V0Model::V0Default));

        let model = parse_v0_model("v0-default");
        assert!(matches!(model, V0Model::V0Default));
    }

    #[test]
    fn test_parse_v0_model_custom() {
        let model = parse_v0_model("custom-model-123");
        assert!(matches!(model, V0Model::Custom(s) if s == "custom-model-123"));
    }

    // ==================== V0Provider Tests ====================

    #[test]
    fn test_v0_provider_new() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_v0_provider_new_or_default() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        assert_eq!(provider.config.api_key, "test-key");
    }

    #[test]
    fn test_v0_provider_clone() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        let cloned = provider.clone();
        assert_eq!(provider.config.api_key, cloned.config.api_key);
    }

    #[test]
    fn test_v0_provider_get_endpoint() {
        let config = V0Config {
            api_base: "https://api.v0.dev/v1".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);

        assert_eq!(
            provider.get_endpoint("chat/completions"),
            "https://api.v0.dev/v1/chat/completions"
        );
        assert_eq!(
            provider.get_endpoint("/models"),
            "https://api.v0.dev/v1/models"
        );
    }

    #[test]
    fn test_v0_provider_get_endpoint_trailing_slash() {
        let config = V0Config {
            api_base: "https://api.v0.dev/v1/".to_string(),
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);

        assert_eq!(
            provider.get_endpoint("chat/completions"),
            "https://api.v0.dev/v1/chat/completions"
        );
    }

    #[test]
    fn test_v0_provider_create_headers() {
        let config = V0Config {
            api_key: "test-key-123".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        let headers = provider.create_headers();

        assert!(headers.contains_key(reqwest::header::AUTHORIZATION));
        assert!(headers.contains_key(reqwest::header::CONTENT_TYPE));
    }

    // ==================== V0Error Tests ====================

    #[test]
    fn test_v0_error_display() {
        let http_err = V0Error::ApiError("test error".to_string());
        assert!(http_err.to_string().contains("test error"));

        let rate_limit = V0Error::RateLimitExceeded;
        assert!(rate_limit.to_string().contains("Rate limit"));

        let auth_failed = V0Error::AuthenticationFailed;
        assert!(auth_failed.to_string().contains("Authentication"));

        let model_not_found = V0Error::ModelNotFound("gpt-5".to_string());
        assert!(model_not_found.to_string().contains("gpt-5"));

        let invalid_request = V0Error::InvalidRequest("bad request".to_string());
        assert!(invalid_request.to_string().contains("bad request"));
    }

    #[test]
    fn test_v0_error_trait_error_type() {
        assert_eq!(V0Error::RateLimitExceeded.error_type(), "rate_limit_error");
        assert_eq!(
            V0Error::AuthenticationFailed.error_type(),
            "authentication_error"
        );
        assert_eq!(
            V0Error::ModelNotFound("x".to_string()).error_type(),
            "model_not_found"
        );
        assert_eq!(
            V0Error::InvalidRequest("x".to_string()).error_type(),
            "invalid_request_error"
        );
        assert_eq!(V0Error::ApiError("x".to_string()).error_type(), "api_error");
    }

    #[test]
    fn test_v0_error_trait_is_retryable() {
        assert!(V0Error::RateLimitExceeded.is_retryable());
        assert!(V0Error::ApiError("server error".to_string()).is_retryable());
        assert!(!V0Error::AuthenticationFailed.is_retryable());
        assert!(!V0Error::InvalidRequest("bad".to_string()).is_retryable());
        assert!(!V0Error::ModelNotFound("x".to_string()).is_retryable());
    }

    #[test]
    fn test_v0_error_trait_retry_delay() {
        assert_eq!(V0Error::RateLimitExceeded.retry_delay(), Some(60));
        assert_eq!(V0Error::ApiError("x".to_string()).retry_delay(), Some(5));
        assert_eq!(V0Error::AuthenticationFailed.retry_delay(), None);
        assert_eq!(V0Error::InvalidRequest("x".to_string()).retry_delay(), None);
    }

    #[test]
    fn test_v0_error_trait_http_status() {
        assert_eq!(V0Error::AuthenticationFailed.http_status(), 401);
        assert_eq!(V0Error::RateLimitExceeded.http_status(), 429);
        assert_eq!(V0Error::ModelNotFound("x".to_string()).http_status(), 404);
        assert_eq!(V0Error::InvalidRequest("x".to_string()).http_status(), 400);
        assert_eq!(V0Error::ApiError("x".to_string()).http_status(), 500);
    }

    #[test]
    fn test_v0_error_not_supported() {
        let err = V0Error::not_supported("vision");
        assert!(matches!(err, V0Error::InvalidRequest(_)));
        assert!(err.to_string().contains("vision"));
    }

    #[test]
    fn test_v0_error_authentication_failed() {
        let err = V0Error::authentication_failed("invalid token");
        assert!(matches!(err, V0Error::AuthenticationFailed));
    }

    #[test]
    fn test_v0_error_rate_limited() {
        let err = V0Error::rate_limited(Some(60));
        assert!(matches!(err, V0Error::RateLimitExceeded));
    }

    #[test]
    fn test_v0_error_network_error() {
        let err = V0Error::network_error("connection refused");
        assert!(matches!(err, V0Error::InvalidRequest(_)));
        assert!(err.to_string().contains("Network error"));
    }

    #[test]
    fn test_v0_error_not_implemented() {
        let err = V0Error::not_implemented("embeddings");
        assert!(matches!(err, V0Error::InvalidRequest(_)));
        assert!(err.to_string().contains("embeddings"));
    }

    // ==================== V0ErrorMapper Tests ====================

    #[test]
    fn test_v0_error_mapper_401() {
        let mapper = V0ErrorMapper;
        let err = mapper.map_http_error(401, "unauthorized");
        assert!(matches!(err, V0Error::AuthenticationFailed));
    }

    #[test]
    fn test_v0_error_mapper_429() {
        let mapper = V0ErrorMapper;
        let err = mapper.map_http_error(429, "rate limited");
        assert!(matches!(err, V0Error::RateLimitExceeded));
    }

    #[test]
    fn test_v0_error_mapper_404() {
        let mapper = V0ErrorMapper;
        let err = mapper.map_http_error(404, "not found");
        assert!(matches!(err, V0Error::ModelNotFound(_)));
    }

    // ==================== LLMProvider Trait Tests ====================

    #[test]
    fn test_v0_provider_name() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        assert_eq!(provider.name(), "v0");
    }

    #[test]
    fn test_v0_provider_capabilities() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        let capabilities = provider.capabilities();

        assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
        assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
        assert!(capabilities.contains(&ProviderCapability::ToolCalling));
        assert!(capabilities.contains(&ProviderCapability::FunctionCalling));
    }

    #[test]
    fn test_v0_provider_models() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        let models = provider.models();

        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "v0-default"));
    }

    #[test]
    fn test_v0_provider_supported_params() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        let params = provider.get_supported_openai_params("v0-default");

        assert!(params.contains(&"messages"));
        assert!(params.contains(&"model"));
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"stream"));
        assert!(params.contains(&"tools"));
    }

    #[test]
    fn test_v0_provider_get_error_mapper() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);
        let _mapper = provider.get_error_mapper();
        // Just verify it compiles and returns
    }

    #[tokio::test]
    async fn test_v0_provider_calculate_cost() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);

        // 1000 input tokens at $0.1/1K = $0.1
        // 1000 output tokens at $0.2/1K = $0.2
        // Total = $0.3
        let cost = provider
            .calculate_cost("v0-default", 1000, 1000)
            .await
            .unwrap();
        assert!((cost - 0.3).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_v0_provider_calculate_cost_zero_tokens() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);

        let cost = provider.calculate_cost("v0-default", 0, 0).await.unwrap();
        assert_eq!(cost, 0.0);
    }

    #[tokio::test]
    async fn test_v0_provider_map_openai_params() {
        let config = V0Config {
            api_key: "test-key".to_string(),
            ..Default::default()
        };
        let provider = V0Provider::new_or_default(config);

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));
        params.insert("stream".to_string(), serde_json::json!(true));

        let mapped = provider
            .map_openai_params(params, "v0-default")
            .await
            .unwrap();

        assert_eq!(mapped.get("temperature"), Some(&serde_json::json!(0.7)));
        assert_eq!(mapped.get("stream"), Some(&serde_json::json!(true)));
    }
}
