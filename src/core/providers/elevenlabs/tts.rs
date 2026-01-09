//! Text-to-Speech (TTS) Module for ElevenLabs
//!
//! Provides text-to-speech capabilities using ElevenLabs' voice synthesis models.

use super::error::ElevenLabsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default output format for TTS
pub const DEFAULT_OUTPUT_FORMAT: &str = "mp3_44100_128";

/// TTS API endpoint path
pub const TTS_ENDPOINT_PATH: &str = "/v1/text-to-speech";

/// Voice mappings from OpenAI voice names to ElevenLabs voice IDs
pub fn get_voice_mappings() -> HashMap<&'static str, &'static str> {
    let mut mappings = HashMap::new();
    mappings.insert("alloy", "21m00Tcm4TlvDq8ikWAM"); // Rachel
    mappings.insert("amber", "5Q0t7uMcjvnagumLfvZi"); // Paul
    mappings.insert("ash", "AZnzlk1XvdvUeBnXmlld"); // Domi
    mappings.insert("august", "D38z5RcWu1voky8WS1ja"); // Fin
    mappings.insert("blue", "2EiwWnXFnvU5JabPnv8n"); // Clyde
    mappings.insert("coral", "9BWtsMINqrJLrRacOk9x"); // Aria
    mappings.insert("lily", "EXAVITQu4vr4xnSDxMaL"); // Sarah
    mappings.insert("onyx", "29vD33N1CtxCmqQRPOHJ"); // Drew
    mappings.insert("sage", "CwhRBWXzGAHq8TQ4Fs17"); // Roger
    mappings.insert("verse", "CYw3kZ02Hs0563khs1Fj"); // Dave
    mappings
}

/// Response format mappings from OpenAI to ElevenLabs
pub fn get_format_mappings() -> HashMap<&'static str, &'static str> {
    let mut mappings = HashMap::new();
    mappings.insert("mp3", "mp3_44100_128");
    mappings.insert("pcm", "pcm_44100");
    mappings.insert("opus", "opus_48000_128");
    // Note: ElevenLabs does not support WAV, AAC, or FLAC formats
    mappings
}

/// Voice settings for fine-tuning the speech output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VoiceSettings {
    /// Stability of the voice (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stability: Option<f32>,

    /// Similarity boost (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub similarity_boost: Option<f32>,

    /// Style (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<f32>,

    /// Use speaker boost
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_speaker_boost: Option<bool>,

    /// Speech speed (0.25 to 4.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,
}

/// Text-to-speech request
#[derive(Debug, Clone, Serialize)]
pub struct TextToSpeechRequest {
    /// Text to convert to speech
    pub text: String,

    /// Model ID for TTS
    /// Options: "eleven_monolingual_v1", "eleven_multilingual_v1", "eleven_multilingual_v2", "eleven_turbo_v2", "eleven_turbo_v2_5"
    pub model_id: String,

    /// Voice settings for fine-tuning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_settings: Option<VoiceSettings>,

    /// Pronunciation dictionary locators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronunciation_dictionary_locators: Option<Vec<PronunciationDictionaryLocator>>,

    /// Seed for deterministic generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,

    /// Previous text for context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_text: Option<String>,

    /// Next text for context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_text: Option<String>,

    /// Previous request IDs for context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_request_ids: Option<Vec<String>>,

    /// Next request IDs for context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_request_ids: Option<Vec<String>>,
}

/// Pronunciation dictionary locator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PronunciationDictionaryLocator {
    /// Dictionary ID
    pub pronunciation_dictionary_id: String,
    /// Version ID
    pub version_id: String,
}

/// Text-to-speech response (binary audio data)
#[derive(Debug)]
pub struct TextToSpeechResponse {
    /// Audio data bytes
    pub audio_data: Vec<u8>,

    /// Content type of the audio
    pub content_type: String,

    /// Character cost (if available from headers)
    pub character_cost: Option<u32>,

    /// Request ID from ElevenLabs
    pub request_id: Option<String>,
}

/// Available TTS models
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TTSModel {
    /// Monolingual English model (v1)
    MonolingualV1,
    /// Multilingual model (v1)
    MultilingualV1,
    /// Multilingual model (v2) - recommended for non-English
    MultilingualV2,
    /// Turbo model (v2) - faster, English optimized
    TurboV2,
    /// Turbo model (v2.5) - latest fast model
    TurboV2_5,
}

impl TTSModel {
    /// Get the model ID string
    pub fn as_str(&self) -> &'static str {
        match self {
            TTSModel::MonolingualV1 => "eleven_monolingual_v1",
            TTSModel::MultilingualV1 => "eleven_multilingual_v1",
            TTSModel::MultilingualV2 => "eleven_multilingual_v2",
            TTSModel::TurboV2 => "eleven_turbo_v2",
            TTSModel::TurboV2_5 => "eleven_turbo_v2_5",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "eleven_monolingual_v1" => Some(TTSModel::MonolingualV1),
            "eleven_multilingual_v1" => Some(TTSModel::MultilingualV1),
            "eleven_multilingual_v2" => Some(TTSModel::MultilingualV2),
            "eleven_turbo_v2" => Some(TTSModel::TurboV2),
            "eleven_turbo_v2_5" => Some(TTSModel::TurboV2_5),
            _ => None,
        }
    }
}

impl Default for TTSModel {
    fn default() -> Self {
        TTSModel::MultilingualV2
    }
}

/// Resolve voice ID from various input formats
pub fn resolve_voice_id(voice: &str) -> Result<String, ElevenLabsError> {
    let voice_mappings = get_voice_mappings();
    let normalized = voice.trim().to_lowercase();

    // Check if it's a known OpenAI voice name
    if let Some(&voice_id) = voice_mappings.get(normalized.as_str()) {
        return Ok(voice_id.to_string());
    }

    // Otherwise, assume it's a direct ElevenLabs voice ID
    if voice.trim().is_empty() {
        return Err(ElevenLabsError::InvalidRequestError(
            "Voice ID is required".to_string(),
        ));
    }

    Ok(voice.trim().to_string())
}

/// Map output format from OpenAI to ElevenLabs format
pub fn map_output_format(format: Option<&str>) -> &'static str {
    let format_mappings = get_format_mappings();

    match format {
        Some(f) => format_mappings
            .get(f)
            .copied()
            .unwrap_or(DEFAULT_OUTPUT_FORMAT),
        None => DEFAULT_OUTPUT_FORMAT,
    }
}

/// Build the complete TTS URL with voice ID and query parameters
pub fn build_tts_url(base_url: &str, voice_id: &str, output_format: Option<&str>) -> String {
    let base = base_url.trim_end_matches('/');
    let format = map_output_format(output_format);

    format!(
        "{}{}/{}?output_format={}",
        base, TTS_ENDPOINT_PATH, voice_id, format
    )
}

/// Supported audio output formats
pub fn supported_output_formats() -> &'static [&'static str] {
    &[
        "mp3_22050_32",
        "mp3_44100_32",
        "mp3_44100_64",
        "mp3_44100_96",
        "mp3_44100_128",
        "mp3_44100_192",
        "pcm_16000",
        "pcm_22050",
        "pcm_24000",
        "pcm_44100",
        "ulaw_8000",
        "opus_32000_24",
        "opus_48000_24",
        "opus_48000_64",
        "opus_48000_128",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_mappings() {
        let mappings = get_voice_mappings();
        assert_eq!(mappings.get("alloy"), Some(&"21m00Tcm4TlvDq8ikWAM"));
        assert_eq!(mappings.get("onyx"), Some(&"29vD33N1CtxCmqQRPOHJ"));
    }

    #[test]
    fn test_format_mappings() {
        let mappings = get_format_mappings();
        assert_eq!(mappings.get("mp3"), Some(&"mp3_44100_128"));
        assert_eq!(mappings.get("pcm"), Some(&"pcm_44100"));
        assert_eq!(mappings.get("opus"), Some(&"opus_48000_128"));
    }

    #[test]
    fn test_resolve_voice_id_openai_name() {
        let voice_id = resolve_voice_id("alloy").unwrap();
        assert_eq!(voice_id, "21m00Tcm4TlvDq8ikWAM");
    }

    #[test]
    fn test_resolve_voice_id_direct_id() {
        let voice_id = resolve_voice_id("custom-voice-id-123").unwrap();
        assert_eq!(voice_id, "custom-voice-id-123");
    }

    #[test]
    fn test_resolve_voice_id_empty() {
        let result = resolve_voice_id("");
        assert!(result.is_err());
    }

    #[test]
    fn test_map_output_format() {
        assert_eq!(map_output_format(Some("mp3")), "mp3_44100_128");
        assert_eq!(map_output_format(Some("pcm")), "pcm_44100");
        assert_eq!(map_output_format(None), "mp3_44100_128");
        assert_eq!(map_output_format(Some("unknown")), "mp3_44100_128");
    }

    #[test]
    fn test_build_tts_url() {
        let url = build_tts_url("https://api.elevenlabs.io", "voice-123", Some("mp3"));
        assert_eq!(
            url,
            "https://api.elevenlabs.io/v1/text-to-speech/voice-123?output_format=mp3_44100_128"
        );
    }

    #[test]
    fn test_tts_model_as_str() {
        assert_eq!(TTSModel::MultilingualV2.as_str(), "eleven_multilingual_v2");
        assert_eq!(TTSModel::TurboV2_5.as_str(), "eleven_turbo_v2_5");
    }

    #[test]
    fn test_tts_model_from_str() {
        assert_eq!(
            TTSModel::from_str("eleven_multilingual_v2"),
            Some(TTSModel::MultilingualV2)
        );
        assert_eq!(TTSModel::from_str("unknown"), None);
    }

    #[test]
    fn test_voice_settings_serialization() {
        let settings = VoiceSettings {
            stability: Some(0.5),
            similarity_boost: Some(0.75),
            speed: Some(1.2),
            ..Default::default()
        };

        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(json["stability"], 0.5);
        assert_eq!(json["similarity_boost"], 0.75);
        assert_eq!(json["speed"], 1.2);
    }
}
