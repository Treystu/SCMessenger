//! Adaptive TTL Based on Peer Activity
//!
//! Keeps routes fresh longer for active peers, expires faster for inactive peers.
//! This reduces unnecessary re-discovery for frequently messaged peers.
//!
//! # Design Principles
//!
//! 1. **Activity-based**: TTL scales with message frequency
//! 2. **Bounded growth**: Maximum TTL prevents unbounded memory usage
//! 3. **Deterministic**: Same activity pattern produces same TTL
//! 4. **Decay over time**: Activity decays when no messages are exchanged

use std::collections::HashMap;
use web_time::{Duration, Instant};

/// Activity history for a peer
#[derive(Debug, Clone)]
pub struct ActivityHistory {
    /// Messages exchanged in recent time window
    pub recent_messages: u32,
    /// Last message timestamp
    pub last_message: Instant,
    /// Calculated adaptive TTL
    pub adaptive_ttl: Duration,
}

impl Default for ActivityHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivityHistory {
    /// Create new activity history
    pub fn new() -> Self {
        ActivityHistory {
            recent_messages: 0,
            last_message: Instant::now(),
            adaptive_ttl: Duration::from_secs(1800), // Default 30 minutes
        }
    }

    /// Record a message exchange
    pub fn record_message(&mut self) {
        self.recent_messages += 1;
        self.last_message = Instant::now();
    }

    /// Calculate TTL based on current activity
    pub fn calculate_ttl(&self, base_ttl: Duration, max_ttl: Duration) -> Duration {
        if self.recent_messages > 10 {
            // Very active peer: use maximum TTL
            max_ttl
        } else if self.recent_messages > 2 {
            // Moderately active peer: double base TTL
            base_ttl * 2
        } else {
            // Inactive peer: use base TTL
            base_ttl
        }
    }

    /// Decay activity over time
    pub fn decay(&mut self, half_life: Duration, base_ttl: Duration, max_ttl: Duration) {
        let elapsed = self.last_message.elapsed();
        let decay_factor = 0.5_f64.powf(elapsed.as_secs_f64() / half_life.as_secs_f64());
        self.recent_messages = (self.recent_messages as f64 * decay_factor).round() as u32;
        self.adaptive_ttl = self.calculate_ttl(base_ttl, max_ttl);
    }
}

/// Manager for adaptive TTL calculation
#[derive(Debug, Clone)]
pub struct AdaptiveTTLManager {
    /// Activity history per peer
    peer_activity: HashMap<String, ActivityHistory>,
    /// Base TTL for inactive peers
    base_ttl: Duration,
    /// Maximum TTL for very active peers
    max_ttl: Duration,
    /// Half-life for activity decay
    half_life: Duration,
}

impl AdaptiveTTLManager {
    /// Create a new adaptive TTL manager
    pub fn new(base_ttl: Duration, max_ttl: Duration, half_life: Duration) -> Self {
        AdaptiveTTLManager {
            peer_activity: HashMap::new(),
            base_ttl,
            max_ttl,
            half_life,
        }
    }

    /// Create with default settings
    pub fn with_defaults() -> Self {
        Self::new(
            Duration::from_secs(1800), // 30 minutes base
            Duration::from_secs(7200), // 2 hours max
            Duration::from_secs(3600), // 1 hour half-life
        )
    }

    /// Calculate TTL for a peer based on activity
    pub fn calculate_ttl(&mut self, peer_id: &str) -> Duration {
        // Get or create activity history
        let activity = self.peer_activity.entry(peer_id.to_string()).or_default();

        // Decay activity before calculating
        activity.decay(self.half_life, self.base_ttl, self.max_ttl);

        // Return calculated TTL
        activity.adaptive_ttl
    }

    /// Record message activity for a peer
    pub fn record_activity(&mut self, peer_id: &str) {
        let activity = self.peer_activity.entry(peer_id.to_string()).or_default();
        activity.record_message();
        activity.adaptive_ttl = activity.calculate_ttl(self.base_ttl, self.max_ttl);
    }

    /// Get current activity for a peer
    pub fn get_activity(&self, peer_id: &str) -> Option<&ActivityHistory> {
        self.peer_activity.get(peer_id)
    }

    /// Clean up old entries
    pub fn cleanup(&mut self, max_age: Duration) -> usize {
        let before = self.peer_activity.len();
        self.peer_activity
            .retain(|_, activity| activity.last_message.elapsed() < max_age);
        before - self.peer_activity.len()
    }

    /// Get number of tracked peers
    pub fn len(&self) -> usize {
        self.peer_activity.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.peer_activity.is_empty()
    }
}

impl Default for AdaptiveTTLManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_ttl_creation() {
        let manager = AdaptiveTTLManager::with_defaults();
        assert_eq!(manager.base_ttl, Duration::from_secs(1800));
        assert_eq!(manager.max_ttl, Duration::from_secs(7200));
    }

    #[test]
    fn test_inactive_peer_ttl() {
        let mut manager = AdaptiveTTLManager::with_defaults();
        let ttl = manager.calculate_ttl("peer1");
        assert_eq!(ttl, Duration::from_secs(1800));
    }

    #[test]
    fn test_active_peer_ttl() {
        let mut manager = AdaptiveTTLManager::with_defaults();

        // Record multiple messages
        for _ in 0..15 {
            manager.record_activity("peer1");
        }

        let ttl = manager.calculate_ttl("peer1");
        assert_eq!(ttl, Duration::from_secs(7200));
    }

    #[test]
    fn test_moderate_peer_ttl() {
        let mut manager = AdaptiveTTLManager::with_defaults();

        // Record moderate activity
        for _ in 0..5 {
            manager.record_activity("peer1");
        }

        let ttl = manager.calculate_ttl("peer1");
        assert_eq!(ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_activity_decay() {
        let mut manager = AdaptiveTTLManager::new(
            Duration::from_secs(100),
            Duration::from_secs(400),
            Duration::from_millis(50), // Very short half-life for test
        );

        // Record enough activity to reach the maximum TTL tier (>10 messages)
        for _ in 0..15 {
            manager.record_activity("peer1");
        }

        let ttl_before = manager.calculate_ttl("peer1");
        assert_eq!(ttl_before, Duration::from_secs(400)); // Max tier

        // Wait long enough for significant decay (4 half-lives at 50ms each)
        std::thread::sleep(Duration::from_millis(200));

        let ttl_after = manager.calculate_ttl("peer1");
        assert!(ttl_after < ttl_before);
    }

    #[test]
    fn test_cleanup_old_entries() {
        let mut manager = AdaptiveTTLManager::with_defaults();

        // Add some activity
        manager.record_activity("peer1");
        manager.record_activity("peer2");

        assert_eq!(manager.len(), 2);

        // Clean up with very short max age
        let removed = manager.cleanup(Duration::from_nanos(1));
        assert_eq!(removed, 2);
        assert_eq!(manager.len(), 0);
    }
}
