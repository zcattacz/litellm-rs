//! Provider Registry - data-driven Tier 1 provider system
//!
//! Instead of maintaining 50+ separate provider implementations that are
//! essentially OpenAI-compatible with different base URLs, this module
//! defines them as static data entries in a catalog.
//!
//! A Tier 1 provider needs zero code — just a `ProviderDefinition` entry.

pub mod catalog;
pub mod definition;

pub use catalog::{PROVIDER_CATALOG, get_definition, is_tier1_provider};
pub use definition::{AuthType, ProviderDefinition};
