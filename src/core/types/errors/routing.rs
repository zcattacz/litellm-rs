//! Routing error types

/// Routing error types
#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("No healthy providers available")]
    NoHealthyProviders,

    #[error("No suitable provider found for request")]
    NoSuitableProvider,

    #[error("All providers failed")]
    AllProvidersFailed,

    #[error("Provider '{provider}' not found")]
    ProviderNotFound { provider: String },

    #[error("Invalid routing strategy: {strategy}")]
    InvalidStrategy { strategy: String },

    #[error("Route selection failed: {reason}")]
    SelectionFailed { reason: String },

    #[error("Circuit breaker is open for provider '{provider}'")]
    CircuitBreakerOpen { provider: String },

    #[error("Load balancing failed: {reason}")]
    LoadBalancingFailed { reason: String },
}

/// Result type alias
pub type RoutingResult<T> = Result<T, RoutingError>;

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Error Variant Tests ====================

    #[test]
    fn test_no_healthy_providers_error() {
        let err = RoutingError::NoHealthyProviders;
        assert!(err.to_string().contains("No healthy providers available"));
    }

    #[test]
    fn test_no_suitable_provider_error() {
        let err = RoutingError::NoSuitableProvider;
        assert!(err.to_string().contains("No suitable provider found"));
    }

    #[test]
    fn test_all_providers_failed_error() {
        let err = RoutingError::AllProvidersFailed;
        assert!(err.to_string().contains("All providers failed"));
    }

    #[test]
    fn test_provider_not_found_error() {
        let err = RoutingError::ProviderNotFound {
            provider: "openai".to_string(),
        };
        assert!(err.to_string().contains("openai"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_invalid_strategy_error() {
        let err = RoutingError::InvalidStrategy {
            strategy: "random_invalid".to_string(),
        };
        assert!(err.to_string().contains("random_invalid"));
        assert!(err.to_string().contains("Invalid routing strategy"));
    }

    #[test]
    fn test_selection_failed_error() {
        let err = RoutingError::SelectionFailed {
            reason: "No providers match the filter criteria".to_string(),
        };
        assert!(err.to_string().contains("No providers match"));
        assert!(err.to_string().contains("selection failed"));
    }

    #[test]
    fn test_circuit_breaker_open_error() {
        let err = RoutingError::CircuitBreakerOpen {
            provider: "anthropic".to_string(),
        };
        assert!(err.to_string().contains("anthropic"));
        assert!(err.to_string().contains("Circuit breaker is open"));
    }

    #[test]
    fn test_load_balancing_failed_error() {
        let err = RoutingError::LoadBalancingFailed {
            reason: "All weights are zero".to_string(),
        };
        assert!(err.to_string().contains("All weights are zero"));
        assert!(err.to_string().contains("Load balancing failed"));
    }

    // ==================== Debug Tests ====================

    #[test]
    fn test_no_healthy_providers_debug() {
        let err = RoutingError::NoHealthyProviders;
        let debug = format!("{:?}", err);
        assert!(debug.contains("NoHealthyProviders"));
    }

    #[test]
    fn test_provider_not_found_debug() {
        let err = RoutingError::ProviderNotFound {
            provider: "azure".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("ProviderNotFound"));
        assert!(debug.contains("azure"));
    }

    #[test]
    fn test_circuit_breaker_open_debug() {
        let err = RoutingError::CircuitBreakerOpen {
            provider: "gemini".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("CircuitBreakerOpen"));
    }

    // ==================== Result Type Tests ====================

    #[test]
    fn test_routing_result_ok() {
        let result: RoutingResult<String> = Ok("provider-1".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "provider-1");
    }

    #[test]
    fn test_routing_result_err() {
        let result: RoutingResult<String> = Err(RoutingError::NoHealthyProviders);
        assert!(result.is_err());
    }

    #[test]
    fn test_routing_result_map() {
        let result: RoutingResult<i32> = Ok(10);
        let mapped = result.map(|v| v + 5);
        assert_eq!(mapped.unwrap(), 15);
    }

    #[test]
    fn test_routing_result_and_then() {
        let result: RoutingResult<i32> = Ok(10);
        let chained = result.and_then(|v| {
            if v > 5 {
                Ok(v * 2)
            } else {
                Err(RoutingError::NoSuitableProvider)
            }
        });
        assert_eq!(chained.unwrap(), 20);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_provider_name() {
        let err = RoutingError::ProviderNotFound {
            provider: "".to_string(),
        };
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_empty_strategy_name() {
        let err = RoutingError::InvalidStrategy {
            strategy: "".to_string(),
        };
        assert!(err.to_string().contains("Invalid routing strategy"));
    }

    #[test]
    fn test_empty_reason() {
        let err = RoutingError::SelectionFailed {
            reason: "".to_string(),
        };
        assert!(err.to_string().contains("selection failed"));
    }

    #[test]
    fn test_special_characters_in_provider() {
        let err = RoutingError::ProviderNotFound {
            provider: "provider-name_v2.1".to_string(),
        };
        assert!(err.to_string().contains("provider-name_v2.1"));
    }

    #[test]
    fn test_unicode_in_reason() {
        let err = RoutingError::LoadBalancingFailed {
            reason: "负载均衡失败".to_string(),
        };
        assert!(err.to_string().contains("负载均衡失败"));
    }

    #[test]
    fn test_long_provider_name() {
        let long_name = "a".repeat(1000);
        let err = RoutingError::CircuitBreakerOpen {
            provider: long_name.clone(),
        };
        assert!(err.to_string().contains(&long_name));
    }

    #[test]
    fn test_multiline_reason() {
        let err = RoutingError::SelectionFailed {
            reason: "Line 1\nLine 2\nLine 3".to_string(),
        };
        assert!(err.to_string().contains("Line 1"));
        assert!(err.to_string().contains("Line 2"));
    }
}
