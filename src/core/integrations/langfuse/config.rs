//! Langfuse Configuration
//!
//! Configuration for connecting to Langfuse LLMOps platform.

use serde::{Deserialize, Serialize};
use std::env;

/// Default Langfuse cloud host
fn default_host() -> String {
    "https://cloud.langfuse.com".to_string()
}

/// Default batch size for ingestion
fn default_batch_size() -> usize {
    10
}

/// Default flush interval in milliseconds
fn default_flush_interval_ms() -> u64 {
    1000
}

/// Default enabled state
fn default_enabled() -> bool {
    true
}

/// Langfuse configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangfuseConfig {
    /// Langfuse public key (LANGFUSE_PUBLIC_KEY)
    #[serde(default)]
    pub public_key: Option<String>,

    /// Langfuse secret key (LANGFUSE_SECRET_KEY)
    #[serde(default)]
    pub secret_key: Option<String>,

    /// Langfuse host URL (LANGFUSE_HOST)
    #[serde(default = "default_host")]
    pub host: String,

    /// Whether Langfuse integration is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Batch size for ingestion events
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Flush interval in milliseconds
    #[serde(default = "default_flush_interval_ms")]
    pub flush_interval_ms: u64,

    /// Debug mode - logs events instead of sending
    #[serde(default)]
    pub debug: bool,

    /// Optional release version tag
    #[serde(default)]
    pub release: Option<String>,
}

impl Default for LangfuseConfig {
    fn default() -> Self {
        Self {
            public_key: None,
            secret_key: None,
            host: default_host(),
            enabled: default_enabled(),
            batch_size: default_batch_size(),
            flush_interval_ms: default_flush_interval_ms(),
            debug: false,
            release: None,
        }
    }
}

impl LangfuseConfig {
    /// Create configuration from environment variables
    ///
    /// Reads:
    /// - LANGFUSE_PUBLIC_KEY
    /// - LANGFUSE_SECRET_KEY
    /// - LANGFUSE_HOST (optional, defaults to cloud.langfuse.com)
    /// - LANGFUSE_DEBUG (optional, defaults to false)
    /// - LANGFUSE_RELEASE (optional)
    pub fn from_env() -> Self {
        Self {
            public_key: env::var("LANGFUSE_PUBLIC_KEY").ok(),
            secret_key: env::var("LANGFUSE_SECRET_KEY").ok(),
            host: env::var("LANGFUSE_HOST").unwrap_or_else(|_| default_host()),
            enabled: env::var("LANGFUSE_ENABLED")
                .map(|v| v.to_lowercase() != "false" && v != "0")
                .unwrap_or(true),
            batch_size: env::var("LANGFUSE_BATCH_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_batch_size),
            flush_interval_ms: env::var("LANGFUSE_FLUSH_INTERVAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or_else(default_flush_interval_ms),
            debug: env::var("LANGFUSE_DEBUG")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            release: env::var("LANGFUSE_RELEASE").ok(),
        }
    }

    /// Check if configuration is valid for making API calls
    pub fn is_valid(&self) -> bool {
        self.enabled && self.public_key.is_some() && self.secret_key.is_some()
    }

    /// Get the ingestion API endpoint
    pub fn ingestion_endpoint(&self) -> String {
        format!("{}/api/public/ingestion", self.host.trim_end_matches('/'))
    }

    /// Get Basic auth header value
    pub fn auth_header(&self) -> Option<String> {
        match (&self.public_key, &self.secret_key) {
            (Some(public), Some(secret)) => {
                let credentials = format!("{}:{}", public, secret);
                let encoded = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    credentials.as_bytes(),
                );
                Some(format!("Basic {}", encoded))
            }
            _ => None,
        }
    }

    /// Merge with another config, preferring non-default values from other
    pub fn merge(mut self, other: Self) -> Self {
        if other.public_key.is_some() {
            self.public_key = other.public_key;
        }
        if other.secret_key.is_some() {
            self.secret_key = other.secret_key;
        }
        if other.host != default_host() {
            self.host = other.host;
        }
        if !other.enabled {
            self.enabled = other.enabled;
        }
        if other.batch_size != default_batch_size() {
            self.batch_size = other.batch_size;
        }
        if other.flush_interval_ms != default_flush_interval_ms() {
            self.flush_interval_ms = other.flush_interval_ms;
        }
        if other.debug {
            self.debug = other.debug;
        }
        if other.release.is_some() {
            self.release = other.release;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LangfuseConfig::default();
        assert!(config.enabled);
        assert!(config.public_key.is_none());
        assert!(config.secret_key.is_none());
        assert_eq!(config.host, "https://cloud.langfuse.com");
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.flush_interval_ms, 1000);
        assert!(!config.debug);
    }

    #[test]
    fn test_config_is_valid() {
        let mut config = LangfuseConfig::default();
        assert!(!config.is_valid());

        config.public_key = Some("pk-test".to_string());
        assert!(!config.is_valid());

        config.secret_key = Some("sk-test".to_string());
        assert!(config.is_valid());

        config.enabled = false;
        assert!(!config.is_valid());
    }

    #[test]
    fn test_ingestion_endpoint() {
        let config = LangfuseConfig::default();
        assert_eq!(
            config.ingestion_endpoint(),
            "https://cloud.langfuse.com/api/public/ingestion"
        );

        let custom = LangfuseConfig {
            host: "https://custom.langfuse.com/".to_string(),
            ..Default::default()
        };
        assert_eq!(
            custom.ingestion_endpoint(),
            "https://custom.langfuse.com/api/public/ingestion"
        );
    }

    #[test]
    fn test_auth_header() {
        let config = LangfuseConfig {
            public_key: Some("pk-test".to_string()),
            secret_key: Some("sk-test".to_string()),
            ..Default::default()
        };

        let auth = config.auth_header().unwrap();
        assert!(auth.starts_with("Basic "));

        // Decode and verify
        let encoded = auth.strip_prefix("Basic ").unwrap();
        let decoded = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            encoded,
        )
        .unwrap();
        let credentials = String::from_utf8(decoded).unwrap();
        assert_eq!(credentials, "pk-test:sk-test");
    }

    #[test]
    fn test_auth_header_missing_keys() {
        let config = LangfuseConfig::default();
        assert!(config.auth_header().is_none());

        let partial = LangfuseConfig {
            public_key: Some("pk-test".to_string()),
            ..Default::default()
        };
        assert!(partial.auth_header().is_none());
    }

    #[test]
    fn test_config_merge() {
        let base = LangfuseConfig::default();
        let other = LangfuseConfig {
            public_key: Some("pk-new".to_string()),
            secret_key: Some("sk-new".to_string()),
            host: "https://self-hosted.com".to_string(),
            batch_size: 50,
            ..Default::default()
        };

        let merged = base.merge(other);
        assert_eq!(merged.public_key, Some("pk-new".to_string()));
        assert_eq!(merged.secret_key, Some("sk-new".to_string()));
        assert_eq!(merged.host, "https://self-hosted.com");
        assert_eq!(merged.batch_size, 50);
    }

    #[test]
    fn test_config_serialization() {
        let config = LangfuseConfig {
            public_key: Some("pk-test".to_string()),
            secret_key: Some("sk-test".to_string()),
            host: "https://cloud.langfuse.com".to_string(),
            enabled: true,
            batch_size: 20,
            flush_interval_ms: 2000,
            debug: false,
            release: Some("v1.0.0".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("pk-test"));
        assert!(json.contains("v1.0.0"));

        let deserialized: LangfuseConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.public_key, config.public_key);
        assert_eq!(deserialized.batch_size, config.batch_size);
    }

    #[test]
    fn test_config_deserialization_with_defaults() {
        let json = r#"{"public_key": "pk-test", "secret_key": "sk-test"}"#;
        let config: LangfuseConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.public_key, Some("pk-test".to_string()));
        assert_eq!(config.host, "https://cloud.langfuse.com");
        assert_eq!(config.batch_size, 10);
        assert!(config.enabled);
    }
}
