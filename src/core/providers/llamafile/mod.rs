//! Llamafile Provider
//!
//! Llamafile provides single-file executables that contain both a model and
//! a local inference server. This implementation supports chat completions
//! through its OpenAI-compatible API.
//!
//! ## Features
//! - Chat completions with streaming support
//! - OpenAI-compatible API format
//! - No API key required (local execution)
//!
//! ## Configuration
//! The default API base is `http://127.0.0.1:8080/v1`.
//! Set `LLAMAFILE_API_BASE` environment variable to customize.

// Core modules
mod config;
mod error;
mod provider;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::LlamafileConfig;
pub use error::{LlamafileError, LlamafileErrorMapper};
pub use provider::LlamafileProvider;
