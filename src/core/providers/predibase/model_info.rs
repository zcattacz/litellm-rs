//! Predibase Model Information

use std::collections::HashMap;
use std::sync::LazyLock;

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

static MODEL_CONFIGS: LazyLock<HashMap<&'static str, ModelInfo>> = LazyLock::new(|| {
    let mut configs = HashMap::new();

    configs.insert(
        "llama-3-8b-instruct",
        ModelInfo {
            model_id: "llama-3-8b-instruct",
            display_name: "Llama 3 8B Instruct",
            context_length: 8192,
            max_output_tokens: 4096,
            supports_tools: false,
            supports_vision: false,
            input_cost_per_million: 0.20,
            output_cost_per_million: 0.20,
        },
    );

    configs.insert(
        "llama-3-70b-instruct",
        ModelInfo {
            model_id: "llama-3-70b-instruct",
            display_name: "Llama 3 70B Instruct",
            context_length: 8192,
            max_output_tokens: 4096,
            supports_tools: false,
            supports_vision: false,
            input_cost_per_million: 1.20,
            output_cost_per_million: 1.20,
        },
    );

    configs.insert(
        "mistral-7b-instruct",
        ModelInfo {
            model_id: "mistral-7b-instruct",
            display_name: "Mistral 7B Instruct",
            context_length: 8192,
            max_output_tokens: 4096,
            supports_tools: false,
            supports_vision: false,
            input_cost_per_million: 0.18,
            output_cost_per_million: 0.18,
        },
    );

    configs
});

pub fn get_model_info(model_id: &str) -> Option<&'static ModelInfo> {
    MODEL_CONFIGS.get(model_id)
}

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
    }
}
