//! External Integrations
//!
//! This module provides integrations with external LLMOps platforms and services.
//!
//! # Available Integrations
//!
//! ## Langfuse
//!
//! Open-source LLMOps platform for tracing, evaluation, and analytics.
//!
//! ```rust,ignore
//! use litellm_rs::core::integrations::langfuse::{LangfuseLogger, LlmRequest};
//!
//! let logger = LangfuseLogger::from_env()?;
//! logger.on_llm_start(LlmRequest::new("req-id", "gpt-4"));
//! ```
//!
//! See the [`langfuse`] module for detailed documentation.

pub mod langfuse;

// Re-export commonly used types
pub use langfuse::{
    LangfuseConfig, LangfuseLogger, LangfuseTracing, LlmCallback, LlmError, LlmRequest, LlmResponse,
};
