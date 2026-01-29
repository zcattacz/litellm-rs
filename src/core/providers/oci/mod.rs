//! Oracle Cloud Infrastructure (OCI) Generative AI Provider
//!
//! OCI Generative AI provides access to foundation models including Cohere Command
//! and Meta Llama models through Oracle Cloud Infrastructure.
//!
//! Supported features:
//! - Chat completions
//! - Streaming chat completions
//! - Tool calling (for supported models)
//!
//! References:
//! - API Docs: <https://docs.oracle.com/en-us/iaas/Content/generative-ai/home.htm>

// Core modules
mod config;
mod error;
mod model_info;
mod provider;
mod streaming;

// Re-export main types for external use
pub use config::OciConfig;
pub use error::{OciError, OciErrorMapper};
pub use model_info::{OciModel, get_available_models, get_model_info};
pub use provider::OciProvider;
