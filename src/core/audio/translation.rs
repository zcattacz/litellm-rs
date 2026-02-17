//! Audio translation functionality

use crate::core::providers::ProviderRegistry;
use crate::utils::error::gateway_error::{GatewayError, Result};
use std::sync::Arc;
use tracing::info;

use super::transcription::parse_model_string;
use super::types::{TranslationRequest, TranslationResponse};

/// Audio service for handling audio translation requests
pub struct TranslationService {
    provider_registry: Arc<ProviderRegistry>,
}

impl TranslationService {
    /// Create a new translation service
    pub fn new(provider_registry: Arc<ProviderRegistry>) -> Self {
        Self { provider_registry }
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

        // For translation, we use transcription with target language = English
        // Most providers use the same endpoint with different parameters
        let (provider_name, _actual_model) = parse_model_string(&request.model);

        let providers = self.provider_registry.all();
        let provider = providers
            .iter()
            .find(|p| p.name() == provider_name)
            .ok_or_else(|| {
                GatewayError::internal(format!(
                    "No provider found for audio translation: {}",
                    provider_name
                ))
            })?;

        Err(GatewayError::internal(format!(
            "Provider {} does not support audio translation",
            provider.name()
        )))
    }
}
