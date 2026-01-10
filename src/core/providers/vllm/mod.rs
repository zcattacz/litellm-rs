//! vLLM Provider
//!
//! vLLM is a high-throughput and memory-efficient inference engine for LLMs.
//! It provides an OpenAI-compatible API for serving various open-source models
//! with optimizations like PagedAttention and continuous batching.

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Feature modules
pub mod streaming;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::VLLMConfig;
pub use error::VLLMError;
pub use model_info::{VLLMModelInfo, get_model_info};
pub use provider::VLLMProvider;
