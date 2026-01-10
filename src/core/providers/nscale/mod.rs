//! Nscale Provider (Nscale AI Inference Platform)
//!
//! Nscale is an AI inference platform providing access to various
//! AI models with fast inference and competitive pricing.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;

pub use client::NscaleClient;
pub use config::NscaleConfig;
pub use error::NscaleErrorMapper;
pub use models::{get_nscale_registry, NscaleModelRegistry};
pub use provider::NscaleProvider;
