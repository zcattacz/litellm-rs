//! Audio API module for speech-to-text and text-to-speech
//!
//! Provides unified audio processing capabilities across providers.

mod speech;
#[cfg(test)]
mod tests;
mod transcription;
mod translation;

// Make types module publicly accessible
pub mod types;

use crate::utils::error::gateway_error::Result;

// Internal service imports
use speech::SpeechService;
use transcription::TranscriptionService;
use translation::TranslationService;

// Import types for AudioService method signatures
use types::{
    SpeechRequest, SpeechResponse, TranscriptionRequest, TranscriptionResponse, TranslationRequest,
    TranslationResponse,
};

/// Audio service for handling audio API requests
pub struct AudioService {
    transcription_service: TranscriptionService,
    translation_service: TranslationService,
    speech_service: SpeechService,
}

impl AudioService {
    /// Create a new audio service
    pub fn new() -> Self {
        Self {
            transcription_service: TranscriptionService::new(),
            translation_service: TranslationService::new(),
            speech_service: SpeechService::new(),
        }
    }

    /// Transcribe audio to text
    pub async fn transcribe(&self, request: TranscriptionRequest) -> Result<TranscriptionResponse> {
        self.transcription_service.transcribe(request).await
    }

    /// Translate audio to English text
    pub async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        self.translation_service.translate(request).await
    }

    /// Convert text to speech
    pub async fn speech(&self, request: SpeechRequest) -> Result<SpeechResponse> {
        self.speech_service.speech(request).await
    }
}
