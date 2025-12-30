//! Bounded collection utilities for metrics

use std::collections::VecDeque;

/// Maximum number of samples to retain for time-series metrics
pub(super) const MAX_METRIC_SAMPLES: usize = 10_000;

/// Maximum number of recent requests/errors to track (for rate calculations)
pub(super) const MAX_RECENT_EVENTS: usize = 1_000;

/// Helper trait for bounded VecDeque operations
pub(super) trait BoundedPush<T> {
    fn push_bounded(&mut self, value: T, max_size: usize);
}

impl<T> BoundedPush<T> for VecDeque<T> {
    /// Push a value while maintaining a maximum size (O(1) amortized)
    #[inline]
    fn push_bounded(&mut self, value: T, max_size: usize) {
        if self.len() >= max_size {
            self.pop_front();
        }
        self.push_back(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Constant Tests ====================

    #[test]
    fn test_max_metric_samples_value() {
        assert_eq!(MAX_METRIC_SAMPLES, 10_000);
    }

    #[test]
    fn test_max_recent_events_value() {
        assert_eq!(MAX_RECENT_EVENTS, 1_000);
    }

    // ==================== BoundedPush Tests ====================

    #[test]
    fn test_push_bounded_empty_deque() {
        let mut deque: VecDeque<i32> = VecDeque::new();
        deque.push_bounded(1, 5);

        assert_eq!(deque.len(), 1);
        assert_eq!(deque[0], 1);
    }

    #[test]
    fn test_push_bounded_below_max() {
        let mut deque: VecDeque<i32> = VecDeque::new();
        deque.push_bounded(1, 5);
        deque.push_bounded(2, 5);
        deque.push_bounded(3, 5);

        assert_eq!(deque.len(), 3);
        assert_eq!(deque[0], 1);
        assert_eq!(deque[1], 2);
        assert_eq!(deque[2], 3);
    }

    #[test]
    fn test_push_bounded_at_max() {
        let mut deque: VecDeque<i32> = VecDeque::new();
        for i in 1..=5 {
            deque.push_bounded(i, 5);
        }

        assert_eq!(deque.len(), 5);
        // Push one more - should remove first element
        deque.push_bounded(6, 5);
        assert_eq!(deque.len(), 5);
        assert_eq!(deque[0], 2); // First element now is 2
        assert_eq!(deque[4], 6); // Last element is 6
    }

    #[test]
    fn test_push_bounded_maintains_max_size() {
        let mut deque: VecDeque<i32> = VecDeque::new();
        let max_size = 3;

        for i in 1..=10 {
            deque.push_bounded(i, max_size);
            assert!(deque.len() <= max_size);
        }

        assert_eq!(deque.len(), 3);
        // Should contain [8, 9, 10]
        assert_eq!(deque[0], 8);
        assert_eq!(deque[1], 9);
        assert_eq!(deque[2], 10);
    }

    #[test]
    fn test_push_bounded_single_element_max() {
        let mut deque: VecDeque<i32> = VecDeque::new();

        deque.push_bounded(1, 1);
        assert_eq!(deque.len(), 1);
        assert_eq!(deque[0], 1);

        deque.push_bounded(2, 1);
        assert_eq!(deque.len(), 1);
        assert_eq!(deque[0], 2);

        deque.push_bounded(3, 1);
        assert_eq!(deque.len(), 1);
        assert_eq!(deque[0], 3);
    }

    #[test]
    fn test_push_bounded_with_strings() {
        let mut deque: VecDeque<String> = VecDeque::new();

        deque.push_bounded("a".to_string(), 2);
        deque.push_bounded("b".to_string(), 2);
        deque.push_bounded("c".to_string(), 2);

        assert_eq!(deque.len(), 2);
        assert_eq!(deque[0], "b");
        assert_eq!(deque[1], "c");
    }

    #[test]
    fn test_push_bounded_with_floats() {
        let mut deque: VecDeque<f64> = VecDeque::new();

        deque.push_bounded(1.5, 3);
        deque.push_bounded(2.5, 3);
        deque.push_bounded(3.5, 3);
        deque.push_bounded(4.5, 3);

        assert_eq!(deque.len(), 3);
        assert!((deque[0] - 2.5).abs() < f64::EPSILON);
        assert!((deque[1] - 3.5).abs() < f64::EPSILON);
        assert!((deque[2] - 4.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_push_bounded_large_max_size() {
        let mut deque: VecDeque<i32> = VecDeque::new();

        for i in 0..100 {
            deque.push_bounded(i, 1000);
        }

        assert_eq!(deque.len(), 100);
        assert_eq!(deque[0], 0);
        assert_eq!(deque[99], 99);
    }

    #[test]
    fn test_push_bounded_order_preserved() {
        let mut deque: VecDeque<i32> = VecDeque::new();

        for i in 1..=5 {
            deque.push_bounded(i, 10);
        }

        let result: Vec<i32> = deque.iter().copied().collect();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_push_bounded_fifo_behavior() {
        let mut deque: VecDeque<char> = VecDeque::new();
        let max_size = 3;

        deque.push_bounded('a', max_size);
        deque.push_bounded('b', max_size);
        deque.push_bounded('c', max_size);
        // Now full: ['a', 'b', 'c']

        deque.push_bounded('d', max_size);
        // Should be: ['b', 'c', 'd'] - 'a' was removed

        assert_eq!(deque.len(), 3);
        assert_eq!(deque.front(), Some(&'b'));
        assert_eq!(deque.back(), Some(&'d'));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_push_bounded_max_size_zero() {
        // Edge case: max_size = 0 means nothing gets stored after comparison
        let mut deque: VecDeque<i32> = VecDeque::new();
        deque.push_bounded(1, 0);

        // len() >= 0 is always true for empty deque, so pop_front does nothing,
        // then push_back adds element
        // Actually, empty deque len() = 0, 0 >= 0 is true
        // But pop_front on empty returns None and does nothing
        // Then push_back adds 1
        // So deque becomes [1]
        // Next push: len() = 1, 1 >= 0, pop_front removes 1, push_back adds 2
        // So deque stays at size 1
        assert_eq!(deque.len(), 1);

        deque.push_bounded(2, 0);
        assert_eq!(deque.len(), 1);
        assert_eq!(deque[0], 2);
    }

    #[test]
    fn test_push_bounded_with_struct() {
        #[derive(Debug, PartialEq, Clone)]
        struct Point {
            x: i32,
            y: i32,
        }

        let mut deque: VecDeque<Point> = VecDeque::new();

        deque.push_bounded(Point { x: 1, y: 1 }, 2);
        deque.push_bounded(Point { x: 2, y: 2 }, 2);
        deque.push_bounded(Point { x: 3, y: 3 }, 2);

        assert_eq!(deque.len(), 2);
        assert_eq!(deque[0], Point { x: 2, y: 2 });
        assert_eq!(deque[1], Point { x: 3, y: 3 });
    }

    #[test]
    fn test_push_bounded_repeated_values() {
        let mut deque: VecDeque<i32> = VecDeque::new();

        for _ in 0..5 {
            deque.push_bounded(42, 3);
        }

        assert_eq!(deque.len(), 3);
        assert!(deque.iter().all(|&x| x == 42));
    }
}
