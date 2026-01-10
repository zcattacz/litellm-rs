//! NVIDIA NIM Provider
//!
//! NVIDIA NIM (NVIDIA Inference Microservices) provides optimized AI inference
//! for various models. This implementation provides access through NVIDIA's
//! OpenAI-compatible API.
//!
//! Reference: https://docs.api.nvidia.com/nim/reference

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Re-export main types for external use
pub use config::NvidiaNimConfig;
pub use error::NvidiaNimError;
pub use model_info::{NvidiaNimModel, get_model_info, get_supported_params};
pub use provider::NvidiaNimProvider;
