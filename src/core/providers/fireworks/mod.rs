//! Fireworks AI Provider
//!
//! Fireworks AI provides fast inference for open-source and custom models.
//! This implementation provides access to various models through Fireworks AI's
//! OpenAI-compatible API with support for function calling and response formatting.
//!
//! Reference: https://docs.fireworks.ai/api-reference/post-chatcompletions

mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use config::FireworksConfig;
pub use error::FireworksError;
pub use model_info::{FireworksModel, get_model_info, is_reasoning_model};
pub use provider::FireworksProvider;
