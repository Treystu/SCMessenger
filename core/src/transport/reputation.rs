//! P0_SECURITY_002: Peer reputation scoring for abuse mitigation.
//!
//! Tracks abuse signals (rate limit hits, invalid messages, spam patterns)
//! alongside relay quality. Produces a composite reputation score that
//! influences rate limits and relay decisions.
//!
//! P0_ANTI_ABUSE_001: Reputation scores persist across sessions via StorageBackend.
//! Time-based decay gradually returns inactive peers toward neutral.

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::store::backend::StorageBackend;

/// Composite reputation score for a peer, ranging from 0.0 (abusive) to 100.0 (trusted).
/// A score of 50.0 is neutral (new peer default).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Epoch seconds when the last signal was recorded (for persistence + decay).
    pub last_signal_epoch_secs: Option<u64>,
    /// In-memory instant for decay calculations (not serialized).
    #[serde(skip)]
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
            last_signal_epoch_secs: None,
            last_signal_at: Some(Instant::now()),
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
        self.last_signal_epoch_secs = Some(current_epoch_secs());
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
/// Supports persistence via StorageBackend and time-based reputation decay.
pub struct AbuseReputationManager {
    peers: RwLock<HashMap<String, PeerAbuseStats>>,
    max_tracked_peers: usize,
    /// Optional storage backend for persisting reputation data across sessions.
    storage: Option<Arc<dyn StorageBackend>>,
    /// Time-to-live for reputation entries: peers with no signals for this long
    /// have their scores decayed toward neutral.
    decay_ttl: Duration,
}

const STORAGE_PREFIX: &[u8] = b"reputation:";

impl AbuseReputationManager {
    /// Create a new in-memory reputation manager (no persistence).
    pub fn new(max_tracked_peers: usize) -> Self {
        Self {
            peers: RwLock::new(HashMap::new()),
            max_tracked_peers,
            storage: None,
            decay_ttl: Duration::from_secs(7 * 24 * 3600), // 7 days default
        }
    }

    /// Create a reputation manager with persistent storage.
    /// Loads existing reputation data from storage on creation.
    pub fn with_backend(max_tracked_peers: usize, backend: Arc<dyn StorageBackend>) -> Self {
        let manager = Self {
            peers: RwLock::new(HashMap::new()),
            max_tracked_peers,
            storage: Some(backend),
            decay_ttl: Duration::from_secs(7 * 24 * 3600),
        };
        manager.load_from_storage();
        manager
    }

    /// Load all persisted reputation entries from storage.
    fn load_from_storage(&self) {
        if let Some(ref storage) = self.storage {
            match storage.scan_prefix(STORAGE_PREFIX) {
                Ok(entries) => {
                    let mut peers = self.peers.write();
                    for (_, value) in entries {
                        if let Ok(stats) = serde_json::from_slice::<PeerAbuseStats>(&value) {
                            // Reconstruct Instant from epoch time (approximate)
                            let stats = PeerAbuseStats {
                                last_signal_at: stats.last_signal_epoch_secs
                                    .map(|_| Instant::now()),
                                ..stats
                            };
                            peers.insert(stats.peer_id.clone(), stats);
                        }
                    }
                    // Enforce capacity limit after loading
                    if peers.len() > self.max_tracked_peers {
                        let mut entries: Vec<_> = peers.iter()
                            .map(|(k, s)| (k.clone(), s.reputation_score.value() as u32))
                            .collect();
                        entries.sort_by_key(|(_, v)| *v);
                        let remove_count = peers.len() - self.max_tracked_peers;
                        for (key, _) in entries.iter().take(remove_count) {
                            peers.remove(key);
                        }
                    }
                    tracing::info!("Loaded {} reputation entries from storage", peers.len());
                }
                Err(e) => {
                    tracing::warn!("Failed to load reputation data from storage: {}", e);
                }
            }
        }
    }

    /// Persist a single peer's reputation data to storage.
    fn persist_peer(&self, peer_id: &str, stats: &PeerAbuseStats) {
        if let Some(ref storage) = self.storage {
            let key = [STORAGE_PREFIX, peer_id.as_bytes()].concat();
            match serde_json::to_vec(stats) {
                Ok(value) => {
                    if let Err(e) = storage.put(&key, &value) {
                        tracing::debug!("Failed to persist reputation for {}: {}", peer_id, e);
                    }
                }
                Err(e) => {
                    tracing::debug!("Failed to serialize reputation for {}: {}", peer_id, e);
                }
            }
        }
    }

    /// Remove a peer's reputation data from storage.
    fn remove_peer_from_storage(&self, peer_id: &str) {
        if let Some(ref storage) = self.storage {
            let key = [STORAGE_PREFIX, peer_id.as_bytes()].concat();
            if let Err(e) = storage.remove(&key) {
                tracing::debug!("Failed to remove reputation for {}: {}", peer_id, e);
            }
        }
    }

    /// Apply time-based reputation decay for all tracked peers.
    /// Peers with no signals for longer than `decay_ttl` have their scores
    /// gradually moved toward neutral (50.0).
    pub fn apply_decay(&self) {
        let now_epoch = current_epoch_secs();
        let decay_secs = self.decay_ttl.as_secs();
        let mut peers = self.peers.write();
        let mut to_persist = Vec::new();

        for (peer_id, stats) in peers.iter_mut() {
            if let Some(last_epoch) = stats.last_signal_epoch_secs {
                let elapsed_secs = now_epoch.saturating_sub(last_epoch);
                if elapsed_secs > decay_secs {
                    // Decay factor: how far past the TTL (capped at 1.0)
                    let decay_ratio = ((elapsed_secs - decay_secs) as f64 / decay_secs as f64).min(1.0);
                    // Move score toward neutral by the decay ratio (max 50% per call)
                    let current = stats.reputation_score.value();
                    let neutral = ReputationScore::NEUTRAL;
                    let new_value = current + (neutral - current) * decay_ratio * 0.5;
                    stats.reputation_score = ReputationScore::new(new_value);

                    // Also decay signal counts proportionally
                    let count_decay = 1.0 - (decay_ratio * 0.3);
                    stats.rate_limit_hits = (stats.rate_limit_hits as f64 * count_decay) as u32;
                    stats.oversized_messages = (stats.oversized_messages as f64 * count_decay) as u32;
                    stats.invalid_format_count = (stats.invalid_format_count as f64 * count_decay) as u32;
                    stats.duplicate_count = (stats.duplicate_count as f64 * count_decay) as u32;
                    stats.invalid_destination_count = (stats.invalid_destination_count as f64 * count_decay) as u32;
                    stats.failed_relays = (stats.failed_relays as f64 * count_decay) as u32;
                    stats.connection_timeouts = (stats.connection_timeouts as f64 * count_decay) as u32;

                    to_persist.push(peer_id.clone());
                }
            }
        }

        // Persist decayed entries
        drop(peers);
        let peers = self.peers.read();
        for peer_id in to_persist {
            if let Some(stats) = peers.get(&peer_id) {
                self.persist_peer(&peer_id, stats);
            }
        }
    }

    /// Flush all reputation data to persistent storage.
    pub fn flush_to_storage(&self) {
        if self.storage.is_none() {
            return;
        }
        let peers = self.peers.read();
        for (peer_id, stats) in peers.iter() {
            self.persist_peer(peer_id, stats);
        }
        if let Some(ref storage) = self.storage {
            if let Err(e) = storage.flush() {
                tracing::debug!("Failed to flush reputation storage: {}", e);
            }
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
                self.remove_peer_from_storage(&lowest_key);
                peers.remove(&lowest_key);
            }
        }

        let stats = peers
            .entry(peer_id.to_string())
            .or_insert_with(|| PeerAbuseStats::new(peer_id.to_string()));
        stats.record_signal(signal);
        let score = stats.reputation_score;
        let stats_clone = stats.clone();
        drop(peers);
        self.persist_peer(peer_id, &stats_clone);
        score
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
    /// Also removes them from persistent storage.
    pub fn prune_stale(&self, max_age: Duration) -> usize {
        let mut peers = self.peers.write();
        let now = Instant::now();
        let before = peers.len();
        let mut to_remove = Vec::new();
        peers.retain(|key, stats| {
            let keep = stats
                .last_signal_at
                .map(|t| now.duration_since(t) < max_age)
                .unwrap_or(false);
            if !keep {
                to_remove.push(key.clone());
            }
            keep
        });
        drop(peers);
        for key in to_remove {
            self.remove_peer_from_storage(&key);
        }
        before - self.peers.read().len()
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

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
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

    #[test]
    fn test_persistence_roundtrip() {
        let backend = Arc::new(crate::store::backend::MemoryStorage::new());
        let manager = AbuseReputationManager::with_backend(100, backend.clone());

        // Record some signals
        manager.record_signal("peer1", AbuseSignal::SuccessfulDelivery);
        manager.record_signal("peer1", AbuseSignal::SuccessfulDelivery);
        manager.record_signal("peer2", AbuseSignal::RateLimited);

        let score1 = manager.get_score("peer1");
        let score2 = manager.get_score("peer2");
        assert!(score1.value() > 50.0);
        assert!(score2.value() < 50.0);

        // Flush to storage
        manager.flush_to_storage();

        // Load into a new manager — should restore the data
        let manager2 = AbuseReputationManager::with_backend(100, backend);
        let restored1 = manager2.get_score("peer1");
        let restored2 = manager2.get_score("peer2");
        assert!((restored1.value() - score1.value()).abs() < 0.01);
        assert!((restored2.value() - score2.value()).abs() < 0.01);
    }

    #[test]
    fn test_persistence_eviction_cleans_storage() {
        let backend = Arc::new(crate::store::backend::MemoryStorage::new());
        let manager = AbuseReputationManager::with_backend(2, backend.clone());

        manager.record_signal("peer1", AbuseSignal::RateLimited);
        manager.record_signal("peer2", AbuseSignal::RateLimited);
        manager.record_signal("peer3", AbuseSignal::SuccessfulDelivery); // evicts lowest

        // peer3 should exist, one of peer1/peer2 should be evicted
        assert_eq!(manager.len(), 2);

        // Storage should not have the evicted peer
        let stored_keys: Vec<_> = backend.scan_prefix(STORAGE_PREFIX)
            .unwrap_or_default()
            .into_iter()
            .map(|(k, _)| String::from_utf8_lossy(&k).to_string())
            .collect();
        assert_eq!(stored_keys.len(), 2);
    }

    #[test]
    fn test_decay_moves_toward_neutral() {
        let manager = AbuseReputationManager::new(100);

        // Create a peer with bad reputation
        for _ in 0..20 {
            manager.record_signal("bad_peer", AbuseSignal::RateLimited);
        }
        let bad_score = manager.get_score("bad_peer");
        assert!(bad_score.value() < 20.0);

        // Manually set the last_signal_epoch to simulate aging
        {
            let mut peers = manager.peers.write();
            if let Some(stats) = peers.get_mut("bad_peer") {
                // Set last signal to 14 days ago (2x the 7-day decay TTL)
                stats.last_signal_epoch_secs = Some(current_epoch_secs() - 14 * 24 * 3600);
            }
        }

        manager.apply_decay();

        // After decay, score should have moved toward neutral
        let decayed_score = manager.get_score("bad_peer");
        assert!(decayed_score.value() > bad_score.value());
        assert!(decayed_score.value() < ReputationScore::NEUTRAL);
    }

    #[test]
    fn test_epoch_secs_recorded() {
        let mut stats = PeerAbuseStats::new("test".to_string());
        assert!(stats.last_signal_epoch_secs.is_none());
        stats.record_signal(AbuseSignal::SuccessfulDelivery);
        assert!(stats.last_signal_epoch_secs.is_some());
        // Should be approximately now
        let now = current_epoch_secs();
        let recorded = stats.last_signal_epoch_secs.unwrap();
        assert!(now >= recorded);
        assert!(now - recorded < 5); // within 5 seconds
    }
}