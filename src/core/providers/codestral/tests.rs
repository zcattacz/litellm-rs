//! Tests for Codestral provider

use super::*;

#[test]
fn test_config_default() {
    let config = CodestralConfig::default();
    assert!(config.api_key.is_none());
    assert_eq!(config.timeout, 60);
}

#[test]
fn test_get_api_base_default() {
    let config = CodestralConfig::default();
    assert_eq!(config.get_api_base(), "https://codestral.mistral.ai/v1");
}

#[test]
fn test_model_info() {
    let info = get_model_info("codestral-latest");
    assert!(info.is_some());
    assert!(info.unwrap().supports_fim);
}

// Note: Error conversion tests removed - CodestralError is now a type alias to ProviderError

#[test]
fn test_fim_request_serialization() {
    use super::provider::FimRequest;
    let request = FimRequest {
        model: "codestral-latest".to_string(),
        prompt: "def hello():".to_string(),
        suffix: Some("    return greeting".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(100),
        stop: None,
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["model"], "codestral-latest");
    assert_eq!(json["prompt"], "def hello():");
}
