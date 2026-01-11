//! NanoGPT Provider
//!
//! NanoGPT provides lightweight, efficient AI inference through their OpenAI-compatible API.

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::NanoGPTConfig;
pub use error::NanoGPTError;
pub use model_info::{NanoGPTModel, get_model_info};
pub use provider::NanoGPTProvider;
