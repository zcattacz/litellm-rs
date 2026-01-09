//! Cerebras Provider
//!
//! Cerebras AI API integration supporting fast inference capabilities.
//! Cerebras uses an OpenAI-compatible API format.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;

pub use client::CerebrasClient;
pub use config::CerebrasConfig;
pub use error::CerebrasErrorMapper;
pub use models::{CerebrasModelRegistry, get_cerebras_registry};
pub use provider::CerebrasProvider;
