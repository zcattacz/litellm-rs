//! Novita AI Provider
//!
//! Novita AI provides OpenAI-compatible API for accessing various AI models.
//! This implementation wraps their API with proper authentication and LiteLLM source header.
//!
//! Reference: https://novita.ai/docs/guides/llm-api

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::NovitaConfig;
pub use error::NovitaError;
pub use model_info::{get_available_models, get_model_info};
pub use provider::NovitaProvider;
