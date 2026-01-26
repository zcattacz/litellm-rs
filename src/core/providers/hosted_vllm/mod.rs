//! Hosted vLLM Provider
//!
//! Provider for vLLM hosted inference servers. vLLM is a high-throughput and
//! memory-efficient inference engine for LLMs that provides an OpenAI-compatible API.
//!
//! This provider connects to self-hosted vLLM instances and supports:
//! - Chat completions (streaming and non-streaming)
//! - Model listing from the vLLM server
//! - Tool/function calling (model dependent)
//! - Embeddings (if enabled on the vLLM server)
//!
//! # Example
//! ```rust,ignore
//! use litellm_rs::core::providers::hosted_vllm::{HostedVLLMProvider, HostedVLLMConfig};
//!
//! let config = HostedVLLMConfig::new("http://localhost:8000/v1");
//! let provider = HostedVLLMProvider::new(config).await?;
//! ```

// Core modules
mod config;
mod models;
mod provider;

// Feature modules
pub mod streaming;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::HostedVLLMConfig;
pub use models::{HostedVLLMModelInfo, get_model_info, get_or_create_model_info};
pub use provider::HostedVLLMProvider;
