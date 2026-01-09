//! Perplexity Provider
//!
//! Perplexity AI provider with search-integrated chat completions.
//! Supports citations, search context, and web search options.

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod provider;
pub mod streaming;

pub use client::PerplexityClient;
pub use config::PerplexityConfig;
pub use error::PerplexityErrorMapper;
pub use models::{PerplexityModelRegistry, get_perplexity_registry};
pub use provider::PerplexityProvider;
