//! Configuration error types

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for field '{field}': {value}")]
    InvalidValue { field: String, value: String },

    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to read configuration file: {path}")]
    ReadError { path: String },

    #[error("Failed to parse configuration: {reason}")]
    ParseError { reason: String },

    #[error("Unsupported configuration format")]
    UnsupportedFormat,

    #[error("Configuration validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Environment variable error: {var}")]
    EnvVarError { var: String },
}

/// Result type alias
pub type ConfigResult<T> = Result<T, ConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Error Variant Tests ====================

    #[test]
    fn test_missing_field_error() {
        let err = ConfigError::MissingField {
            field: "api_key".to_string(),
        };
        assert!(err.to_string().contains("api_key"));
        assert!(err.to_string().contains("Missing required field"));
    }

    #[test]
    fn test_invalid_value_error() {
        let err = ConfigError::InvalidValue {
            field: "timeout".to_string(),
            value: "-1".to_string(),
        };
        assert!(err.to_string().contains("timeout"));
        assert!(err.to_string().contains("-1"));
        assert!(err.to_string().contains("Invalid value"));
    }

    #[test]
    fn test_file_not_found_error() {
        let err = ConfigError::FileNotFound {
            path: "/etc/config.yaml".to_string(),
        };
        assert!(err.to_string().contains("/etc/config.yaml"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_read_error() {
        let err = ConfigError::ReadError {
            path: "/etc/config.yaml".to_string(),
        };
        assert!(err.to_string().contains("/etc/config.yaml"));
        assert!(err.to_string().contains("Failed to read"));
    }

    #[test]
    fn test_parse_error() {
        let err = ConfigError::ParseError {
            reason: "Invalid YAML syntax at line 10".to_string(),
        };
        assert!(err.to_string().contains("Invalid YAML syntax"));
        assert!(err.to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_unsupported_format_error() {
        let err = ConfigError::UnsupportedFormat;
        assert!(err.to_string().contains("Unsupported configuration format"));
    }

    #[test]
    fn test_validation_failed_error() {
        let err = ConfigError::ValidationFailed {
            reason: "Port must be between 1 and 65535".to_string(),
        };
        assert!(err.to_string().contains("Port must be between"));
        assert!(err.to_string().contains("validation failed"));
    }

    #[test]
    fn test_env_var_error() {
        let err = ConfigError::EnvVarError {
            var: "OPENAI_API_KEY".to_string(),
        };
        assert!(err.to_string().contains("OPENAI_API_KEY"));
        assert!(err.to_string().contains("Environment variable"));
    }

    // ==================== Debug Tests ====================

    #[test]
    fn test_missing_field_debug() {
        let err = ConfigError::MissingField {
            field: "model".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("MissingField"));
        assert!(debug.contains("model"));
    }

    #[test]
    fn test_invalid_value_debug() {
        let err = ConfigError::InvalidValue {
            field: "rate".to_string(),
            value: "abc".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("InvalidValue"));
    }

    #[test]
    fn test_unsupported_format_debug() {
        let err = ConfigError::UnsupportedFormat;
        let debug = format!("{:?}", err);
        assert!(debug.contains("UnsupportedFormat"));
    }

    // ==================== Result Type Tests ====================

    #[test]
    fn test_config_result_ok() {
        let result: ConfigResult<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_config_result_err() {
        let result: ConfigResult<String> = Err(ConfigError::UnsupportedFormat);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_result_map() {
        let result: ConfigResult<i32> = Ok(42);
        let mapped = result.map(|v| v * 2);
        assert_eq!(mapped.unwrap(), 84);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_field_name() {
        let err = ConfigError::MissingField {
            field: "".to_string(),
        };
        assert!(err.to_string().contains("Missing required field"));
    }

    #[test]
    fn test_empty_path() {
        let err = ConfigError::FileNotFound {
            path: "".to_string(),
        };
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_empty_reason() {
        let err = ConfigError::ParseError {
            reason: "".to_string(),
        };
        assert!(err.to_string().contains("Failed to parse"));
    }

    #[test]
    fn test_special_characters_in_field() {
        let err = ConfigError::MissingField {
            field: "field.nested[0].value".to_string(),
        };
        assert!(err.to_string().contains("field.nested[0].value"));
    }

    #[test]
    fn test_special_characters_in_path() {
        let err = ConfigError::FileNotFound {
            path: "/path/with spaces/config.yaml".to_string(),
        };
        assert!(err.to_string().contains("/path/with spaces/config.yaml"));
    }

    #[test]
    fn test_unicode_in_error() {
        let err = ConfigError::ValidationFailed {
            reason: "配置验证失败".to_string(),
        };
        assert!(err.to_string().contains("配置验证失败"));
    }
}
