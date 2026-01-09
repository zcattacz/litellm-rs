//! Integration tests for GitHub Copilot provider

use super::*;
use crate::core::types::requests::{ChatMessage, MessageContent, MessageRole};

#[tokio::test]
async fn test_github_copilot_provider_creation_default() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await;
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_github_copilot_config_from_env() {
    std::env::set_var("GITHUB_COPILOT_TOKEN_DIR", "/custom/path");
    let config = GitHubCopilotConfig::default();
    assert_eq!(config.get_token_dir(), "/custom/path");
    std::env::remove_var("GITHUB_COPILOT_TOKEN_DIR");
}

#[tokio::test]
async fn test_github_copilot_provider_transform_request() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await.unwrap();

    let request = crate::core::types::requests::ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            audio: None,
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
        top_p: None,
        n: None,
        stream: false,
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        seed: None,
        tools: None,
        tool_choice: None,
        parallel_tool_calls: None,
        response_format: None,
        logprobs: None,
        top_logprobs: None,
        max_completion_tokens: None,
        modalities: None,
        prediction: None,
        audio: None,
        service_tier: None,
        store: None,
        reasoning_effort: None,
        metadata: None,
        stream_options: None,
        extra: None,
    };

    let context = crate::core::types::common::RequestContext {
        request_id: "test-123".to_string(),
        trace_id: None,
        span_id: None,
        client_id: None,
        user_id: None,
        start_time: std::time::Instant::now(),
        metadata: std::collections::HashMap::new(),
    };

    let result = provider.transform_request(request, context).await;
    assert!(result.is_ok());

    let json = result.unwrap();
    assert_eq!(json["model"], "gpt-4o");
    assert_eq!(json["temperature"], 0.7);
    assert_eq!(json["max_tokens"], 100);
}

#[tokio::test]
async fn test_github_copilot_provider_map_params() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await.unwrap();

    let mut params = std::collections::HashMap::new();
    params.insert(
        "temperature".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(0.8).unwrap()),
    );
    params.insert(
        "max_tokens".to_string(),
        serde_json::Value::Number(serde_json::Number::from(200)),
    );

    let result = provider.map_openai_params(params.clone(), "gpt-4o").await;
    assert!(result.is_ok());

    let mapped = result.unwrap();
    assert_eq!(mapped["temperature"], params["temperature"]);
    assert_eq!(mapped["max_tokens"], params["max_tokens"]);
}

#[test]
fn test_github_copilot_error_conversions() {
    let errors = vec![
        GitHubCopilotError::ApiError("api error".to_string()),
        GitHubCopilotError::AuthenticationError("auth error".to_string()),
        GitHubCopilotError::RateLimitError("rate limit".to_string()),
        GitHubCopilotError::DeviceCodeError("device error".to_string()),
        GitHubCopilotError::AccessTokenError("token error".to_string()),
        GitHubCopilotError::ApiKeyExpiredError("expired".to_string()),
        GitHubCopilotError::RefreshApiKeyError("refresh error".to_string()),
    ];

    for error in errors {
        let provider_error: crate::core::providers::unified_provider::ProviderError = error.into();
        // Just ensure the conversion doesn't panic
        let _ = format!("{:?}", provider_error);
    }
}

#[test]
fn test_github_copilot_model_info_completeness() {
    // Ensure all models have required fields populated
    for model_id in get_available_models() {
        let info = get_model_info(model_id).unwrap();
        assert!(!info.model_id.is_empty());
        assert!(!info.display_name.is_empty());
        assert!(info.context_length > 0);
        assert!(info.max_output_tokens > 0);
    }
}

#[test]
fn test_github_copilot_config_serialization_roundtrip() {
    let config = GitHubCopilotConfig {
        token_dir: Some("/custom/path".to_string()),
        api_base: Some("https://custom.api.com".to_string()),
        timeout: 45,
        max_retries: 5,
        disable_system_to_assistant: true,
        debug: true,
        ..Default::default()
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: GitHubCopilotConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.token_dir, deserialized.token_dir);
    assert_eq!(config.api_base, deserialized.api_base);
    assert_eq!(config.timeout, deserialized.timeout);
    assert_eq!(config.max_retries, deserialized.max_retries);
    assert_eq!(
        config.disable_system_to_assistant,
        deserialized.disable_system_to_assistant
    );
    assert_eq!(config.debug, deserialized.debug);
}

#[test]
fn test_message_transformation() {
    let config = GitHubCopilotConfig::default();
    let authenticator = CopilotAuthenticator::new(&config);
    let provider = GitHubCopilotProvider {
        config,
        authenticator,
        models: vec![],
        cached_api_key: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        cached_api_base: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
    };

    let mut messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: Some(MessageContent::Text("You are helpful".to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            audio: None,
        },
        ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            audio: None,
        },
    ];

    provider.transform_messages(&mut messages);

    // System message should be converted to assistant
    assert_eq!(messages[0].role, MessageRole::Assistant);
    assert_eq!(messages[1].role, MessageRole::User);
}

#[test]
fn test_message_transformation_disabled() {
    let config = GitHubCopilotConfig {
        disable_system_to_assistant: true,
        ..Default::default()
    };
    let authenticator = CopilotAuthenticator::new(&config);
    let provider = GitHubCopilotProvider {
        config,
        authenticator,
        models: vec![],
        cached_api_key: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        cached_api_base: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
    };

    let mut messages = vec![ChatMessage {
        role: MessageRole::System,
        content: Some(MessageContent::Text("You are helpful".to_string())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        refusal: None,
        audio: None,
    }];

    provider.transform_messages(&mut messages);

    // System message should NOT be converted
    assert_eq!(messages[0].role, MessageRole::System);
}

#[test]
fn test_has_vision_content() {
    let config = GitHubCopilotConfig::default();
    let authenticator = CopilotAuthenticator::new(&config);
    let provider = GitHubCopilotProvider {
        config,
        authenticator,
        models: vec![],
        cached_api_key: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        cached_api_base: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
    };

    // Text only message
    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text("Hello".to_string())),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        refusal: None,
        audio: None,
    }];
    assert!(!provider.has_vision_content(&messages));

    // Message with image
    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Array(vec![
            crate::core::types::requests::ContentPart::Text {
                text: "What's in this image?".to_string(),
            },
            crate::core::types::requests::ContentPart::ImageUrl {
                image_url: crate::core::types::requests::ImageUrl {
                    url: "https://example.com/image.png".to_string(),
                    detail: None,
                },
            },
        ])),
        name: None,
        tool_calls: None,
        tool_call_id: None,
        refusal: None,
        audio: None,
    }];
    assert!(provider.has_vision_content(&messages));
}

#[test]
fn test_authenticator_creation() {
    let config = GitHubCopilotConfig::default();
    let auth = CopilotAuthenticator::new(&config);

    // Authenticator should be created successfully
    let _ = format!("{:?}", auth);
}

#[test]
fn test_api_key_info_serialization() {
    use super::authenticator::ApiKeyInfo;

    let info = ApiKeyInfo {
        token: "test-token".to_string(),
        expires_at: 1234567890,
        endpoints: super::authenticator::Endpoints {
            api: Some("https://api.example.com".to_string()),
        },
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: ApiKeyInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(info.token, deserialized.token);
    assert_eq!(info.expires_at, deserialized.expires_at);
    assert_eq!(info.endpoints.api, deserialized.endpoints.api);
}
