use crate::core::providers::unified_provider::ProviderError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::Level;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = ProviderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DEBUG" => Ok(LogLevel::Debug),
            "INFO" => Ok(LogLevel::Info),
            "WARN" | "WARNING" => Ok(LogLevel::Warn),
            "ERROR" => Ok(LogLevel::Error),
            _ => Err(ProviderError::InvalidRequest {
                provider: "unknown",
                message: format!("Invalid log level: {}", s),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub module: Option<String>,
    pub request_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            level: format!("{:?}", level).to_uppercase(),
            message,
            module: None,
            request_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_module(mut self, module: String) -> Self {
        self.module = Some(module);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== LogLevel Tests ====================

    #[test]
    fn test_log_level_debug() {
        let level = LogLevel::Debug;
        assert_eq!(level, LogLevel::Debug);
    }

    #[test]
    fn test_log_level_info() {
        let level = LogLevel::Info;
        assert_eq!(level, LogLevel::Info);
    }

    #[test]
    fn test_log_level_warn() {
        let level = LogLevel::Warn;
        assert_eq!(level, LogLevel::Warn);
    }

    #[test]
    fn test_log_level_error() {
        let level = LogLevel::Error;
        assert_eq!(level, LogLevel::Error);
    }

    #[test]
    fn test_log_level_clone() {
        let level = LogLevel::Info;
        let cloned = level.clone();
        assert_eq!(level, cloned);
    }

    #[test]
    fn test_log_level_debug_trait() {
        let level = LogLevel::Info;
        let debug_str = format!("{:?}", level);
        assert_eq!(debug_str, "Info");
    }

    #[test]
    fn test_log_level_from_str_debug() {
        let level: LogLevel = "DEBUG".parse().unwrap();
        assert_eq!(level, LogLevel::Debug);
    }

    #[test]
    fn test_log_level_from_str_info() {
        let level: LogLevel = "INFO".parse().unwrap();
        assert_eq!(level, LogLevel::Info);
    }

    #[test]
    fn test_log_level_from_str_warn() {
        let level: LogLevel = "WARN".parse().unwrap();
        assert_eq!(level, LogLevel::Warn);
    }

    #[test]
    fn test_log_level_from_str_warning() {
        let level: LogLevel = "WARNING".parse().unwrap();
        assert_eq!(level, LogLevel::Warn);
    }

    #[test]
    fn test_log_level_from_str_error() {
        let level: LogLevel = "ERROR".parse().unwrap();
        assert_eq!(level, LogLevel::Error);
    }

    #[test]
    fn test_log_level_from_str_lowercase() {
        let level: LogLevel = "info".parse().unwrap();
        assert_eq!(level, LogLevel::Info);
    }

    #[test]
    fn test_log_level_from_str_mixed_case() {
        let level: LogLevel = "WaRn".parse().unwrap();
        assert_eq!(level, LogLevel::Warn);
    }

    #[test]
    fn test_log_level_from_str_invalid() {
        let result: Result<LogLevel, _> = "INVALID".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_log_level_to_tracing_debug() {
        let level = LogLevel::Debug;
        let tracing_level: Level = level.into();
        assert_eq!(tracing_level, Level::DEBUG);
    }

    #[test]
    fn test_log_level_to_tracing_info() {
        let level = LogLevel::Info;
        let tracing_level: Level = level.into();
        assert_eq!(tracing_level, Level::INFO);
    }

    #[test]
    fn test_log_level_to_tracing_warn() {
        let level = LogLevel::Warn;
        let tracing_level: Level = level.into();
        assert_eq!(tracing_level, Level::WARN);
    }

    #[test]
    fn test_log_level_to_tracing_error() {
        let level = LogLevel::Error;
        let tracing_level: Level = level.into();
        assert_eq!(tracing_level, Level::ERROR);
    }

    // ==================== LogEntry Tests ====================

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new(LogLevel::Info, "Test message".to_string());

        assert_eq!(entry.level, "INFO");
        assert_eq!(entry.message, "Test message");
        assert!(entry.module.is_none());
        assert!(entry.request_id.is_none());
        assert!(entry.metadata.is_empty());
    }

    #[test]
    fn test_log_entry_new_debug() {
        let entry = LogEntry::new(LogLevel::Debug, "Debug message".to_string());
        assert_eq!(entry.level, "DEBUG");
    }

    #[test]
    fn test_log_entry_new_warn() {
        let entry = LogEntry::new(LogLevel::Warn, "Warning".to_string());
        assert_eq!(entry.level, "WARN");
    }

    #[test]
    fn test_log_entry_new_error() {
        let entry = LogEntry::new(LogLevel::Error, "Error occurred".to_string());
        assert_eq!(entry.level, "ERROR");
    }

    #[test]
    fn test_log_entry_with_module() {
        let entry =
            LogEntry::new(LogLevel::Info, "Test".to_string()).with_module("gateway".to_string());

        assert_eq!(entry.module, Some("gateway".to_string()));
    }

    #[test]
    fn test_log_entry_with_request_id() {
        let entry = LogEntry::new(LogLevel::Info, "Test".to_string())
            .with_request_id("req-123".to_string());

        assert_eq!(entry.request_id, Some("req-123".to_string()));
    }

    #[test]
    fn test_log_entry_with_metadata() {
        let entry = LogEntry::new(LogLevel::Info, "Test".to_string())
            .with_metadata("user_id".to_string(), serde_json::json!("user-456"));

        assert!(entry.metadata.contains_key("user_id"));
        assert_eq!(
            entry.metadata.get("user_id").unwrap(),
            &serde_json::json!("user-456")
        );
    }

    #[test]
    fn test_log_entry_with_multiple_metadata() {
        let entry = LogEntry::new(LogLevel::Info, "Test".to_string())
            .with_metadata("key1".to_string(), serde_json::json!("value1"))
            .with_metadata("key2".to_string(), serde_json::json!(123))
            .with_metadata("key3".to_string(), serde_json::json!(true));

        assert_eq!(entry.metadata.len(), 3);
    }

    #[test]
    fn test_log_entry_builder_chain() {
        let entry = LogEntry::new(LogLevel::Info, "Request completed".to_string())
            .with_module("http".to_string())
            .with_request_id("req-789".to_string())
            .with_metadata("status".to_string(), serde_json::json!(200))
            .with_metadata("duration_ms".to_string(), serde_json::json!(150));

        assert_eq!(entry.module, Some("http".to_string()));
        assert_eq!(entry.request_id, Some("req-789".to_string()));
        assert_eq!(entry.metadata.len(), 2);
    }

    #[test]
    fn test_log_entry_clone() {
        let entry =
            LogEntry::new(LogLevel::Info, "Test".to_string()).with_module("test".to_string());

        let cloned = entry.clone();
        assert_eq!(cloned.level, entry.level);
        assert_eq!(cloned.message, entry.message);
        assert_eq!(cloned.module, entry.module);
    }

    #[test]
    fn test_log_entry_debug() {
        let entry = LogEntry::new(LogLevel::Info, "Test".to_string());
        let debug_str = format!("{:?}", entry);

        assert!(debug_str.contains("LogEntry"));
        assert!(debug_str.contains("INFO"));
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry::new(LogLevel::Info, "Test".to_string())
            .with_module("api".to_string())
            .with_request_id("req-abc".to_string());

        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["level"], "INFO");
        assert_eq!(json["message"], "Test");
        assert_eq!(json["module"], "api");
    }

    #[test]
    fn test_log_entry_deserialization() {
        let json = r#"{
            "timestamp": "2024-01-01T00:00:00Z",
            "level": "ERROR",
            "message": "Something went wrong",
            "module": "handler",
            "request_id": "req-123",
            "metadata": {"error_code": 500}
        }"#;

        let entry: LogEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.level, "ERROR");
        assert_eq!(entry.message, "Something went wrong");
        assert_eq!(entry.module, Some("handler".to_string()));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_log_level_roundtrip() {
        let original = LogLevel::Warn;
        let string = format!("{:?}", original).to_uppercase();
        let parsed: LogLevel = string.parse().unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_all_log_levels_parse() {
        let levels = vec!["DEBUG", "INFO", "WARN", "WARNING", "ERROR"];

        for level_str in levels {
            let result: Result<LogLevel, _> = level_str.parse();
            assert!(result.is_ok(), "Failed to parse: {}", level_str);
        }
    }

    #[test]
    fn test_log_entry_timestamp_is_recent() {
        let before = chrono::Utc::now();
        let entry = LogEntry::new(LogLevel::Info, "Test".to_string());
        let after = chrono::Utc::now();

        assert!(entry.timestamp >= before);
        assert!(entry.timestamp <= after);
    }
}
