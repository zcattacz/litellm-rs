//! Integration tests for GitHub Models provider

use super::*;
use crate::core::types::requests::{ChatMessage, MessageContent, MessageRole};

#[tokio::test]
async fn test_github_provider_new_without_key() {
    let config = GitHubConfig::default();
    // Clear env var for this test
    std::env::remove_var("GITHUB_TOKEN");
    let provider = GitHubProvider::new(config).await;
    assert!(provider.is_err());
}

#[tokio::test]
async fn test_github_config_from_env() {
    std::env::set_var("GITHUB_TOKEN", "ghp_test_env_key");
    let config = GitHubConfig::default();
    assert_eq!(config.get_api_key(), Some("ghp_test_env_key".to_string()));
    std::env::remove_var("GITHUB_TOKEN");
}

#[tokio::test]
async fn test_github_provider_transform_request() {
    let provider = GitHubProvider::with_api_key("ghp_test123").await.unwrap();

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
async fn test_github_provider_map_params() {
    let provider = GitHubProvider::with_api_key("ghp_test123").await.unwrap();

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
fn test_github_error_conversions() {
    // Test all error types convert correctly
    let errors = vec![
        GitHubError::ApiError("api error".to_string()),
        GitHubError::AuthenticationError("auth error".to_string()),
        GitHubError::RateLimitError("rate limit".to_string()),
        GitHubError::InvalidRequestError("invalid".to_string()),
        GitHubError::ModelNotFoundError("model not found".to_string()),
        GitHubError::ServiceUnavailableError("unavailable".to_string()),
        GitHubError::StreamingError("stream error".to_string()),
        GitHubError::ConfigurationError("config error".to_string()),
        GitHubError::NetworkError("network error".to_string()),
        GitHubError::UnknownError("unknown".to_string()),
    ];

    for error in errors {
        let provider_error: crate::core::providers::unified_provider::ProviderError = error.into();
        // Just ensure the conversion doesn't panic
        let _ = format!("{:?}", provider_error);
    }
}

#[test]
fn test_github_model_info_completeness() {
    // Ensure all models have required fields populated
    for model_id in get_available_models() {
        let info = get_model_info(model_id).unwrap();
        assert!(!info.model_id.is_empty());
        assert!(!info.display_name.is_empty());
        assert!(info.context_length > 0);
        assert!(info.max_output_tokens > 0);
        // Costs can be zero for free models
        assert!(info.input_cost_per_million >= 0.0);
        assert!(info.output_cost_per_million >= 0.0);
    }
}

#[tokio::test]
async fn test_github_provider_embeddings_not_supported() {
    let provider = GitHubProvider::with_api_key("ghp_test123").await.unwrap();

    let request = crate::core::types::requests::EmbeddingRequest {
        model: "text-embedding-ada-002".to_string(),
        input: crate::core::types::requests::EmbeddingInput::Single("test".to_string()),
        encoding_format: None,
        user: None,
        dimensions: None,
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

    let result = provider.embeddings(request, context).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        GitHubError::InvalidRequestError(_)
    ));
}

#[test]
fn test_github_config_serialization_roundtrip() {
    let config = GitHubConfig {
        api_key: Some("ghp_test123".to_string()),
        api_base: Some("https://custom.api.com".to_string()),
        timeout: 45,
        max_retries: 5,
        debug: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: GitHubConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.api_key, deserialized.api_key);
    assert_eq!(config.api_base, deserialized.api_base);
    assert_eq!(config.timeout, deserialized.timeout);
    assert_eq!(config.max_retries, deserialized.max_retries);
    assert_eq!(config.debug, deserialized.debug);
}
