//! Deployment runtime metrics
//!
//! This module defines metrics tracking for deployment monitoring and observability.

use std::sync::atomic::{AtomicU32, AtomicU64};

/// Deployment runtime metrics
#[derive(Debug)]
pub struct DeploymentMetrics {
    /// Total requests processed
    pub total_requests: AtomicU64,
    /// Successful requests
    pub successful_requests: AtomicU64,
    /// Failed requests
    pub failed_requests: AtomicU64,
    /// Total tokens processed
    pub total_tokens: AtomicU64,
    /// Total cost incurred
    pub total_cost: parking_lot::RwLock<f64>,
    /// Active connections
    pub active_connections: AtomicU32,
    /// Queue size
    pub queue_size: AtomicU32,
    /// Last request timestamp
    pub last_request: AtomicU64,
    /// Request rate (requests per minute)
    pub request_rate: AtomicU32,
    /// Token rate (tokens per minute)
    pub token_rate: AtomicU32,
    /// Average response time
    pub avg_response_time: AtomicU64,
    /// P95 response time
    pub p95_response_time: AtomicU64,
    /// P99 response time
    pub p99_response_time: AtomicU64,
}

impl Default for DeploymentMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl DeploymentMetrics {
    /// Create new deployment metrics
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_cost: parking_lot::RwLock::new(0.0),
            active_connections: AtomicU32::new(0),
            queue_size: AtomicU32::new(0),
            last_request: AtomicU64::new(0),
            request_rate: AtomicU32::new(0),
            token_rate: AtomicU32::new(0),
            avg_response_time: AtomicU64::new(0),
            p95_response_time: AtomicU64::new(0),
            p99_response_time: AtomicU64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    // ==================== Construction Tests ====================

    #[test]
    fn test_deployment_metrics_new() {
        let metrics = DeploymentMetrics::new();

        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.successful_requests.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.failed_requests.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.total_tokens.load(Ordering::SeqCst), 0);
        assert_eq!(*metrics.total_cost.read(), 0.0);
        assert_eq!(metrics.active_connections.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.queue_size.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.last_request.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.request_rate.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.token_rate.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.avg_response_time.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.p95_response_time.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.p99_response_time.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_deployment_metrics_default() {
        let metrics = DeploymentMetrics::default();

        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.successful_requests.load(Ordering::SeqCst), 0);
    }

    // ==================== Atomic Counter Tests ====================

    #[test]
    fn test_deployment_metrics_total_requests_increment() {
        let metrics = DeploymentMetrics::new();

        metrics.total_requests.fetch_add(1, Ordering::SeqCst);
        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 1);

        metrics.total_requests.fetch_add(5, Ordering::SeqCst);
        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 6);
    }

    #[test]
    fn test_deployment_metrics_successful_requests() {
        let metrics = DeploymentMetrics::new();

        metrics.successful_requests.fetch_add(10, Ordering::SeqCst);
        assert_eq!(metrics.successful_requests.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_deployment_metrics_failed_requests() {
        let metrics = DeploymentMetrics::new();

        metrics.failed_requests.fetch_add(3, Ordering::SeqCst);
        assert_eq!(metrics.failed_requests.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_deployment_metrics_total_tokens() {
        let metrics = DeploymentMetrics::new();

        metrics.total_tokens.fetch_add(1000, Ordering::SeqCst);
        assert_eq!(metrics.total_tokens.load(Ordering::SeqCst), 1000);

        metrics.total_tokens.fetch_add(500, Ordering::SeqCst);
        assert_eq!(metrics.total_tokens.load(Ordering::SeqCst), 1500);
    }

    #[test]
    fn test_deployment_metrics_total_cost() {
        let metrics = DeploymentMetrics::new();

        {
            let mut cost = metrics.total_cost.write();
            *cost = 1.50;
        }
        assert_eq!(*metrics.total_cost.read(), 1.50);

        {
            let mut cost = metrics.total_cost.write();
            *cost += 0.75;
        }
        assert!(((*metrics.total_cost.read()) - 2.25).abs() < 0.001);
    }

    #[test]
    fn test_deployment_metrics_active_connections() {
        let metrics = DeploymentMetrics::new();

        metrics.active_connections.fetch_add(5, Ordering::SeqCst);
        assert_eq!(metrics.active_connections.load(Ordering::SeqCst), 5);

        metrics.active_connections.fetch_sub(2, Ordering::SeqCst);
        assert_eq!(metrics.active_connections.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_deployment_metrics_queue_size() {
        let metrics = DeploymentMetrics::new();

        metrics.queue_size.store(100, Ordering::SeqCst);
        assert_eq!(metrics.queue_size.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_deployment_metrics_last_request() {
        let metrics = DeploymentMetrics::new();

        let timestamp = 1704067200u64; // Example timestamp
        metrics.last_request.store(timestamp, Ordering::SeqCst);
        assert_eq!(metrics.last_request.load(Ordering::SeqCst), timestamp);
    }

    #[test]
    fn test_deployment_metrics_request_rate() {
        let metrics = DeploymentMetrics::new();

        metrics.request_rate.store(150, Ordering::SeqCst);
        assert_eq!(metrics.request_rate.load(Ordering::SeqCst), 150);
    }

    #[test]
    fn test_deployment_metrics_token_rate() {
        let metrics = DeploymentMetrics::new();

        metrics.token_rate.store(5000, Ordering::SeqCst);
        assert_eq!(metrics.token_rate.load(Ordering::SeqCst), 5000);
    }

    #[test]
    fn test_deployment_metrics_avg_response_time() {
        let metrics = DeploymentMetrics::new();

        metrics.avg_response_time.store(250, Ordering::SeqCst);
        assert_eq!(metrics.avg_response_time.load(Ordering::SeqCst), 250);
    }

    #[test]
    fn test_deployment_metrics_p95_response_time() {
        let metrics = DeploymentMetrics::new();

        metrics.p95_response_time.store(500, Ordering::SeqCst);
        assert_eq!(metrics.p95_response_time.load(Ordering::SeqCst), 500);
    }

    #[test]
    fn test_deployment_metrics_p99_response_time() {
        let metrics = DeploymentMetrics::new();

        metrics.p99_response_time.store(1000, Ordering::SeqCst);
        assert_eq!(metrics.p99_response_time.load(Ordering::SeqCst), 1000);
    }

    // ==================== Combined Operations Tests ====================

    #[test]
    fn test_deployment_metrics_simulate_request_success() {
        let metrics = DeploymentMetrics::new();

        // Simulate successful request
        metrics.total_requests.fetch_add(1, Ordering::SeqCst);
        metrics.successful_requests.fetch_add(1, Ordering::SeqCst);
        metrics.total_tokens.fetch_add(150, Ordering::SeqCst);
        {
            let mut cost = metrics.total_cost.write();
            *cost += 0.003;
        }

        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.successful_requests.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.failed_requests.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.total_tokens.load(Ordering::SeqCst), 150);
    }

    #[test]
    fn test_deployment_metrics_simulate_request_failure() {
        let metrics = DeploymentMetrics::new();

        // Simulate failed request
        metrics.total_requests.fetch_add(1, Ordering::SeqCst);
        metrics.failed_requests.fetch_add(1, Ordering::SeqCst);

        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.successful_requests.load(Ordering::SeqCst), 0);
        assert_eq!(metrics.failed_requests.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_deployment_metrics_simulate_multiple_requests() {
        let metrics = DeploymentMetrics::new();

        // Simulate 10 requests: 8 success, 2 failures
        for _ in 0..8 {
            metrics.total_requests.fetch_add(1, Ordering::SeqCst);
            metrics.successful_requests.fetch_add(1, Ordering::SeqCst);
            metrics.total_tokens.fetch_add(100, Ordering::SeqCst);
        }
        for _ in 0..2 {
            metrics.total_requests.fetch_add(1, Ordering::SeqCst);
            metrics.failed_requests.fetch_add(1, Ordering::SeqCst);
        }

        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 10);
        assert_eq!(metrics.successful_requests.load(Ordering::SeqCst), 8);
        assert_eq!(metrics.failed_requests.load(Ordering::SeqCst), 2);
        assert_eq!(metrics.total_tokens.load(Ordering::SeqCst), 800);
    }

    #[test]
    fn test_deployment_metrics_connection_tracking() {
        let metrics = DeploymentMetrics::new();

        // Simulate connection lifecycle
        metrics.active_connections.fetch_add(1, Ordering::SeqCst);
        metrics.active_connections.fetch_add(1, Ordering::SeqCst);
        metrics.active_connections.fetch_add(1, Ordering::SeqCst);
        assert_eq!(metrics.active_connections.load(Ordering::SeqCst), 3);

        metrics.active_connections.fetch_sub(1, Ordering::SeqCst);
        assert_eq!(metrics.active_connections.load(Ordering::SeqCst), 2);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_deployment_metrics_large_values() {
        let metrics = DeploymentMetrics::new();

        let large_value = u64::MAX / 2;
        metrics.total_tokens.store(large_value, Ordering::SeqCst);
        assert_eq!(metrics.total_tokens.load(Ordering::SeqCst), large_value);
    }

    #[test]
    fn test_deployment_metrics_cost_precision() {
        let metrics = DeploymentMetrics::new();

        {
            let mut cost = metrics.total_cost.write();
            *cost = 0.000001;
        }
        assert!((*metrics.total_cost.read() - 0.000001).abs() < 1e-10);
    }

    #[test]
    fn test_deployment_metrics_debug() {
        let metrics = DeploymentMetrics::new();
        let debug_str = format!("{:?}", metrics);

        assert!(debug_str.contains("DeploymentMetrics"));
        assert!(debug_str.contains("total_requests"));
    }

    // ==================== Thread Safety Tests ====================

    #[test]
    fn test_deployment_metrics_concurrent_increments() {
        use std::sync::Arc;
        use std::thread;

        let metrics = Arc::new(DeploymentMetrics::new());
        let mut handles = vec![];

        // Spawn 10 threads, each incrementing 100 times
        for _ in 0..10 {
            let m = Arc::clone(&metrics);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    m.total_requests.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(metrics.total_requests.load(Ordering::SeqCst), 1000);
    }

    #[test]
    fn test_deployment_metrics_concurrent_cost_updates() {
        use std::sync::Arc;
        use std::thread;

        let metrics = Arc::new(DeploymentMetrics::new());
        let mut handles = vec![];

        // Spawn 10 threads, each adding 0.1 to cost 10 times
        for _ in 0..10 {
            let m = Arc::clone(&metrics);
            handles.push(thread::spawn(move || {
                for _ in 0..10 {
                    let mut cost = m.total_cost.write();
                    *cost += 0.1;
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should be approximately 10.0 (10 threads * 10 iterations * 0.1)
        let final_cost = *metrics.total_cost.read();
        assert!((final_cost - 10.0).abs() < 0.001);
    }
}
