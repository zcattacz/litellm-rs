//! Infinity Provider
//!
//! Infinity is a high-performance open-source embedding and reranking server.
//! This provider supports both embedding and reranking capabilities.
//!
//! Reference: https://infinity.modal.michaelfeil.eu/docs

// Core modules
mod config;
mod error;
mod provider;

// Tests
#[cfg(test)]
mod tests;

// Re-export main types for external use
pub use config::InfinityConfig;
pub use error::{InfinityError, InfinityErrorMapper};
pub use provider::InfinityProvider;
