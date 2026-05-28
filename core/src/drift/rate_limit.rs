//! Rate limiting for sync protocol to prevent flooding
//!
//! This module provides a token bucket-style rate limiter that tracks
//! sync requests per peer and enforces limits to prevent DoS attacks.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Rate limiter for sync protocol requests
///
/// Uses a sliding window approach to track sync requests per peer.
/// When a peer exceeds the configured rate limit, their sync requests
/// are denied until the window slides forward.
pub struct SyncRateLimiter {
    /// Map of peer ID to list of recent sync timestamps
    limits: HashMap<String, Vec<Instant>>,
    /// Time window for rate limiting
    window: Duration,
    /// Maximum sync requests allowed per window
    max_per_window: usize,
}

impl SyncRateLimiter {
    /// Create a new rate limiter with the specified window and limit
    ///
    /// # Arguments
    /// * `window` - Time window for rate limiting (e.g., Duration::from_secs(60))
    /// * `max_per_window` - Maximum sync requests allowed in the window
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use scmessenger_core::drift::rate_limit::SyncRateLimiter;
    ///
    /// // Allow 10 syncs per minute
    /// let limiter = SyncRateLimiter::new(Duration::from_secs(60), 10);
    /// ```
    pub fn new(window: Duration, max_per_window: usize) -> Self {
        Self {
            limits: HashMap::new(),
            window,
            max_per_window,
        }
    }

    /// Check if a sync request from a peer should be allowed
    ///
    /// Returns `true` if the request is allowed, `false` if rate limited.
    /// If allowed, the request is recorded for future rate limit checks.
    ///
    /// # Arguments
    /// * `peer_id` - Identifier for the peer making the sync request
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// use scmessenger_core::drift::rate_limit::SyncRateLimiter;
    ///
    /// let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 10);
    ///
    /// // First request is allowed
    /// assert!(limiter.allow_sync("peer1"));
    /// ```
    pub fn allow_sync(&mut self, peer_id: &str) -> bool {
        let now = Instant::now();

        // Get or create entry for peer
        let timestamps = self.limits.entry(peer_id.to_string()).or_default();

        // Remove expired timestamps (outside the window)
        timestamps.retain(|&t| now.duration_since(t) < self.window);

        // Check if under limit
        if timestamps.len() >= self.max_per_window {
            return false;
        }

        // Record this sync
        timestamps.push(now);
        true
    }

    /// Clean up expired entries to prevent unbounded memory growth
    ///
    /// This should be called periodically (e.g., every few minutes) to
    /// remove peers that haven't made requests recently.
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.limits.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < self.window);
            !timestamps.is_empty()
        });
    }

    /// Get the number of peers currently being tracked
    pub fn tracked_peer_count(&self) -> usize {
        self.limits.len()
    }

    /// Get the number of recent sync requests for a specific peer
    pub fn peer_sync_count(&self, peer_id: &str) -> usize {
        self.limits.get(peer_id).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = SyncRateLimiter::new(Duration::from_secs(60), 10);
        assert_eq!(limiter.tracked_peer_count(), 0);
    }

    #[test]
    fn test_allow_sync_under_limit() {
        let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 5);

        // First 5 requests should be allowed
        for _ in 0..5 {
            assert!(limiter.allow_sync("peer1"));
        }

        assert_eq!(limiter.peer_sync_count("peer1"), 5);
    }

    #[test]
    fn test_deny_sync_over_limit() {
        let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 3);

        // First 3 requests allowed
        for _ in 0..3 {
            assert!(limiter.allow_sync("peer1"));
        }

        // 4th request should be denied
        assert!(!limiter.allow_sync("peer1"));
        assert_eq!(limiter.peer_sync_count("peer1"), 3);
    }

    #[test]
    fn test_different_peers_independent() {
        let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 2);

        assert!(limiter.allow_sync("peer1"));
        assert!(limiter.allow_sync("peer1"));
        assert!(!limiter.allow_sync("peer1")); // peer1 at limit

        // peer2 should still be allowed
        assert!(limiter.allow_sync("peer2"));
        assert!(limiter.allow_sync("peer2"));
        assert!(!limiter.allow_sync("peer2")); // peer2 at limit

        assert_eq!(limiter.tracked_peer_count(), 2);
    }

    #[test]
    fn test_window_expiry() {
        let mut limiter = SyncRateLimiter::new(Duration::from_millis(100), 2);

        // Use up the limit
        assert!(limiter.allow_sync("peer1"));
        assert!(limiter.allow_sync("peer1"));
        assert!(!limiter.allow_sync("peer1"));

        // Wait for window to expire
        thread::sleep(Duration::from_millis(150));

        // Should be allowed again
        assert!(limiter.allow_sync("peer1"));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut limiter = SyncRateLimiter::new(Duration::from_millis(100), 5);

        limiter.allow_sync("peer1");
        limiter.allow_sync("peer2");
        limiter.allow_sync("peer3");

        assert_eq!(limiter.tracked_peer_count(), 3);

        // Wait for entries to expire
        thread::sleep(Duration::from_millis(150));

        // Cleanup should remove all expired entries
        limiter.cleanup_expired();
        assert_eq!(limiter.tracked_peer_count(), 0);
    }

    #[test]
    fn test_cleanup_preserves_active() {
        let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 5);

        limiter.allow_sync("peer1");
        limiter.allow_sync("peer2");

        // Cleanup should not remove active entries
        limiter.cleanup_expired();
        assert_eq!(limiter.tracked_peer_count(), 2);
    }

    #[test]
    fn test_sliding_window() {
        let mut limiter = SyncRateLimiter::new(Duration::from_millis(200), 2);

        // First request
        assert!(limiter.allow_sync("peer1"));
        thread::sleep(Duration::from_millis(50));

        // Second request
        assert!(limiter.allow_sync("peer1"));

        // Third request denied (within window)
        assert!(!limiter.allow_sync("peer1"));

        // Wait for first request to expire
        thread::sleep(Duration::from_millis(160));

        // Should be allowed now (first request expired)
        assert!(limiter.allow_sync("peer1"));
    }

    #[test]
    fn test_peer_sync_count() {
        let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 10);

        assert_eq!(limiter.peer_sync_count("peer1"), 0);

        limiter.allow_sync("peer1");
        assert_eq!(limiter.peer_sync_count("peer1"), 1);

        limiter.allow_sync("peer1");
        limiter.allow_sync("peer1");
        assert_eq!(limiter.peer_sync_count("peer1"), 3);
    }

    #[test]
    fn test_zero_limit() {
        let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 0);

        // All requests should be denied
        assert!(!limiter.allow_sync("peer1"));
        assert!(!limiter.allow_sync("peer1"));
    }

    #[test]
    fn test_large_limit() {
        let mut limiter = SyncRateLimiter::new(Duration::from_secs(60), 1000);

        // Should allow many requests
        for _ in 0..1000 {
            assert!(limiter.allow_sync("peer1"));
        }

        // 1001st should be denied
        assert!(!limiter.allow_sync("peer1"));
    }
}
