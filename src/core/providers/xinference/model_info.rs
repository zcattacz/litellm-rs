//! Xinference Model Information

use std::collections::HashMap;
use std::sync::LazyLock;

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub model_id: &'static str,
    pub display_name: &'static str,
    pub context_length: u32,
    pub max_output_tokens: u32,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub input_cost_per_million: f64,
    pub output_cost_per_million: f64,
}

/// Static model configurations for common Xinference models
static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    // Common LLaMA models
    configs.insert(
        "llama-2-7b",
        ModelInfo {
            model_id: "llama-2-7b",
            display_name: "LLaMA 2 7B",
            context_length: 4096,
            max_output_tokens: 4096,
            supports_tools: false,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    configs.insert(
        "llama-2-13b",
        ModelInfo {
            model_id: "llama-2-13b",
            display_name: "LLaMA 2 13B",
            context_length: 4096,
            max_output_tokens: 4096,
            supports_tools: false,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    configs.insert(
        "llama-3-8b-instruct",
        ModelInfo {
            model_id: "llama-3-8b-instruct",
            display_name: "LLaMA 3 8B Instruct",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    // Qwen models
    configs.insert(
        "qwen-7b-chat",
        ModelInfo {
            model_id: "qwen-7b-chat",
            display_name: "Qwen 7B Chat",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    configs.insert(
        "qwen-14b-chat",
        ModelInfo {
            model_id: "qwen-14b-chat",
            display_name: "Qwen 14B Chat",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    // ChatGLM models
    configs.insert(
        "chatglm3-6b",
        ModelInfo {
            model_id: "chatglm3-6b",
            display_name: "ChatGLM3 6B",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    // Mistral models
    configs.insert(
        "mistral-7b-instruct",
        ModelInfo {
            model_id: "mistral-7b-instruct",
            display_name: "Mistral 7B Instruct",
            context_length: 8192,
            max_output_tokens: 8192,
            supports_tools: true,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    // Code models
    configs.insert(
        "code-llama-7b",
        ModelInfo {
            model_id: "code-llama-7b",
            display_name: "Code LLaMA 7B",
            context_length: 16384,
            max_output_tokens: 16384,
            supports_tools: false,
            supports_vision: false,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    // Vision models
    configs.insert(
        "llava-v1.5-7b",
        ModelInfo {
            model_id: "llava-v1.5-7b",
            display_name: "LLaVA v1.5 7B",
            context_length: 4096,
            max_output_tokens: 4096,
            supports_tools: false,
            supports_vision: true,
            input_cost_per_million: 0.0,
            output_cost_per_million: 0.0,
        },
    );

    configs
});

/// Get model information
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
    fn test_get_model_info() {
        let info = get_model_info("llama-3-8b-instruct");
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.model_id, "llama-3-8b-instruct");
        assert!(info.supports_tools);
    }

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"llama-2-7b"));
    }
}
