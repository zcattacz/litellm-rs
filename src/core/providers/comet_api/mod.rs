//! Comet API Provider Implementation

pub mod config;
pub mod error_mapper;
pub mod model_info;
pub mod provider;

pub use config::CometApiConfig;
pub use error_mapper::CometApiErrorMapper;
pub use provider::CometApiProvider;

pub const PROVIDER_NAME: &str = "cometapi";
pub const DEFAULT_BASE_URL: &str = "https://api.comet.com/v1";
