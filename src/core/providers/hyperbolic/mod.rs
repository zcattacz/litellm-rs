//! Hyperbolic Provider
//!
//! Hyperbolic provides OpenAI-compatible API for accessing various AI models.
//! This implementation provides access to their models through their standard endpoints.
//!
//! Reference: <https://docs.hyperbolic.xyz/docs/rest-api>

// Core modules
mod config;
mod error;
mod model_info;
mod provider;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::HyperbolicConfig;
pub use error::HyperbolicError;
pub use model_info::{get_available_models, get_model_info};
pub use provider::HyperbolicProvider;
