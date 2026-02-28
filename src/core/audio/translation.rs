//! Audio translation functionality

use crate::utils::error::gateway_error::{GatewayError, Result};
use tracing::info;

use super::types::{TranslationRequest, TranslationResponse};

/// Audio service for handling audio translation requests
pub struct TranslationService;

impl TranslationService {
    /// Create a new translation service
    pub fn new() -> Self {
        Self
    }

    /// Translate audio to English text
    pub async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        info!(
            "Translating audio: model={}, file_size={}",
            request.model,
            request.file.len()
        );

        // Validate file size (max 25MB)
        if request.file.len() > 25 * 1024 * 1024 {
            return Err(GatewayError::validation("Audio file too large (max 25MB)"));
        }

        Err(GatewayError::not_implemented(format!(
            "Audio translation is not implemented for model {}",
            request.model
        )))
    }
}
