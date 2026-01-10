//! Volcengine Provider (ByteDance Cloud AI Platform)
//!
//! Volcengine (火山引擎) is ByteDance's cloud AI platform providing
//! access to Doubao and other AI models.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;

pub use client::VolcengineClient;
pub use config::VolcengineConfig;
pub use error::VolcengineErrorMapper;
pub use models::{get_volcengine_registry, VolcengineModelRegistry};
pub use provider::VolcengineProvider;
