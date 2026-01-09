//! ElevenLabs Provider
//!
//! ElevenLabs provides high-quality text-to-speech (TTS) and speech-to-text (STT) APIs.
//! This implementation provides access to their voice synthesis and audio transcription
//! capabilities through a unified provider interface.

// Core modules
mod config;
mod error;
mod provider;

// Feature modules
pub mod stt;
pub mod tts;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::ElevenLabsConfig;
pub use error::{ElevenLabsError, ElevenLabsErrorMapper};
pub use provider::ElevenLabsProvider;

// Re-export feature types
pub use stt::TranscriptionRequest;
pub use tts::TextToSpeechRequest;
