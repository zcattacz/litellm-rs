//! Image response types

use serde::{Deserialize, Serialize};

/// Image response (simple format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResponse {
    /// Creation timestamp
    pub created: i64,

    /// Image data list
    pub data: Vec<ImageData>,
}

/// Image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// Image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Base64 encoded image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b64_json: Option<String>,

    /// Revised prompt (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_prompt: Option<String>,
}

/// Image generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Creation timestamp
    pub created: u64,

    /// Generated image list
    pub data: Vec<ImageData>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ImageResponse Tests ====================

    #[test]
    fn test_image_response_creation() {
        let response = ImageResponse {
            created: 1234567890,
            data: vec![],
        };
        assert_eq!(response.created, 1234567890);
        assert!(response.data.is_empty());
    }

    #[test]
    fn test_image_response_with_url() {
        let response = ImageResponse {
            created: 1234567890,
            data: vec![ImageData {
                url: Some("https://example.com/image.png".to_string()),
                b64_json: None,
                revised_prompt: None,
            }],
        };
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].url.is_some());
    }

    #[test]
    fn test_image_response_with_b64() {
        let response = ImageResponse {
            created: 1234567890,
            data: vec![ImageData {
                url: None,
                b64_json: Some("base64encodeddata".to_string()),
                revised_prompt: None,
            }],
        };
        assert!(response.data[0].b64_json.is_some());
        assert!(response.data[0].url.is_none());
    }

    #[test]
    fn test_image_response_serialization() {
        let response = ImageResponse {
            created: 1000,
            data: vec![ImageData {
                url: Some("https://example.com/img.png".to_string()),
                b64_json: None,
                revised_prompt: Some("A revised prompt".to_string()),
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("1000"));
        assert!(json.contains("https://example.com/img.png"));
        assert!(json.contains("revised_prompt"));
    }

    #[test]
    fn test_image_response_deserialization() {
        let json = r#"{
            "created": 1699999999,
            "data": [
                {"url": "https://cdn.example.com/image1.png"},
                {"url": "https://cdn.example.com/image2.png"}
            ]
        }"#;
        let response: ImageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.created, 1699999999);
        assert_eq!(response.data.len(), 2);
    }

    // ==================== ImageData Tests ====================

    #[test]
    fn test_image_data_url_only() {
        let data = ImageData {
            url: Some("https://example.com/image.png".to_string()),
            b64_json: None,
            revised_prompt: None,
        };
        assert!(data.url.is_some());
        assert!(data.b64_json.is_none());
    }

    #[test]
    fn test_image_data_b64_only() {
        let data = ImageData {
            url: None,
            b64_json: Some("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string()),
            revised_prompt: None,
        };
        assert!(data.b64_json.is_some());
        assert!(data.url.is_none());
    }

    #[test]
    fn test_image_data_with_revised_prompt() {
        let data = ImageData {
            url: Some("https://example.com/img.png".to_string()),
            b64_json: None,
            revised_prompt: Some("A beautiful sunset over the ocean".to_string()),
        };
        assert_eq!(
            data.revised_prompt,
            Some("A beautiful sunset over the ocean".to_string())
        );
    }

    #[test]
    fn test_image_data_serialization_minimal() {
        let data = ImageData {
            url: None,
            b64_json: None,
            revised_prompt: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_image_data_serialization_with_url() {
        let data = ImageData {
            url: Some("https://api.example.com/images/123.png".to_string()),
            b64_json: None,
            revised_prompt: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("url"));
        assert!(!json.contains("b64_json"));
    }

    #[test]
    fn test_image_data_deserialization() {
        let json = r#"{
            "url": "https://example.com/image.jpg",
            "revised_prompt": "A cat sitting on a mat"
        }"#;
        let data: ImageData = serde_json::from_str(json).unwrap();
        assert_eq!(data.url, Some("https://example.com/image.jpg".to_string()));
        assert_eq!(
            data.revised_prompt,
            Some("A cat sitting on a mat".to_string())
        );
    }

    // ==================== ImageGenerationResponse Tests ====================

    #[test]
    fn test_image_generation_response_creation() {
        let response = ImageGenerationResponse {
            created: 1700000000,
            data: vec![],
        };
        assert_eq!(response.created, 1700000000);
    }

    #[test]
    fn test_image_generation_response_with_data() {
        let response = ImageGenerationResponse {
            created: 1700000000,
            data: vec![
                ImageData {
                    url: Some("https://example.com/1.png".to_string()),
                    b64_json: None,
                    revised_prompt: None,
                },
                ImageData {
                    url: Some("https://example.com/2.png".to_string()),
                    b64_json: None,
                    revised_prompt: None,
                },
            ],
        };
        assert_eq!(response.data.len(), 2);
    }

    #[test]
    fn test_image_generation_response_serialization() {
        let response = ImageGenerationResponse {
            created: 1700000001,
            data: vec![ImageData {
                url: Some("https://cdn.example.com/generated.png".to_string()),
                b64_json: None,
                revised_prompt: Some("Revised".to_string()),
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("1700000001"));
        assert!(json.contains("cdn.example.com"));
    }

    #[test]
    fn test_image_generation_response_deserialization() {
        let json = r#"{"created": 1699999999, "data": []}"#;
        let response: ImageGenerationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.created, 1699999999);
        assert!(response.data.is_empty());
    }

    // ==================== Clone and Debug Tests ====================

    #[test]
    fn test_image_response_clone() {
        let response = ImageResponse {
            created: 1234567890,
            data: vec![ImageData {
                url: Some("https://example.com/img.png".to_string()),
                b64_json: None,
                revised_prompt: None,
            }],
        };
        let cloned = response.clone();
        assert_eq!(cloned.created, 1234567890);
        assert_eq!(cloned.data.len(), 1);
    }

    #[test]
    fn test_image_data_clone() {
        let data = ImageData {
            url: Some("https://example.com/img.png".to_string()),
            b64_json: None,
            revised_prompt: Some("Prompt".to_string()),
        };
        let cloned = data.clone();
        assert_eq!(cloned.url, data.url);
        assert_eq!(cloned.revised_prompt, data.revised_prompt);
    }

    #[test]
    fn test_image_response_debug() {
        let response = ImageResponse {
            created: 1234567890,
            data: vec![],
        };
        let debug = format!("{:?}", response);
        assert!(debug.contains("ImageResponse"));
        assert!(debug.contains("1234567890"));
    }

    #[test]
    fn test_image_data_debug() {
        let data = ImageData {
            url: Some("test".to_string()),
            b64_json: None,
            revised_prompt: None,
        };
        let debug = format!("{:?}", data);
        assert!(debug.contains("ImageData"));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_image_data_all_fields() {
        let data = ImageData {
            url: Some("https://example.com/image.png".to_string()),
            b64_json: Some("base64data".to_string()),
            revised_prompt: Some("Revised prompt".to_string()),
        };
        assert!(data.url.is_some());
        assert!(data.b64_json.is_some());
        assert!(data.revised_prompt.is_some());
    }

    #[test]
    fn test_image_response_zero_timestamp() {
        let response = ImageResponse {
            created: 0,
            data: vec![],
        };
        assert_eq!(response.created, 0);
    }

    #[test]
    fn test_image_data_empty_strings() {
        let data = ImageData {
            url: Some("".to_string()),
            b64_json: Some("".to_string()),
            revised_prompt: Some("".to_string()),
        };
        assert!(data.url.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_image_response_roundtrip() {
        let original = ImageResponse {
            created: 1700000000,
            data: vec![ImageData {
                url: Some("https://example.com/image.png".to_string()),
                b64_json: None,
                revised_prompt: Some("A test prompt".to_string()),
            }],
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ImageResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.created, original.created);
        assert_eq!(deserialized.data.len(), original.data.len());
    }
}
