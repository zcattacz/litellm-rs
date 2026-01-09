//! Audio types for OpenAI-compatible API
//!
//! This module defines audio-related structures for multimodal interactions
//! including audio content, parameters, and delta updates for streaming.

use serde::{Deserialize, Serialize};

/// Audio parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParams {
    /// Voice to use
    pub voice: String,
    /// Audio format
    pub format: String,
}

/// Audio content
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct AudioContent {
    /// Audio data (base64 encoded)
    pub data: String,
    /// Audio format
    pub format: String,
}

/// Audio delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDelta {
    /// Audio data delta
    pub data: Option<String>,
    /// Transcript delta
    pub transcript: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== AudioParams Tests ====================

    #[test]
    fn test_audio_params_creation() {
        let params = AudioParams {
            voice: "alloy".to_string(),
            format: "mp3".to_string(),
        };
        assert_eq!(params.voice, "alloy");
        assert_eq!(params.format, "mp3");
    }

    #[test]
    fn test_audio_params_serialization() {
        let params = AudioParams {
            voice: "nova".to_string(),
            format: "wav".to_string(),
        };
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"voice\":\"nova\""));
        assert!(json.contains("\"format\":\"wav\""));
    }

    #[test]
    fn test_audio_params_deserialization() {
        let json = r#"{"voice":"shimmer","format":"flac"}"#;
        let params: AudioParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.voice, "shimmer");
        assert_eq!(params.format, "flac");
    }

    #[test]
    fn test_audio_params_clone() {
        let params = AudioParams {
            voice: "echo".to_string(),
            format: "opus".to_string(),
        };
        let cloned = params.clone();
        assert_eq!(cloned.voice, params.voice);
        assert_eq!(cloned.format, params.format);
    }

    #[test]
    fn test_audio_params_debug() {
        let params = AudioParams {
            voice: "fable".to_string(),
            format: "aac".to_string(),
        };
        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("AudioParams"));
        assert!(debug_str.contains("fable"));
        assert!(debug_str.contains("aac"));
    }

    #[test]
    fn test_audio_params_various_voices() {
        let voices = ["alloy", "echo", "fable", "onyx", "nova", "shimmer"];
        for voice in voices {
            let params = AudioParams {
                voice: voice.to_string(),
                format: "mp3".to_string(),
            };
            let json = serde_json::to_string(&params).unwrap();
            let parsed: AudioParams = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.voice, voice);
        }
    }

    #[test]
    fn test_audio_params_various_formats() {
        let formats = ["mp3", "wav", "flac", "opus", "aac", "pcm"];
        for format in formats {
            let params = AudioParams {
                voice: "alloy".to_string(),
                format: format.to_string(),
            };
            let json = serde_json::to_string(&params).unwrap();
            let parsed: AudioParams = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.format, format);
        }
    }

    // ==================== AudioContent Tests ====================

    #[test]
    fn test_audio_content_creation() {
        let content = AudioContent {
            data: "SGVsbG8gV29ybGQ=".to_string(),
            format: "mp3".to_string(),
        };
        assert_eq!(content.data, "SGVsbG8gV29ybGQ=");
        assert_eq!(content.format, "mp3");
    }

    #[test]
    fn test_audio_content_serialization() {
        let content = AudioContent {
            data: "base64data==".to_string(),
            format: "wav".to_string(),
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains("\"data\":\"base64data==\""));
        assert!(json.contains("\"format\":\"wav\""));
    }

    #[test]
    fn test_audio_content_deserialization() {
        let json = r#"{"data":"YXVkaW9kYXRh","format":"flac"}"#;
        let content: AudioContent = serde_json::from_str(json).unwrap();
        assert_eq!(content.data, "YXVkaW9kYXRh");
        assert_eq!(content.format, "flac");
    }

    #[test]
    fn test_audio_content_clone() {
        let content = AudioContent {
            data: "test_data".to_string(),
            format: "opus".to_string(),
        };
        let cloned = content.clone();
        assert_eq!(cloned.data, content.data);
        assert_eq!(cloned.format, content.format);
    }

    #[test]
    fn test_audio_content_debug() {
        let content = AudioContent {
            data: "debug_data".to_string(),
            format: "mp3".to_string(),
        };
        let debug_str = format!("{:?}", content);
        assert!(debug_str.contains("AudioContent"));
        assert!(debug_str.contains("debug_data"));
    }

    #[test]
    fn test_audio_content_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        fn calculate_hash<T: Hash>(t: &T) -> u64 {
            let mut s = DefaultHasher::new();
            t.hash(&mut s);
            s.finish()
        }

        let content1 = AudioContent {
            data: "data1".to_string(),
            format: "mp3".to_string(),
        };
        let content2 = AudioContent {
            data: "data1".to_string(),
            format: "mp3".to_string(),
        };
        let content3 = AudioContent {
            data: "data2".to_string(),
            format: "mp3".to_string(),
        };

        // Same content should produce same hash
        assert_eq!(calculate_hash(&content1), calculate_hash(&content2));
        // Different content should produce different hash
        assert_ne!(calculate_hash(&content1), calculate_hash(&content3));
    }

    #[test]
    fn test_audio_content_empty_data() {
        let content = AudioContent {
            data: "".to_string(),
            format: "mp3".to_string(),
        };
        assert!(content.data.is_empty());
        let json = serde_json::to_string(&content).unwrap();
        let parsed: AudioContent = serde_json::from_str(&json).unwrap();
        assert!(parsed.data.is_empty());
    }

    // ==================== AudioDelta Tests ====================

    #[test]
    fn test_audio_delta_creation() {
        let delta = AudioDelta {
            data: Some("chunk_data".to_string()),
            transcript: Some("Hello".to_string()),
        };
        assert_eq!(delta.data, Some("chunk_data".to_string()));
        assert_eq!(delta.transcript, Some("Hello".to_string()));
    }

    #[test]
    fn test_audio_delta_serialization() {
        let delta = AudioDelta {
            data: Some("delta_data".to_string()),
            transcript: Some("text".to_string()),
        };
        let json = serde_json::to_string(&delta).unwrap();
        assert!(json.contains("\"data\":\"delta_data\""));
        assert!(json.contains("\"transcript\":\"text\""));
    }

    #[test]
    fn test_audio_delta_deserialization() {
        let json = r#"{"data":"YXVkaW8=","transcript":"audio transcript"}"#;
        let delta: AudioDelta = serde_json::from_str(json).unwrap();
        assert_eq!(delta.data, Some("YXVkaW8=".to_string()));
        assert_eq!(delta.transcript, Some("audio transcript".to_string()));
    }

    #[test]
    fn test_audio_delta_with_only_data() {
        let delta = AudioDelta {
            data: Some("only_data".to_string()),
            transcript: None,
        };
        assert!(delta.data.is_some());
        assert!(delta.transcript.is_none());

        let json = serde_json::to_string(&delta).unwrap();
        let parsed: AudioDelta = serde_json::from_str(&json).unwrap();
        assert!(parsed.data.is_some());
    }

    #[test]
    fn test_audio_delta_with_only_transcript() {
        let delta = AudioDelta {
            data: None,
            transcript: Some("only transcript".to_string()),
        };
        assert!(delta.data.is_none());
        assert!(delta.transcript.is_some());
    }

    #[test]
    fn test_audio_delta_both_none() {
        let delta = AudioDelta {
            data: None,
            transcript: None,
        };
        assert!(delta.data.is_none());
        assert!(delta.transcript.is_none());

        let json = serde_json::to_string(&delta).unwrap();
        let parsed: AudioDelta = serde_json::from_str(&json).unwrap();
        assert!(parsed.data.is_none());
        assert!(parsed.transcript.is_none());
    }

    #[test]
    fn test_audio_delta_clone() {
        let delta = AudioDelta {
            data: Some("clone_test".to_string()),
            transcript: Some("transcript".to_string()),
        };
        let cloned = delta.clone();
        assert_eq!(cloned.data, delta.data);
        assert_eq!(cloned.transcript, delta.transcript);
    }

    #[test]
    fn test_audio_delta_debug() {
        let delta = AudioDelta {
            data: Some("debug".to_string()),
            transcript: None,
        };
        let debug_str = format!("{:?}", delta);
        assert!(debug_str.contains("AudioDelta"));
        assert!(debug_str.contains("debug"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_audio_params_to_content_workflow() {
        // Simulate creating audio from params
        let params = AudioParams {
            voice: "alloy".to_string(),
            format: "mp3".to_string(),
        };

        // After generation, we'd have content
        let content = AudioContent {
            data: "generated_audio_base64".to_string(),
            format: params.format.clone(),
        };

        assert_eq!(content.format, params.format);
    }

    #[test]
    fn test_streaming_delta_sequence() {
        // Simulate streaming audio deltas
        let deltas = vec![
            AudioDelta {
                data: Some("chunk1".to_string()),
                transcript: None,
            },
            AudioDelta {
                data: Some("chunk2".to_string()),
                transcript: Some("Hello".to_string()),
            },
            AudioDelta {
                data: Some("chunk3".to_string()),
                transcript: Some(" World".to_string()),
            },
            AudioDelta {
                data: None,
                transcript: None,
            }, // End marker
        ];

        let mut full_data = String::new();
        let mut full_transcript = String::new();

        for delta in deltas {
            if let Some(data) = delta.data {
                full_data.push_str(&data);
            }
            if let Some(transcript) = delta.transcript {
                full_transcript.push_str(&transcript);
            }
        }

        assert_eq!(full_data, "chunk1chunk2chunk3");
        assert_eq!(full_transcript, "Hello World");
    }
}
