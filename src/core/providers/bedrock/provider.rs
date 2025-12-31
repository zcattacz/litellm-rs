//! Main Bedrock Provider Implementation
//!
//! Contains the BedrockProvider struct and its LLMProvider trait implementation.

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;
use std::collections::HashMap;
use std::pin::Pin;
use tracing::debug;

use super::client::BedrockClient;
use super::config::BedrockConfig;
use super::error::{BedrockError, BedrockErrorMapper};
use super::model_config::{BedrockModelFamily, get_model_config};
use super::utils::{CostCalculator, validate_region};
use crate::core::traits::ProviderConfig as _;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{
    ChatMessage, FinishReason, MessageContent, MessageRole,
    common::{HealthStatus, ModelInfo, ProviderCapability, RequestContext},
    requests::{ChatRequest, EmbeddingRequest},
    responses::{ChatChunk, ChatResponse, EmbeddingResponse},
};

/// Static capabilities for Bedrock provider
const BEDROCK_CAPABILITIES: &[ProviderCapability] = &[
    ProviderCapability::ChatCompletion,
    ProviderCapability::ChatCompletionStream,
    ProviderCapability::FunctionCalling,
    ProviderCapability::Embeddings,
];

/// AWS Bedrock provider implementation
#[derive(Debug, Clone)]
pub struct BedrockProvider {
    client: BedrockClient,
    models: Vec<ModelInfo>,
}

impl BedrockProvider {
    /// Create a new Bedrock provider instance
    pub async fn new(config: BedrockConfig) -> Result<Self, BedrockError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ProviderError::configuration("bedrock", e))?;

        // Validate AWS region
        validate_region(&config.aws_region)?;

        // Create Bedrock client
        let client = BedrockClient::new(config)?;

        // Define supported models using cost calculator data
        let mut models = Vec::new();
        let available_models = CostCalculator::get_all_models();

        for model_id in available_models {
            if let Some(pricing) = CostCalculator::get_model_pricing(model_id) {
                if let Ok(model_config) = get_model_config(model_id) {
                    models.push(ModelInfo {
                        id: model_id.to_string(),
                        name: format!(
                            "{} (Bedrock)",
                            model_id.split('.').next_back().unwrap_or(model_id)
                        ),
                        provider: "bedrock".to_string(),
                        max_context_length: model_config.max_context_length,
                        max_output_length: model_config.max_output_length,
                        supports_streaming: model_config.supports_streaming,
                        supports_tools: model_config.supports_function_calling,
                        supports_multimodal: model_config.supports_multimodal,
                        input_cost_per_1k_tokens: Some(pricing.input_cost_per_1k),
                        output_cost_per_1k_tokens: Some(pricing.output_cost_per_1k),
                        currency: pricing.currency.to_string(),
                        capabilities: vec![],
                        created_at: None,
                        updated_at: None,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        Ok(Self { client, models })
    }

    /// Generate images using Bedrock image models
    pub async fn generate_image(
        &self,
        request: &crate::core::types::requests::ImageGenerationRequest,
    ) -> Result<crate::core::types::responses::ImageGenerationResponse, BedrockError> {
        super::images::execute_image_generation(&self.client, request).await
    }

    /// Access the Agents client
    pub fn agents_client(&self) -> super::agents::AgentClient<'_> {
        super::agents::AgentClient::new(&self.client)
    }

    /// Access the Knowledge Bases client
    pub fn knowledge_bases_client(&self) -> super::knowledge_bases::KnowledgeBaseClient<'_> {
        super::knowledge_bases::KnowledgeBaseClient::new(&self.client)
    }

    /// Access the Batch processing client
    pub fn batch_client(&self) -> super::batch::BatchClient<'_> {
        super::batch::BatchClient::new(&self.client)
    }

    /// Access the Guardrails client
    pub fn guardrails_client(&self) -> super::guardrails::GuardrailClient<'_> {
        super::guardrails::GuardrailClient::new(&self.client)
    }

    /// Check if model is an embedding model
    fn is_embedding_model(&self, model: &str) -> bool {
        model.contains("embed")
    }

    /// Convert messages to a single prompt string for models that require it
    fn messages_to_prompt(&self, messages: &[ChatMessage]) -> Result<String, BedrockError> {
        let mut prompt = String::new();

        for message in messages {
            let content = match &message.content {
                Some(MessageContent::Text(text)) => text.clone(),
                Some(MessageContent::Parts(parts)) => {
                    // Extract text from parts
                    parts
                        .iter()
                        .filter_map(|part| {
                            if let crate::core::types::requests::ContentPart::Text { text } = part {
                                Some(text.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                }
                None => continue,
            };

            match message.role {
                MessageRole::System => prompt.push_str(&format!("System: {}\n\n", content)),
                MessageRole::User => prompt.push_str(&format!("Human: {}\n\n", content)),
                MessageRole::Assistant => prompt.push_str(&format!("Assistant: {}\n\n", content)),
                MessageRole::Function | MessageRole::Tool => {
                    prompt.push_str(&format!("Tool: {}\n\n", content));
                }
            }
        }

        // Add Assistant prompt at the end for completion
        prompt.push_str("Assistant:");

        Ok(prompt)
    }
}

#[async_trait]
impl LLMProvider for BedrockProvider {
    type Config = BedrockConfig;
    type Error = BedrockError;
    type ErrorMapper = BedrockErrorMapper;

    fn name(&self) -> &'static str {
        "bedrock"
    }

    fn capabilities(&self) -> &'static [ProviderCapability] {
        BEDROCK_CAPABILITIES
    }

    fn models(&self) -> &[ModelInfo] {
        &self.models
    }

    fn get_supported_openai_params(&self, _model: &str) -> &'static [&'static str] {
        &[
            "temperature",
            "top_p",
            "max_tokens",
            "stream",
            "stop",
            "tools",
            "tool_choice",
        ]
    }

    async fn map_openai_params(
        &self,
        params: HashMap<String, Value>,
        _model: &str,
    ) -> Result<HashMap<String, Value>, Self::Error> {
        // Bedrock has some differences from OpenAI format
        let mut mapped = HashMap::new();

        for (key, value) in params {
            match key.as_str() {
                // Map OpenAI parameters to Bedrock format
                "max_tokens" => mapped.insert("max_tokens_to_sample".to_string(), value),
                "temperature" | "top_p" | "stream" | "stop" => mapped.insert(key, value),
                // Skip unsupported parameters
                _ => None,
            };
        }

        Ok(mapped)
    }

    async fn transform_request(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<Value, Self::Error> {
        // Get model configuration
        let model_config = get_model_config(&request.model)?;

        // Route based on model family
        match model_config.family {
            BedrockModelFamily::Claude => {
                // Claude models on Bedrock use anthropic messages format
                let mut body = serde_json::json!({
                    "messages": request.messages,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                    "anthropic_version": "bedrock-2023-05-20"
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                if let Some(top_p) = request.top_p {
                    body["top_p"] =
                        Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::TitanText => {
                // Titan models use different format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "inputText": prompt,
                    "textGenerationConfig": {
                        "maxTokenCount": request.max_tokens.unwrap_or(4096),
                    }
                });

                if let Some(temp) = request.temperature {
                    body["textGenerationConfig"]["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                if let Some(top_p) = request.top_p {
                    body["textGenerationConfig"]["topP"] =
                        Value::Number(serde_json::Number::from_f64(top_p.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Nova => {
                // Nova models use converse API format similar to Claude
                let mut body = serde_json::json!({
                    "messages": request.messages,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Llama => {
                // Meta Llama models use similar format to Claude
                let mut body = serde_json::json!({
                    "messages": request.messages,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Mistral => {
                // Mistral models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::AI21 => {
                // AI21 models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "maxTokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::Cohere => {
                // Cohere models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::DeepSeek => {
                // DeepSeek models use their own format
                let prompt = self.messages_to_prompt(&request.messages)?;
                let mut body = serde_json::json!({
                    "prompt": prompt,
                    "max_tokens": request.max_tokens.unwrap_or(4096),
                });

                if let Some(temp) = request.temperature {
                    body["temperature"] =
                        Value::Number(serde_json::Number::from_f64(temp.into()).unwrap());
                }

                Ok(body)
            }
            BedrockModelFamily::TitanEmbedding
            | BedrockModelFamily::TitanImage
            | BedrockModelFamily::StabilityAI => {
                // These are not chat models
                Err(ProviderError::invalid_request(
                    "bedrock",
                    format!(
                        "Model family {:?} is not supported for chat completion",
                        model_config.family
                    ),
                ))
            }
        }
    }

    async fn transform_response(
        &self,
        raw_response: &[u8],
        model: &str,
        _request_id: &str,
    ) -> Result<ChatResponse, Self::Error> {
        use crate::core::types::responses::{ChatChoice, Usage};

        let response: Value = serde_json::from_slice(raw_response)
            .map_err(|e| ProviderError::response_parsing("bedrock", e.to_string()))?;

        // Get model configuration
        let model_config = get_model_config(model)?;

        let choices = match model_config.family {
            BedrockModelFamily::Claude => {
                // Claude response format
                let content = response
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        thinking: None,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::TitanText => {
                // Titan response format
                let content = response
                    .get("results")
                    .and_then(|r| r.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("outputText"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        thinking: None,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::Nova | BedrockModelFamily::Llama => {
                // Nova and Llama use similar format to Claude
                let content = response
                    .get("content")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        thinking: None,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::Mistral => {
                // Mistral response format
                let content = response
                    .get("outputs")
                    .and_then(|o| o.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        thinking: None,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::AI21 => {
                // AI21 response format
                let content = response
                    .get("completions")
                    .and_then(|c| c.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|item| item.get("data"))
                    .and_then(|data| data.get("text"))
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        thinking: None,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::Cohere => {
                // Cohere response format
                let content = response
                    .get("text")
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        thinking: None,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            BedrockModelFamily::DeepSeek => {
                // DeepSeek response format
                let content = response
                    .get("completion")
                    .and_then(|text| text.as_str())
                    .unwrap_or("")
                    .to_string();

                vec![ChatChoice {
                    index: 0,
                    message: ChatMessage {
                        role: MessageRole::Assistant,
                        content: Some(MessageContent::Text(content)),
                        thinking: None,
                        name: None,
                        function_call: None,
                        tool_calls: None,
                        tool_call_id: None,
                    },
                    finish_reason: Some(FinishReason::Stop),
                    logprobs: None,
                }]
            }
            _ => {
                // Unsupported model family
                return Err(ProviderError::invalid_request(
                    "bedrock",
                    format!(
                        "Model family {:?} is not supported for response parsing",
                        model_config.family
                    ),
                ));
            }
        };

        // Extract usage information based on model family
        let usage = match model_config.family {
            BedrockModelFamily::Claude | BedrockModelFamily::Nova | BedrockModelFamily::Llama => {
                response.get("usage").map(|u| Usage {
                    prompt_tokens: u.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0)
                        as u32,
                    completion_tokens: u.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0)
                        as u32,
                    total_tokens: 0, // Will be calculated below
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                    thinking_usage: None,
                })
            }
            BedrockModelFamily::TitanText => {
                response.get("inputTextTokenCount").and_then(|input| {
                    response.get("results").and_then(|results| {
                        results.as_array().and_then(|arr| {
                            arr.first().and_then(|r| {
                                r.get("tokenCount").map(|output| Usage {
                                    prompt_tokens: input.as_u64().unwrap_or(0) as u32,
                                    completion_tokens: output.as_u64().unwrap_or(0) as u32,
                                    total_tokens: 0, // Will be calculated below
                                    prompt_tokens_details: None,
                                    completion_tokens_details: None,
                                    thinking_usage: None,
                                })
                            })
                        })
                    })
                })
            }
            _ => None,
        };

        let mut final_usage = usage;
        if let Some(ref mut usage) = final_usage {
            usage.total_tokens = usage.prompt_tokens + usage.completion_tokens;
        }

        Ok(ChatResponse {
            id: format!("bedrock-{}", uuid::Uuid::new_v4()),
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: model.to_string(),
            choices,
            usage: final_usage,
            system_fingerprint: None,
        })
    }

    async fn chat_completion(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, Self::Error> {
        debug!("Bedrock chat request: model={}", request.model);

        // Check if it's an embedding model
        if self.is_embedding_model(&request.model) {
            return Err(ProviderError::invalid_request(
                "bedrock",
                "Use embeddings endpoint for embedding models".to_string(),
            ));
        }

        // Use the chat module's routing logic
        let response_value = super::chat::route_chat_request(&self.client, &request).await?;

        // Convert the response to bytes for transform_response
        let response_bytes = serde_json::to_vec(&response_value)
            .map_err(|e| ProviderError::serialization("bedrock", e.to_string()))?;

        self.transform_response(&response_bytes, &request.model, "bedrock-request")
            .await
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
        context: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, Self::Error>> + Send>>, Self::Error>
    {
        debug!("Bedrock streaming chat request: model={}", request.model);

        // Check if it's an embedding model
        if self.is_embedding_model(&request.model) {
            return Err(ProviderError::invalid_request(
                "bedrock",
                "Use embeddings endpoint for embedding models".to_string(),
            ));
        }

        // Get model configuration
        let model_config = get_model_config(&request.model)?;

        if !model_config.supports_streaming {
            return Err(ProviderError::not_supported(
                "bedrock",
                format!("Model {} does not support streaming", request.model),
            ));
        }

        // Transform request
        let body = self.transform_request(request.clone(), context).await?;

        // Use streaming endpoint
        let operation = match model_config.api_type {
            super::model_config::BedrockApiType::ConverseStream => "converse-stream",
            super::model_config::BedrockApiType::InvokeStream => "invoke-with-response-stream",
            _ => {
                return Err(ProviderError::not_supported(
                    "bedrock",
                    format!(
                        "Model {} does not support streaming with API type {:?}",
                        request.model, model_config.api_type
                    ),
                ));
            }
        };

        // Send streaming request
        let response = self
            .client
            .send_streaming_request(&request.model, operation, &body)
            .await?;

        // Create BedrockStream
        let stream = super::streaming::BedrockStream::new(
            response.bytes_stream(),
            model_config.family.clone(),
        );

        Ok(Box::pin(stream))
    }

    async fn embeddings(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, Self::Error> {
        debug!("Bedrock embedding request: model={}", request.model);

        // Use the embeddings module
        super::embeddings::execute_embedding(&self.client, &request).await
    }

    async fn health_check(&self) -> HealthStatus {
        match self.client.health_check().await {
            Ok(is_healthy) => {
                if is_healthy {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Unhealthy
                }
            }
            Err(_) => HealthStatus::Unhealthy,
        }
    }

    fn get_error_mapper(&self) -> Self::ErrorMapper {
        BedrockErrorMapper
    }

    async fn calculate_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Result<f64, Self::Error> {
        CostCalculator::calculate_cost(model, input_tokens, output_tokens)
            .ok_or_else(|| ProviderError::model_not_found("bedrock", model.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::requests::ContentPart;

    fn create_test_config() -> BedrockConfig {
        BedrockConfig {
            aws_access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            aws_secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }

    fn create_test_provider() -> BedrockProvider {
        let config = create_test_config();
        BedrockProvider {
            client: BedrockClient::new(config).unwrap(),
            models: vec![],
        }
    }

    // ==================== Provider Creation Tests ====================

    #[tokio::test]
    async fn test_bedrock_provider_creation() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test_secret".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let provider = BedrockProvider::new(config).await;
        assert!(provider.is_ok());

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "bedrock");
        assert!(
            provider
                .capabilities()
                .contains(&ProviderCapability::ChatCompletion)
        );
    }

    #[tokio::test]
    async fn test_bedrock_provider_creation_with_session_token() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test_secret".to_string(),
            aws_session_token: Some("session_token".to_string()),
            aws_region: "us-west-2".to_string(),
            timeout_seconds: 60,
            max_retries: 5,
        };

        let provider = BedrockProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_bedrock_provider_creation_invalid_region() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test_secret".to_string(),
            aws_session_token: None,
            aws_region: "invalid-region-xyz".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let provider = BedrockProvider::new(config).await;
        assert!(provider.is_err());
    }

    #[tokio::test]
    async fn test_bedrock_provider_creation_empty_credentials() {
        let config = BedrockConfig {
            aws_access_key_id: "".to_string(),
            aws_secret_access_key: "test_secret".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let provider = BedrockProvider::new(config).await;
        assert!(provider.is_err());
    }

    // ==================== Provider Capabilities Tests ====================

    #[test]
    fn test_provider_name() {
        let provider = create_test_provider();
        assert_eq!(provider.name(), "bedrock");
    }

    #[test]
    fn test_provider_capabilities() {
        let provider = create_test_provider();
        let caps = provider.capabilities();

        assert!(caps.contains(&ProviderCapability::ChatCompletion));
        assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
        assert!(caps.contains(&ProviderCapability::FunctionCalling));
        assert!(caps.contains(&ProviderCapability::Embeddings));
    }

    #[test]
    fn test_provider_supported_openai_params() {
        let provider = create_test_provider();
        let params = provider.get_supported_openai_params("anthropic.claude-3-sonnet-20240229");

        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"top_p"));
        assert!(params.contains(&"max_tokens"));
        assert!(params.contains(&"stream"));
        assert!(params.contains(&"stop"));
        assert!(params.contains(&"tools"));
        assert!(params.contains(&"tool_choice"));
    }

    #[test]
    fn test_provider_models_empty_initially() {
        let provider = create_test_provider();
        assert!(provider.models().is_empty());
    }

    // ==================== Embedding Model Detection Tests ====================

    #[test]
    fn test_embedding_model_detection() {
        let provider = create_test_provider();

        assert!(provider.is_embedding_model("amazon.titan-embed-text-v1"));
        assert!(provider.is_embedding_model("cohere.embed-english-v3"));
        assert!(provider.is_embedding_model("my-embed-model"));
        assert!(!provider.is_embedding_model("anthropic.claude-3-sonnet"));
        assert!(!provider.is_embedding_model("amazon.titan-text-express-v1"));
    }

    // ==================== Messages to Prompt Conversion Tests ====================

    #[test]
    fn test_messages_to_prompt_simple_user_message() {
        let provider = create_test_provider();

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello, how are you?".to_string())),
            ..Default::default()
        }];

        let prompt = provider.messages_to_prompt(&messages).unwrap();
        assert!(prompt.contains("Human: Hello, how are you?"));
        assert!(prompt.ends_with("Assistant:"));
    }

    #[test]
    fn test_messages_to_prompt_system_message() {
        let provider = create_test_provider();

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("You are a helpful assistant.".to_string())),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            },
        ];

        let prompt = provider.messages_to_prompt(&messages).unwrap();
        assert!(prompt.contains("System: You are a helpful assistant."));
        assert!(prompt.contains("Human: Hello"));
    }

    #[test]
    fn test_messages_to_prompt_assistant_message() {
        let provider = create_test_provider();

        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: Some(MessageContent::Text("Hi there!".to_string())),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("How are you?".to_string())),
                ..Default::default()
            },
        ];

        let prompt = provider.messages_to_prompt(&messages).unwrap();
        assert!(prompt.contains("Human: Hello"));
        assert!(prompt.contains("Assistant: Hi there!"));
        assert!(prompt.contains("Human: How are you?"));
    }

    #[test]
    fn test_messages_to_prompt_tool_message() {
        let provider = create_test_provider();

        let messages = vec![ChatMessage {
            role: MessageRole::Tool,
            content: Some(MessageContent::Text("Tool output".to_string())),
            ..Default::default()
        }];

        let prompt = provider.messages_to_prompt(&messages).unwrap();
        assert!(prompt.contains("Tool: Tool output"));
    }

    #[test]
    fn test_messages_to_prompt_function_message() {
        let provider = create_test_provider();

        let messages = vec![ChatMessage {
            role: MessageRole::Function,
            content: Some(MessageContent::Text("Function result".to_string())),
            ..Default::default()
        }];

        let prompt = provider.messages_to_prompt(&messages).unwrap();
        assert!(prompt.contains("Tool: Function result"));
    }

    #[test]
    fn test_messages_to_prompt_with_content_parts() {
        let provider = create_test_provider();

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Parts(vec![
                ContentPart::Text { text: "Hello".to_string() },
                ContentPart::Text { text: "World".to_string() },
            ])),
            ..Default::default()
        }];

        let prompt = provider.messages_to_prompt(&messages).unwrap();
        assert!(prompt.contains("Human: Hello World"));
    }

    #[test]
    fn test_messages_to_prompt_none_content() {
        let provider = create_test_provider();

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: None,
            ..Default::default()
        }];

        let prompt = provider.messages_to_prompt(&messages).unwrap();
        assert!(prompt.ends_with("Assistant:"));
    }

    // ==================== OpenAI Params Mapping Tests ====================

    #[tokio::test]
    async fn test_map_openai_params_max_tokens() {
        let provider = create_test_provider();

        let mut params = HashMap::new();
        params.insert("max_tokens".to_string(), Value::Number(100.into()));

        let mapped = provider.map_openai_params(params, "anthropic.claude-3-sonnet-20240229").await.unwrap();

        assert!(mapped.contains_key("max_tokens_to_sample"));
        assert_eq!(mapped.get("max_tokens_to_sample").unwrap(), &Value::Number(100.into()));
    }

    #[tokio::test]
    async fn test_map_openai_params_temperature() {
        let provider = create_test_provider();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.7));

        let mapped = provider.map_openai_params(params, "anthropic.claude-3-sonnet-20240229").await.unwrap();

        assert!(mapped.contains_key("temperature"));
    }

    #[tokio::test]
    async fn test_map_openai_params_unsupported_ignored() {
        let provider = create_test_provider();

        let mut params = HashMap::new();
        params.insert("unsupported_param".to_string(), Value::String("value".to_string()));

        let mapped = provider.map_openai_params(params, "anthropic.claude-3-sonnet-20240229").await.unwrap();

        assert!(!mapped.contains_key("unsupported_param"));
    }

    #[tokio::test]
    async fn test_map_openai_params_multiple() {
        let provider = create_test_provider();

        let mut params = HashMap::new();
        params.insert("temperature".to_string(), serde_json::json!(0.5));
        params.insert("top_p".to_string(), serde_json::json!(0.9));
        params.insert("stream".to_string(), Value::Bool(true));
        params.insert("stop".to_string(), serde_json::json!(["END"]));

        let mapped = provider.map_openai_params(params, "anthropic.claude-3-sonnet-20240229").await.unwrap();

        assert!(mapped.contains_key("temperature"));
        assert!(mapped.contains_key("top_p"));
        assert!(mapped.contains_key("stream"));
        assert!(mapped.contains_key("stop"));
    }

    // ==================== Transform Request Tests ====================

    #[tokio::test]
    async fn test_transform_request_claude() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "anthropic.claude-3-sonnet-20240229".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.get("messages").is_some());
        assert_eq!(body.get("max_tokens").unwrap(), 1000);
        assert!(body.get("anthropic_version").is_some());
    }

    #[tokio::test]
    async fn test_transform_request_titan() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "amazon.titan-text-express-v1".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            max_tokens: Some(500),
            temperature: Some(0.5),
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.get("inputText").is_some());
        assert!(body.get("textGenerationConfig").is_some());
    }

    #[tokio::test]
    async fn test_transform_request_nova() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "amazon.nova-pro-v1:0".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            max_tokens: Some(2000),
            temperature: Some(0.8),
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.get("messages").is_some());
        assert_eq!(body.get("max_tokens").unwrap(), 2000);
    }

    #[tokio::test]
    async fn test_transform_request_llama() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "meta.llama3-70b-instruct-v1:0".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            max_tokens: Some(1500),
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transform_request_mistral() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "mistral.mistral-large-2407-v1:0".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.get("prompt").is_some());
    }

    #[tokio::test]
    async fn test_transform_request_ai21() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "ai21.jamba-1-5-large-v1:0".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.get("prompt").is_some());
        assert!(body.get("maxTokens").is_some());
    }

    #[tokio::test]
    async fn test_transform_request_cohere() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "cohere.command-r-plus-v1:0".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_ok());
        let body = result.unwrap();
        assert!(body.get("prompt").is_some());
    }

    #[tokio::test]
    async fn test_transform_request_embedding_model_error() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "amazon.titan-embed-text-v1".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transform_request_unknown_model() {
        let provider = create_test_provider();

        let request = ChatRequest {
            model: "unknown.model-v1".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let context = RequestContext::default();
        let result = provider.transform_request(request, context).await;

        assert!(result.is_err());
    }

    // ==================== Transform Response Tests ====================

    #[tokio::test]
    async fn test_transform_response_claude() {
        let provider = create_test_provider();

        let response = serde_json::json!({
            "content": [{"text": "Hello! I'm doing well."}],
            "usage": {
                "input_tokens": 10,
                "output_tokens": 20
            }
        });
        let response_bytes = serde_json::to_vec(&response).unwrap();

        let result = provider.transform_response(
            &response_bytes,
            "anthropic.claude-3-sonnet-20240229",
            "test-request-id"
        ).await;

        assert!(result.is_ok());
        let chat_response = result.unwrap();
        assert_eq!(chat_response.model, "anthropic.claude-3-sonnet-20240229");
        assert!(!chat_response.choices.is_empty());
    }

    #[tokio::test]
    async fn test_transform_response_titan() {
        let provider = create_test_provider();

        let response = serde_json::json!({
            "results": [{"outputText": "Hello from Titan!"}],
            "inputTextTokenCount": 5
        });
        let response_bytes = serde_json::to_vec(&response).unwrap();

        let result = provider.transform_response(
            &response_bytes,
            "amazon.titan-text-express-v1",
            "test-request-id"
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transform_response_nova() {
        let provider = create_test_provider();

        let response = serde_json::json!({
            "content": [{"text": "Nova response"}],
            "usage": {
                "input_tokens": 15,
                "output_tokens": 25
            }
        });
        let response_bytes = serde_json::to_vec(&response).unwrap();

        let result = provider.transform_response(
            &response_bytes,
            "amazon.nova-pro-v1:0",
            "test-request-id"
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transform_response_mistral() {
        let provider = create_test_provider();

        let response = serde_json::json!({
            "outputs": [{"text": "Mistral response"}]
        });
        let response_bytes = serde_json::to_vec(&response).unwrap();

        let result = provider.transform_response(
            &response_bytes,
            "mistral.mistral-large-2407-v1:0",
            "test-request-id"
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transform_response_ai21() {
        let provider = create_test_provider();

        let response = serde_json::json!({
            "completions": [{"data": {"text": "AI21 response"}}]
        });
        let response_bytes = serde_json::to_vec(&response).unwrap();

        let result = provider.transform_response(
            &response_bytes,
            "ai21.jamba-1-5-large-v1:0",
            "test-request-id"
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transform_response_cohere() {
        let provider = create_test_provider();

        let response = serde_json::json!({
            "text": "Cohere response"
        });
        let response_bytes = serde_json::to_vec(&response).unwrap();

        let result = provider.transform_response(
            &response_bytes,
            "cohere.command-r-plus-v1:0",
            "test-request-id"
        ).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transform_response_invalid_json() {
        let provider = create_test_provider();

        let response_bytes = b"not valid json";

        let result = provider.transform_response(
            response_bytes,
            "anthropic.claude-3-sonnet-20240229",
            "test-request-id"
        ).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transform_response_unknown_model() {
        let provider = create_test_provider();

        let response = serde_json::json!({"text": "response"});
        let response_bytes = serde_json::to_vec(&response).unwrap();

        let result = provider.transform_response(
            &response_bytes,
            "unknown.model-v1",
            "test-request-id"
        ).await;

        assert!(result.is_err());
    }

    // ==================== Cost Calculation Tests ====================

    #[tokio::test]
    async fn test_calculate_cost_known_model() {
        let provider = create_test_provider();

        let cost = provider.calculate_cost(
            "anthropic.claude-3-opus-20240229",
            1000,
            500
        ).await;

        assert!(cost.is_ok());
        let cost_value = cost.unwrap();
        assert!(cost_value > 0.0);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let provider = create_test_provider();

        let cost = provider.calculate_cost(
            "unknown.model-v1",
            1000,
            500
        ).await;

        assert!(cost.is_err());
    }

    #[tokio::test]
    async fn test_calculate_cost_zero_tokens() {
        let provider = create_test_provider();

        let cost = provider.calculate_cost(
            "anthropic.claude-3-haiku-20240307",
            0,
            0
        ).await;

        assert!(cost.is_ok());
        assert!((cost.unwrap() - 0.0).abs() < 0.0001);
    }

    // ==================== Error Mapper Tests ====================

    #[test]
    fn test_get_error_mapper() {
        let provider = create_test_provider();
        let mapper = provider.get_error_mapper();

        // Test that we can get an error mapper (it's a struct)
        let _ = format!("{:?}", mapper);
    }

    // ==================== Client Access Tests ====================

    #[test]
    fn test_agents_client_access() {
        let provider = create_test_provider();
        let _agents_client = provider.agents_client();
        // Just verify we can access the agents client
    }

    #[test]
    fn test_knowledge_bases_client_access() {
        let provider = create_test_provider();
        let _kb_client = provider.knowledge_bases_client();
        // Just verify we can access the knowledge bases client
    }

    #[test]
    fn test_batch_client_access() {
        let provider = create_test_provider();
        let _batch_client = provider.batch_client();
        // Just verify we can access the batch client
    }

    #[test]
    fn test_guardrails_client_access() {
        let provider = create_test_provider();
        let _guardrails_client = provider.guardrails_client();
        // Just verify we can access the guardrails client
    }

    // ==================== Capabilities Constants Tests ====================

    #[test]
    fn test_bedrock_capabilities_constant() {
        assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
        assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
        assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::FunctionCalling));
        assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::Embeddings));
        assert_eq!(BEDROCK_CAPABILITIES.len(), 4);
    }

    // ==================== Provider Clone/Debug Tests ====================

    #[test]
    fn test_provider_clone() {
        let provider = create_test_provider();
        let cloned = provider.clone();

        assert_eq!(provider.name(), cloned.name());
        assert_eq!(provider.capabilities().len(), cloned.capabilities().len());
    }

    #[test]
    fn test_provider_debug() {
        let provider = create_test_provider();
        let debug_str = format!("{:?}", provider);

        assert!(debug_str.contains("BedrockProvider"));
    }
}
