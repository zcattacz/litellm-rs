//! Unit tests for Oobabooga provider

use super::*;
use crate::core::types::requests::{ChatMessage, ChatRequest, MessageContent, MessageRole};

#[test]
fn test_oobabooga_provider_name() {
    // Test that the provider name is correct
    let config = OobaboogaConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_oobabooga_config_creation() {
    let config = OobaboogaConfig {
        api_key: Some("test-token".to_string()),
        api_base: Some("http://localhost:5000".to_string()),
        timeout: 60,
        max_retries: 2,
        debug: true,
    };

    assert_eq!(config.api_key, Some("test-token".to_string()));
    assert_eq!(config.api_base, Some("http://localhost:5000".to_string()));
    assert_eq!(config.timeout, 60);
    assert_eq!(config.max_retries, 2);
    assert!(config.debug);
}

#[test]
fn test_oobabooga_config_default_values() {
    let config = OobaboogaConfig::default();

    assert!(config.api_key.is_none());
    assert!(config.api_base.is_none());
    assert_eq!(config.timeout, 120);
    assert_eq!(config.max_retries, 3);
    assert!(!config.debug);
}

#[test]
fn test_oobabooga_config_validation() {
    // Valid config
    let config = OobaboogaConfig::default();
    assert!(config.validate().is_ok());

    // Invalid: zero timeout
    let config = OobaboogaConfig {
        timeout: 0,
        ..Default::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_oobabooga_error_types() {
    use crate::core::types::errors::ProviderErrorTrait;

    let api_error = OobaboogaError::ApiError("test".to_string());
    assert_eq!(api_error.error_type(), "api_error");
    assert!(!api_error.is_retryable());

    let network_error = OobaboogaError::NetworkError("test".to_string());
    assert_eq!(network_error.error_type(), "network_error");
    assert!(network_error.is_retryable());

    let timeout_error = OobaboogaError::TimeoutError("test".to_string());
    assert_eq!(timeout_error.error_type(), "timeout_error");
    assert!(timeout_error.is_retryable());
    assert_eq!(timeout_error.retry_delay(), Some(10));
}

#[test]
fn test_oobabooga_error_conversion() {
    use crate::core::providers::unified_provider::ProviderError;

    let oobabooga_error = OobaboogaError::AuthenticationError("invalid token".to_string());
    let provider_error: ProviderError = oobabooga_error.into();

    assert!(matches!(provider_error, ProviderError::Authentication { .. }));
}

#[test]
fn test_oobabooga_error_mapper() {
    use crate::core::traits::error_mapper::trait_def::ErrorMapper;

    let mapper = OobaboogaErrorMapper;

    // Test 400 error
    let error = mapper.map_http_error(400, "Bad request");
    assert!(matches!(error, OobaboogaError::InvalidRequestError(_)));

    // Test 401 error
    let error = mapper.map_http_error(401, "Unauthorized");
    assert!(matches!(error, OobaboogaError::AuthenticationError(_)));

    // Test 404 error
    let error = mapper.map_http_error(404, "Not found");
    assert!(matches!(error, OobaboogaError::ModelNotFoundError(_)));

    // Test 503 error
    let error = mapper.map_http_error(503, "Service unavailable");
    assert!(matches!(error, OobaboogaError::ServiceUnavailableError(_)));

    // Test pattern matching for model not found
    let error = mapper.map_http_error(400, "model 'llama' not found");
    assert!(matches!(error, OobaboogaError::ModelNotFoundError(_)));
}

#[test]
fn test_oobabooga_config_endpoints() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    assert_eq!(
        config.get_chat_endpoint().unwrap(),
        "http://localhost:5000/v1/chat/completions"
    );
    assert_eq!(
        config.get_embeddings_endpoint().unwrap(),
        "http://localhost:5000/v1/embeddings"
    );
    assert_eq!(
        config.get_models_endpoint().unwrap(),
        "http://localhost:5000/v1/models"
    );
}

#[test]
fn test_oobabooga_config_endpoints_with_trailing_slash() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000/".to_string()),
        ..Default::default()
    };

    assert_eq!(
        config.get_chat_endpoint().unwrap(),
        "http://localhost:5000/v1/chat/completions"
    );
}

#[test]
fn test_oobabooga_config_build_auth_headers() {
    // Without API key
    let config = OobaboogaConfig::default();
    let headers = config.build_auth_headers();
    assert!(headers
        .iter()
        .any(|(k, v)| k == "accept" && v == "application/json"));
    assert!(headers
        .iter()
        .any(|(k, v)| k == "content-type" && v == "application/json"));

    // With API key
    let config = OobaboogaConfig {
        api_key: Some("my-token".to_string()),
        ..Default::default()
    };
    let headers = config.build_auth_headers();
    assert!(headers
        .iter()
        .any(|(k, v)| k == "Authorization" && v == "Token my-token"));
}

#[test]
fn test_oobabooga_capabilities() {
    // Test that capabilities constant is properly defined
    assert!(OOBABOOGA_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
    assert!(OOBABOOGA_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
    assert!(OOBABOOGA_CAPABILITIES.contains(&ProviderCapability::Embeddings));
}

#[tokio::test]
async fn test_oobabooga_provider_creation() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    let provider = OobaboogaProvider::new(config).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "oobabooga");
    assert_eq!(provider.capabilities(), OOBABOOGA_CAPABILITIES);
}

#[tokio::test]
async fn test_oobabooga_provider_with_base_url() {
    let provider = OobaboogaProvider::with_base_url("http://192.168.1.100:5000").await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(
        provider.config.api_base,
        Some("http://192.168.1.100:5000".to_string())
    );
}

#[tokio::test]
async fn test_oobabooga_build_chat_request() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    let provider = OobaboogaProvider::new(config).await.unwrap();

    let request = ChatRequest {
        model: "oobabooga/llama-7b".to_string(),
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
async fn test_oobabooga_parse_chat_response() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    let provider = OobaboogaProvider::new(config).await.unwrap();

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
    assert_eq!(response.model, "oobabooga/llama-7b");
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
async fn test_oobabooga_parse_chat_response_with_error() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    let provider = OobaboogaProvider::new(config).await.unwrap();

    let response_json = serde_json::json!({
        "error": "Model not loaded"
    });

    let result = provider.parse_chat_response(response_json, "llama-7b");
    assert!(result.is_err());
    if let Err(OobaboogaError::ApiError(msg)) = result {
        assert!(msg.contains("Model not loaded"));
    } else {
        panic!("Expected ApiError");
    }
}

#[tokio::test]
async fn test_oobabooga_map_openai_params() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    let provider = OobaboogaProvider::new(config).await.unwrap();

    let mut params = std::collections::HashMap::new();
    params.insert("temperature".to_string(), serde_json::json!(0.7));
    params.insert("max_tokens".to_string(), serde_json::json!(100));

    let mapped = provider
        .map_openai_params(params.clone(), "llama-7b")
        .await
        .unwrap();

    // Oobabooga uses OpenAI-compatible API, so params should pass through unchanged
    assert_eq!(mapped["temperature"], serde_json::json!(0.7));
    assert_eq!(mapped["max_tokens"], serde_json::json!(100));
}

#[tokio::test]
async fn test_oobabooga_calculate_cost() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    let provider = OobaboogaProvider::new(config).await.unwrap();

    // Oobabooga is free, cost should always be 0
    let cost = provider
        .calculate_cost("llama-7b", 1000, 500)
        .await
        .unwrap();
    assert_eq!(cost, 0.0);
}

#[tokio::test]
async fn test_oobabooga_get_supported_params() {
    let config = OobaboogaConfig {
        api_base: Some("http://localhost:5000".to_string()),
        ..Default::default()
    };

    let provider = OobaboogaProvider::new(config).await.unwrap();

    let params = provider.get_supported_openai_params("llama-7b");

    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"seed"));
}

#[test]
fn test_oobabooga_error_http_status_codes() {
    use crate::core::types::errors::ProviderErrorTrait;

    assert_eq!(
        OobaboogaError::AuthenticationError("".to_string()).http_status(),
        401
    );
    assert_eq!(
        OobaboogaError::InvalidRequestError("".to_string()).http_status(),
        400
    );
    assert_eq!(
        OobaboogaError::ModelNotFoundError("".to_string()).http_status(),
        404
    );
    assert_eq!(
        OobaboogaError::ServiceUnavailableError("".to_string()).http_status(),
        503
    );
    assert_eq!(
        OobaboogaError::TimeoutError("".to_string()).http_status(),
        504
    );
    assert_eq!(OobaboogaError::ApiError("".to_string()).http_status(), 500);
    assert_eq!(
        OobaboogaError::ConnectionRefusedError("".to_string()).http_status(),
        503
    );
    assert_eq!(
        OobaboogaError::ContextLengthExceeded {
            max: 4096,
            actual: 5000
        }
        .http_status(),
        400
    );
}
