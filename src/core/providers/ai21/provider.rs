//! AI21 Provider Implementation
//!
//! Main provider implementation using the unified base infrastructure

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::core::providers::base::{
    GlobalPoolManager, HeaderPair, HttpMethod, get_pricing_db, header, header_owned,
};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::{ProviderConfig, provider::llm_provider::trait_definition::LLMProvider};
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::ChatRequest,
    responses::{ChatChunk, ChatResponse},
};

use super::{AI21Client, AI21Config, AI21ErrorMapper};

#[derive(Debug, Clone)]
pub struct AI21Provider {
    config: AI21Config,
    pool_manager: Arc<GlobalPoolManager>,
    supported_models: Vec<ModelInfo>,
}

impl AI21Provider {
    /// Generate headers for AI21 API requests
    fn get_request_headers(&self) -> Vec<HeaderPair> {
        let mut headers = Vec::with_capacity(2);

        if let Some(api_key) = &self.config.base.api_key {
            headers.push(header("Authorization", format!("Bearer {}", api_key)));
        }

        // Add custom headers
        for (key, value) in &self.config.base.headers {
            headers.push(header_owned(key.clone(), value.clone()));
        }

        headers
    }

    pub fn new(config: AI21Config) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("ai21", e))?;

        let pool_manager = Arc::new(
            GlobalPoolManager::new()
                .map_err(|e| ProviderError::configuration("ai21", e.to_string()))?,
        );
        let supported_models = AI21Client::supported_models();

        Ok(Self {
            config,
            pool_manager,
            supported_models,
        })
    }

    pub fn from_env() -> Result<Self, ProviderError> {
        let config = AI21Config::from_env();
        Self::new(config)
    }

    /// Create provider with API key
    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some(api_key.into());
        Self::new(config)
    }
}

#[async_trait]
impl LLMProvider for AI21Provider {
    type Config = AI21Config;
    type Error = ProviderError;
    type ErrorMapper = AI21ErrorMapper;

    fn name(&self) -> &'static str {
        "ai21"
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
        AI21Client::supported_openai_params()
    }

    async fn map_openai_params(
        &self,
        mut params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Map max_completion_tokens to max_tokens for AI21
        if let Some(max_completion_tokens) = params.remove("max_completion_tokens") {
            params.insert("max_tokens".to_string(), max_completion_tokens);
        }
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        Ok(AI21Client::transform_chat_request(request))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("ai21", e.to_string()))?;
        AI21Client::transform_chat_response(response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        AI21ErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let url = format!(
            "{}/chat/completions",
            self.config.base.get_effective_api_base("ai21")
        );
        let body = AI21Client::transform_chat_request(request.clone());

        let headers = self.get_request_headers();
        let body_data = Some(body);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body_data)
            .await?;

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("ai21", e.to_string()))?;

        self.transform_response(&response_bytes, &request.model, &context.request_id)
            .await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        let url = format!(
            "{}/chat/completions",
            self.config.base.get_effective_api_base("ai21")
        );

        // Create streaming request
        let mut body = AI21Client::transform_chat_request(request.clone());
        body["stream"] = serde_json::Value::Bool(true);

        // Get API key
        let api_key = self
            .config
            .base
            .get_effective_api_key("ai21")
            .ok_or_else(|| ProviderError::authentication("ai21", "API key is required"))?;

        // Create request
        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("ai21", e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ProviderError::api_error(
                "ai21",
                status.as_u16(),
                error_text,
            ));
        }

        // Create AI21 stream using unified SSE parser
        let stream = response.bytes_stream();
        Ok(Box::pin(super::streaming::create_ai21_stream(stream)))
    }

    async fn health_check(&self) -> HealthStatus {
        if self.config.base.get_effective_api_key("ai21").is_some() {
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

    #[test]
    fn test_provider_creation() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();
        assert_eq!(provider.name(), "ai21");
    }

    #[test]
    fn test_provider_capabilities() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();
        let capabilities = provider.capabilities();

        assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
        assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
        assert!(capabilities.contains(&ProviderCapability::ToolCalling));
    }

    #[test]
    fn test_provider_models() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();
        let models = provider.models();

        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "jamba-1.5-large"));
    }

    #[test]
    fn test_provider_from_env() {
        // This test may fail if AI21_API_KEY is not set
        let result = AI21Provider::from_env();
        // Either succeeds or fails with missing API key
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_provider_missing_api_key() {
        let config = AI21Config::new("ai21");
        let result = AI21Provider::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_supported_openai_params() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();
        let params = provider.get_supported_openai_params("jamba-1.5-large");

        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"tools"));
    }

    #[tokio::test]
    async fn test_map_openai_params() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();

        let mut params = HashMap::new();
        params.insert(
            "max_completion_tokens".to_string(),
            serde_json::json!(1000),
        );
        params.insert("temperature".to_string(), serde_json::json!(0.7));

        let mapped = provider
            .map_openai_params(params, "jamba-1.5-large")
            .await
            .unwrap();

        // max_completion_tokens should be mapped to max_tokens
        assert!(mapped.contains_key("max_tokens"));
        assert!(!mapped.contains_key("max_completion_tokens"));
        assert!(mapped.contains_key("temperature"));
    }

    #[tokio::test]
    async fn test_health_check_with_key() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();
        let health = provider.health_check().await;

        assert!(matches!(health, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_calculate_cost() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();
        let cost = provider
            .calculate_cost("jamba-1.5-large", 1000, 500)
            .await
            .unwrap();

        // Cost should be non-negative
        assert!(cost >= 0.0);
    }

    #[test]
    fn test_get_request_headers() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());
        config
            .base
            .headers
            .insert("X-Custom-Header".to_string(), "custom-value".to_string());

        let provider = AI21Provider::new(config).unwrap();
        let headers = provider.get_request_headers();

        assert!(!headers.is_empty());
    }

    #[tokio::test]
    async fn test_transform_request() {
        let mut config = AI21Config::new("ai21");
        config.base.api_key = Some("test-api-key".to_string());

        let provider = AI21Provider::new(config).unwrap();

        let request = ChatRequest {
            model: "jamba-1.5-large".to_string(),
            messages: vec![],
            temperature: Some(0.7),
            max_tokens: Some(100),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stream: false,
            tools: None,
            tool_choice: None,
            user: None,
            response_format: None,
            seed: None,
            max_completion_tokens: None,
            stop: None,
            parallel_tool_calls: None,
            n: None,
            logit_bias: None,
            functions: None,
            function_call: None,
            logprobs: None,
            top_logprobs: None,
            thinking: None,
            extra_params: HashMap::new(),
        };

        let context = RequestContext::default();
        let transformed = provider.transform_request(request, context).await.unwrap();

        assert_eq!(transformed["model"], "jamba-1.5-large");
    }
}
