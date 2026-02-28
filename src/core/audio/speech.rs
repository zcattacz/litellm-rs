//! Text-to-speech functionality

use crate::utils::error::gateway_error::{GatewayError, Result};
use tracing::info;

use super::types::{SpeechRequest, SpeechResponse};

/// Audio service for handling text-to-speech requests
pub struct SpeechService;

impl SpeechService {
    /// Create a new speech service
    pub fn new() -> Self {
        Self
    }

    /// Convert text to speech
    pub async fn speech(&self, request: SpeechRequest) -> Result<SpeechResponse> {
        info!(
            "Generating speech: model={}, voice={}, text_len={}",
            request.model,
            request.voice,
            request.input.len()
        );

        // Validate input length (max 4096 characters for most providers)
        if request.input.len() > 4096 {
            return Err(GatewayError::validation(
                "Input text too long (max 4096 characters)",
            ));
        }

        Err(GatewayError::not_implemented(format!(
            "Text-to-speech is not implemented for model {}",
            request.model
        )))
    }
}
