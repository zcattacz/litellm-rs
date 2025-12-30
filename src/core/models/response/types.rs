//! Core response types for the Gateway

use super::super::Metadata;
use super::super::openai::*;
use super::completion::CompletionResponse;
use super::embedding::EmbeddingResponse;
use super::error::ErrorResponse;
use super::media::{AudioTranscriptionResponse, ImageGenerationResponse};
use super::metadata::{CacheInfo, ProviderInfo, ResponseMetrics};
use super::moderation::ModerationResponse;
use super::rerank::RerankResponse;
use serde::{Deserialize, Serialize};

/// Response type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    /// Chat completion response
    ChatCompletion,
    /// Text completion response
    Completion,
    /// Embedding response
    Embedding,
    /// Image generation response
    ImageGeneration,
    /// Audio transcription response
    AudioTranscription,
    /// Moderation response
    Moderation,
    /// Rerank response
    Rerank,
    /// Error response
    Error,
}

/// Response data union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseData {
    /// Chat completion response data
    #[serde(rename = "chat_completion")]
    ChatCompletion(ChatCompletionResponse),
    /// Text completion response data
    #[serde(rename = "completion")]
    Completion(CompletionResponse),
    /// Embedding response data
    #[serde(rename = "embedding")]
    Embedding(EmbeddingResponse),
    /// Image generation response data
    #[serde(rename = "image_generation")]
    ImageGeneration(ImageGenerationResponse),
    /// Audio transcription response data
    #[serde(rename = "audio_transcription")]
    AudioTranscription(AudioTranscriptionResponse),
    /// Moderation response data
    #[serde(rename = "moderation")]
    Moderation(ModerationResponse),
    /// Rerank response data
    #[serde(rename = "rerank")]
    Rerank(RerankResponse),
    /// Error response data
    #[serde(rename = "error")]
    Error(ErrorResponse),
}

/// Internal gateway response wrapper
#[derive(Debug, Clone)]
pub struct GatewayResponse {
    /// Response metadata
    pub metadata: Metadata,
    /// Response type
    pub response_type: ResponseType,
    /// Response data
    pub data: ResponseData,
    /// Provider information
    pub provider_info: ProviderInfo,
    /// Performance metrics
    pub metrics: ResponseMetrics,
    /// Caching information
    pub cache_info: CacheInfo,
}

impl GatewayResponse {
    /// Create a new gateway response
    pub fn new(response_type: ResponseType, data: ResponseData) -> Self {
        Self {
            metadata: Metadata::new(),
            response_type,
            data,
            provider_info: ProviderInfo::default(),
            metrics: ResponseMetrics::default(),
            cache_info: CacheInfo::default(),
        }
    }

    /// Set provider information
    pub fn with_provider_info(mut self, provider_info: ProviderInfo) -> Self {
        self.provider_info = provider_info;
        self
    }

    /// Set metrics
    pub fn with_metrics(mut self, metrics: ResponseMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set cache information
    pub fn with_cache_info(mut self, cache_info: CacheInfo) -> Self {
        self.cache_info = cache_info;
        self
    }

    /// Check if response is an error
    pub fn is_error(&self) -> bool {
        matches!(self.response_type, ResponseType::Error)
    }

    /// Get usage information if available
    pub fn usage(&self) -> Option<&Usage> {
        match &self.data {
            ResponseData::ChatCompletion(resp) => resp.usage.as_ref(),
            ResponseData::Completion(resp) => resp.usage.as_ref(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Helper Functions ====================

    fn create_test_chat_response() -> ChatCompletionResponse {
        ChatCompletionResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            system_fingerprint: None,
            choices: vec![],
            usage: None,
        }
    }

    fn create_test_chat_response_with_usage() -> ChatCompletionResponse {
        ChatCompletionResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            system_fingerprint: None,
            choices: vec![],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            }),
        }
    }

    fn create_test_error_response() -> ErrorResponse {
        ErrorResponse {
            error: crate::core::models::response::error::ErrorDetail {
                message: "Test error".to_string(),
                error_type: "invalid_request".to_string(),
                code: Some("400".to_string()),
                param: None,
            },
        }
    }

    // ==================== ResponseType Tests ====================

    #[test]
    fn test_response_type_serialization() {
        let chat = ResponseType::ChatCompletion;
        let json = serde_json::to_string(&chat).unwrap();
        assert_eq!(json, "\"chat_completion\"");

        let completion = ResponseType::Completion;
        let json = serde_json::to_string(&completion).unwrap();
        assert_eq!(json, "\"completion\"");

        let embedding = ResponseType::Embedding;
        let json = serde_json::to_string(&embedding).unwrap();
        assert_eq!(json, "\"embedding\"");

        let image = ResponseType::ImageGeneration;
        let json = serde_json::to_string(&image).unwrap();
        assert_eq!(json, "\"image_generation\"");

        let audio = ResponseType::AudioTranscription;
        let json = serde_json::to_string(&audio).unwrap();
        assert_eq!(json, "\"audio_transcription\"");

        let moderation = ResponseType::Moderation;
        let json = serde_json::to_string(&moderation).unwrap();
        assert_eq!(json, "\"moderation\"");

        let rerank = ResponseType::Rerank;
        let json = serde_json::to_string(&rerank).unwrap();
        assert_eq!(json, "\"rerank\"");

        let error = ResponseType::Error;
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, "\"error\"");
    }

    #[test]
    fn test_response_type_deserialization() {
        let chat: ResponseType = serde_json::from_str("\"chat_completion\"").unwrap();
        assert!(matches!(chat, ResponseType::ChatCompletion));

        let error: ResponseType = serde_json::from_str("\"error\"").unwrap();
        assert!(matches!(error, ResponseType::Error));
    }

    // ==================== GatewayResponse Creation Tests ====================

    #[test]
    fn test_gateway_response_creation() {
        let chat_response = create_test_chat_response();

        let data = ResponseData::ChatCompletion(chat_response);
        let response = GatewayResponse::new(ResponseType::ChatCompletion, data);

        assert!(matches!(
            response.response_type,
            ResponseType::ChatCompletion
        ));
        assert!(!response.is_error());
    }

    #[test]
    fn test_error_response() {
        let error_response = create_test_error_response();

        let data = ResponseData::Error(error_response);
        let response = GatewayResponse::new(ResponseType::Error, data);

        assert!(response.is_error());
    }

    #[test]
    fn test_gateway_response_defaults() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);
        let response = GatewayResponse::new(ResponseType::ChatCompletion, data);

        // Check that metadata is initialized
        assert!(!response.metadata.id.is_nil());
    }

    // ==================== Builder Pattern Tests ====================

    #[test]
    fn test_with_provider_info() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);

        let provider_info = ProviderInfo {
            name: "openai".to_string(),
            model: "gpt-4".to_string(),
            region: Some("us-east-1".to_string()),
            ..Default::default()
        };

        let response = GatewayResponse::new(ResponseType::ChatCompletion, data)
            .with_provider_info(provider_info.clone());

        assert_eq!(response.provider_info.name, "openai");
        assert_eq!(response.provider_info.model, "gpt-4");
        assert_eq!(response.provider_info.region, Some("us-east-1".to_string()));
    }

    #[test]
    fn test_with_metrics() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);

        let metrics = ResponseMetrics {
            total_time_ms: 150,
            provider_time_ms: 100,
            queue_time_ms: 10,
            processing_time_ms: 40,
            retry_count: 0,
            from_cache: false,
            cache_type: None,
        };

        let response = GatewayResponse::new(ResponseType::ChatCompletion, data)
            .with_metrics(metrics);

        assert_eq!(response.metrics.total_time_ms, 150);
        assert_eq!(response.metrics.provider_time_ms, 100);
        assert_eq!(response.metrics.retry_count, 0);
    }

    #[test]
    fn test_with_cache_info() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);

        let cache_info = CacheInfo {
            cached: true,
            hit: true,
            cache_key: Some("cache-key-123".to_string()),
            ttl_seconds: Some(3600),
            cache_type: Some("memory".to_string()),
            similarity_score: None,
        };

        let response = GatewayResponse::new(ResponseType::ChatCompletion, data)
            .with_cache_info(cache_info);

        assert!(response.cache_info.hit);
        assert!(response.cache_info.cached);
        assert_eq!(response.cache_info.cache_key, Some("cache-key-123".to_string()));
        assert_eq!(response.cache_info.ttl_seconds, Some(3600));
    }

    #[test]
    fn test_builder_chain() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);

        let provider_info = ProviderInfo {
            name: "openai".to_string(),
            model: "gpt-4".to_string(),
            ..Default::default()
        };

        let metrics = ResponseMetrics {
            total_time_ms: 100,
            provider_time_ms: 75,
            queue_time_ms: 5,
            processing_time_ms: 20,
            retry_count: 0,
            from_cache: false,
            cache_type: None,
        };

        let cache_info = CacheInfo {
            cached: false,
            hit: false,
            cache_key: None,
            ttl_seconds: None,
            cache_type: None,
            similarity_score: None,
        };

        let response = GatewayResponse::new(ResponseType::ChatCompletion, data)
            .with_provider_info(provider_info)
            .with_metrics(metrics)
            .with_cache_info(cache_info);

        assert_eq!(response.provider_info.name, "openai");
        assert_eq!(response.metrics.total_time_ms, 100);
        assert!(!response.cache_info.hit);
    }

    // ==================== Usage Tests ====================

    #[test]
    fn test_usage_with_chat_completion() {
        let chat_response = create_test_chat_response_with_usage();
        let data = ResponseData::ChatCompletion(chat_response);
        let response = GatewayResponse::new(ResponseType::ChatCompletion, data);

        let usage = response.usage();
        assert!(usage.is_some());
        let usage = usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 20);
        assert_eq!(usage.total_tokens, 30);
    }

    #[test]
    fn test_usage_with_chat_completion_no_usage() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);
        let response = GatewayResponse::new(ResponseType::ChatCompletion, data);

        assert!(response.usage().is_none());
    }

    #[test]
    fn test_usage_with_error_response() {
        let error_response = create_test_error_response();
        let data = ResponseData::Error(error_response);
        let response = GatewayResponse::new(ResponseType::Error, data);

        assert!(response.usage().is_none());
    }

    // ==================== is_error Tests ====================

    #[test]
    fn test_is_error_for_chat_completion() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);
        let response = GatewayResponse::new(ResponseType::ChatCompletion, data);

        assert!(!response.is_error());
    }

    #[test]
    fn test_is_error_for_error_response() {
        let error_response = create_test_error_response();
        let data = ResponseData::Error(error_response);
        let response = GatewayResponse::new(ResponseType::Error, data);

        assert!(response.is_error());
    }

    // ==================== ResponseData Serialization Tests ====================

    #[test]
    fn test_response_data_chat_completion_serialization() {
        let chat_response = create_test_chat_response();
        let data = ResponseData::ChatCompletion(chat_response);

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"type\":\"chat_completion\""));
    }

    #[test]
    fn test_response_data_error_serialization() {
        let error_response = create_test_error_response();
        let data = ResponseData::Error(error_response);

        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"type\":\"error\""));
        assert!(json.contains("Test error"));
    }
}
