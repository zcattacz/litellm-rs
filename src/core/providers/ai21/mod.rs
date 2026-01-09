//! AI21 Provider
//!
//! AI21 Labs API integration supporting Jamba model family.
//! AI21 uses an OpenAI-compatible API format.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;

pub use client::AI21Client;
pub use config::AI21Config;
pub use error::AI21ErrorMapper;
pub use models::{AI21ModelRegistry, get_ai21_registry};
pub use provider::AI21Provider;
