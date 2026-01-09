//! Legacy fallback configuration for load balancer
//!
//! **DEPRECATED**: This module is part of the legacy load balancer system.
//! For new code, use `crate::core::router::fallback::FallbackConfig` instead,
//! which provides a more ergonomic builder pattern and thread-safe DashMap storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for error-specific fallbacks
///
/// **DEPRECATED**: Use `crate::core::router::fallback::FallbackConfig` for new code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FallbackConfig {
    /// General fallbacks for any error (model -> fallback models)
    #[serde(default)]
    pub general_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for content policy violations
    #[serde(default)]
    pub content_policy_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for context window exceeded errors
    #[serde(default)]
    pub context_window_fallbacks: HashMap<String, Vec<String>>,
    /// Fallbacks for rate limit errors
    #[serde(default)]
    pub rate_limit_fallbacks: HashMap<String, Vec<String>>,
}

impl FallbackConfig {
    /// Create a new fallback config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a general fallback
    pub fn add_general_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.general_fallbacks.insert(model.to_string(), fallbacks);
        self
    }

    /// Add a content policy fallback
    pub fn add_content_policy_fallback(
        &mut self,
        model: &str,
        fallbacks: Vec<String>,
    ) -> &mut Self {
        self.content_policy_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add a context window fallback
    pub fn add_context_window_fallback(
        &mut self,
        model: &str,
        fallbacks: Vec<String>,
    ) -> &mut Self {
        self.context_window_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }

    /// Add a rate limit fallback
    pub fn add_rate_limit_fallback(&mut self, model: &str, fallbacks: Vec<String>) -> &mut Self {
        self.rate_limit_fallbacks
            .insert(model.to_string(), fallbacks);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Default Tests ====================

    #[test]
    fn test_fallback_config_default() {
        let config = FallbackConfig::default();
        assert!(config.general_fallbacks.is_empty());
        assert!(config.content_policy_fallbacks.is_empty());
        assert!(config.context_window_fallbacks.is_empty());
        assert!(config.rate_limit_fallbacks.is_empty());
    }

    #[test]
    fn test_fallback_config_new() {
        let config = FallbackConfig::new();
        assert!(config.general_fallbacks.is_empty());
    }

    // ==================== General Fallback Tests ====================

    #[test]
    fn test_add_general_fallback() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        assert_eq!(config.general_fallbacks.len(), 1);
        assert_eq!(
            config.general_fallbacks.get("gpt-4"),
            Some(&vec!["gpt-3.5-turbo".to_string()])
        );
    }

    #[test]
    fn test_add_general_fallback_multiple_fallbacks() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback(
            "gpt-4",
            vec!["gpt-4-turbo".to_string(), "gpt-3.5-turbo".to_string()],
        );

        let fallbacks = config.general_fallbacks.get("gpt-4").unwrap();
        assert_eq!(fallbacks.len(), 2);
    }

    #[test]
    fn test_add_general_fallback_chaining() {
        let mut config = FallbackConfig::new();
        config
            .add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()])
            .add_general_fallback("claude-3", vec!["claude-2".to_string()]);

        assert_eq!(config.general_fallbacks.len(), 2);
    }

    #[test]
    fn test_add_general_fallback_override() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["old-model".to_string()]);
        config.add_general_fallback("gpt-4", vec!["new-model".to_string()]);

        assert_eq!(
            config.general_fallbacks.get("gpt-4"),
            Some(&vec!["new-model".to_string()])
        );
    }

    // ==================== Content Policy Fallback Tests ====================

    #[test]
    fn test_add_content_policy_fallback() {
        let mut config = FallbackConfig::new();
        config.add_content_policy_fallback("gpt-4", vec!["claude-3".to_string()]);

        assert_eq!(config.content_policy_fallbacks.len(), 1);
        assert_eq!(
            config.content_policy_fallbacks.get("gpt-4"),
            Some(&vec!["claude-3".to_string()])
        );
    }

    #[test]
    fn test_add_content_policy_fallback_chaining() {
        let mut config = FallbackConfig::new();
        config
            .add_content_policy_fallback("gpt-4", vec!["claude-3".to_string()])
            .add_content_policy_fallback("gemini", vec!["llama".to_string()]);

        assert_eq!(config.content_policy_fallbacks.len(), 2);
    }

    // ==================== Context Window Fallback Tests ====================

    #[test]
    fn test_add_context_window_fallback() {
        let mut config = FallbackConfig::new();
        config.add_context_window_fallback("gpt-4", vec!["gpt-4-32k".to_string()]);

        assert_eq!(config.context_window_fallbacks.len(), 1);
        assert_eq!(
            config.context_window_fallbacks.get("gpt-4"),
            Some(&vec!["gpt-4-32k".to_string()])
        );
    }

    #[test]
    fn test_add_context_window_fallback_to_larger_models() {
        let mut config = FallbackConfig::new();
        config.add_context_window_fallback(
            "gpt-4",
            vec!["gpt-4-turbo".to_string(), "claude-3-opus".to_string()],
        );

        let fallbacks = config.context_window_fallbacks.get("gpt-4").unwrap();
        assert_eq!(fallbacks.len(), 2);
    }

    // ==================== Rate Limit Fallback Tests ====================

    #[test]
    fn test_add_rate_limit_fallback() {
        let mut config = FallbackConfig::new();
        config.add_rate_limit_fallback("gpt-4", vec!["gpt-4-backup".to_string()]);

        assert_eq!(config.rate_limit_fallbacks.len(), 1);
        assert_eq!(
            config.rate_limit_fallbacks.get("gpt-4"),
            Some(&vec!["gpt-4-backup".to_string()])
        );
    }

    #[test]
    fn test_add_rate_limit_fallback_multiple_providers() {
        let mut config = FallbackConfig::new();
        config.add_rate_limit_fallback(
            "gpt-4",
            vec!["azure-gpt-4".to_string(), "anthropic-claude".to_string()],
        );

        let fallbacks = config.rate_limit_fallbacks.get("gpt-4").unwrap();
        assert_eq!(fallbacks.len(), 2);
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_fallback_config_serialization() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec!["gpt-3.5-turbo".to_string()]);

        let json = serde_json::to_value(&config).unwrap();
        assert!(json["general_fallbacks"]["gpt-4"].is_array());
    }

    #[test]
    fn test_fallback_config_deserialization() {
        let json = r#"{
            "general_fallbacks": {"model-a": ["model-b"]},
            "content_policy_fallbacks": {},
            "context_window_fallbacks": {},
            "rate_limit_fallbacks": {}
        }"#;

        let config: FallbackConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.general_fallbacks.len(), 1);
        assert!(config.general_fallbacks.contains_key("model-a"));
    }

    #[test]
    fn test_fallback_config_deserialization_missing_fields() {
        let json = r#"{}"#;
        let config: FallbackConfig = serde_json::from_str(json).unwrap();

        assert!(config.general_fallbacks.is_empty());
        assert!(config.content_policy_fallbacks.is_empty());
    }

    #[test]
    fn test_fallback_config_roundtrip() {
        let mut original = FallbackConfig::new();
        original.add_general_fallback("gpt-4", vec!["backup".to_string()]);
        original.add_rate_limit_fallback("claude", vec!["claude-backup".to_string()]);

        let json = serde_json::to_string(&original).unwrap();
        let restored: FallbackConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(original.general_fallbacks, restored.general_fallbacks);
        assert_eq!(original.rate_limit_fallbacks, restored.rate_limit_fallbacks);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_fallback_config_clone() {
        let mut original = FallbackConfig::new();
        original.add_general_fallback("gpt-4", vec!["backup".to_string()]);

        let cloned = original.clone();
        assert_eq!(original.general_fallbacks, cloned.general_fallbacks);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_mixed_fallback_types() {
        let mut config = FallbackConfig::new();
        config
            .add_general_fallback("model", vec!["general-backup".to_string()])
            .add_content_policy_fallback("model", vec!["policy-backup".to_string()])
            .add_context_window_fallback("model", vec!["context-backup".to_string()])
            .add_rate_limit_fallback("model", vec!["rate-backup".to_string()]);

        assert_eq!(
            config.general_fallbacks.get("model").unwrap()[0],
            "general-backup"
        );
        assert_eq!(
            config.content_policy_fallbacks.get("model").unwrap()[0],
            "policy-backup"
        );
        assert_eq!(
            config.context_window_fallbacks.get("model").unwrap()[0],
            "context-backup"
        );
        assert_eq!(
            config.rate_limit_fallbacks.get("model").unwrap()[0],
            "rate-backup"
        );
    }

    #[test]
    fn test_empty_fallback_list() {
        let mut config = FallbackConfig::new();
        config.add_general_fallback("gpt-4", vec![]);

        assert!(config.general_fallbacks.get("gpt-4").unwrap().is_empty());
    }
}
