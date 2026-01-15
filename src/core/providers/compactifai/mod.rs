//! CompactifAI Provider Implementation

pub mod config;
pub mod error_mapper;
pub mod model_info;
pub mod provider;

pub use config::CompactifaiConfig;
pub use error_mapper::CompactifAiErrorMapper;
pub use provider::CompactifaiProvider;

pub const PROVIDER_NAME: &str = "compactifai";
pub const DEFAULT_BASE_URL: &str = "https://api.compactif.ai/v1";
