//! O-Series Response Transformation for Azure

use super::{AzureProcessedResponse, AzureResponseMetadata, ResponseMetrics};
use serde::{Deserialize, Serialize};

/// O-Series specific response processor
#[derive(Debug, Clone, Default)]
pub struct OSeriesResponseProcessor;

impl OSeriesResponseProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Process O-series model responses
    pub fn process_response<T: Serialize + for<'de> Deserialize<'de> + Clone>(
        &self,
        response: T,
    ) -> Result<AzureProcessedResponse<T>, String> {
        let start_time = std::time::Instant::now();

        // Convert to JSON for processing
        let json_response = serde_json::to_value(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))?;

        // Extract O-series specific metadata
        let metadata = self.extract_o_series_metadata(&json_response);

        // Check for reasoning tokens in usage
        let has_reasoning = self.has_reasoning_tokens(&json_response);

        let processing_time = start_time.elapsed().as_millis() as u64;

        let response_size = serde_json::to_vec(&response).map_or(0, |v| v.len());

        Ok(AzureProcessedResponse {
            data: response,
            metadata,
            content_filtered: false, // O-series models typically have different filtering
            warnings: if has_reasoning {
                vec!["Response includes reasoning tokens".to_string()]
            } else {
                vec![]
            },
            metrics: ResponseMetrics {
                total_time_ms: processing_time,
                transformation_time_ms: processing_time,
                filtering_time_ms: 0,
                response_size_bytes: response_size,
            },
        })
    }

    fn extract_o_series_metadata(&self, response: &serde_json::Value) -> AzureResponseMetadata {
        let mut metadata = AzureResponseMetadata::default();

        // Extract deployment info
        if let Some(model) = response.get("model").and_then(|m| m.as_str()) {
            metadata.deployment_id = Some(model.to_string());
        }

        metadata
    }

    fn has_reasoning_tokens(&self, response: &serde_json::Value) -> bool {
        if let Some(usage) = response.get("usage") {
            return usage.get("reasoning_tokens").is_some();
        }
        false
    }
}

/// O-Series response transformation
#[derive(Debug, Clone, Default)]
pub struct OSeriesResponseTransformation;

impl OSeriesResponseTransformation {
    pub fn new() -> Self {
        Self
    }

    /// Transform O-series response for compatibility
    /// Takes ownership to avoid unnecessary cloning
    pub fn transform_response(
        &self,
        mut response: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Handle reasoning tokens in usage
        if let Some(usage) = response.get_mut("usage") {
            self.transform_usage_with_reasoning(usage)?;
        }

        // Handle choices with reasoning content
        if let Some(choices) = response.get_mut("choices").and_then(|c| c.as_array_mut()) {
            for choice in choices {
                self.transform_o_series_choice(choice)?;
            }
        }

        Ok(response)
    }

    fn transform_usage_with_reasoning(&self, usage: &mut serde_json::Value) -> Result<(), String> {
        // O-series models may include reasoning_tokens
        // Ensure they're properly accounted for in total_tokens

        if let Some(usage_obj) = usage.as_object_mut() {
            let prompt_tokens = usage_obj
                .get("prompt_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);

            let completion_tokens = usage_obj
                .get("completion_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);

            let reasoning_tokens = usage_obj
                .get("reasoning_tokens")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);

            // Update total_tokens to include reasoning tokens
            let total_tokens = prompt_tokens + completion_tokens + reasoning_tokens;
            usage_obj.insert("total_tokens".to_string(), serde_json::json!(total_tokens));
        }

        Ok(())
    }

    fn transform_o_series_choice(&self, _choice: &mut serde_json::Value) -> Result<(), String> {
        // O-series models might have special handling for reasoning
        // For now, pass through as-is
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ==================== OSeriesResponseProcessor Tests ====================

    #[test]
    fn test_o_series_processor_new() {
        let processor = OSeriesResponseProcessor::new();
        // Just verify construction works
        let _ = processor;
    }

    #[test]
    fn test_o_series_processor_default() {
        let processor = OSeriesResponseProcessor;
        let _ = processor;
    }

    #[test]
    fn test_process_response_basic() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "id": "chatcmpl-123",
            "model": "o1-preview",
            "choices": [{"message": {"content": "Hello"}}]
        });

        let result = processor.process_response(response.clone());
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.data, response);
        assert!(!processed.content_filtered);
        assert!(processed.warnings.is_empty());
    }

    #[test]
    fn test_process_response_with_reasoning_tokens() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "id": "chatcmpl-456",
            "model": "o1-preview",
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "reasoning_tokens": 200
            }
        });

        let result = processor.process_response(response);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(!processed.warnings.is_empty());
        assert!(processed.warnings[0].contains("reasoning tokens"));
    }

    #[test]
    fn test_process_response_extracts_model_metadata() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "model": "o1-mini-deployment",
            "choices": []
        });

        let result = processor.process_response(response);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(
            processed.metadata.deployment_id,
            Some("o1-mini-deployment".to_string())
        );
    }

    #[test]
    fn test_process_response_no_model() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "choices": []
        });

        let result = processor.process_response(response);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metadata.deployment_id.is_none());
    }

    #[test]
    fn test_process_response_metrics() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "id": "test",
            "choices": []
        });

        let result = processor.process_response(response);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert!(processed.metrics.response_size_bytes > 0);
    }

    #[test]
    fn test_has_reasoning_tokens_true() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "usage": {
                "prompt_tokens": 100,
                "reasoning_tokens": 50
            }
        });

        assert!(processor.has_reasoning_tokens(&response));
    }

    #[test]
    fn test_has_reasoning_tokens_false() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50
            }
        });

        assert!(!processor.has_reasoning_tokens(&response));
    }

    #[test]
    fn test_has_reasoning_tokens_no_usage() {
        let processor = OSeriesResponseProcessor::new();

        let response = json!({
            "choices": []
        });

        assert!(!processor.has_reasoning_tokens(&response));
    }

    // ==================== OSeriesResponseTransformation Tests ====================

    #[test]
    fn test_o_series_transformation_new() {
        let transformation = OSeriesResponseTransformation::new();
        let _ = transformation;
    }

    #[test]
    fn test_o_series_transformation_default() {
        let transformation = OSeriesResponseTransformation;
        let _ = transformation;
    }

    #[test]
    fn test_transform_response_basic() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "id": "chatcmpl-123",
            "choices": [{"message": {"content": "Hello"}}]
        });

        let result = transformation.transform_response(response.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_response_with_reasoning_tokens() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "id": "chatcmpl-456",
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "reasoning_tokens": 200
            }
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();

        // Check total_tokens includes reasoning tokens
        let total = transformed["usage"]["total_tokens"].as_u64().unwrap();
        assert_eq!(total, 350); // 100 + 50 + 200
    }

    #[test]
    fn test_transform_response_no_reasoning_tokens() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50
            }
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();

        let total = transformed["usage"]["total_tokens"].as_u64().unwrap();
        assert_eq!(total, 150); // 100 + 50 + 0
    }

    #[test]
    fn test_transform_response_no_usage() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "choices": []
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_response_with_choices() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello from O1"
                    },
                    "finish_reason": "stop"
                },
                {
                    "index": 1,
                    "message": {
                        "role": "assistant",
                        "content": "Another response"
                    },
                    "finish_reason": "stop"
                }
            ]
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed["choices"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_transform_usage_updates_total_correctly() {
        let transformation = OSeriesResponseTransformation::new();

        // Test case with large token counts
        let response = json!({
            "usage": {
                "prompt_tokens": 5000,
                "completion_tokens": 3000,
                "reasoning_tokens": 10000
            }
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();

        let total = transformed["usage"]["total_tokens"].as_u64().unwrap();
        assert_eq!(total, 18000); // 5000 + 3000 + 10000
    }

    #[test]
    fn test_transform_response_preserves_other_fields() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "id": "chatcmpl-789",
            "object": "chat.completion",
            "created": 1234567890,
            "model": "o1-preview",
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5
            },
            "system_fingerprint": "fp_abc123"
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();

        assert_eq!(transformed["id"], "chatcmpl-789");
        assert_eq!(transformed["object"], "chat.completion");
        assert_eq!(transformed["created"], 1234567890);
        assert_eq!(transformed["model"], "o1-preview");
        assert_eq!(transformed["system_fingerprint"], "fp_abc123");
    }

    #[test]
    fn test_transform_response_empty_choices() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "choices": []
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed["choices"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_transform_usage_missing_prompt_tokens() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "usage": {
                "completion_tokens": 50,
                "reasoning_tokens": 100
            }
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();

        // Missing prompt_tokens defaults to 0
        let total = transformed["usage"]["total_tokens"].as_u64().unwrap();
        assert_eq!(total, 150); // 0 + 50 + 100
    }

    #[test]
    fn test_transform_usage_missing_completion_tokens() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "usage": {
                "prompt_tokens": 100,
                "reasoning_tokens": 200
            }
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();

        let total = transformed["usage"]["total_tokens"].as_u64().unwrap();
        assert_eq!(total, 300); // 100 + 0 + 200
    }

    #[test]
    fn test_transform_usage_zero_tokens() {
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "usage": {
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "reasoning_tokens": 0
            }
        });

        let result = transformation.transform_response(response);
        assert!(result.is_ok());
        let transformed = result.unwrap();

        let total = transformed["usage"]["total_tokens"].as_u64().unwrap();
        assert_eq!(total, 0);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_processor_and_transformation_work_together() {
        let processor = OSeriesResponseProcessor::new();
        let transformation = OSeriesResponseTransformation::new();

        let response = json!({
            "id": "chatcmpl-integration",
            "model": "o1-preview",
            "choices": [{
                "message": {"content": "Test response"}
            }],
            "usage": {
                "prompt_tokens": 100,
                "completion_tokens": 50,
                "reasoning_tokens": 150
            }
        });

        // Transform first
        let transformed = transformation.transform_response(response.clone()).unwrap();

        // Then process
        let processed = processor.process_response(transformed).unwrap();

        // Check warnings about reasoning tokens
        assert!(!processed.warnings.is_empty());

        // Check total tokens was updated
        let total = processed.data["usage"]["total_tokens"].as_u64().unwrap();
        assert_eq!(total, 300); // 100 + 50 + 150
    }

    #[test]
    fn test_process_response_with_serializable_struct() {
        let processor = OSeriesResponseProcessor::new();

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct SimpleResponse {
            id: String,
            content: String,
        }

        let response = SimpleResponse {
            id: "test-id".to_string(),
            content: "Hello".to_string(),
        };

        let result = processor.process_response(response);
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.data.id, "test-id");
        assert_eq!(processed.data.content, "Hello");
    }
}
