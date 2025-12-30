//! Health status types and check results
//!
//! This module defines the core types for health monitoring including
//! health status levels and health check results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but degraded
    Degraded,
    /// Service is unhealthy but may recover
    Unhealthy,
    /// Service is completely unavailable
    Down,
}

impl HealthStatus {
    /// Check if the status allows requests
    pub fn allows_requests(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Get numeric score for routing (higher is better)
    pub fn score(&self) -> u32 {
        match self {
            HealthStatus::Healthy => 100,
            HealthStatus::Degraded => 70,
            HealthStatus::Unhealthy => 30,
            HealthStatus::Down => 0,
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Health status
    pub status: HealthStatus,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional details about the health check
    pub details: Option<String>,
    /// Error message if unhealthy
    pub error: Option<String>,
    /// Metrics collected during health check
    pub metrics: HashMap<String, f64>,
}

impl HealthCheckResult {
    /// Create a healthy result
    pub fn healthy(response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Healthy,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: None,
            error: None,
            metrics: HashMap::new(),
        }
    }

    /// Create an unhealthy result
    pub fn unhealthy(error: String, response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: None,
            error: Some(error),
            metrics: HashMap::new(),
        }
    }

    /// Create a degraded result
    pub fn degraded(reason: String, response_time_ms: u64) -> Self {
        Self {
            status: HealthStatus::Degraded,
            response_time_ms,
            timestamp: chrono::Utc::now(),
            details: Some(reason),
            error: None,
            metrics: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // ==================== HealthStatus Tests ====================

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::Healthy;
        assert!(status.allows_requests());
        assert_eq!(status.score(), 100);
    }

    #[test]
    fn test_health_status_degraded() {
        let status = HealthStatus::Degraded;
        assert!(status.allows_requests());
        assert_eq!(status.score(), 70);
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus::Unhealthy;
        assert!(!status.allows_requests());
        assert_eq!(status.score(), 30);
    }

    #[test]
    fn test_health_status_down() {
        let status = HealthStatus::Down;
        assert!(!status.allows_requests());
        assert_eq!(status.score(), 0);
    }

    #[test]
    fn test_health_status_equality() {
        let s1 = HealthStatus::Healthy;
        let s2 = HealthStatus::Healthy;
        let s3 = HealthStatus::Degraded;

        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_health_status_clone() {
        let status = HealthStatus::Degraded;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Healthy"));

        let parsed: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_all_variants_serialization() {
        let statuses = vec![
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
            HealthStatus::Down,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: HealthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_health_status_score_ordering() {
        assert!(HealthStatus::Healthy.score() > HealthStatus::Degraded.score());
        assert!(HealthStatus::Degraded.score() > HealthStatus::Unhealthy.score());
        assert!(HealthStatus::Unhealthy.score() > HealthStatus::Down.score());
    }

    // ==================== HealthCheckResult Tests ====================

    #[test]
    fn test_health_check_result_healthy() {
        let result = HealthCheckResult::healthy(50);

        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.response_time_ms, 50);
        assert!(result.error.is_none());
        assert!(result.details.is_none());
        assert!(result.metrics.is_empty());
    }

    #[test]
    fn test_health_check_result_unhealthy() {
        let result = HealthCheckResult::unhealthy("Connection refused".to_string(), 1000);

        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert_eq!(result.response_time_ms, 1000);
        assert!(result.error.is_some());
        assert_eq!(result.error.as_ref().unwrap(), "Connection refused");
        assert!(result.details.is_none());
    }

    #[test]
    fn test_health_check_result_degraded() {
        let result = HealthCheckResult::degraded("High latency detected".to_string(), 500);

        assert_eq!(result.status, HealthStatus::Degraded);
        assert_eq!(result.response_time_ms, 500);
        assert!(result.details.is_some());
        assert_eq!(result.details.as_ref().unwrap(), "High latency detected");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_health_check_result_timestamp() {
        let before = Utc::now();
        let result = HealthCheckResult::healthy(100);
        let after = Utc::now();

        assert!(result.timestamp >= before);
        assert!(result.timestamp <= after);
    }

    #[test]
    fn test_health_check_result_creation_with_full_fields() {
        let now = Utc::now();
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 45.5);
        metrics.insert("memory_usage".to_string(), 62.3);

        let result = HealthCheckResult {
            status: HealthStatus::Healthy,
            response_time_ms: 75,
            timestamp: now,
            details: Some("All systems operational".to_string()),
            error: None,
            metrics,
        };

        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.response_time_ms, 75);
        assert!(result.details.is_some());
        assert_eq!(result.metrics.len(), 2);
        assert_eq!(result.metrics.get("cpu_usage"), Some(&45.5));
    }

    #[test]
    fn test_health_check_result_clone() {
        let result = HealthCheckResult::healthy(100);
        let cloned = result.clone();

        assert_eq!(cloned.status, result.status);
        assert_eq!(cloned.response_time_ms, result.response_time_ms);
        assert_eq!(cloned.timestamp, result.timestamp);
    }

    #[test]
    fn test_health_check_result_serialization() {
        let result = HealthCheckResult::healthy(75);

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Healthy"));
        assert!(json.contains("75"));

        let parsed: HealthCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, HealthStatus::Healthy);
        assert_eq!(parsed.response_time_ms, 75);
    }

    #[test]
    fn test_health_check_result_unhealthy_serialization() {
        let result = HealthCheckResult::unhealthy("Timeout".to_string(), 5000);

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Unhealthy"));
        assert!(json.contains("Timeout"));

        let parsed: HealthCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, HealthStatus::Unhealthy);
        assert_eq!(parsed.error.as_ref().unwrap(), "Timeout");
    }

    #[test]
    fn test_health_check_result_with_metrics_serialization() {
        let mut metrics = HashMap::new();
        metrics.insert("latency_p99".to_string(), 150.0);

        let result = HealthCheckResult {
            status: HealthStatus::Degraded,
            response_time_ms: 200,
            timestamp: Utc::now(),
            details: Some("High p99 latency".to_string()),
            error: None,
            metrics,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("latency_p99"));
        assert!(json.contains("150"));

        let parsed: HealthCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.metrics.get("latency_p99"), Some(&150.0));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_health_check_result_allows_requests_check() {
        let healthy = HealthCheckResult::healthy(50);
        let degraded = HealthCheckResult::degraded("Slow".to_string(), 200);
        let unhealthy = HealthCheckResult::unhealthy("Error".to_string(), 500);

        assert!(healthy.status.allows_requests());
        assert!(degraded.status.allows_requests());
        assert!(!unhealthy.status.allows_requests());
    }

    #[test]
    fn test_health_check_routing_score_comparison() {
        let healthy = HealthCheckResult::healthy(50);
        let degraded = HealthCheckResult::degraded("High load".to_string(), 150);
        let unhealthy = HealthCheckResult::unhealthy("API error".to_string(), 1000);

        // In routing decisions, prefer healthy over degraded over unhealthy
        assert!(healthy.status.score() > degraded.status.score());
        assert!(degraded.status.score() > unhealthy.status.score());
    }

    #[test]
    fn test_health_check_results_collection() {
        let results = vec![
            HealthCheckResult::healthy(50),
            HealthCheckResult::healthy(60),
            HealthCheckResult::degraded("Slow".to_string(), 200),
            HealthCheckResult::unhealthy("Error".to_string(), 500),
        ];

        // Count healthy results
        let healthy_count = results.iter()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count();
        assert_eq!(healthy_count, 2);

        // Calculate average response time
        let avg_response: u64 = results.iter()
            .map(|r| r.response_time_ms)
            .sum::<u64>() / results.len() as u64;
        assert_eq!(avg_response, 202); // (50 + 60 + 200 + 500) / 4

        // Find results that allow requests
        let requestable = results.iter()
            .filter(|r| r.status.allows_requests())
            .count();
        assert_eq!(requestable, 3);
    }

    #[test]
    fn test_provider_health_simulation() {
        // Simulate health checks for multiple providers
        let provider_checks = vec![
            ("openai", HealthCheckResult::healthy(45)),
            ("anthropic", HealthCheckResult::healthy(55)),
            ("azure", HealthCheckResult::degraded("High latency".to_string(), 300)),
            ("offline-provider", HealthCheckResult::unhealthy("Connection refused".to_string(), 5000)),
        ];

        // Find best provider by score
        let best = provider_checks.iter()
            .max_by_key(|(_, result)| result.status.score())
            .unwrap();
        assert!(best.0 == "openai" || best.0 == "anthropic"); // Both are healthy with same score

        // Filter to only available providers
        let available: Vec<_> = provider_checks.iter()
            .filter(|(_, result)| result.status.allows_requests())
            .collect();
        assert_eq!(available.len(), 3);
    }

    #[test]
    fn test_health_status_transition_logic() {
        // Test typical health status transitions
        let transitions = vec![
            (HealthStatus::Healthy, HealthStatus::Degraded, true),   // Getting worse
            (HealthStatus::Degraded, HealthStatus::Unhealthy, true), // Getting worse
            (HealthStatus::Unhealthy, HealthStatus::Healthy, true),  // Recovery
            (HealthStatus::Down, HealthStatus::Healthy, true),       // Full recovery
        ];

        for (from, to, expected_change) in transitions {
            let score_changed = from.score() != to.score();
            assert_eq!(score_changed, expected_change);
        }
    }

    #[test]
    fn test_health_check_with_zero_response_time() {
        let result = HealthCheckResult::healthy(0);
        assert_eq!(result.response_time_ms, 0);
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_check_with_max_response_time() {
        let result = HealthCheckResult::unhealthy(
            "Extreme timeout".to_string(),
            u64::MAX,
        );
        assert_eq!(result.response_time_ms, u64::MAX);
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }
}
