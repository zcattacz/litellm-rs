//! Custom HTTPX Provider Implementation
//!
//! A flexible provider for custom HTTP-based LLM endpoints

pub mod config;
pub mod error_mapper;
pub mod model_info;
pub mod provider;

pub use config::CustomHttpxConfig;
pub use error_mapper::CustomApiErrorMapper;
pub use provider::CustomHttpxProvider;

pub const PROVIDER_NAME: &str = "custom_httpx";
