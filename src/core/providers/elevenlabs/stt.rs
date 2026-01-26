//! Speech-to-Text (STT) Module for ElevenLabs
//!
//! Provides audio transcription capabilities using ElevenLabs' speech-to-text API.

use crate::core::providers::unified_provider::ProviderError;
use serde::{Deserialize, Serialize};

/// Provider name constant
const PROVIDER_NAME: &str = "elevenlabs";

/// STT API endpoint path
pub const STT_ENDPOINT_PATH: &str = "/v1/speech-to-text";

/// Transcription request
#[derive(Debug, Clone)]
pub struct TranscriptionRequest {
    /// Audio file bytes
    pub file: Vec<u8>,

    /// Model ID for transcription
    /// Options: "scribe_v1"
    pub model_id: String,

    /// Language code (ISO 639-1)
    /// Example: "en", "es", "fr", "de", "zh"
    pub language_code: Option<String>,

    /// Temperature for sampling (0.0 to 1.0)
    pub temperature: Option<f32>,

    /// Original filename (for content type detection)
    pub filename: Option<String>,
}

/// Transcription response from ElevenLabs
#[derive(Debug, Clone, Deserialize)]
pub struct TranscriptionResponse {
    /// Transcribed text
    pub text: String,

    /// Detected language code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,

    /// Word-level timestamps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<WordInfo>>,
}

/// Word information with timestamps
#[derive(Debug, Clone, Deserialize)]
pub struct WordInfo {
    /// The word text
    pub text: String,

    /// Start time in seconds
    pub start: f32,

    /// End time in seconds
    pub end: f32,

    /// Type: "word", "spacing", or "audio_event"
    #[serde(rename = "type")]
    pub word_type: String,
}

/// OpenAI-compatible transcription response
#[derive(Debug, Clone, Serialize)]
pub struct OpenAITranscriptionResponse {
    /// Transcribed text
    pub text: String,

    /// Task type (always "transcribe")
    pub task: String,

    /// Language
    pub language: String,

    /// Word timestamps (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<Vec<OpenAIWordInfo>>,
}

/// OpenAI-compatible word info
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIWordInfo {
    /// The word
    pub word: String,

    /// Start time in seconds
    pub start: f32,

    /// End time in seconds
    pub end: f32,
}

impl From<TranscriptionResponse> for OpenAITranscriptionResponse {
    fn from(response: TranscriptionResponse) -> Self {
        let words = response.words.map(|words| {
            words
                .into_iter()
                .filter(|w| w.word_type == "word")
                .map(|w| OpenAIWordInfo {
                    word: w.text,
                    start: w.start,
                    end: w.end,
                })
                .collect()
        });

        OpenAITranscriptionResponse {
            text: response.text,
            task: "transcribe".to_string(),
            language: response
                .language_code
                .unwrap_or_else(|| "unknown".to_string()),
            words,
        }
    }
}

/// Create multipart form for audio upload
pub fn create_multipart_form(
    request: &TranscriptionRequest,
) -> Result<reqwest::multipart::Form, ProviderError> {
    use reqwest::multipart;

    let mut form = multipart::Form::new();

    // Detect content type from filename or default to audio/mpeg
    let (filename, mime_type) = match &request.filename {
        Some(name) => {
            let mime = detect_audio_mime_type(name);
            (name.clone(), mime)
        }
        None => ("audio.mp3".to_string(), "audio/mpeg"),
    };

    // Add audio file
    let file_part = multipart::Part::bytes(request.file.clone())
        .file_name(filename)
        .mime_str(mime_type)
        .map_err(|e| {
            ProviderError::invalid_request(PROVIDER_NAME, format!("Invalid MIME type: {}", e))
        })?;
    form = form.part("file", file_part);

    // Add model_id
    form = form.text("model_id", request.model_id.clone());

    // Add optional parameters
    if let Some(language_code) = &request.language_code {
        form = form.text("language_code", language_code.clone());
    }

    if let Some(temperature) = request.temperature {
        form = form.text("temperature", temperature.to_string());
    }

    Ok(form)
}

/// Detect MIME type from filename extension
fn detect_audio_mime_type(filename: &str) -> &'static str {
    let extension = filename
        .rsplit('.')
        .next()
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match extension.as_str() {
        "mp3" => "audio/mpeg",
        "mp4" | "m4a" => "audio/mp4",
        "wav" => "audio/wav",
        "webm" => "audio/webm",
        "ogg" | "oga" => "audio/ogg",
        "flac" => "audio/flac",
        _ => "audio/mpeg",
    }
}

/// Supported audio input formats
pub fn supported_audio_formats() -> &'static [&'static str] {
    &["mp3", "mp4", "m4a", "wav", "webm", "ogg", "flac"]
}

/// Maximum file size in bytes (100MB for ElevenLabs)
pub const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Available STT models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum STTModel {
    /// Scribe v1 - Main transcription model
    #[default]
    ScribeV1,
}

impl STTModel {
    /// Get the model ID string
    pub fn as_str(&self) -> &'static str {
        match self {
            STTModel::ScribeV1 => "scribe_v1",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "scribe_v1" => Some(STTModel::ScribeV1),
            _ => None,
        }
    }
}

/// Build the complete STT URL
pub fn build_stt_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    format!("{}{}", base, STT_ENDPOINT_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_audio_mime_type() {
        assert_eq!(detect_audio_mime_type("audio.mp3"), "audio/mpeg");
        assert_eq!(detect_audio_mime_type("audio.wav"), "audio/wav");
        assert_eq!(detect_audio_mime_type("audio.m4a"), "audio/mp4");
        assert_eq!(detect_audio_mime_type("audio.webm"), "audio/webm");
        assert_eq!(detect_audio_mime_type("audio.unknown"), "audio/mpeg");
    }

    #[test]
    fn test_stt_model_as_str() {
        assert_eq!(STTModel::ScribeV1.as_str(), "scribe_v1");
    }

    #[test]
    fn test_stt_model_from_str() {
        assert_eq!(STTModel::parse("scribe_v1"), Some(STTModel::ScribeV1));
        assert_eq!(STTModel::parse("unknown"), None);
    }

    #[test]
    fn test_build_stt_url() {
        let url = build_stt_url("https://api.elevenlabs.io");
        assert_eq!(url, "https://api.elevenlabs.io/v1/speech-to-text");
    }

    #[test]
    fn test_transcription_response_to_openai() {
        let response = TranscriptionResponse {
            text: "Hello world".to_string(),
            language_code: Some("en".to_string()),
            words: Some(vec![
                WordInfo {
                    text: "Hello".to_string(),
                    start: 0.0,
                    end: 0.5,
                    word_type: "word".to_string(),
                },
                WordInfo {
                    text: " ".to_string(),
                    start: 0.5,
                    end: 0.6,
                    word_type: "spacing".to_string(),
                },
                WordInfo {
                    text: "world".to_string(),
                    start: 0.6,
                    end: 1.0,
                    word_type: "word".to_string(),
                },
            ]),
        };

        let openai_response: OpenAITranscriptionResponse = response.into();
        assert_eq!(openai_response.text, "Hello world");
        assert_eq!(openai_response.task, "transcribe");
        assert_eq!(openai_response.language, "en");

        let words = openai_response.words.unwrap();
        assert_eq!(words.len(), 2); // Only "word" types, not "spacing"
        assert_eq!(words[0].word, "Hello");
        assert_eq!(words[1].word, "world");
    }

    #[test]
    fn test_supported_audio_formats() {
        let formats = supported_audio_formats();
        assert!(formats.contains(&"mp3"));
        assert!(formats.contains(&"wav"));
        assert!(formats.contains(&"webm"));
    }
}
