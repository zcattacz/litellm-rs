//! NVIDIA NIM Model Information
//!
//! Contains model metadata and supported parameters for NVIDIA NIM models.

use std::collections::HashMap;

/// NVIDIA NIM model information
#[derive(Debug, Clone)]
pub struct NvidiaNimModel {
    /// Model identifier
    pub model_id: &'static str,
    /// Display name
    pub display_name: &'static str,
    /// Maximum context length
    pub context_length: usize,
    /// Maximum output tokens
    pub max_output_tokens: usize,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
    /// Whether the model supports tools/function calling
    pub supports_tools: bool,
    /// Whether the model supports vision/images
    pub supports_vision: bool,
    /// Input cost per million tokens (USD)
    pub input_cost_per_million: f64,
    /// Output cost per million tokens (USD)
    pub output_cost_per_million: f64,
}

/// Get available models
pub fn get_available_models() -> &'static [&'static str] {
    &[
        // Meta Llama models
        "meta/llama3-70b-instruct",
        "meta/llama3-8b-instruct",
        "meta/llama2-70b",
        "meta/codellama-70b",
        // Mistral models
        "mistralai/mistral-large",
        "mistralai/mixtral-8x22b-instruct-v0.1",
        "mistralai/mixtral-8x7b-instruct-v0.1",
        "mistralai/mistral-7b-instruct-v0.3",
        "mistralai/mistral-7b-instruct-v0.2",
        "mistralai/codestral-22b-instruct-v0.1",
        // Microsoft Phi models
        "microsoft/phi-3-small-8k-instruct",
        "microsoft/phi-3-small-128k-instruct",
        "microsoft/phi-3-mini-4k-instruct",
        "microsoft/phi-3-mini-128k-instruct",
        "microsoft/phi-3-medium-4k-instruct",
        "microsoft/phi-3-medium-128k-instruct",
        // Google models
        "google/recurrentgemma-2b",
        "google/gemma-2-27b-it",
        "google/gemma-2-9b-it",
        "google/codegemma-1.1-7b",
        // NVIDIA models
        "nvidia/nemotron-4-340b-instruct",
        "nvidia/nemotron-4-340b-reward",
        "nvidia/llama3-chatqa-1.5-8b",
        "nvidia/llama3-chatqa-1.5-70b",
        // Other models
        "upstage/solar-10.7b-instruct",
        "snowflake/arctic",
        "seallms/seallm-7b-v2.5",
    ]
}

/// Get model information by model ID
pub fn get_model_info(model_id: &str) -> Option<NvidiaNimModel> {
    // Default values for most models
    let base_model = NvidiaNimModel {
        model_id,
        display_name: model_id,
        context_length: 8192,
        max_output_tokens: 4096,
        supports_streaming: true,
        supports_tools: true,
        supports_vision: false,
        input_cost_per_million: 0.0,  // Pricing varies
        output_cost_per_million: 0.0, // Pricing varies
    };

    // Return model-specific configurations
    Some(match model_id {
        // Meta Llama 3 70B
        "meta/llama3-70b-instruct" => NvidiaNimModel {
            display_name: "Llama 3 70B Instruct",
            context_length: 8192,
            max_output_tokens: 4096,
            supports_tools: true,
            ..base_model
        },
        // Meta Llama 3 8B
        "meta/llama3-8b-instruct" => NvidiaNimModel {
            display_name: "Llama 3 8B Instruct",
            context_length: 8192,
            max_output_tokens: 4096,
            supports_tools: true,
            ..base_model
        },
        // Mistral Large
        "mistralai/mistral-large" => NvidiaNimModel {
            display_name: "Mistral Large",
            context_length: 32768,
            max_output_tokens: 8192,
            supports_tools: true,
            ..base_model
        },
        // Mixtral 8x22B
        "mistralai/mixtral-8x22b-instruct-v0.1" => NvidiaNimModel {
            display_name: "Mixtral 8x22B Instruct",
            context_length: 65536,
            max_output_tokens: 8192,
            supports_tools: true,
            ..base_model
        },
        // Mixtral 8x7B
        "mistralai/mixtral-8x7b-instruct-v0.1" => NvidiaNimModel {
            display_name: "Mixtral 8x7B Instruct",
            context_length: 32768,
            max_output_tokens: 8192,
            supports_tools: true,
            ..base_model
        },
        // Codestral
        "mistralai/codestral-22b-instruct-v0.1" => NvidiaNimModel {
            display_name: "Codestral 22B Instruct",
            context_length: 32768,
            max_output_tokens: 8192,
            supports_tools: true,
            ..base_model
        },
        // Phi-3 models
        "microsoft/phi-3-small-128k-instruct" => NvidiaNimModel {
            display_name: "Phi-3 Small 128K Instruct",
            context_length: 131072,
            max_output_tokens: 4096,
            supports_tools: false,
            ..base_model
        },
        "microsoft/phi-3-medium-128k-instruct" => NvidiaNimModel {
            display_name: "Phi-3 Medium 128K Instruct",
            context_length: 131072,
            max_output_tokens: 4096,
            supports_tools: false,
            ..base_model
        },
        // Google Gemma models
        "google/gemma-2-27b-it" | "google/gemma-2-9b-it" | "google/recurrentgemma-2b" => {
            NvidiaNimModel {
                display_name: if model_id.contains("27b") {
                    "Gemma 2 27B IT"
                } else if model_id.contains("9b") {
                    "Gemma 2 9B IT"
                } else {
                    "RecurrentGemma 2B"
                },
                context_length: 8192,
                max_output_tokens: 4096,
                supports_tools: false,
                ..base_model
            }
        }
        // NVIDIA Nemotron
        "nvidia/nemotron-4-340b-instruct" => NvidiaNimModel {
            display_name: "Nemotron 4 340B Instruct",
            context_length: 4096,
            max_output_tokens: 4096,
            supports_tools: false,
            ..base_model
        },
        "nvidia/nemotron-4-340b-reward" => NvidiaNimModel {
            display_name: "Nemotron 4 340B Reward",
            context_length: 4096,
            max_output_tokens: 4096,
            supports_streaming: true,
            supports_tools: false,
            ..base_model
        },
        // Default for any other model
        _ => base_model,
    })
}

/// Get supported OpenAI parameters for a specific model
pub fn get_supported_params(model: &str) -> &'static [&'static str] {
    // Google Gemma models have limited parameters
    if model.starts_with("google/recurrentgemma")
        || model.starts_with("google/gemma-2")
        || model == "gemma-2-9b-it"
    {
        return &["stream", "temperature", "top_p", "max_tokens", "stop", "seed"];
    }

    // NVIDIA Nemotron Instruct
    if model == "nvidia/nemotron-4-340b-instruct" {
        return &[
            "stream",
            "temperature",
            "top_p",
            "max_tokens",
            "max_completion_tokens",
        ];
    }

    // NVIDIA Nemotron Reward (only streaming)
    if model == "nvidia/nemotron-4-340b-reward" {
        return &["stream"];
    }

    // Google CodeGemma (no seed support)
    if model.contains("codegemma") {
        return &[
            "stream",
            "temperature",
            "top_p",
            "frequency_penalty",
            "presence_penalty",
            "max_tokens",
            "max_completion_tokens",
            "stop",
        ];
    }

    // Default - most NVIDIA NIM models support these
    &[
        "stream",
        "temperature",
        "top_p",
        "frequency_penalty",
        "presence_penalty",
        "max_tokens",
        "max_completion_tokens",
        "stop",
        "seed",
        "tools",
        "tool_choice",
        "parallel_tool_calls",
        "response_format",
    ]
}

/// Check if a model supports tools/function calling
pub fn supports_tools(model: &str) -> bool {
    // Models that don't support tools
    let no_tools_models = [
        "google/recurrentgemma-2b",
        "google/gemma-2-27b-it",
        "google/gemma-2-9b-it",
        "gemma-2-9b-it",
        "nvidia/nemotron-4-340b-instruct",
        "nvidia/nemotron-4-340b-reward",
        "google/codegemma-1.1-7b",
    ];

    !no_tools_models.contains(&model)
}

/// Get all models as a HashMap for quick lookup
pub fn get_models_map() -> HashMap<&'static str, NvidiaNimModel> {
    let mut map = HashMap::new();
    for model_id in get_available_models() {
        if let Some(info) = get_model_info(model_id) {
            map.insert(*model_id, info);
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"meta/llama3-70b-instruct"));
        assert!(models.contains(&"mistralai/mistral-large"));
    }

    #[test]
    fn test_get_model_info_llama() {
        let info = get_model_info("meta/llama3-70b-instruct").unwrap();
        assert_eq!(info.display_name, "Llama 3 70B Instruct");
        assert!(info.supports_streaming);
        assert!(info.supports_tools);
    }

    #[test]
    fn test_get_model_info_gemma() {
        let info = get_model_info("google/gemma-2-27b-it").unwrap();
        assert!(info.supports_streaming);
        assert!(!info.supports_tools);
    }

    #[test]
    fn test_get_model_info_unknown() {
        let info = get_model_info("unknown/model");
        assert!(info.is_some()); // Returns default
    }

    #[test]
    fn test_get_supported_params_default() {
        let params = get_supported_params("meta/llama3-70b-instruct");
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"tools"));
        assert!(params.contains(&"stream"));
    }

    #[test]
    fn test_get_supported_params_gemma() {
        let params = get_supported_params("google/gemma-2-9b-it");
        assert!(params.contains(&"temperature"));
        assert!(!params.contains(&"tools"));
    }

    #[test]
    fn test_get_supported_params_nemotron_reward() {
        let params = get_supported_params("nvidia/nemotron-4-340b-reward");
        assert!(params.contains(&"stream"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_supports_tools() {
        assert!(supports_tools("meta/llama3-70b-instruct"));
        assert!(supports_tools("mistralai/mistral-large"));
        assert!(!supports_tools("google/gemma-2-9b-it"));
        assert!(!supports_tools("nvidia/nemotron-4-340b-reward"));
    }

    #[test]
    fn test_get_models_map() {
        let map = get_models_map();
        assert!(!map.is_empty());
        assert!(map.contains_key("meta/llama3-70b-instruct"));
    }
}
