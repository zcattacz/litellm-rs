//! Nebius Provider (Nebius AI Cloud Platform)
//!
//! Nebius is an AI cloud platform providing access to various AI models
//! including Llama, Mistral, and Qwen series.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;

pub use client::NebiusClient;
pub use config::NebiusConfig;
pub use error::NebiusErrorMapper;
pub use models::{get_nebius_registry, NebiusModelRegistry};
pub use provider::NebiusProvider;
