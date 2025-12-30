//! Rate limiter types and data structures

use std::time::Instant;

/// Rate limit result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Current request count in the window
    pub current_count: u32,
    /// Maximum requests allowed
    pub limit: u32,
    /// Remaining requests in the window
    pub remaining: u32,
    /// Time until the window resets (in seconds)
    pub reset_after_secs: u64,
    /// Retry after (in seconds, only set when not allowed)
    pub retry_after_secs: Option<u64>,
}

/// Rate limit entry for tracking request counts
#[derive(Debug, Clone)]
pub(super) struct RateLimitEntry {
    /// Request timestamps for sliding window
    pub(super) timestamps: Vec<Instant>,
    /// Token count for token bucket
    pub(super) tokens: f64,
    /// Last token refill time
    pub(super) last_refill: Instant,
}

impl Default for RateLimitEntry {
    fn default() -> Self {
        Self {
            timestamps: Vec::new(),
            tokens: 0.0,
            last_refill: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RateLimitResult Tests ====================

    #[test]
    fn test_rate_limit_result_allowed() {
        let result = RateLimitResult {
            allowed: true,
            current_count: 5,
            limit: 100,
            remaining: 95,
            reset_after_secs: 60,
            retry_after_secs: None,
        };

        assert!(result.allowed);
        assert_eq!(result.current_count, 5);
        assert_eq!(result.remaining, 95);
        assert!(result.retry_after_secs.is_none());
    }

    #[test]
    fn test_rate_limit_result_denied() {
        let result = RateLimitResult {
            allowed: false,
            current_count: 100,
            limit: 100,
            remaining: 0,
            reset_after_secs: 30,
            retry_after_secs: Some(30),
        };

        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
        assert_eq!(result.retry_after_secs, Some(30));
    }

    #[test]
    fn test_rate_limit_result_clone() {
        let result = RateLimitResult {
            allowed: true,
            current_count: 10,
            limit: 50,
            remaining: 40,
            reset_after_secs: 120,
            retry_after_secs: None,
        };

        let cloned = result.clone();
        assert_eq!(cloned.allowed, result.allowed);
        assert_eq!(cloned.current_count, result.current_count);
        assert_eq!(cloned.remaining, result.remaining);
    }

    #[test]
    fn test_rate_limit_result_debug() {
        let result = RateLimitResult {
            allowed: true,
            current_count: 1,
            limit: 10,
            remaining: 9,
            reset_after_secs: 60,
            retry_after_secs: None,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("RateLimitResult"));
        assert!(debug_str.contains("allowed: true"));
    }

    #[test]
    fn test_rate_limit_result_near_limit() {
        let result = RateLimitResult {
            allowed: true,
            current_count: 99,
            limit: 100,
            remaining: 1,
            reset_after_secs: 5,
            retry_after_secs: None,
        };

        assert!(result.allowed);
        assert_eq!(result.remaining, 1);
    }

    #[test]
    fn test_rate_limit_result_at_limit() {
        let result = RateLimitResult {
            allowed: false,
            current_count: 100,
            limit: 100,
            remaining: 0,
            reset_after_secs: 60,
            retry_after_secs: Some(60),
        };

        assert!(!result.allowed);
        assert_eq!(result.current_count, result.limit);
        assert_eq!(result.remaining, 0);
    }

    #[test]
    fn test_rate_limit_result_zero_limit() {
        let result = RateLimitResult {
            allowed: false,
            current_count: 0,
            limit: 0,
            remaining: 0,
            reset_after_secs: 0,
            retry_after_secs: Some(0),
        };

        assert!(!result.allowed);
        assert_eq!(result.limit, 0);
    }

    #[test]
    fn test_rate_limit_result_high_limit() {
        let result = RateLimitResult {
            allowed: true,
            current_count: 1000,
            limit: 1_000_000,
            remaining: 999_000,
            reset_after_secs: 3600,
            retry_after_secs: None,
        };

        assert!(result.allowed);
        assert_eq!(result.limit, 1_000_000);
    }

    #[test]
    fn test_rate_limit_result_usage_percentage() {
        let result = RateLimitResult {
            allowed: true,
            current_count: 50,
            limit: 100,
            remaining: 50,
            reset_after_secs: 60,
            retry_after_secs: None,
        };

        let usage_percentage = if result.limit > 0 {
            (result.current_count as f64 / result.limit as f64) * 100.0
        } else {
            0.0
        };

        assert!((usage_percentage - 50.0).abs() < f64::EPSILON);
    }

    // ==================== RateLimitEntry Tests ====================

    #[test]
    fn test_rate_limit_entry_default() {
        let entry = RateLimitEntry::default();

        assert!(entry.timestamps.is_empty());
        assert!((entry.tokens - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rate_limit_entry_clone() {
        let entry = RateLimitEntry {
            timestamps: vec![Instant::now()],
            tokens: 10.0,
            last_refill: Instant::now(),
        };

        let cloned = entry.clone();
        assert_eq!(cloned.timestamps.len(), entry.timestamps.len());
        assert_eq!(cloned.tokens, entry.tokens);
    }

    #[test]
    fn test_rate_limit_entry_debug() {
        let entry = RateLimitEntry::default();
        let debug_str = format!("{:?}", entry);

        assert!(debug_str.contains("RateLimitEntry"));
        assert!(debug_str.contains("timestamps"));
        assert!(debug_str.contains("tokens"));
    }

    #[test]
    fn test_rate_limit_entry_with_timestamps() {
        let now = Instant::now();
        let entry = RateLimitEntry {
            timestamps: vec![now, now, now],
            tokens: 0.0,
            last_refill: now,
        };

        assert_eq!(entry.timestamps.len(), 3);
    }

    #[test]
    fn test_rate_limit_entry_with_tokens() {
        let entry = RateLimitEntry {
            timestamps: Vec::new(),
            tokens: 100.5,
            last_refill: Instant::now(),
        };

        assert!((entry.tokens - 100.5).abs() < f64::EPSILON);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_rate_limit_check_simulation() {
        let limit = 10u32;
        let mut current_count = 0u32;

        // Simulate requests
        for _ in 0..limit {
            current_count += 1;
            let remaining = limit.saturating_sub(current_count);
            let allowed = current_count <= limit;

            if current_count < limit {
                assert!(allowed);
                assert!(remaining > 0);
            }
        }

        // Final request should hit limit
        let result = RateLimitResult {
            allowed: current_count < limit,
            current_count,
            limit,
            remaining: limit.saturating_sub(current_count),
            reset_after_secs: 60,
            retry_after_secs: if current_count >= limit {
                Some(60)
            } else {
                None
            },
        };

        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
    }

    #[test]
    fn test_sliding_window_cleanup_simulation() {
        let now = Instant::now();
        let window_size = std::time::Duration::from_secs(60);

        let entry = RateLimitEntry {
            timestamps: vec![now, now, now],
            tokens: 0.0,
            last_refill: now,
        };

        // Simulate cleanup - all timestamps within window
        let valid_count = entry
            .timestamps
            .iter()
            .filter(|&&ts| now.duration_since(ts) < window_size)
            .count();

        assert_eq!(valid_count, 3);
    }

    #[test]
    fn test_token_bucket_refill_simulation() {
        let capacity = 100.0;
        let refill_rate = 10.0; // tokens per second
        let mut entry = RateLimitEntry {
            timestamps: Vec::new(),
            tokens: 50.0,
            last_refill: Instant::now(),
        };

        // Simulate time passing and refill
        let elapsed_secs = 3.0;
        let tokens_to_add = refill_rate * elapsed_secs;
        entry.tokens = (entry.tokens + tokens_to_add).min(capacity);

        assert!((entry.tokens - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_rate_limits() {
        let rpm_result = RateLimitResult {
            allowed: true,
            current_count: 50,
            limit: 60,
            remaining: 10,
            reset_after_secs: 30,
            retry_after_secs: None,
        };

        let tpm_result = RateLimitResult {
            allowed: true,
            current_count: 50000,
            limit: 100000,
            remaining: 50000,
            reset_after_secs: 30,
            retry_after_secs: None,
        };

        // Both must be allowed for request to proceed
        let overall_allowed = rpm_result.allowed && tpm_result.allowed;
        assert!(overall_allowed);
    }

    #[test]
    fn test_rate_limit_exceeded_with_retry() {
        let result = RateLimitResult {
            allowed: false,
            current_count: 60,
            limit: 60,
            remaining: 0,
            reset_after_secs: 45,
            retry_after_secs: Some(45),
        };

        assert!(!result.allowed);
        assert!(result.retry_after_secs.is_some());

        // Client should wait retry_after_secs before retrying
        let wait_time = result.retry_after_secs.unwrap();
        assert_eq!(wait_time, 45);
    }

    #[test]
    fn test_rate_limit_headers_simulation() {
        let result = RateLimitResult {
            allowed: true,
            current_count: 25,
            limit: 100,
            remaining: 75,
            reset_after_secs: 120,
            retry_after_secs: None,
        };

        // Simulate HTTP headers
        let x_ratelimit_limit = result.limit;
        let x_ratelimit_remaining = result.remaining;
        let x_ratelimit_reset = result.reset_after_secs;

        assert_eq!(x_ratelimit_limit, 100);
        assert_eq!(x_ratelimit_remaining, 75);
        assert_eq!(x_ratelimit_reset, 120);
    }
}
