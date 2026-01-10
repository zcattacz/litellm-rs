//! LM Studio Provider
//!
//! LM Studio provides a local AI inference server with an OpenAI-compatible API.
//! This implementation supports chat completions (streaming and non-streaming),
//! and embeddings through its local server.
//!
//! ## Features
//! - Chat completions with streaming support
//! - Embeddings generation
//! - OpenAI-compatible API format
//! - JSON schema response format support
//! - Tool/function calling for supported models
//!
//! ## Configuration
//! The default API base is configurable via environment variable `LM_STUDIO_API_BASE`.
//! LM Studio does not require an API key for local usage.

// Core modules
mod config;
mod error;
mod provider;

// Re-export main types for external use
pub use config::LMStudioConfig;
pub use error::LMStudioError;
pub use provider::LMStudioProvider;
