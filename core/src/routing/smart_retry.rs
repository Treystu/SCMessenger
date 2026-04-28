//! Smart retry mechanisms with exponential backoff and delivery triggers.
//!
//! This module provides functionality for calculating retry delays using
//! exponential backoff strategies and defining triggers for message delivery.

use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for exponential backoff strategy.
#[derive(Debug, Clone, PartialEq)]
pub struct BackoffStrategy {
    /// Base delay in milliseconds
    pub base_ms: u64,
    /// Maximum delay in milliseconds
    pub max_ms: u64,
    /// Multiplier for each subsequent attempt
    pub multiplier: f64,
}

/// Calculate the next attempt time based on exponential backoff strategy.
///
/// # Arguments
///
/// * `attempt_count` - The number of previous attempts (0 for first attempt)
/// * `strategy` - The backoff strategy to use
///
/// # Returns
///
/// Unix timestamp (in milliseconds) when the next attempt should occur.
pub fn calculate_next_attempt(attempt_count: u32, strategy: &BackoffStrategy) -> u64 {
    // Calculate delay in milliseconds using exponential backoff
    let delay_ms = if attempt_count == 0 {
        0 // No delay for the first attempt
    } else {
        let exponential_delay =
            strategy.base_ms as f64 * strategy.multiplier.powi(attempt_count as i32 - 1);
        exponential_delay.min(strategy.max_ms as f64) as u64
    };

    // Get current timestamp in milliseconds
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    now_ms + delay_ms
}

/// Represents the reason for triggering an outbox flush operation.
#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryTrigger {
    /// Flush triggered by a scheduled timer
    ScheduledTimer,
    /// Flush triggered when a peer becomes available
    PeerDiscovered(String),
    /// Flush triggered when routing information changes
    RouteUpdated(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Helper function to get current timestamp for testing
    fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
    }

    #[test]
    fn test_first_attempt_no_delay() {
        let strategy = BackoffStrategy {
            base_ms: 1000,
            max_ms: 10000,
            multiplier: 2.0,
        };

        let now = get_current_timestamp();
        let result = calculate_next_attempt(0, &strategy);

        // First attempt should happen immediately (no delay)
        // Allow small variance for execution time
        assert!(
            result >= now && result <= now + 100,
            "First attempt should have no delay. Now: {}, Result: {}",
            now,
            result
        );
    }

    #[test]
    fn test_exponential_backoff_increases() {
        let strategy = BackoffStrategy {
            base_ms: 1000,
            max_ms: 10000,
            multiplier: 2.0,
        };

        let now = get_current_timestamp();

        // Test multiple attempt counts
        for attempt in 1..=5 {
            let result = calculate_next_attempt(attempt, &strategy);
            let mut expected_delay =
                (strategy.base_ms as f64 * strategy.multiplier.powi(attempt as i32 - 1)) as u64;
            if expected_delay > strategy.max_ms {
                expected_delay = strategy.max_ms;
            }
            let expected_time = now + expected_delay;

            // Allow some variance for execution time
            assert!(
                result >= expected_time && result <= expected_time + 100,
                "Attempt {}: Expected delay {}, got {}",
                attempt,
                expected_delay,
                result - now
            );
        }
    }

    #[test]
    fn test_max_delay_enforcement() {
        let strategy = BackoffStrategy {
            base_ms: 1000,
            max_ms: 5000,
            multiplier: 10.0, // Very large multiplier
        };

        let now = get_current_timestamp();

        // Attempt 1: base_ms * multiplier^0 = 1000ms
        let result1 = calculate_next_attempt(1, &strategy);
        assert!(result1 >= now + 1000 && result1 <= now + 1100);

        // Attempt 2: base_ms * multiplier^1 = 10000ms, but capped at max_ms = 5000ms
        let result2 = calculate_next_attempt(2, &strategy);
        assert!(result2 >= now + 5000 && result2 <= now + 5100);

        // Attempt 3: base_ms * multiplier^2 = 100000ms, but capped at max_ms = 5000ms
        let result3 = calculate_next_attempt(3, &strategy);
        assert!(result3 >= now + 5000 && result3 <= now + 5100);

        // Attempt 10: Should still be capped at max_ms
        let result10 = calculate_next_attempt(10, &strategy);
        assert!(result10 >= now + 5000 && result10 <= now + 5100);
    }

    #[test]
    fn test_zero_base_delay() {
        let strategy = BackoffStrategy {
            base_ms: 0,
            max_ms: 10000,
            multiplier: 2.0,
        };

        let now = get_current_timestamp();

        // First attempt should still work
        let result1 = calculate_next_attempt(0, &strategy);
        assert!(result1 >= now && result1 <= now + 100);

        // Subsequent attempts should have 0 delay (base_ms = 0)
        for attempt in 1..=5 {
            let result = calculate_next_attempt(attempt, &strategy);
            assert!(result >= now && result <= now + 100);
        }
    }

    #[test]
    fn test_multiplier_of_one() {
        let strategy = BackoffStrategy {
            base_ms: 1000,
            max_ms: 10000,
            multiplier: 1.0, // No exponential growth
        };

        let now = get_current_timestamp();

        // All attempts should have the same delay (base_ms)
        for attempt in 1..=5 {
            let result = calculate_next_attempt(attempt, &strategy);
            assert!(
                result >= now + 1000 && result <= now + 1100,
                "Attempt {} should have constant delay",
                attempt
            );
        }
    }

    #[test]
    fn test_fractional_multiplier() {
        let strategy = BackoffStrategy {
            base_ms: 1000,
            max_ms: 10000,
            multiplier: 0.5, // Decreasing delay
        };

        let now = get_current_timestamp();

        // Attempt 1: 1000 * 0.5^0 = 1000ms
        let result1 = calculate_next_attempt(1, &strategy);
        assert!(result1 >= now + 1000 && result1 <= now + 1100);

        // Attempt 2: 1000 * 0.5^1 = 500ms
        let result2 = calculate_next_attempt(2, &strategy);
        assert!(result2 >= now + 500 && result2 <= now + 600);

        // Attempt 3: 1000 * 0.5^2 = 250ms
        let result3 = calculate_next_attempt(3, &strategy);
        assert!(result3 >= now + 250 && result3 <= now + 350);
    }

    #[test]
    fn test_large_attempt_counts() {
        let strategy = BackoffStrategy {
            base_ms: 1000,
            max_ms: 5000,
            multiplier: 2.0,
        };

        let now = get_current_timestamp();

        // Test with very large attempt counts
        // The delay should be capped at max_ms
        for attempt in 10..=100 {
            let result = calculate_next_attempt(attempt, &strategy);
            assert!(
                result >= now + 5000 && result <= now + 5100,
                "Attempt {} should be capped at max_ms",
                attempt
            );
        }
    }

    #[test]
    fn test_edge_case_max_equals_base() {
        let strategy = BackoffStrategy {
            base_ms: 1000,
            max_ms: 1000, // max equals base
            multiplier: 2.0,
        };

        let now = get_current_timestamp();

        // First attempt
        let result1 = calculate_next_attempt(0, &strategy);
        assert!(result1 >= now && result1 <= now + 100);

        // Second attempt: base_ms * multiplier^0 = 1000, capped at 1000
        let result2 = calculate_next_attempt(1, &strategy);
        assert!(result2 >= now + 1000 && result2 <= now + 1100);

        // Third attempt: base_ms * multiplier^1 = 2000, capped at 1000
        let result3 = calculate_next_attempt(2, &strategy);
        assert!(result3 >= now + 1000 && result3 <= now + 1100);
    }

    #[test]
    fn test_delivery_trigger_variants() {
        let timer_trigger = DeliveryTrigger::ScheduledTimer;
        let peer_trigger = DeliveryTrigger::PeerDiscovered("peer1".to_string());
        let route_trigger = DeliveryTrigger::RouteUpdated("route1".to_string());

        match timer_trigger {
            DeliveryTrigger::ScheduledTimer => {}
            _ => panic!("Expected ScheduledTimer variant"),
        }

        match peer_trigger {
            DeliveryTrigger::PeerDiscovered(peer_id) => {
                assert_eq!(peer_id, "peer1");
            }
            _ => panic!("Expected PeerDiscovered variant"),
        }

        match route_trigger {
            DeliveryTrigger::RouteUpdated(route_id) => {
                assert_eq!(route_id, "route1");
            }
            _ => panic!("Expected RouteUpdated variant"),
        }
    }

    #[test]
    fn test_delivery_trigger_equality() {
        let trigger1 = DeliveryTrigger::PeerDiscovered("peer1".to_string());
        let trigger2 = DeliveryTrigger::PeerDiscovered("peer1".to_string());
        let trigger3 = DeliveryTrigger::PeerDiscovered("peer2".to_string());

        assert_eq!(trigger1, trigger2);
        assert_ne!(trigger1, trigger3);
        assert_ne!(trigger2, trigger3);
    }

    #[test]
    fn test_backoff_strategy_equality() {
        let strategy1 = BackoffStrategy {
            base_ms: 1000,
            max_ms: 10000,
            multiplier: 2.0,
        };

        let strategy2 = BackoffStrategy {
            base_ms: 1000,
            max_ms: 10000,
            multiplier: 2.0,
        };

        let strategy3 = BackoffStrategy {
            base_ms: 500,
            max_ms: 10000,
            multiplier: 2.0,
        };

        assert_eq!(strategy1, strategy2);
        assert_ne!(strategy1, strategy3);
    }
}
