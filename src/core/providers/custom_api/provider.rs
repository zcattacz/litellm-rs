//! Custom HTTPX Provider Implementation

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

use super::config::CustomHttpxConfig;
use super::model_info;

#[derive(Debug, Clone)]
pub struct CustomHttpxProvider {
    config: CustomHttpxConfig,
    http_client: reqwest::Client,
    supported_models: Vec<ModelInfo>,
}

impl CustomHttpxProvider {
    pub fn new(config: CustomHttpxConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("custom_httpx", e))?;

        let http_client = reqwest::Client::builder()
            .timeout(config.timeout())
            .build()
            .map_err(|e| {
                ProviderError::initialization(
                    "custom_httpx",
                    format!("Failed to create HTTP client: {}", e),
                )
            })?;

        Ok(Self {
            config,
            http_client,
            supported_models: model_info::get_supported_models(),
        })
    }

    pub fn with_endpoint(endpoint_url: impl Into<String>) -> Result<Self, ProviderError> {
        let config = CustomHttpxConfig::new(endpoint_url);
        Self::new(config)
    }

    fn build_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Some(api_key) = &self.config.base.api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));
        }

        headers.insert("Content-Type".to_string(), "application/json".to_string());

        for (key, value) in &self.config.base.headers {
            headers.insert(key.clone(), value.clone());
        }

        headers
    }
}

#[async_trait]
impl LLMProvider for CustomHttpxProvider {
    type Config = CustomHttpxConfig;
    type Error = ProviderError;
    type ErrorMapper = super::error_mapper::CustomApiErrorMapper;

    fn name(&self) -> &'static str {
        super::PROVIDER_NAME
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[ProviderCapability::ChatCompletion]
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
        if let Some(template) = &self.config.request_template {
            let req_str = template.replace("{model}", &request.model).replace(
                "{messages}",
                &serde_json::to_string(&request.messages)
                    .map_err(|e| ProviderError::serialization("custom_httpx", e.to_string()))?,
            );

            serde_json::from_str(&req_str)
                .map_err(|e| ProviderError::serialization("custom_httpx", e.to_string()))
        } else {
            let mut req = serde_json::json!({
                "model": request.model,
                "messages": request.messages,
            });

            if let Some(max_tokens) = request.max_tokens {
                req["max_tokens"] = Value::Number(max_tokens.into());
            }

            if let Some(temperature) = request.temperature {
                req["temperature"] = serde_json::to_value(temperature)
                    .map_err(|e| ProviderError::serialization("custom_httpx", e.to_string()))?;
            }

            Ok(req)
        }
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_text = String::from_utf8_lossy(raw_response);
        let response: ChatResponse = serde_json::from_str(&response_text)
            .map_err(|e| ProviderError::serialization("custom_httpx", e.to_string()))?;
        Ok(response)
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        super::error_mapper::CustomApiErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let body = self.transform_request(request.clone(), context).await?;
        let headers = self.build_headers();

        let mut req_builder = match self.config.http_method.to_uppercase().as_str() {
            "GET" => self.http_client.get(&self.config.endpoint_url),
            "POST" => self.http_client.post(&self.config.endpoint_url),
            "PUT" => self.http_client.put(&self.config.endpoint_url),
            _ => self.http_client.post(&self.config.endpoint_url),
        };

        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::network("custom_httpx", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();

            return Err(match status {
                401 => ProviderError::authentication("custom_httpx", error_text),
                429 => ProviderError::rate_limit("custom_httpx", None),
                404 => ProviderError::model_not_found("custom_httpx", request.model),
                _ => ProviderError::api_error("custom_httpx", status, error_text),
            });
        }

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("custom_httpx", e.to_string()))?;

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
            "custom_httpx",
            "Streaming not yet implemented",
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    async fn calculate_cost(
        &self,
        _model: &str,
        _input_tokens: u32,
        _output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        Ok(0.0)
    }
}
