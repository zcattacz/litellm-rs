//! Token usage models

use serde::{Deserialize, Serialize};

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// Input tokens
    pub input_tokens: u32,
    /// Output tokens
    pub output_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
    /// Cached tokens
    pub cached_tokens: Option<u32>,
    /// Reasoning tokens
    pub reasoning_tokens: Option<u32>,
    /// Audio tokens
    pub audio_tokens: Option<u32>,
}

impl TokenUsage {
    /// Create new token usage
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cached_tokens: None,
            reasoning_tokens: None,
            audio_tokens: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TokenUsage Creation Tests ====================

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_token_usage_default() {
        let usage = TokenUsage::default();
        assert_eq!(usage.input_tokens, 0);
        assert_eq!(usage.output_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
        assert!(usage.cached_tokens.is_none());
        assert!(usage.reasoning_tokens.is_none());
        assert!(usage.audio_tokens.is_none());
    }

    #[test]
    fn test_token_usage_zero_tokens() {
        let usage = TokenUsage::new(0, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_token_usage_large_values() {
        let usage = TokenUsage::new(100_000, 50_000);
        assert_eq!(usage.total_tokens, 150_000);
    }

    #[test]
    fn test_token_usage_max_values() {
        let usage = TokenUsage::new(u32::MAX / 2, u32::MAX / 2);
        assert_eq!(usage.total_tokens, u32::MAX - 1);
    }

    // ==================== Optional Fields Tests ====================

    #[test]
    fn test_token_usage_with_cached_tokens() {
        let mut usage = TokenUsage::new(100, 50);
        usage.cached_tokens = Some(25);
        assert_eq!(usage.cached_tokens, Some(25));
    }

    #[test]
    fn test_token_usage_with_reasoning_tokens() {
        let mut usage = TokenUsage::new(100, 50);
        usage.reasoning_tokens = Some(30);
        assert_eq!(usage.reasoning_tokens, Some(30));
    }

    #[test]
    fn test_token_usage_with_audio_tokens() {
        let mut usage = TokenUsage::new(100, 50);
        usage.audio_tokens = Some(20);
        assert_eq!(usage.audio_tokens, Some(20));
    }

    #[test]
    fn test_token_usage_with_all_optional_fields() {
        let mut usage = TokenUsage::new(1000, 500);
        usage.cached_tokens = Some(100);
        usage.reasoning_tokens = Some(200);
        usage.audio_tokens = Some(50);

        assert_eq!(usage.cached_tokens, Some(100));
        assert_eq!(usage.reasoning_tokens, Some(200));
        assert_eq!(usage.audio_tokens, Some(50));
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn test_token_usage_serialization() {
        let usage = TokenUsage::new(100, 50);
        let json = serde_json::to_value(&usage).unwrap();
        assert_eq!(json["input_tokens"], 100);
        assert_eq!(json["output_tokens"], 50);
        assert_eq!(json["total_tokens"], 150);
    }

    #[test]
    fn test_token_usage_deserialization() {
        let json = r#"{
            "input_tokens": 200,
            "output_tokens": 100,
            "total_tokens": 300,
            "cached_tokens": 50,
            "reasoning_tokens": null,
            "audio_tokens": null
        }"#;
        let usage: TokenUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 200);
        assert_eq!(usage.output_tokens, 100);
        assert_eq!(usage.total_tokens, 300);
        assert_eq!(usage.cached_tokens, Some(50));
    }

    #[test]
    fn test_token_usage_deserialization_minimal() {
        let json = r#"{
            "input_tokens": 10,
            "output_tokens": 5,
            "total_tokens": 15
        }"#;
        let usage: TokenUsage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert!(usage.cached_tokens.is_none());
    }

    #[test]
    fn test_token_usage_roundtrip() {
        let mut original = TokenUsage::new(500, 250);
        original.cached_tokens = Some(100);
        original.reasoning_tokens = Some(50);

        let json = serde_json::to_string(&original).unwrap();
        let restored: TokenUsage = serde_json::from_str(&json).unwrap();

        assert_eq!(original.input_tokens, restored.input_tokens);
        assert_eq!(original.output_tokens, restored.output_tokens);
        assert_eq!(original.cached_tokens, restored.cached_tokens);
        assert_eq!(original.reasoning_tokens, restored.reasoning_tokens);
    }

    // ==================== Clone Tests ====================

    #[test]
    fn test_token_usage_clone() {
        let mut original = TokenUsage::new(100, 50);
        original.cached_tokens = Some(25);

        let cloned = original.clone();
        assert_eq!(original.input_tokens, cloned.input_tokens);
        assert_eq!(original.output_tokens, cloned.output_tokens);
        assert_eq!(original.cached_tokens, cloned.cached_tokens);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_token_usage_input_only() {
        let usage = TokenUsage::new(100, 0);
        assert_eq!(usage.total_tokens, 100);
    }

    #[test]
    fn test_token_usage_output_only() {
        let usage = TokenUsage::new(0, 100);
        assert_eq!(usage.total_tokens, 100);
    }
}
