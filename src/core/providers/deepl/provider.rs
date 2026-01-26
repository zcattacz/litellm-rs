//! DeepL Translation Provider Implementation
//!
//! Note: DeepL is a translation service, not a chat completion model.
//! This provider adapts DeepL's translation API to the LLMProvider interface
//! by treating translation requests as special chat completions.

use crate::core::traits::provider::ProviderConfig;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    message::{MessageContent, MessageRole},
    requests::ChatRequest,
    responses::{ChatChunk, ChatResponse},
};

use super::config::DeepLConfig;
use super::model_info;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeepLTranslateRequest {
    text: Vec<String>,
    target_lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    formality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeepLTranslation {
    detected_source_language: String,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeepLTranslateResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Clone)]
pub struct DeepLProvider {
    config: DeepLConfig,
    http_client: reqwest::Client,
    supported_models: Vec<ModelInfo>,
}

impl DeepLProvider {
    pub fn new(config: DeepLConfig) -> Result<Self, ProviderError> {
        config
            .validate()
            .map_err(|e| ProviderError::configuration("deepl", e))?;

        let http_client = reqwest::Client::builder()
            .timeout(config.timeout())
            .build()
            .map_err(|e| {
                ProviderError::initialization(
                    "deepl",
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
        let config = DeepLConfig::new(api_key);
        Self::new(config)
    }

    fn build_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Some(api_key) = &self.config.base.api_key {
            headers.insert(
                "Authorization".to_string(),
                format!("DeepL-Auth-Key {}", api_key),
            );
        }

        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    /// Extract translation parameters from chat request
    /// Expected message format: "Translate to {target_lang}: {text}"
    /// Or with source language: "Translate from {source_lang} to {target_lang}: {text}"
    fn extract_translation_params(
        &self,
        request: &ChatRequest,
    ) -> Result<(String, Option<String>, String), ProviderError> {
        // Get the last user message
        let user_message = request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .ok_or_else(|| ProviderError::invalid_request("deepl", "No user message found"))?;

        let content = user_message
            .content
            .as_ref()
            .ok_or_else(|| ProviderError::invalid_request("deepl", "Message content is empty"))?;

        // Extract text from MessageContent
        let text = match content {
            MessageContent::Text(text_content) => text_content.as_str(),
            MessageContent::Parts(parts) => {
                // Find text part
                use crate::core::types::content::ContentPart;
                parts
                    .iter()
                    .find_map(|p| match p {
                        ContentPart::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        ProviderError::invalid_request("deepl", "No text content found in message")
                    })?
            }
        };

        // Parse translation instruction
        let parts: Vec<&str> = text.split(':').collect();
        if parts.len() < 2 {
            return Err(ProviderError::invalid_request(
                "deepl",
                "Invalid translation format. Expected: 'Translate to {lang}: {text}'",
            ));
        }

        let instruction = parts[0].trim();
        let text_to_translate = parts[1..].join(":").trim().to_string();

        let (source_lang, target_lang) =
            if instruction.contains("from") && instruction.contains("to") {
                // Parse "Translate from EN to DE"
                let lang_parts: Vec<&str> = instruction.split_whitespace().collect();
                let from_idx = lang_parts
                    .iter()
                    .position(|&s| s == "from")
                    .ok_or_else(|| {
                        ProviderError::invalid_request("deepl", "Invalid translation format")
                    })?;
                let to_idx = lang_parts.iter().position(|&s| s == "to").ok_or_else(|| {
                    ProviderError::invalid_request("deepl", "Invalid translation format")
                })?;

                let source = if from_idx + 1 < lang_parts.len() {
                    Some(lang_parts[from_idx + 1].to_uppercase())
                } else {
                    None
                };

                let target = if to_idx + 1 < lang_parts.len() {
                    lang_parts[to_idx + 1].to_uppercase()
                } else {
                    return Err(ProviderError::invalid_request(
                        "deepl",
                        "Target language not specified",
                    ));
                };

                (source, target)
            } else if instruction.contains("to") {
                // Parse "Translate to DE"
                let lang_parts: Vec<&str> = instruction.split_whitespace().collect();
                let to_idx = lang_parts.iter().position(|&s| s == "to").ok_or_else(|| {
                    ProviderError::invalid_request("deepl", "Invalid translation format")
                })?;

                let target = if to_idx + 1 < lang_parts.len() {
                    lang_parts[to_idx + 1].to_uppercase()
                } else {
                    return Err(ProviderError::invalid_request(
                        "deepl",
                        "Target language not specified",
                    ));
                };

                (None, target)
            } else {
                return Err(ProviderError::invalid_request(
                    "deepl",
                    "Invalid translation format. Expected: 'Translate to {lang}: {text}'",
                ));
            };

        Ok((target_lang, source_lang, text_to_translate))
    }
}

#[async_trait]
impl LLMProvider for DeepLProvider {
    type Config = DeepLConfig;
    type Error = ProviderError;
    type ErrorMapper = super::error_mapper::DeepLErrorMapper;

    fn name(&self) -> &'static str {
        super::PROVIDER_NAME
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        &[ProviderCapability::AudioTranslation]
    }

    fn models(&self) -> &[ModelInfo] {
        &self.supported_models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature", // Used to control formality (mapped to formality parameter)
        ]
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
        let (target_lang, source_lang, text) = self.extract_translation_params(&request)?;

        let translate_request = DeepLTranslateRequest {
            text: vec![text],
            target_lang,
            source_lang,
            formality: None, // Can be mapped from temperature or other params
        };

        serde_json::to_value(translate_request)
            .map_err(|e| ProviderError::serialization("deepl", e.to_string()))
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        _model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        let response_text = String::from_utf8_lossy(raw_response);
        let deepl_response: DeepLTranslateResponse = serde_json::from_str(&response_text)
            .map_err(|e| ProviderError::serialization("deepl", e.to_string()))?;

        // Convert DeepL response to ChatResponse format
        let translation = deepl_response
            .translations
            .first()
            .ok_or_else(|| ProviderError::api_error("deepl", 500, "No translation returned"))?;

        // Create a chat response with the translation
        let response = serde_json::json!({
            "id": format!("deepl-{}", uuid::Uuid::new_v4()),
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": "deepl-translate",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": translation.text.clone(),
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }
        });

        serde_json::from_value(response)
            .map_err(|e| ProviderError::serialization("deepl", e.to_string()))
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        super::error_mapper::DeepLErrorMapper
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        let url = format!(
            "{}/translate",
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
            .map_err(|e| ProviderError::network("deepl", e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.unwrap_or_default();

            return Err(match status {
                401 | 403 => ProviderError::authentication("deepl", error_text),
                429 => ProviderError::rate_limit("deepl", None),
                456 => ProviderError::quota_exceeded("deepl", "Quota exceeded"),
                400 => ProviderError::invalid_request("deepl", error_text),
                _ => ProviderError::api_error("deepl", status, error_text),
            });
        }

        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| ProviderError::network("deepl", e.to_string()))?;

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
            "deepl",
            "Streaming is not supported for translation",
        ))
    }

    async fn health_check(&self) -> HealthStatus {
        // Try to get usage information as a health check
        let url = format!(
            "{}/usage",
            self.config
                .base
                .api_base
                .as_ref()
                .unwrap_or(&super::DEFAULT_BASE_URL.to_string())
        );

        let headers = self.build_headers();
        let mut req_builder = self.http_client.get(&url);
        for (key, value) in headers {
            req_builder = req_builder.header(key, value);
        }

        match req_builder.send().await {
            Ok(response) if response.status().is_success() => HealthStatus::Healthy,
            Ok(_) => HealthStatus::Unhealthy,
            Err(_) => HealthStatus::Unhealthy,
        }
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
            .ok_or_else(|| ProviderError::model_not_found("deepl", model.to_string()))?;

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
        let config = DeepLConfig::new("test-key");
        let provider = DeepLProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_capabilities() {
        let config = DeepLConfig::new("test-key");
        let provider = DeepLProvider::new(config).unwrap();

        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::AudioTranslation));
        assert!(!caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[test]
    fn test_provider_models() {
        let config = DeepLConfig::new("test-key");
        let provider = DeepLProvider::new(config).unwrap();

        let models = provider.models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.id == "deepl-translate"));
    }

    #[test]
    fn test_provider_name() {
        let config = DeepLConfig::new("test-key");
        let provider = DeepLProvider::new(config).unwrap();
        assert_eq!(provider.name(), "deepl");
    }
}
