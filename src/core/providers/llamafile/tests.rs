//! Unit tests for Llamafile provider

use super::*;
use crate::core::types::requests::{ChatMessage, ChatRequest, MessageContent, MessageRole};

#[test]
fn test_llamafile_provider_name() {
    // Test that the provider name is correct
    let config = LlamafileConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_llamafile_config_creation() {
    let config = LlamafileConfig {
        api_key: Some("test-key".to_string()),
        api_base: Some("http://localhost:8080/v1".to_string()),
        timeout: 60,
        max_retries: 2,
        debug: true,
    };

    assert_eq!(config.api_key, Some("test-key".to_string()));
    assert_eq!(config.api_base, Some("http://localhost:8080/v1".to_string()));
    assert_eq!(config.timeout, 60);
    assert_eq!(config.max_retries, 2);
    assert!(config.debug);
}

#[test]
fn test_llamafile_config_default_values() {
    let config = LlamafileConfig::default();

    assert!(config.api_key.is_none());
    assert!(config.api_base.is_none());
    assert_eq!(config.timeout, 120);
    assert_eq!(config.max_retries, 3);
    assert!(!config.debug);
}

#[test]
fn test_llamafile_config_validation() {
    // Valid config
    let config = LlamafileConfig::default();
    assert!(config.validate().is_ok());

    // Invalid: zero timeout
    let config = LlamafileConfig {
        timeout: 0,
        ..Default::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_llamafile_error_types() {
    use crate::core::types::errors::ProviderErrorTrait;

    let api_error = LlamafileError::ApiError("test".to_string());
    assert_eq!(api_error.error_type(), "api_error");
    assert!(!api_error.is_retryable());

    let network_error = LlamafileError::NetworkError("test".to_string());
    assert_eq!(network_error.error_type(), "network_error");
    assert!(network_error.is_retryable());

    let timeout_error = LlamafileError::TimeoutError("test".to_string());
    assert_eq!(timeout_error.error_type(), "timeout_error");
    assert!(timeout_error.is_retryable());
    assert_eq!(timeout_error.retry_delay(), Some(10));
}

#[test]
fn test_llamafile_error_conversion() {
    use crate::core::providers::unified_provider::ProviderError;

    let llamafile_error = LlamafileError::AuthenticationError("invalid key".to_string());
    let provider_error: ProviderError = llamafile_error.into();

    assert!(matches!(provider_error, ProviderError::Authentication { .. }));
}

#[test]
fn test_llamafile_error_mapper() {
    use crate::core::traits::error_mapper::trait_def::ErrorMapper;

    let mapper = LlamafileErrorMapper;

    // Test 400 error
    let error = mapper.map_http_error(400, "Bad request");
    assert!(matches!(error, LlamafileError::InvalidRequestError(_)));

    // Test 401 error
    let error = mapper.map_http_error(401, "Unauthorized");
    assert!(matches!(error, LlamafileError::AuthenticationError(_)));

    // Test 404 error
    let error = mapper.map_http_error(404, "Not found");
    assert!(matches!(error, LlamafileError::ModelNotFoundError(_)));

    // Test 503 error
    let error = mapper.map_http_error(503, "Service unavailable");
    assert!(matches!(error, LlamafileError::ServiceUnavailableError(_)));

    // Test pattern matching for model not found
    let error = mapper.map_http_error(400, "model 'llama' not found");
    assert!(matches!(error, LlamafileError::ModelNotFoundError(_)));
}

#[test]
fn test_llamafile_config_endpoints() {
    let config = LlamafileConfig::default();

    assert_eq!(
        config.get_chat_endpoint(),
        "http://127.0.0.1:8080/v1/chat/completions"
    );
    assert_eq!(
        config.get_completions_endpoint(),
        "http://127.0.0.1:8080/v1/completions"
    );
    assert_eq!(
        config.get_models_endpoint(),
        "http://127.0.0.1:8080/v1/models"
    );
}

#[test]
fn test_llamafile_config_endpoints_custom() {
    let config = LlamafileConfig {
        api_base: Some("http://192.168.1.100:9000/v1".to_string()),
        ..Default::default()
    };

    assert_eq!(
        config.get_chat_endpoint(),
        "http://192.168.1.100:9000/v1/chat/completions"
    );
}

#[test]
fn test_llamafile_config_endpoints_with_trailing_slash() {
    let config = LlamafileConfig {
        api_base: Some("http://localhost:8080/v1/".to_string()),
        ..Default::default()
    };

    assert_eq!(
        config.get_chat_endpoint(),
        "http://localhost:8080/v1/chat/completions"
    );
}

#[test]
fn test_llamafile_config_get_api_key() {
    // Test with explicit key
    let config = LlamafileConfig {
        api_key: Some("my-key".to_string()),
        ..Default::default()
    };
    assert_eq!(config.get_api_key(), "my-key");

    // Test without key (returns fake-api-key)
    let config = LlamafileConfig::default();
    assert_eq!(config.get_api_key(), "fake-api-key");
}

#[test]
fn test_llamafile_capabilities() {
    // Test that capabilities constant is properly defined
    assert!(LLAMAFILE_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
    assert!(LLAMAFILE_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
    // Llamafile doesn't support embeddings in basic mode
    assert!(!LLAMAFILE_CAPABILITIES.contains(&ProviderCapability::Embeddings));
}

#[tokio::test]
async fn test_llamafile_provider_creation() {
    let config = LlamafileConfig::default();

    let provider = LlamafileProvider::new(config).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "llamafile");
    assert_eq!(provider.capabilities(), LLAMAFILE_CAPABILITIES);
}

#[tokio::test]
async fn test_llamafile_provider_default_local() {
    let provider = LlamafileProvider::default_local().await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(
        provider.config.get_api_base(),
        "http://127.0.0.1:8080/v1"
    );
}

#[tokio::test]
async fn test_llamafile_provider_with_base_url() {
    let provider = LlamafileProvider::with_base_url("http://192.168.1.100:8080/v1").await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(
        provider.config.api_base,
        Some("http://192.168.1.100:8080/v1".to_string())
    );
}

#[tokio::test]
async fn test_llamafile_build_chat_request() {
    let config = LlamafileConfig::default();

    let provider = LlamafileProvider::new(config).await.unwrap();

    let request = ChatRequest {
        model: "llamafile/llama-7b".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant.".to_string(),
                )),
                thinking: None,
                tool_calls: None,
                function_call: None,
                name: None,
                refusal: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello!".to_string())),
                thinking: None,
                tool_calls: None,
                function_call: None,
                name: None,
                refusal: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
        ..Default::default()
    };

    let body = provider.build_chat_request(&request, false).unwrap();

    assert_eq!(body["model"], "llama-7b");
    assert_eq!(body["stream"], false);
    assert_eq!(body["temperature"], 0.7);
    assert_eq!(body["max_tokens"], 100);

    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[0]["content"], "You are a helpful assistant.");
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[1]["content"], "Hello!");
}

#[tokio::test]
async fn test_llamafile_parse_chat_response() {
    let config = LlamafileConfig::default();

    let provider = LlamafileProvider::new(config).await.unwrap();

    let response_json = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "llama-7b",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 8,
            "total_tokens": 18
        }
    });

    let response = provider
        .parse_chat_response(response_json, "llama-7b")
        .unwrap();

    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.model, "llamafile/llama-7b");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.role, MessageRole::Assistant);

    if let Some(MessageContent::Text(content)) = &response.choices[0].message.content {
        assert_eq!(content, "Hello! How can I help you?");
    } else {
        panic!("Expected text content");
    }

    let usage = response.usage.unwrap();
    assert_eq!(usage.prompt_tokens, 10);
    assert_eq!(usage.completion_tokens, 8);
    assert_eq!(usage.total_tokens, 18);
}

#[tokio::test]
async fn test_llamafile_map_openai_params() {
    let config = LlamafileConfig::default();

    let provider = LlamafileProvider::new(config).await.unwrap();

    let mut params = std::collections::HashMap::new();
    params.insert("temperature".to_string(), serde_json::json!(0.7));
    params.insert("max_tokens".to_string(), serde_json::json!(100));

    let mapped = provider
        .map_openai_params(params.clone(), "llama-7b")
        .await
        .unwrap();

    // Llamafile uses OpenAI-compatible API, so params should pass through unchanged
    assert_eq!(mapped["temperature"], serde_json::json!(0.7));
    assert_eq!(mapped["max_tokens"], serde_json::json!(100));
}

#[tokio::test]
async fn test_llamafile_calculate_cost() {
    let config = LlamafileConfig::default();

    let provider = LlamafileProvider::new(config).await.unwrap();

    // Llamafile is free, cost should always be 0
    let cost = provider
        .calculate_cost("llama-7b", 1000, 500)
        .await
        .unwrap();
    assert_eq!(cost, 0.0);
}

#[tokio::test]
async fn test_llamafile_get_supported_params() {
    let config = LlamafileConfig::default();

    let provider = LlamafileProvider::new(config).await.unwrap();

    let params = provider.get_supported_openai_params("llama-7b");

    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"seed"));
}

#[test]
fn test_llamafile_error_http_status_codes() {
    use crate::core::types::errors::ProviderErrorTrait;

    assert_eq!(
        LlamafileError::AuthenticationError("".to_string()).http_status(),
        401
    );
    assert_eq!(
        LlamafileError::InvalidRequestError("".to_string()).http_status(),
        400
    );
    assert_eq!(
        LlamafileError::ModelNotFoundError("".to_string()).http_status(),
        404
    );
    assert_eq!(
        LlamafileError::ServiceUnavailableError("".to_string()).http_status(),
        503
    );
    assert_eq!(
        LlamafileError::TimeoutError("".to_string()).http_status(),
        504
    );
    assert_eq!(LlamafileError::ApiError("".to_string()).http_status(), 500);
    assert_eq!(
        LlamafileError::ConnectionRefusedError("".to_string()).http_status(),
        503
    );
    assert_eq!(
        LlamafileError::ContextLengthExceeded {
            max: 4096,
            actual: 5000
        }
        .http_status(),
        400
    );
}
