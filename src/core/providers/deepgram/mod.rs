//! Deepgram Provider
//!
//! Deepgram provides advanced speech-to-text (STT) API with features like
//! diarization, punctuation, and language detection.
//! This implementation provides access to their audio transcription
//! capabilities through a unified provider interface.

// Core modules
mod config;
mod error;
mod provider;

// Feature modules
pub mod stt;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::DeepgramConfig;
pub use error::{DeepgramError, DeepgramErrorMapper};
pub use provider::DeepgramProvider;

// Re-export feature types
pub use stt::TranscriptionRequest;
