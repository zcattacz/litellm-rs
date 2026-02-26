//! Middleware configuration types

use super::defaults::*;
use super::rate_limit::RateLimitConfig;
use super::retry::RetryConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    /// Cache configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheConfig>,
    /// Retry configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
    /// Rate limit configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitConfig>,
    /// Auth configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    /// CORS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<CorsConfig>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CacheConfig {
    /// Memory cache
    #[serde(rename = "memory")]
    Memory {
        max_size: usize,
        #[serde(with = "super::duration_serde")]
        ttl: Duration,
    },
    /// Redis cache
    #[serde(rename = "redis")]
    Redis {
        url: String,
        #[serde(with = "super::duration_serde")]
        ttl: Duration,
        #[serde(default = "default_pool_size")]
        pool_size: u32,
    },
    /// Tiered cache
    #[serde(rename = "tiered")]
    Tiered {
        l1: Box<CacheConfig>,
        l2: Box<CacheConfig>,
        l3: Option<Box<CacheConfig>>,
    },
}

/// Auth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enabled authentication methods
    pub methods: Vec<AuthMethod>,
    /// JWT configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,
    /// API key configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<ApiKeyConfig>,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Jwt,
    ApiKey,
    Basic,
    Custom { handler: String },
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Signing key
    pub secret: String,
    /// Algorithm
    #[serde(default = "default_jwt_algorithm")]
    pub algorithm: String,
    /// Expiration time (seconds)
    #[serde(default = "default_jwt_expiration")]
    pub expiration_seconds: u64,
    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,
    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Header name
    #[serde(default = "default_api_key_header")]
    pub header_name: String,
    /// Prefix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Valid API keys
    #[serde(default)]
    pub valid_keys: Vec<String>,
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    #[serde(default = "default_cors_methods")]
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    #[serde(default = "default_cors_headers")]
    pub allowed_headers: Vec<String>,
    /// Allow credentials
    #[serde(default)]
    pub allow_credentials: bool,
    /// Maximum age (seconds)
    #[serde(default = "default_cors_max_age")]
    pub max_age_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== MiddlewareConfig Tests ====================

    #[test]
    fn test_middleware_config_creation() {
        let config = MiddlewareConfig {
            cache: None,
            retry: None,
            rate_limit: None,
            auth: None,
            cors: None,
        };
        assert!(config.cache.is_none());
        assert!(config.retry.is_none());
        assert!(config.rate_limit.is_none());
        assert!(config.auth.is_none());
        assert!(config.cors.is_none());
    }

    #[test]
    fn test_middleware_config_serialization_empty() {
        let config = MiddlewareConfig {
            cache: None,
            retry: None,
            rate_limit: None,
            auth: None,
            cors: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_middleware_config_deserialization_empty() {
        let json = r#"{}"#;
        let config: MiddlewareConfig = serde_json::from_str(json).unwrap();
        assert!(config.cache.is_none());
        assert!(config.auth.is_none());
    }

    // ==================== CacheConfig Tests ====================

    #[test]
    fn test_cache_config_memory() {
        let cache = CacheConfig::Memory {
            max_size: 1000,
            ttl: Duration::from_secs(300),
        };
        match cache {
            CacheConfig::Memory { max_size, ttl } => {
                assert_eq!(max_size, 1000);
                assert_eq!(ttl, Duration::from_secs(300));
            }
            _ => panic!("Expected Memory cache"),
        }
    }

    #[test]
    fn test_cache_config_redis() {
        let cache = CacheConfig::Redis {
            url: "redis://localhost:6379".to_string(),
            ttl: Duration::from_secs(600),
            pool_size: 10,
        };
        match cache {
            CacheConfig::Redis { url, pool_size, .. } => {
                assert_eq!(url, "redis://localhost:6379");
                assert_eq!(pool_size, 10);
            }
            _ => panic!("Expected Redis cache"),
        }
    }

    #[test]
    fn test_cache_config_tiered() {
        let l1 = CacheConfig::Memory {
            max_size: 100,
            ttl: Duration::from_secs(60),
        };
        let l2 = CacheConfig::Redis {
            url: "redis://localhost:6379".to_string(),
            ttl: Duration::from_secs(300),
            pool_size: 5,
        };
        let cache = CacheConfig::Tiered {
            l1: Box::new(l1),
            l2: Box::new(l2),
            l3: None,
        };
        assert!(matches!(cache, CacheConfig::Tiered { .. }));
    }

    #[test]
    fn test_cache_config_memory_serialization() {
        let cache = CacheConfig::Memory {
            max_size: 500,
            ttl: Duration::from_secs(120),
        };
        let json = serde_json::to_string(&cache).unwrap();
        assert!(json.contains("memory"));
        assert!(json.contains("500"));
    }

    #[test]
    fn test_cache_config_redis_serialization() {
        let cache = CacheConfig::Redis {
            url: "redis://127.0.0.1:6379".to_string(),
            ttl: Duration::from_secs(300),
            pool_size: 20,
        };
        let json = serde_json::to_string(&cache).unwrap();
        assert!(json.contains("redis"));
        assert!(json.contains("127.0.0.1"));
    }

    // ==================== AuthConfig Tests ====================

    #[test]
    fn test_auth_config_creation() {
        let config = AuthConfig {
            methods: vec![AuthMethod::Jwt, AuthMethod::ApiKey],
            jwt: None,
            api_key: None,
        };
        assert_eq!(config.methods.len(), 2);
    }

    #[test]
    fn test_auth_config_with_jwt() {
        let config = AuthConfig {
            methods: vec![AuthMethod::Jwt],
            jwt: Some(JwtConfig {
                secret: "my-secret".to_string(),
                algorithm: "HS256".to_string(),
                expiration_seconds: 3600,
                issuer: Some("my-app".to_string()),
                audience: None,
            }),
            api_key: None,
        };
        assert!(config.jwt.is_some());
        assert_eq!(config.jwt.unwrap().secret, "my-secret");
    }

    #[test]
    fn test_auth_config_with_api_key() {
        let config = AuthConfig {
            methods: vec![AuthMethod::ApiKey],
            jwt: None,
            api_key: Some(ApiKeyConfig {
                header_name: "X-API-Key".to_string(),
                prefix: Some("Bearer".to_string()),
                valid_keys: vec!["key1".to_string(), "key2".to_string()],
            }),
        };
        assert!(config.api_key.is_some());
        let api_key = config.api_key.unwrap();
        assert_eq!(api_key.header_name, "X-API-Key");
        assert_eq!(api_key.valid_keys.len(), 2);
    }

    #[test]
    fn test_auth_config_serialization() {
        let config = AuthConfig {
            methods: vec![AuthMethod::Basic],
            jwt: None,
            api_key: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("basic"));
    }

    // ==================== AuthMethod Tests ====================

    #[test]
    fn test_auth_method_jwt() {
        let method = AuthMethod::Jwt;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"jwt\"");
    }

    #[test]
    fn test_auth_method_api_key() {
        let method = AuthMethod::ApiKey;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"apikey\"");
    }

    #[test]
    fn test_auth_method_basic() {
        let method = AuthMethod::Basic;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"basic\"");
    }

    #[test]
    fn test_auth_method_custom() {
        let method = AuthMethod::Custom {
            handler: "my_handler".to_string(),
        };
        let json = serde_json::to_string(&method).unwrap();
        assert!(json.contains("custom"));
        assert!(json.contains("my_handler"));
    }

    #[test]
    fn test_auth_method_custom_deserialization() {
        let json = r#"{"custom": {"handler": "auth.CustomHandler"}}"#;
        let method: AuthMethod = serde_json::from_str(json).unwrap();
        match method {
            AuthMethod::Custom { handler } => {
                assert_eq!(handler, "auth.CustomHandler");
            }
            _ => panic!("Expected Custom"),
        }
    }

    // ==================== JwtConfig Tests ====================

    #[test]
    fn test_jwt_config_creation() {
        let config = JwtConfig {
            secret: "super-secret".to_string(),
            algorithm: "RS256".to_string(),
            expiration_seconds: 7200,
            issuer: Some("api.example.com".to_string()),
            audience: Some("client-app".to_string()),
        };
        assert_eq!(config.secret, "super-secret");
        assert_eq!(config.algorithm, "RS256");
        assert_eq!(config.expiration_seconds, 7200);
    }

    #[test]
    fn test_jwt_config_serialization() {
        let config = JwtConfig {
            secret: "test".to_string(),
            algorithm: "HS256".to_string(),
            expiration_seconds: 3600,
            issuer: None,
            audience: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("HS256"));
        assert!(json.contains("3600"));
        assert!(!json.contains("issuer"));
    }

    #[test]
    fn test_jwt_config_deserialization_with_defaults() {
        let json = r#"{"secret": "my-secret"}"#;
        let config: JwtConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.secret, "my-secret");
        assert_eq!(config.algorithm, "HS256");
        assert_eq!(config.expiration_seconds, 86400);
    }

    // ==================== ApiKeyConfig Tests ====================

    #[test]
    fn test_api_key_config_creation() {
        let config = ApiKeyConfig {
            header_name: "Authorization".to_string(),
            prefix: Some("Bearer".to_string()),
            valid_keys: vec!["key1".to_string()],
        };
        assert_eq!(config.header_name, "Authorization");
        assert_eq!(config.prefix, Some("Bearer".to_string()));
    }

    #[test]
    fn test_api_key_config_serialization() {
        let config = ApiKeyConfig {
            header_name: "X-API-Key".to_string(),
            prefix: None,
            valid_keys: vec!["abc123".to_string()],
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("X-API-Key"));
        assert!(!json.contains("prefix"));
    }

    #[test]
    fn test_api_key_config_deserialization_with_defaults() {
        let json = r#"{}"#;
        let config: ApiKeyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.header_name, "Authorization");
        assert!(config.prefix.is_none());
        assert!(config.valid_keys.is_empty());
    }

    // ==================== CorsConfig Tests ====================

    #[test]
    fn test_cors_config_creation() {
        let config = CorsConfig {
            allowed_origins: vec!["https://example.com".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string()],
            allow_credentials: true,
            max_age_seconds: 3600,
        };
        assert_eq!(config.allowed_origins.len(), 1);
        assert!(config.allow_credentials);
    }

    #[test]
    fn test_cors_config_serialization() {
        let config = CorsConfig {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string()],
            allowed_headers: vec![],
            allow_credentials: false,
            max_age_seconds: 7200,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("*"));
        assert!(json.contains("7200"));
    }

    #[test]
    fn test_cors_config_deserialization_with_defaults() {
        let json = r#"{"allowed_origins": ["https://example.com"]}"#;
        let config: CorsConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.allowed_origins.len(), 1);
        assert_eq!(config.allowed_methods.len(), 4);
        assert_eq!(config.allowed_headers.len(), 2);
        assert!(!config.allow_credentials);
        assert_eq!(config.max_age_seconds, 3600);
    }

    #[test]
    fn test_cors_config_default_methods() {
        let json = r#"{"allowed_origins": []}"#;
        let config: CorsConfig = serde_json::from_str(json).unwrap();
        assert!(config.allowed_methods.contains(&"GET".to_string()));
        assert!(config.allowed_methods.contains(&"POST".to_string()));
        assert!(config.allowed_methods.contains(&"PUT".to_string()));
        assert!(config.allowed_methods.contains(&"DELETE".to_string()));
    }

    #[test]
    fn test_cors_config_default_headers() {
        let json = r#"{"allowed_origins": []}"#;
        let config: CorsConfig = serde_json::from_str(json).unwrap();
        assert!(config.allowed_headers.contains(&"Content-Type".to_string()));
        assert!(
            config
                .allowed_headers
                .contains(&"Authorization".to_string())
        );
    }

    // ==================== Clone and Debug Tests ====================

    #[test]
    fn test_middleware_config_clone() {
        let config = MiddlewareConfig {
            cache: None,
            retry: None,
            rate_limit: None,
            auth: None,
            cors: None,
        };
        let cloned = config.clone();
        assert!(cloned.cache.is_none());
    }

    #[test]
    fn test_auth_config_clone() {
        let config = AuthConfig {
            methods: vec![AuthMethod::Jwt],
            jwt: None,
            api_key: None,
        };
        let cloned = config.clone();
        assert_eq!(cloned.methods.len(), 1);
    }

    #[test]
    fn test_cors_config_clone() {
        let config = CorsConfig {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string()],
            allowed_headers: vec![],
            allow_credentials: false,
            max_age_seconds: 3600,
        };
        let cloned = config.clone();
        assert_eq!(cloned.allowed_origins.len(), 1);
    }

    #[test]
    fn test_auth_method_debug() {
        let method = AuthMethod::Jwt;
        let debug = format!("{:?}", method);
        assert!(debug.contains("Jwt"));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_cors_config_empty_origins() {
        let config = CorsConfig {
            allowed_origins: vec![],
            allowed_methods: vec![],
            allowed_headers: vec![],
            allow_credentials: false,
            max_age_seconds: 0,
        };
        assert!(config.allowed_origins.is_empty());
        assert_eq!(config.max_age_seconds, 0);
    }

    #[test]
    fn test_api_key_config_empty_keys() {
        let config = ApiKeyConfig {
            header_name: "X-API-Key".to_string(),
            prefix: None,
            valid_keys: vec![],
        };
        assert!(config.valid_keys.is_empty());
    }

    #[test]
    fn test_jwt_config_zero_expiration() {
        let config = JwtConfig {
            secret: "test".to_string(),
            algorithm: "HS256".to_string(),
            expiration_seconds: 0,
            issuer: None,
            audience: None,
        };
        assert_eq!(config.expiration_seconds, 0);
    }

    #[test]
    fn test_cache_config_zero_size() {
        let cache = CacheConfig::Memory {
            max_size: 0,
            ttl: Duration::from_secs(0),
        };
        match cache {
            CacheConfig::Memory { max_size, ttl } => {
                assert_eq!(max_size, 0);
                assert_eq!(ttl, Duration::from_secs(0));
            }
            _ => panic!("Expected Memory cache"),
        }
    }

    #[test]
    fn test_auth_config_multiple_methods() {
        let config = AuthConfig {
            methods: vec![
                AuthMethod::Jwt,
                AuthMethod::ApiKey,
                AuthMethod::Basic,
                AuthMethod::Custom {
                    handler: "my_auth".to_string(),
                },
            ],
            jwt: None,
            api_key: None,
        };
        assert_eq!(config.methods.len(), 4);
    }
}
