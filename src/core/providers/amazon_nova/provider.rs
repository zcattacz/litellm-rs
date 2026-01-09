//! Amazon Nova Provider Implementation
//!
//! Main provider implementation for Amazon Nova multimodal models
//! Amazon Nova uses OpenAI-compatible API format

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::providers::base::{
    GlobalPoolManager, HeaderPair, HttpMethod, get_pricing_db, header,
};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{ProviderConfig, provider::llm_provider::trait_definition::LLMProvider};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::ChatRequest,
    responses::{ChatChoice, ChatChunk, ChatResponse, Usage},
};

use super::{AmazonNovaConfig, AmazonNovaErrorMapper, AmazonNovaModelRegistry};

/// Amazon Nova Provider for multimodal chat completions
#[derive(Debug, Clone)]
pub struct AmazonNovaProvider {
    config: AmazonNovaConfig,
    pool_manager: Arc<GlobalPoolManager>,
    model_registry: AmazonNovaModelRegistry,
    supported_models: Vec<ModelInfo>,
}

impl AmazonNovaProvider {
    /// Create a new Amazon Nova provider with configuration
    pub fn new(config: AmazonNovaConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("amazon_nova", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("amazon_nova", e.to_string()))?,
        );

        let model_registry = AmazonNovaModelRegistry::new();
        let supported_models = Self::build_model_info(&model_registry);

        Ok(Self {
            config,
            pool_manager,
            model_registry,
            supported_models,
        })
    }

    /// Create provider from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let config = AmazonNovaConfig::from_env();
        Self::new(config)
    }

    /// Create provider with API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = AmazonNovaConfig::with_api_key(api_key);
        Self::new(config)
    }

    /// Build ModelInfo list from registry
    fn build_model_info(registry: &AmazonNovaModelRegistry) -> Vec<ModelInfo> {
        registry
            .list_models()
            .iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                provider: "amazon_nova".to_string(),
                max_context_length: m.context_length,
                max_output_length: Some(m.max_output_tokens),
                supports_streaming: m.supports_streaming,
                supports_tools: m.supports_tools,
                supports_multimodal: m.supports_vision,
                input_cost_per_1k_tokens: Some(m.input_cost_per_1k),
                output_cost_per_1k_tokens: Some(m.output_cost_per_1k),
                currency: "USD".to_string(),
                capabilities: vec![
                    ProviderCapability::ChatCompletion,
                    ProviderCapability::ChatCompletionStream,
                ],
                created_at: None,
                updated_at: None,
                metadata: std::collections::HashMap::new(),
            })
            .collect()
    }

    /// Generate request headers for Amazon Nova API
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(2);

        if let Some(api_key) = self.config.get_api_key() {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        headers.push(header("Content-Type", "application/json".to_string()));
        headers
    }

    /// Transform ChatRequest to Amazon Nova format (OpenAI compatible)
    fn transform_chat_request(&self, request: ChatRequest) -> Value {
        let mut body = serde_json::json!({
            "model": self.normalize_model_name(&request.model),
            "messages": request.messages,
        });

        // Handle max_tokens with preference for max_completion_tokens
        if let Some(max_completion_tokens) = request.max_completion_tokens {
            body["max_tokens"] = serde_json::json!(max_completion_tokens);
        } else if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(top_p) = request.top_p {
            body["top_p"] = serde_json::json!(top_p);
        }

        if let Some(stop) = &request.stop {
            body["stop"] = serde_json::json!(stop);
        }

        if request.stream {
            body["stream"] = serde_json::json!(true);
        }

        if let Some(tools) = &request.tools {
            body["tools"] = serde_json::json!(tools);
        }

        if let Some(tool_choice) = &request.tool_choice {
            body["tool_choice"] = serde_json::json!(tool_choice);
        }

        body
    }

    /// Normalize model name to full format
    fn normalize_model_name(&self, model: &str) -> String {
        // If already fully qualified, return as-is
        if model.starts_with("amazon.nova") {
            return model.to_string();
        }

        // Map short names to full names
        match model {
            "nova-pro" => "amazon.nova-pro-v1:0".to_string(),
            "nova-lite" => "amazon.nova-lite-v1:0".to_string(),
            "nova-micro" => "amazon.nova-micro-v1:0".to_string(),
            "nova-premier" => "amazon.nova-premier-v1:0".to_string(),
            _ => model.to_string(),
        }
    }

    /// Transform Amazon Nova response to ChatResponse
    fn transform_chat_response(
        &self,
        response_data: Value,
        model: &str,
        request_id: &str,
    ) -> Result<ChatResponse, ProviderError> {
        let id = response_data
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| request_id.to_string());

        let created = response_data
            .get("created")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as i64;

        let response_model = response_data
            .get("model")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| format!("amazon-nova/{}", model));

        // Parse choices
        let choices: Vec<ChatChoice> = response_data
            .get("choices")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        // Parse usage
        let usage: Option<Usage> = response_data
            .get("usage")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        Ok(ChatResponse {
            id,
            object: "chat.completion".to_string(),
            created,
            model: response_model,
            choices,
            usage,
            system_fingerprint: response_data
                .get("system_fingerprint")
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }
}

#[async_trait]
impl LLMProvider for AmazonNovaProvider {
    type Config = AmazonNovaConfig;
    type Error = ProviderError;
    type ErrorMapper = AmazonNovaErrorMapper;

    fn name(&self) -> &'static str {
        "amazon_nova"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
            ProviderCapability::ToolCalling,
        ]
    }

    fn models(&self) -> &[ModelInfo] {
        &self.supported_models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        super::models::SUPPORTED_OPENAI_PARAMS
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Amazon Nova is OpenAI compatible, pass through most params
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        Ok(self.transform_chat_request(request))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_data: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("amazon_nova", e.to_string()))?;

        self.transform_chat_response(response_data, model, request_id)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        AmazonNovaErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let url = self.config.get_chat_endpoint();
        let body = self.transform_chat_request(request.clone());

        let headers = self.get_request_headers();

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("amazon_nova", e.to_string()))?;

        if !status.is_success() {
            let error_text = String::from_utf8_lossy(&response_bytes);
            return Err(ProviderError::api_error(
                "amazon_nova",
                status.as_u16(),
                error_text.to_string(),
            ));
        }

        self.transform_response(&response_bytes, &request.model, &context.request_id)
            .await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        let url = self.config.get_chat_endpoint();

        let mut body = self.transform_chat_request(request);
        body["stream"] = serde_json::Value::Bool(true);

        let api_key = self
            .config
            .get_api_key()
            .ok_or_else(|| ProviderError::authentication("amazon_nova", "API key is required"))?;

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("amazon_nova", e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProviderError::api_error(
                "amazon_nova",
                status.as_u16(),
                error_text,
            ));
        }

        // Create stream using OpenAI-compatible SSE format
        use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
        let stream = response.bytes_stream();
        let transformer = OpenAICompatibleTransformer::new("amazon_nova");
        Ok(Box::pin(UnifiedSSEStream::new(stream, transformer)))
    }

    async fn health_check(&self) -> HealthStatus {
        if self.config.get_api_key().is_some() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy
        }
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        // First try model registry
        let cost = self
            .model_registry
            .calculate_cost(model, input_tokens, output_tokens);
        if cost > 0.0 {
            return Ok(cost);
        }

        // Fall back to pricing database
        let usage = crate::core::providers::base::pricing::Usage {
            prompt_tokens: input_tokens,
            completion_tokens: output_tokens,
            total_tokens: input_tokens + output_tokens,
            reasoning_tokens: None,
        };

        Ok(get_pricing_db().calculate(model, &usage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::chat::ChatMessage;
    use crate::core::types::message::{MessageContent, MessageRole};

    #[test]
    fn test_provider_creation_fails_without_api_key() {
        let config = AmazonNovaConfig::default();
        let result = AmazonNovaProvider::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_creation_with_api_key() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let result = AmazonNovaProvider::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();
        assert_eq!(provider.name(), "amazon_nova");
    }

    #[test]
    fn test_provider_capabilities() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();
        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
        assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
        assert!(caps.contains(&ProviderCapability::ToolCalling));
    }

    #[test]
    fn test_provider_models() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();
        let models = provider.models();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_normalize_model_name() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();

        assert_eq!(
            provider.normalize_model_name("nova-pro"),
            "amazon.nova-pro-v1:0"
        );
        assert_eq!(
            provider.normalize_model_name("nova-lite"),
            "amazon.nova-lite-v1:0"
        );
        assert_eq!(
            provider.normalize_model_name("amazon.nova-pro-v1:0"),
            "amazon.nova-pro-v1:0"
        );
    }

    #[test]
    fn test_transform_chat_request() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "nova-pro".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                tool_calls: None,
                tool_call_id: None,
                ..Default::default()
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            ..Default::default()
        };

        let body = provider.transform_chat_request(request);
        assert_eq!(body["model"], "amazon.nova-pro-v1:0");
        let temp = body["temperature"].as_f64().unwrap();
        assert!((temp - 0.7).abs() < 0.01);
        assert_eq!(body["max_tokens"], 100);
    }

    #[test]
    fn test_transform_chat_request_with_max_completion_tokens() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();

        let request = ChatRequest {
            model: "nova-pro".to_string(),
            messages: vec![],
            max_tokens: Some(100),
            max_completion_tokens: Some(200), // Should take precedence
            ..Default::default()
        };

        let body = provider.transform_chat_request(request);
        assert_eq!(body["max_tokens"], 200);
    }

    #[test]
    fn test_transform_chat_response() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();

        let response_data = serde_json::json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677858242,
            "model": "amazon.nova-pro-v1:0",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello! How can I help you?"
                    },
                    "finish_reason": "stop"
                }
            ],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 20,
                "total_tokens": 30
            }
        });

        let result = provider.transform_chat_response(response_data, "nova-pro", "req-123");
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.choices.len(), 1);
        assert!(response.usage.is_some());
    }

    #[test]
    fn test_supported_openai_params() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();
        let params = provider.get_supported_openai_params("any-model");
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"tools"));
    }

    #[tokio::test]
    async fn test_calculate_cost() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();

        let cost = provider
            .calculate_cost("amazon.nova-pro-v1:0", 1000, 500)
            .await;
        assert!(cost.is_ok());
        assert!(cost.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_health_check_with_api_key() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();

        let status = provider.health_check().await;
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_map_openai_params() {
        let config = AmazonNovaConfig::with_api_key("test-key");
        let provider = AmazonNovaProvider::new(config).unwrap();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));
        params.insert("max_tokens".to_string(), serde_json::json!(100));

        let result = provider.map_openai_params(params.clone(), "nova-pro").await;
        assert!(result.is_ok());

        let mapped = result.unwrap();
        assert_eq!(mapped.get("temperature"), params.get("temperature"));
        assert_eq!(mapped.get("max_tokens"), params.get("max_tokens"));
    }
}
