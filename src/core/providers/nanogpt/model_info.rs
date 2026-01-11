//! NanoGPT Model Information
//!
//! Contains model configurations and capabilities for NanoGPT-supported models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// NanoGPT model identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NanoGPTModel {
    /// NanoGPT Base 7B model
    NanoBase7B,
    /// NanoGPT Pro 13B model
    NanoPro13B,
    /// NanoGPT Ultra 70B model
    NanoUltra70B,
}

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model ID as used in API calls
    pub model_id: &'static str,

    /// Human-friendly model name
    pub display_name: &'static str,

    /// Maximum context length (tokens)
    pub context_length: u32,

    /// Maximum output tokens
    pub max_output_tokens: u32,

    /// Whether the model supports tool/function calling
    pub supports_tools: bool,

    /// Whether this is a reasoning model
    pub is_reasoning: bool,

    /// Whether the model supports vision
    pub supports_vision: bool,

    /// Cost per 1M input tokens (USD)
    pub input_cost_per_million: f64,

    /// Cost per 1M output tokens (USD)
    pub output_cost_per_million: f64,
}

/// Static model configurations
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    configs.insert(
        "nano-base-7b",
        ModelInfo {
            model_id: "nano-base-7b",
            display_name: "NanoGPT Base 7B",
            context_length: 8192,
            max_output_tokens: 4096,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            input_cost_per_million: 0.10,
            output_cost_per_million: 0.20,
        },
    );

    configs.insert(
        "nano-pro-13b",
        ModelInfo {
            model_id: "nano-pro-13b",
            display_name: "NanoGPT Pro 13B",
            context_length: 16384,
            max_output_tokens: 8192,
            supports_tools: true,
            is_reasoning: false,
            supports_vision: false,
            input_cost_per_million: 0.30,
            output_cost_per_million: 0.60,
        },
    );

    configs.insert(
        "nano-ultra-70b",
        ModelInfo {
            model_id: "nano-ultra-70b",
            display_name: "NanoGPT Ultra 70B",
            context_length: 32768,
            max_output_tokens: 16384,
            supports_tools: true,
            is_reasoning: true,
            supports_vision: false,
            input_cost_per_million: 0.80,
            output_cost_per_million: 1.60,
        },
    );

    configs
});

/// Get model information for a given model ID
pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    MODEL_CONFIGS.get(model_id)
}

/// Get all available model IDs
pub fn get_available_models() -> Vec<&'static str> {
    MODEL_CONFIGS.keys().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_info_valid() {
        let info = get_model_info("nano-base-7b");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.model_id, "nano-base-7b");
        assert_eq!(info.display_name, "NanoGPT Base 7B");
    }

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"nano-base-7b"));
    }
}
