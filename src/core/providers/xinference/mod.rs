//! Xinference Provider
//!
//! Local LLM inference server provider.

pub mod config;
pub mod error;
pub mod model_info;
pub mod provider;

#[cfg(test)]
mod tests;

pub use config::XinferenceConfig;
pub use error::{XinferenceError, XinferenceErrorMapper};
pub use provider::XinferenceProvider;
