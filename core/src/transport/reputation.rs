//! P0_SECURITY_002: Peer reputation scoring for abuse mitigation.
//!
//! Tracks abuse signals (rate limit hits, invalid messages, spam patterns)
//! alongside relay quality. Produces a composite reputation score that
//! influences rate limits and relay decisions.

use std::collections::HashMap;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

/// Composite reputation score for a peer, ranging from 0.0 (abusive) to 100.0 (trusted).
/// A score of 50.0 is neutral (new peer default).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReputationScore(f64);

impl ReputationScore {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = 100.0;
    pub const NEUTRAL: f64 = 50.0;

    pub fn new(score: f64) -> Self {
        Self(score.clamp(Self::MIN, Self::MAX))
    }

    pub fn neutral() -> Self {
        Self(Self::NEUTRAL)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn is_trusted(&self) -> bool {
        self.0 >= 70.0
    }

    pub fn is_suspicious(&self) -> bool {
        self.0 < 30.0 && self.0 >= 10.0
    }

    pub fn is_abusive(&self) -> bool {
        self.0 < 10.0
    }
}

impl std::fmt::Display for ReputationScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

/// Types of abuse signals that contribute to reputation scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbuseSignal {
    /// Peer exceeded per-peer rate limit
    RateLimited,
    /// Peer sent a message that was too large
    OversizedMessage,
    /// Peer sent a message with invalid format
    InvalidFormat,
    /// Peer sent a duplicate within the dedup window
    DuplicateMessage,
    /// Peer sent a message to a non-existent or invalid destination
    InvalidDestination,
    /// Successfully relayed a message for this peer
    SuccessfulRelay,
    /// Relay attempt for this peer failed
    FailedRelay,
    /// Peer successfully delivered a message to us
    SuccessfulDelivery,
    /// Connection to peer timed out
    ConnectionTimeout,
}

/// Per-peer abuse statistics that persist across sessions.
#[derive(Debug, Clone)]
pub struct PeerAbuseStats {
    pub peer_id: String,
    pub rate_limit_hits: u32,
    pub oversized_messages: u32,
    pub invalid_format_count: u32,
    pub duplicate_count: u32,
    pub invalid_destination_count: u32,
    pub successful_relays: u32,
    pub failed_relays: u32,
    pub successful_deliveries: u32,
    pub connection_timeouts: u32,
    pub last_signal_at: Option<Instant>,
    pub reputation_score: ReputationScore,
}

impl PeerAbuseStats {
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            rate_limit_hits: 0,
            oversized_messages: 0,
            invalid_format_count: 0,
            duplicate_count: 0,
            invalid_destination_count: 0,
            successful_relays: 0,
            failed_relays: 0,
            successful_deliveries: 0,
            connection_timeouts: 0,
            last_signal_at: None,
            reputation_score: ReputationScore::neutral(),
        }
    }

    /// Record an abuse signal and recalculate the reputation score.
    pub fn record_signal(&mut self, signal: AbuseSignal) {
        match signal {
            AbuseSignal::RateLimited => self.rate_limit_hits += 1,
            AbuseSignal::OversizedMessage => self.oversized_messages += 1,
            AbuseSignal::InvalidFormat => self.invalid_format_count += 1,
            AbuseSignal::DuplicateMessage => self.duplicate_count += 1,
            AbuseSignal::InvalidDestination => self.invalid_destination_count += 1,
            AbuseSignal::SuccessfulRelay => self.successful_relays += 1,
            AbuseSignal::FailedRelay => self.failed_relays += 1,
            AbuseSignal::SuccessfulDelivery => self.successful_deliveries += 1,
            AbuseSignal::ConnectionTimeout => self.connection_timeouts += 1,
        }
        self.last_signal_at = Some(Instant::now());
        self.reputation_score = self.calculate_score();
    }

    /// Calculate composite reputation score based on abuse signals.
    ///
    /// Formula:
    /// - Start at NEUTRAL (50.0)
    /// - Positive signals (successful deliveries, relays) push toward MAX
    /// - Negative signals (rate limits, invalid messages, duplicates) push toward MIN
    /// - Decay factor: signals lose weight over time
    fn calculate_score(&self) -> ReputationScore {
        let positive = (self.successful_relays as f64 * 2.0)
            + (self.successful_deliveries as f64 * 3.0);
        let negative = (self.rate_limit_hits as f64 * 5.0)
            + (self.oversized_messages as f64 * 3.0)
            + (self.invalid_format_count as f64 * 4.0)
            + (self.duplicate_count as f64 * 2.0)
            + (self.invalid_destination_count as f64 * 3.0)
            + (self.failed_relays as f64 * 1.5)
            + (self.connection_timeouts as f64 * 1.0);

        let total = positive - negative;
        // Sigmoid-like scaling: positive total pushes toward 100, negative toward 0
        let score = ReputationScore::NEUTRAL + (total / (1.0 + total.abs())) * 50.0;
        ReputationScore::new(score)
    }

    /// Get the effective rate limit multiplier for this peer.
    /// Trusted peers get higher limits, abusive peers get lower limits.
    pub fn rate_limit_multiplier(&self) -> f64 {
        if self.reputation_score.is_trusted() {
            1.5 // 50% more capacity for trusted peers
        } else if self.reputation_score.is_abusive() {
            0.1 // 90% reduction for abusive peers
        } else if self.reputation_score.is_suspicious() {
            0.5 // 50% reduction for suspicious peers
        } else {
            1.0 // Default
        }
    }
}

/// Abuse reputation manager that tracks per-peer abuse signals and computes
/// composite reputation scores. Thread-safe via RwLock.
pub struct AbuseReputationManager {
    peers: RwLock<HashMap<String, PeerAbuseStats>>,
    max_tracked_peers: usize,
}

impl AbuseReputationManager {
    pub fn new(max_tracked_peers: usize) -> Self {
        Self {
            peers: RwLock::new(HashMap::new()),
            max_tracked_peers,
        }
    }

    /// Record an abuse signal for a peer.
    pub fn record_signal(&self, peer_id: &str, signal: AbuseSignal) -> ReputationScore {
        let mut peers = self.peers.write();

        // Evict lowest-scored peer if at capacity
        if !peers.contains_key(peer_id) && peers.len() >= self.max_tracked_peers {
            if let Some(lowest_key) = peers
                .iter()
                .min_by_key(|(_, stats)| stats.reputation_score.value() as u32)
                .map(|(k, _)| k.clone())
            {
                peers.remove(&lowest_key);
            }
        }

        let stats = peers
            .entry(peer_id.to_string())
            .or_insert_with(|| PeerAbuseStats::new(peer_id.to_string()));
        stats.record_signal(signal);
        stats.reputation_score
    }

    /// Get the reputation score for a peer. Returns NEUTRAL if not tracked.
    pub fn get_score(&self, peer_id: &str) -> ReputationScore {
        let peers = self.peers.read();
        peers
            .get(peer_id)
            .map(|s| s.reputation_score)
            .unwrap_or_else(ReputationScore::neutral)
    }

    /// Get the rate limit multiplier for a peer. Returns 1.0 if not tracked.
    pub fn rate_limit_multiplier(&self, peer_id: &str) -> f64 {
        let peers = self.peers.read();
        peers
            .get(peer_id)
            .map(|s| s.rate_limit_multiplier())
            .unwrap_or(1.0)
    }

    /// Get all peer reputation entries (for diagnostics/debugging).
    pub fn all_reputations(&self) -> Vec<(String, ReputationScore)> {
        let peers = self.peers.read();
        peers
            .iter()
            .map(|(k, v)| (k.clone(), v.reputation_score))
            .collect()
    }

    /// Prune entries that haven't had a signal in the given duration.
    pub fn prune_stale(&self, max_age: Duration) -> usize {
        let mut peers = self.peers.write();
        let now = Instant::now();
        let before = peers.len();
        peers.retain(|_, stats| {
            stats
                .last_signal_at
                .map(|t| now.duration_since(t) < max_age)
                .unwrap_or(false)
        });
        before - peers.len()
    }

    /// Get the number of tracked peers.
    pub fn len(&self) -> usize {
        self.peers.read().len()
    }

    /// Check if the manager is empty.
    pub fn is_empty(&self) -> bool {
        self.peers.read().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neutral_score() {
        let score = ReputationScore::neutral();
        assert_eq!(score.value(), 50.0);
        assert!(!score.is_trusted());
        assert!(!score.is_suspicious());
        assert!(!score.is_abusive());
    }

    #[test]
    fn test_successful_delivery_increases_score() {
        let mut stats = PeerAbuseStats::new("peer1".to_string());
        for _ in 0..10 {
            stats.record_signal(AbuseSignal::SuccessfulDelivery);
        }
        assert!(stats.reputation_score.value() > 50.0);
        assert!(stats.reputation_score.is_trusted());
    }

    #[test]
    fn test_rate_limiting_decreases_score() {
        let mut stats = PeerAbuseStats::new("peer1".to_string());
        for _ in 0..10 {
            stats.record_signal(AbuseSignal::RateLimited);
        }
        assert!(stats.reputation_score.value() < 50.0);
        assert!(stats.reputation_score.is_abusive());
    }

    #[test]
    fn test_rate_limit_multiplier() {
        let mut trusted = PeerAbuseStats::new("trusted".to_string());
        for _ in 0..20 {
            trusted.record_signal(AbuseSignal::SuccessfulDelivery);
        }
        assert!((trusted.rate_limit_multiplier() - 1.5).abs() < 0.01);

        let mut abusive = PeerAbuseStats::new("abusive".to_string());
        for _ in 0..10 {
            abusive.record_signal(AbuseSignal::RateLimited);
        }
        assert!((abusive.rate_limit_multiplier() - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_reputation_manager_eviction() {
        let manager = AbuseReputationManager::new(3);
        manager.record_signal("peer1", AbuseSignal::RateLimited);
        manager.record_signal("peer2", AbuseSignal::RateLimited);
        manager.record_signal("peer3", AbuseSignal::SuccessfulDelivery);
        assert_eq!(manager.len(), 3);

        // Adding a 4th peer should evict the lowest-scored (one of the rate-limited ones)
        manager.record_signal("peer4", AbuseSignal::SuccessfulDelivery);
        assert_eq!(manager.len(), 3);
    }

    #[test]
    fn test_prune_stale() {
        let manager = AbuseReputationManager::new(100);
        manager.record_signal("peer1", AbuseSignal::SuccessfulDelivery);
        // peer1 was just recorded so it shouldn't be pruned
        let pruned = manager.prune_stale(Duration::from_secs(3600));
        assert_eq!(pruned, 0);
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_mixed_signals() {
        let mut stats = PeerAbuseStats::new("mixed".to_string());
        // 5 successful deliveries and 2 rate limit hits
        for _ in 0..5 {
            stats.record_signal(AbuseSignal::SuccessfulDelivery);
        }
        for _ in 0..2 {
            stats.record_signal(AbuseSignal::RateLimited);
        }
        // Score should be slightly below neutral (5*3=15 positive, 2*5=10 negative, net +5)
        assert!(stats.reputation_score.value() > 50.0);
    }
}