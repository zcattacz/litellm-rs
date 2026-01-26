//! Maritalk Provider Implementation

use crate::core::traits::provider::ProviderConfig;
use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::ChatRequest,
    responses::{ChatChunk, ChatResponse},
};

use super::config::MaritalkConfig;
use super::model_info;

#[derive(Debug, Clone)]
pub struct MaritalkProvider {
    config: MaritalkConfig,
    http_client: reqwest::Client,
    supported_models: Vec<ModelInfo>,
}

impl MaritalkProvider {
    pub fn new(config: MaritalkConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("maritalk", e))?;

        let http_client = reqwest::Client::builder()
            .timeout(config.timeout())
            .build()
            .map_err(|e| {
                ProviderError::initialization(
                    "maritalk",
                    format!("Failed to create HTTP client: {}", e),
                )
            })?;

        Ok(Self {
            config,
            http_client,
            supported_models: model_info::get_supported_models(),
        })
    }

    pub async fn with_api_key(api_key: impl Into<String>) -> Result<Self, ProviderError> {
        let config = MaritalkConfig::new(api_key);
        Self::new(config)
    }

    fn build_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Some(api_key) = &self.config.base.api_key {
            headers.insert("Authorization".to_string(), format!("Key {}", api_key));
        }

        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }
}

#[async_trait]
impl LLMProvider for MaritalkProvider {
    type Config = MaritalkConfig;
    type Error = ProviderError;
    type ErrorMapper = super::error_mapper::MaritalkErrorMapper;

    fn name(&self) -> &'static str {
        super::PROVIDER_NAME
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[
            ProviderCapability::ChatCompletion,
            ProviderCapability::ChatCompletionStream,
        ]
    }

    fn models(&self) -> &[ModelInfo] {
        &self.supported_models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &["temperature", "max_tokens", "top_p", "stream", "stop"]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        Ok(params)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        let mut req = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
        });

        if let Some(max_tokens) = request.max_tokens {
            req["max_tokens"] = Value::Number(max_tokens.into());
        }

        if let Some(temperature) = request.temperature {
            req["temperature"] = serde_json::to_value(temperature)
                .map_err(|e| ProviderError::serialization("maritalk", e.to_string()))?;
        }

        if let Some(top_p) = request.top_p {
            req["top_p"] = serde_json::to_value(top_p)
                .map_err(|e| ProviderError::serialization("maritalk", e.to_string()))?;
        }

        if request.stream {
            req["stream"] = Value::Bool(true);
        }

        Ok(req)
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_text = String::from_utf8_lossy(raw_response);
        let response: ChatResponse = serde_json::from_str(&response_text)
            .map_err(|e| ProviderError::serialization("maritalk", e.to_string()))?;
        Ok(response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        super::error_mapper::MaritalkErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let url = format!(
            "{}/chat/completions",
            self.config
                .base
                .api_base
                .as_ref()
                .unwrap_or(&super::DEFAULT_BASE_URL.to_string())
        );

        let body = self.transform_request(request.clone(), context).await?;
        let headers = self.build_headers();

        let mut req_builder = self.http_client.post(&url);
        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("maritalk", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();

            return Err(match status {
                401 => ProviderError::authentication("maritalk", error_text),
                429 => ProviderError::rate_limit("maritalk", None),
                404 => ProviderError::model_not_found("maritalk", request.model),
                _ => ProviderError::api_error("maritalk", status, error_text),
            });
        }

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("maritalk", e.to_string()))?;

        self.transform_response(&response_bytes, &request.model, "")
            .await
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        Err(ProviderError::not_implemented(
            "maritalk",
            "Streaming not yet implemented",
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        let model_info = self
            .supported_models
            .iter()
            .find(|m| m.id == model)
            .ok_or_else(|| ProviderError::model_not_found("maritalk", model.to_string()))?;

        let input_cost =
            model_info.input_cost_per_1k_tokens.unwrap_or(0.0) * input_tokens as f64 / 1000.0;
        let output_cost =
            model_info.output_cost_per_1k_tokens.unwrap_or(0.0) * output_tokens as f64 / 1000.0;

        Ok(input_cost + output_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "sabia-2-medium"));
        assert!(models.iter().any(|m| m.id == "sabia-2-small"));
    }

    #[test]
    fn test_build_headers() {
        let config = MaritalkConfig::new("test-api-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let headers = provider.build_headers();
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Key test-api-key".to_string())
        );
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[tokio::test]
    async fn test_calculate_cost() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        // Test sabia-2-medium cost calculation
        let cost = provider.calculate_cost("sabia-2-medium", 1000, 1000).await;
        assert!(cost.is_ok());

        // Expected: (0.00002 * 1000/1000) + (0.00004 * 1000/1000) = 0.00006
        let cost_value = cost.unwrap();
        assert!((cost_value - 0.00006).abs() < 0.000001);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let config = MaritalkConfig::new("test-key");
        let provider = MaritalkProvider::new(config).unwrap();

        let cost = provider.calculate_cost("unknown-model", 1000, 1000).await;
        assert!(cost.is_err());
    }
}
