//! Routing strategy types and definitions

use std::collections::HashMap;

/// Routing strategies for provider selection
#[derive(Debug, Clone, Default)]
pub enum RoutingStrategy {
    /// Round-robin selection
    #[default]
    RoundRobin,
    /// Least latency first
    LeastLatency,
    /// Least cost first
    LeastCost,
    /// Random selection
    Random,
    /// Weighted selection based on provider weights
    Weighted,
    /// Priority-based selection
    Priority,
    /// A/B testing with traffic split
    ABTest {
        /// Split ratio for A/B testing (0.0 to 1.0)
        split_ratio: f64,
    },
    /// Route to provider with lowest TPM/RPM usage
    UsageBased,
    /// Route to provider with fewest active concurrent requests
    LeastBusy,
    /// Custom strategy with user-defined logic
    Custom(String),
}

/// Usage metrics for a provider
#[derive(Debug, Clone, Default)]
pub struct ProviderUsage {
    /// Tokens per minute (TPM) usage
    pub tpm: u64,
    /// Requests per minute (RPM) usage
    pub rpm: u64,
    /// Active concurrent requests
    pub active_requests: usize,
    /// TPM limit (if known)
    pub tpm_limit: Option<u64>,
    /// RPM limit (if known)
    pub rpm_limit: Option<u64>,
}

impl ProviderUsage {
    /// Calculate usage percentage (0.0 to 1.0) based on limits
    pub fn usage_percentage(&self) -> f64 {
        let tpm_pct = self
            .tpm_limit
            .map(|limit| self.tpm as f64 / limit as f64)
            .unwrap_or(0.0);
        let rpm_pct = self
            .rpm_limit
            .map(|limit| self.rpm as f64 / limit as f64)
            .unwrap_or(0.0);

        // Return the higher of the two percentages
        tpm_pct.max(rpm_pct)
    }
}

/// Consolidated routing data for all strategies
#[derive(Debug, Default)]
pub(super) struct RoutingData {
    /// Provider weights for weighted strategy
    pub weights: HashMap<String, f64>,
    /// Provider latencies for latency-based routing
    pub latencies: HashMap<String, f64>,
    /// Provider costs for cost-based routing
    pub costs: HashMap<String, f64>,
    /// Provider priorities
    pub priorities: HashMap<String, u32>,
    /// Provider usage metrics for usage-based routing
    pub usage: HashMap<String, ProviderUsage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RoutingStrategy Tests ====================

    #[test]
    fn test_routing_strategy_default() {
        let strategy = RoutingStrategy::default();
        assert!(matches!(strategy, RoutingStrategy::RoundRobin));
    }

    #[test]
    fn test_routing_strategy_all_variants() {
        let strategies = vec![
            RoutingStrategy::RoundRobin,
            RoutingStrategy::LeastLatency,
            RoutingStrategy::LeastCost,
            RoutingStrategy::Random,
            RoutingStrategy::Weighted,
            RoutingStrategy::Priority,
            RoutingStrategy::ABTest { split_ratio: 0.5 },
            RoutingStrategy::UsageBased,
            RoutingStrategy::LeastBusy,
            RoutingStrategy::Custom("my_strategy".to_string()),
        ];

        assert_eq!(strategies.len(), 10);
    }

    #[test]
    fn test_routing_strategy_ab_test() {
        let strategy = RoutingStrategy::ABTest { split_ratio: 0.7 };

        if let RoutingStrategy::ABTest { split_ratio } = strategy {
            assert!((split_ratio - 0.7).abs() < f64::EPSILON);
        } else {
            panic!("Expected ABTest variant");
        }
    }

    #[test]
    fn test_routing_strategy_custom() {
        let strategy = RoutingStrategy::Custom("custom_logic".to_string());

        if let RoutingStrategy::Custom(name) = strategy {
            assert_eq!(name, "custom_logic");
        } else {
            panic!("Expected Custom variant");
        }
    }

    #[test]
    fn test_routing_strategy_clone() {
        let original = RoutingStrategy::LeastLatency;
        let cloned = original.clone();
        assert!(matches!(cloned, RoutingStrategy::LeastLatency));
    }

    #[test]
    fn test_routing_strategy_clone_ab_test() {
        let original = RoutingStrategy::ABTest { split_ratio: 0.3 };
        let cloned = original.clone();

        if let RoutingStrategy::ABTest { split_ratio } = cloned {
            assert!((split_ratio - 0.3).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_routing_strategy_debug() {
        let strategy = RoutingStrategy::LeastCost;
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("LeastCost"));
    }

    // ==================== ProviderUsage Tests ====================

    #[test]
    fn test_provider_usage_default() {
        let usage = ProviderUsage::default();

        assert_eq!(usage.tpm, 0);
        assert_eq!(usage.rpm, 0);
        assert_eq!(usage.active_requests, 0);
        assert!(usage.tpm_limit.is_none());
        assert!(usage.rpm_limit.is_none());
    }

    #[test]
    fn test_provider_usage_creation() {
        let usage = ProviderUsage {
            tpm: 5000,
            rpm: 100,
            active_requests: 10,
            tpm_limit: Some(100000),
            rpm_limit: Some(1000),
        };

        assert_eq!(usage.tpm, 5000);
        assert_eq!(usage.rpm, 100);
        assert_eq!(usage.active_requests, 10);
    }

    #[test]
    fn test_provider_usage_percentage_tpm_based() {
        let usage = ProviderUsage {
            tpm: 50000,
            rpm: 0,
            active_requests: 0,
            tpm_limit: Some(100000),
            rpm_limit: None,
        };

        let pct = usage.usage_percentage();
        assert!((pct - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_usage_percentage_rpm_based() {
        let usage = ProviderUsage {
            tpm: 0,
            rpm: 300,
            active_requests: 0,
            tpm_limit: None,
            rpm_limit: Some(1000),
        };

        let pct = usage.usage_percentage();
        assert!((pct - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_usage_percentage_max_of_both() {
        let usage = ProviderUsage {
            tpm: 30000,  // 30% of limit
            rpm: 800,    // 80% of limit
            active_requests: 5,
            tpm_limit: Some(100000),
            rpm_limit: Some(1000),
        };

        let pct = usage.usage_percentage();
        // Should return the higher percentage (RPM = 80%)
        assert!((pct - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_usage_percentage_no_limits() {
        let usage = ProviderUsage {
            tpm: 50000,
            rpm: 500,
            active_requests: 10,
            tpm_limit: None,
            rpm_limit: None,
        };

        let pct = usage.usage_percentage();
        assert!((pct - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_usage_percentage_at_limit() {
        let usage = ProviderUsage {
            tpm: 100000,
            rpm: 1000,
            active_requests: 10,
            tpm_limit: Some(100000),
            rpm_limit: Some(1000),
        };

        let pct = usage.usage_percentage();
        assert!((pct - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_usage_percentage_over_limit() {
        let usage = ProviderUsage {
            tpm: 150000, // 150% over limit
            rpm: 500,
            active_requests: 10,
            tpm_limit: Some(100000),
            rpm_limit: Some(1000),
        };

        let pct = usage.usage_percentage();
        assert!(pct > 1.0);
        assert!((pct - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_usage_clone() {
        let usage = ProviderUsage {
            tpm: 1000,
            rpm: 50,
            active_requests: 5,
            tpm_limit: Some(10000),
            rpm_limit: Some(500),
        };

        let cloned = usage.clone();
        assert_eq!(cloned.tpm, usage.tpm);
        assert_eq!(cloned.rpm, usage.rpm);
        assert_eq!(cloned.tpm_limit, usage.tpm_limit);
    }

    #[test]
    fn test_provider_usage_debug() {
        let usage = ProviderUsage {
            tpm: 1000,
            rpm: 50,
            active_requests: 3,
            tpm_limit: Some(10000),
            rpm_limit: None,
        };

        let debug_str = format!("{:?}", usage);
        assert!(debug_str.contains("ProviderUsage"));
        assert!(debug_str.contains("1000"));
    }

    // ==================== RoutingData Tests ====================

    #[test]
    fn test_routing_data_default() {
        let data = RoutingData::default();

        assert!(data.weights.is_empty());
        assert!(data.latencies.is_empty());
        assert!(data.costs.is_empty());
        assert!(data.priorities.is_empty());
        assert!(data.usage.is_empty());
    }

    #[test]
    fn test_routing_data_with_weights() {
        let mut data = RoutingData::default();
        data.weights.insert("openai".to_string(), 0.7);
        data.weights.insert("anthropic".to_string(), 0.3);

        assert_eq!(data.weights.len(), 2);
        assert_eq!(data.weights.get("openai"), Some(&0.7));
    }

    #[test]
    fn test_routing_data_with_latencies() {
        let mut data = RoutingData::default();
        data.latencies.insert("openai".to_string(), 150.0);
        data.latencies.insert("anthropic".to_string(), 200.0);
        data.latencies.insert("azure".to_string(), 100.0);

        // Find provider with lowest latency
        let best = data.latencies.iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k);

        assert_eq!(best, Some(&"azure".to_string()));
    }

    #[test]
    fn test_routing_data_with_costs() {
        let mut data = RoutingData::default();
        data.costs.insert("gpt-4".to_string(), 0.03);
        data.costs.insert("gpt-3.5".to_string(), 0.002);
        data.costs.insert("claude-3".to_string(), 0.015);

        let cheapest = data.costs.iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k);

        assert_eq!(cheapest, Some(&"gpt-3.5".to_string()));
    }

    #[test]
    fn test_routing_data_with_priorities() {
        let mut data = RoutingData::default();
        data.priorities.insert("primary".to_string(), 1);
        data.priorities.insert("secondary".to_string(), 2);
        data.priorities.insert("fallback".to_string(), 3);

        let highest_priority = data.priorities.iter()
            .min_by_key(|&(_, v)| v)
            .map(|(k, _)| k);

        assert_eq!(highest_priority, Some(&"primary".to_string()));
    }

    #[test]
    fn test_routing_data_with_usage() {
        let mut data = RoutingData::default();

        data.usage.insert("provider_a".to_string(), ProviderUsage {
            tpm: 80000,
            rpm: 800,
            active_requests: 50,
            tpm_limit: Some(100000),
            rpm_limit: Some(1000),
        });

        data.usage.insert("provider_b".to_string(), ProviderUsage {
            tpm: 20000,
            rpm: 200,
            active_requests: 10,
            tpm_limit: Some(100000),
            rpm_limit: Some(1000),
        });

        // Find provider with lowest usage
        let least_used = data.usage.iter()
            .min_by(|a, b| {
                a.1.usage_percentage().partial_cmp(&b.1.usage_percentage()).unwrap()
            })
            .map(|(k, _)| k);

        assert_eq!(least_used, Some(&"provider_b".to_string()));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_strategy_selection_workflow() {
        // Simulate selecting a provider based on strategy
        let mut data = RoutingData::default();

        // Setup provider data
        data.latencies.insert("fast".to_string(), 50.0);
        data.latencies.insert("slow".to_string(), 200.0);
        data.costs.insert("fast".to_string(), 0.05);
        data.costs.insert("slow".to_string(), 0.01);

        // For LeastLatency strategy
        let strategy = RoutingStrategy::LeastLatency;
        if matches!(strategy, RoutingStrategy::LeastLatency) {
            let selected = data.latencies.iter()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(k, _)| k.clone());
            assert_eq!(selected, Some("fast".to_string()));
        }

        // For LeastCost strategy
        let strategy = RoutingStrategy::LeastCost;
        if matches!(strategy, RoutingStrategy::LeastCost) {
            let selected = data.costs.iter()
                .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .map(|(k, _)| k.clone());
            assert_eq!(selected, Some("slow".to_string()));
        }
    }

    #[test]
    fn test_ab_test_split_ratios() {
        let ratios = vec![0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];

        for ratio in ratios {
            let strategy = RoutingStrategy::ABTest { split_ratio: ratio };
            if let RoutingStrategy::ABTest { split_ratio } = strategy {
                assert!(split_ratio >= 0.0 && split_ratio <= 1.0);
            }
        }
    }

    #[test]
    fn test_usage_based_provider_selection() {
        let providers = vec![
            ("openai", ProviderUsage {
                tpm: 90000,
                rpm: 900,
                active_requests: 50,
                tpm_limit: Some(100000),
                rpm_limit: Some(1000),
            }),
            ("anthropic", ProviderUsage {
                tpm: 30000,
                rpm: 300,
                active_requests: 15,
                tpm_limit: Some(100000),
                rpm_limit: Some(1000),
            }),
            ("azure", ProviderUsage {
                tpm: 50000,
                rpm: 500,
                active_requests: 25,
                tpm_limit: Some(100000),
                rpm_limit: Some(1000),
            }),
        ];

        let best = providers.iter()
            .min_by(|a, b| {
                a.1.usage_percentage().partial_cmp(&b.1.usage_percentage()).unwrap()
            })
            .map(|(name, _)| *name);

        assert_eq!(best, Some("anthropic"));
    }
}
