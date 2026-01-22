//! Custom HTTPX Configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::core::providers::base::BaseConfig;
use crate::core::traits::provider::ProviderConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomHttpxConfig {
    #[serde(flatten)]
    pub base: BaseConfig,

    /// Custom endpoint URL
    pub endpoint_url: String,

    /// HTTP method (GET, POST, etc.)
    pub http_method: String,

    /// Request body template
    pub request_template: Option<String>,

    /// Response parser configuration
    pub response_parser: Option<String>,
}

impl Default for CustomHttpxConfig {
    fn default() -> Self {
        Self {
            base: BaseConfig {
                api_key: None,
                api_base: None,
                timeout: 60,
                max_retries: 3,
                headers: HashMap::new(),
                organization: None,
                api_version: None,
            },
            endpoint_url: String::new(),
            http_method: "POST".to_string(),
            request_template: None,
            response_parser: None,
        }
    }
}

impl CustomHttpxConfig {
    pub fn new(endpoint_url: impl Into<String>) -> Self {
        Self {
            endpoint_url: endpoint_url.into(),
            ..Self::default()
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let endpoint_url = std::env::var("CUSTOM_HTTPX_ENDPOINT")
            .map_err(|_| "CUSTOM_HTTPX_ENDPOINT environment variable is required")?;

        let mut config = Self::new(endpoint_url);

        if let Ok(api_key) = std::env::var("CUSTOM_HTTPX_API_KEY") {
            config.base.api_key = Some(api_key);
        }

        if let Ok(method) = std::env::var("CUSTOM_HTTPX_METHOD") {
            config.http_method = method;
        }

        Ok(config)
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.base.api_key = Some(api_key.into());
        self
    }

    pub fn with_http_method(mut self, method: impl Into<String>) -> Self {
        self.http_method = method.into();
        self
    }

    pub fn with_request_template(mut self, template: impl Into<String>) -> Self {
        self.request_template = Some(template.into());
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.base.headers.insert(key.into(), value.into());
        self
    }
}

impl ProviderConfig for CustomHttpxConfig {
    fn validate(&self) -> Result<(), String> {
        if self.endpoint_url.is_empty() {
            return Err("Endpoint URL is required".to_string());
        }

        if !self.endpoint_url.starts_with("http://") && !self.endpoint_url.starts_with("https://") {
            return Err("Endpoint URL must start with http:// or https://".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.base.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.base.api_base.as_deref()
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.base.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.base.max_retries
    }
}
