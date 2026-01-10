//! Oobabooga (text-generation-webui) Provider
//!
//! Oobabooga text-generation-webui provides a local web interface for running
//! large language models. This implementation supports chat completions and
//! embeddings through its OpenAI-compatible API.
//!
//! ## Features
//! - Chat completions with streaming support
//! - Embeddings generation
//! - OpenAI-compatible API format
//! - Token-based authentication support
//!
//! ## Configuration
//! The API base must be configured via `api_base` parameter or
//! `OOBABOOGA_API_BASE` environment variable.

// Core modules
mod config;
mod error;
mod provider;

// Re-export main types for external use
pub use config::OobaboogaConfig;
pub use error::OobaboogaError;
pub use provider::OobaboogaProvider;
