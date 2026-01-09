//! Types for metrics storage

use std::collections::HashMap;
use std::collections::VecDeque;

/// Consolidated metrics storage - single lock for all metrics
#[derive(Debug, Default)]
pub(super) struct MetricsStorage {
    pub(super) request: RequestMetricsStorage,
    pub(super) provider: ProviderMetricsStorage,
    pub(super) system: SystemMetricsStorage,
    pub(super) error: ErrorMetricsStorage,
    pub(super) performance: PerformanceMetricsStorage,
}

/// Storage for request metrics
#[derive(Debug, Default)]
pub(super) struct RequestMetricsStorage {
    pub(super) total_requests: u64,
    pub(super) response_times: VecDeque<f64>,
    pub(super) status_codes: HashMap<u16, u64>,
    pub(super) endpoints: HashMap<String, u64>,
    pub(super) last_minute_requests: VecDeque<std::time::Instant>,
}

/// Storage for provider metrics
#[derive(Debug, Default)]
pub(super) struct ProviderMetricsStorage {
    pub(super) total_requests: u64,
    pub(super) provider_requests: HashMap<String, u64>,
    pub(super) provider_response_times: HashMap<String, VecDeque<f64>>,
    pub(super) provider_errors: HashMap<String, u64>,
    pub(super) token_usage: HashMap<String, u64>,
    pub(super) costs: HashMap<String, f64>,
}

/// Storage for system metrics
#[derive(Debug, Default)]
pub(super) struct SystemMetricsStorage {
    pub(super) cpu_samples: VecDeque<f64>,
    pub(super) memory_samples: VecDeque<u64>,
    pub(super) disk_samples: VecDeque<u64>,
    pub(super) network_in_samples: VecDeque<u64>,
    pub(super) network_out_samples: VecDeque<u64>,
    pub(super) connection_samples: VecDeque<u32>,
}

/// Storage for error metrics
#[derive(Debug, Default)]
pub(super) struct ErrorMetricsStorage {
    pub(super) total_errors: u64,
    pub(super) error_types: HashMap<String, u64>,
    pub(super) error_endpoints: HashMap<String, u64>,
    pub(super) critical_errors: u64,
    pub(super) warnings: u64,
    pub(super) last_minute_errors: VecDeque<std::time::Instant>,
}

/// Storage for performance metrics
#[derive(Debug, Default)]
pub(super) struct PerformanceMetricsStorage {
    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,
    pub(super) db_query_times: VecDeque<f64>,
    pub(super) queue_depths: VecDeque<u32>,
    pub(super) throughput_samples: VecDeque<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    // ==================== MetricsStorage Tests ====================

    #[test]
    fn test_metrics_storage_default() {
        let storage = MetricsStorage::default();

        assert_eq!(storage.request.total_requests, 0);
        assert_eq!(storage.provider.total_requests, 0);
        assert_eq!(storage.error.total_errors, 0);
        assert_eq!(storage.performance.cache_hits, 0);
    }

    #[test]
    fn test_metrics_storage_debug() {
        let storage = MetricsStorage::default();
        let debug_str = format!("{:?}", storage);

        assert!(debug_str.contains("MetricsStorage"));
    }

    // ==================== RequestMetricsStorage Tests ====================

    #[test]
    fn test_request_metrics_storage_default() {
        let storage = RequestMetricsStorage::default();

        assert_eq!(storage.total_requests, 0);
        assert!(storage.response_times.is_empty());
        assert!(storage.status_codes.is_empty());
        assert!(storage.endpoints.is_empty());
        assert!(storage.last_minute_requests.is_empty());
    }

    #[test]
    fn test_request_metrics_storage_add_request() {
        let mut storage = RequestMetricsStorage::default();

        storage.total_requests += 1;
        storage.response_times.push_back(150.0);
        storage.status_codes.insert(200, 1);
        storage.endpoints.insert("/api/chat".to_string(), 1);
        storage.last_minute_requests.push_back(Instant::now());

        assert_eq!(storage.total_requests, 1);
        assert_eq!(storage.response_times.len(), 1);
        assert_eq!(storage.status_codes.get(&200), Some(&1));
    }

    #[test]
    fn test_request_metrics_storage_multiple_requests() {
        let mut storage = RequestMetricsStorage::default();

        for i in 0..100 {
            storage.total_requests += 1;
            storage.response_times.push_back(50.0 + i as f64);
            *storage.status_codes.entry(200).or_insert(0) += 1;
        }

        assert_eq!(storage.total_requests, 100);
        assert_eq!(storage.response_times.len(), 100);
        assert_eq!(storage.status_codes.get(&200), Some(&100));
    }

    #[test]
    fn test_request_metrics_storage_response_time_stats() {
        let mut storage = RequestMetricsStorage::default();

        storage.response_times.push_back(100.0);
        storage.response_times.push_back(200.0);
        storage.response_times.push_back(300.0);

        let sum: f64 = storage.response_times.iter().sum();
        let avg = sum / storage.response_times.len() as f64;

        assert!((avg - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_request_metrics_storage_status_code_distribution() {
        let mut storage = RequestMetricsStorage::default();

        storage.status_codes.insert(200, 80);
        storage.status_codes.insert(404, 10);
        storage.status_codes.insert(500, 5);
        storage.status_codes.insert(429, 5);

        let total: u64 = storage.status_codes.values().sum();
        assert_eq!(total, 100);

        let error_rate = (storage.status_codes.get(&500).unwrap_or(&0)
            + storage.status_codes.get(&429).unwrap_or(&0)) as f64
            / total as f64;
        assert!((error_rate - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_request_metrics_storage_debug() {
        let storage = RequestMetricsStorage::default();
        let debug_str = format!("{:?}", storage);

        assert!(debug_str.contains("RequestMetricsStorage"));
    }

    // ==================== ProviderMetricsStorage Tests ====================

    #[test]
    fn test_provider_metrics_storage_default() {
        let storage = ProviderMetricsStorage::default();

        assert_eq!(storage.total_requests, 0);
        assert!(storage.provider_requests.is_empty());
        assert!(storage.provider_errors.is_empty());
        assert!(storage.token_usage.is_empty());
        assert!(storage.costs.is_empty());
    }

    #[test]
    fn test_provider_metrics_storage_add_provider_request() {
        let mut storage = ProviderMetricsStorage::default();

        storage.total_requests += 1;
        *storage
            .provider_requests
            .entry("openai".to_string())
            .or_insert(0) += 1;
        storage
            .provider_response_times
            .entry("openai".to_string())
            .or_insert_with(VecDeque::new)
            .push_back(250.0);
        *storage.token_usage.entry("openai".to_string()).or_insert(0) += 1500;
        *storage.costs.entry("openai".to_string()).or_insert(0.0) += 0.03;

        assert_eq!(storage.provider_requests.get("openai"), Some(&1));
        assert_eq!(storage.token_usage.get("openai"), Some(&1500));
    }

    #[test]
    fn test_provider_metrics_storage_multiple_providers() {
        let mut storage = ProviderMetricsStorage::default();

        let providers = vec!["openai", "anthropic", "azure", "google"];
        for provider in &providers {
            *storage
                .provider_requests
                .entry(provider.to_string())
                .or_insert(0) += 10;
            *storage.token_usage.entry(provider.to_string()).or_insert(0) += 1000;
        }

        assert_eq!(storage.provider_requests.len(), 4);
        assert_eq!(storage.token_usage.len(), 4);
    }

    #[test]
    fn test_provider_metrics_storage_error_tracking() {
        let mut storage = ProviderMetricsStorage::default();

        storage.provider_errors.insert("openai".to_string(), 5);
        storage.provider_errors.insert("anthropic".to_string(), 2);
        storage.provider_requests.insert("openai".to_string(), 100);
        storage
            .provider_requests
            .insert("anthropic".to_string(), 50);

        let openai_error_rate = *storage.provider_errors.get("openai").unwrap_or(&0) as f64
            / *storage.provider_requests.get("openai").unwrap_or(&1) as f64;

        assert!((openai_error_rate - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_metrics_storage_cost_tracking() {
        let mut storage = ProviderMetricsStorage::default();

        storage.costs.insert("openai".to_string(), 10.50);
        storage.costs.insert("anthropic".to_string(), 8.25);
        storage.costs.insert("azure".to_string(), 5.00);

        let total_cost: f64 = storage.costs.values().sum();
        assert!((total_cost - 23.75).abs() < 0.01);
    }

    #[test]
    fn test_provider_metrics_storage_debug() {
        let storage = ProviderMetricsStorage::default();
        let debug_str = format!("{:?}", storage);

        assert!(debug_str.contains("ProviderMetricsStorage"));
    }

    // ==================== SystemMetricsStorage Tests ====================

    #[test]
    fn test_system_metrics_storage_default() {
        let storage = SystemMetricsStorage::default();

        assert!(storage.cpu_samples.is_empty());
        assert!(storage.memory_samples.is_empty());
        assert!(storage.disk_samples.is_empty());
        assert!(storage.network_in_samples.is_empty());
        assert!(storage.network_out_samples.is_empty());
        assert!(storage.connection_samples.is_empty());
    }

    #[test]
    fn test_system_metrics_storage_add_samples() {
        let mut storage = SystemMetricsStorage::default();

        storage.cpu_samples.push_back(45.5);
        storage.memory_samples.push_back(8_000_000_000);
        storage.disk_samples.push_back(100_000_000_000);
        storage.network_in_samples.push_back(1_000_000);
        storage.network_out_samples.push_back(500_000);
        storage.connection_samples.push_back(100);

        assert_eq!(storage.cpu_samples.len(), 1);
        assert_eq!(storage.memory_samples.len(), 1);
    }

    #[test]
    fn test_system_metrics_storage_rolling_window() {
        let mut storage = SystemMetricsStorage::default();
        let max_samples = 60;

        // Add more samples than the window size
        for i in 0..100 {
            storage.cpu_samples.push_back(i as f64);
            if storage.cpu_samples.len() > max_samples {
                storage.cpu_samples.pop_front();
            }
        }

        assert_eq!(storage.cpu_samples.len(), max_samples);
        assert_eq!(*storage.cpu_samples.front().unwrap() as u32, 40);
    }

    #[test]
    fn test_system_metrics_storage_cpu_average() {
        let mut storage = SystemMetricsStorage::default();

        storage.cpu_samples.push_back(30.0);
        storage.cpu_samples.push_back(50.0);
        storage.cpu_samples.push_back(70.0);

        let avg: f64 = storage.cpu_samples.iter().sum::<f64>() / storage.cpu_samples.len() as f64;
        assert!((avg - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_system_metrics_storage_debug() {
        let storage = SystemMetricsStorage::default();
        let debug_str = format!("{:?}", storage);

        assert!(debug_str.contains("SystemMetricsStorage"));
    }

    // ==================== ErrorMetricsStorage Tests ====================

    #[test]
    fn test_error_metrics_storage_default() {
        let storage = ErrorMetricsStorage::default();

        assert_eq!(storage.total_errors, 0);
        assert!(storage.error_types.is_empty());
        assert!(storage.error_endpoints.is_empty());
        assert_eq!(storage.critical_errors, 0);
        assert_eq!(storage.warnings, 0);
    }

    #[test]
    fn test_error_metrics_storage_add_error() {
        let mut storage = ErrorMetricsStorage::default();

        storage.total_errors += 1;
        *storage
            .error_types
            .entry("timeout".to_string())
            .or_insert(0) += 1;
        *storage
            .error_endpoints
            .entry("/api/chat".to_string())
            .or_insert(0) += 1;
        storage.last_minute_errors.push_back(Instant::now());

        assert_eq!(storage.total_errors, 1);
        assert_eq!(storage.error_types.get("timeout"), Some(&1));
    }

    #[test]
    fn test_error_metrics_storage_error_types() {
        let mut storage = ErrorMetricsStorage::default();

        storage.error_types.insert("timeout".to_string(), 10);
        storage.error_types.insert("rate_limit".to_string(), 5);
        storage.error_types.insert("auth".to_string(), 3);
        storage.error_types.insert("internal".to_string(), 2);

        let total: u64 = storage.error_types.values().sum();
        assert_eq!(total, 20);

        let most_common = storage
            .error_types
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(k, _)| k);
        assert_eq!(most_common, Some(&"timeout".to_string()));
    }

    #[test]
    fn test_error_metrics_storage_critical_vs_warnings() {
        let storage = ErrorMetricsStorage {
            critical_errors: 5,
            warnings: 50,
            total_errors: 55,
            ..Default::default()
        };

        let critical_ratio = storage.critical_errors as f64 / storage.total_errors as f64;
        assert!(critical_ratio < 0.1);
    }

    #[test]
    fn test_error_metrics_storage_debug() {
        let storage = ErrorMetricsStorage::default();
        let debug_str = format!("{:?}", storage);

        assert!(debug_str.contains("ErrorMetricsStorage"));
    }

    // ==================== PerformanceMetricsStorage Tests ====================

    #[test]
    fn test_performance_metrics_storage_default() {
        let storage = PerformanceMetricsStorage::default();

        assert_eq!(storage.cache_hits, 0);
        assert_eq!(storage.cache_misses, 0);
        assert!(storage.db_query_times.is_empty());
        assert!(storage.queue_depths.is_empty());
        assert!(storage.throughput_samples.is_empty());
    }

    #[test]
    fn test_performance_metrics_storage_cache_stats() {
        let storage = PerformanceMetricsStorage {
            cache_hits: 800,
            cache_misses: 200,
            ..Default::default()
        };

        let total = storage.cache_hits + storage.cache_misses;
        let hit_rate = storage.cache_hits as f64 / total as f64;

        assert!((hit_rate - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_metrics_storage_db_query_times() {
        let mut storage = PerformanceMetricsStorage::default();

        storage.db_query_times.push_back(5.0);
        storage.db_query_times.push_back(10.0);
        storage.db_query_times.push_back(15.0);

        let avg: f64 =
            storage.db_query_times.iter().sum::<f64>() / storage.db_query_times.len() as f64;
        assert!((avg - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_metrics_storage_throughput() {
        let mut storage = PerformanceMetricsStorage::default();

        storage.throughput_samples.push_back(100.0);
        storage.throughput_samples.push_back(150.0);
        storage.throughput_samples.push_back(120.0);

        let max = storage
            .throughput_samples
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((max - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_metrics_storage_queue_depths() {
        let mut storage = PerformanceMetricsStorage::default();

        storage.queue_depths.push_back(10);
        storage.queue_depths.push_back(20);
        storage.queue_depths.push_back(5);

        let max_depth = *storage.queue_depths.iter().max().unwrap();
        assert_eq!(max_depth, 20);
    }

    #[test]
    fn test_performance_metrics_storage_debug() {
        let storage = PerformanceMetricsStorage::default();
        let debug_str = format!("{:?}", storage);

        assert!(debug_str.contains("PerformanceMetricsStorage"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_metrics_workflow() {
        let mut storage = MetricsStorage::default();

        // Simulate a request
        storage.request.total_requests += 1;
        storage.request.response_times.push_back(150.0);
        *storage.request.status_codes.entry(200).or_insert(0) += 1;

        // Track provider
        storage.provider.total_requests += 1;
        *storage
            .provider
            .provider_requests
            .entry("openai".to_string())
            .or_insert(0) += 1;
        *storage
            .provider
            .token_usage
            .entry("openai".to_string())
            .or_insert(0) += 1000;
        *storage
            .provider
            .costs
            .entry("openai".to_string())
            .or_insert(0.0) += 0.02;

        // Track performance
        storage.performance.cache_misses += 1;
        storage.performance.db_query_times.push_back(5.0);

        assert_eq!(storage.request.total_requests, 1);
        assert_eq!(storage.provider.total_requests, 1);
        assert_eq!(storage.performance.cache_misses, 1);
    }

    #[test]
    fn test_metrics_aggregation() {
        let mut storage = MetricsStorage::default();

        // Add multiple requests
        for i in 0..100 {
            storage.request.total_requests += 1;
            storage
                .request
                .response_times
                .push_back(100.0 + (i % 50) as f64);

            if i % 10 == 0 {
                storage.error.total_errors += 1;
            }
        }

        let avg_response: f64 = storage.request.response_times.iter().sum::<f64>()
            / storage.request.response_times.len() as f64;

        let error_rate = storage.error.total_errors as f64 / storage.request.total_requests as f64;

        assert!(avg_response > 100.0);
        assert!((error_rate - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_comparison() {
        let mut storage = MetricsStorage::default();

        storage
            .provider
            .provider_requests
            .insert("openai".to_string(), 500);
        storage
            .provider
            .provider_requests
            .insert("anthropic".to_string(), 300);
        storage
            .provider
            .provider_requests
            .insert("azure".to_string(), 200);

        storage.provider.costs.insert("openai".to_string(), 15.0);
        storage.provider.costs.insert("anthropic".to_string(), 10.0);
        storage.provider.costs.insert("azure".to_string(), 8.0);

        // Find most used provider
        let most_used = storage
            .provider
            .provider_requests
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(k, _)| k);
        assert_eq!(most_used, Some(&"openai".to_string()));

        // Find cheapest provider (cost per request)
        let cheapest = storage
            .provider
            .provider_requests
            .iter()
            .filter_map(|(provider, requests)| {
                storage
                    .provider
                    .costs
                    .get(provider)
                    .map(|cost| (provider, cost / *requests as f64))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(k, _)| k);
        assert_eq!(cheapest, Some(&"openai".to_string())); // 15/500 = 0.03
    }
}
