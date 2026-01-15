//! Bytez AI Provider Implementation

pub mod config;
pub mod error_mapper;
pub mod model_info;
pub mod provider;

pub use config::BytezConfig;
pub use error_mapper::BytezErrorMapper;
pub use provider::BytezProvider;

pub const PROVIDER_NAME: &str = "bytez";
pub const DEFAULT_BASE_URL: &str = "https://api.bytez.com/v1";
