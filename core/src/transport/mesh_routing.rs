// Mesh Routing: Relay, Reputation, and Retry Logic (Phases 3-6)
//
// Implements the sovereign mesh routing system where:
// - Every node can relay messages for others (Phase 3)
// - Nodes track relay performance and reputation (Phase 5)
// - Message delivery uses multi-path retry with continuous adaptation (Phase 6)
// - Any node can bootstrap from any other node (Phase 4)

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================================================
// PHASE 3: RELAY CAPABILITY
// ============================================================================

/// Relay statistics for a peer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelayStats {
    /// Total messages relayed through this peer
    pub messages_relayed: u64,
    /// Total bytes relayed
    pub bytes_relayed: u64,
    /// Messages successfully delivered
    pub successful_deliveries: u64,
    /// Messages that failed or timed out
    pub failed_deliveries: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// When this peer was last used as a relay
    pub last_used: u64,
}

// ============================================================================
// PHASE 5: REPUTATION TRACKING
// ============================================================================

/// Reputation score for a relay peer
#[derive(Debug, Clone)]
pub struct RelayReputation {
    /// Peer ID
    pub peer_id: PeerId,
    /// Statistics
    pub stats: RelayStats,
    /// Calculated reputation score (0-100)
    pub score: f64,
    /// Is this peer currently considered reliable?
    pub is_reliable: bool,
}

impl RelayReputation {
    /// Calculate reputation score based on statistics
    pub fn calculate_score(&mut self) {
        if self.stats.messages_relayed == 0 {
            self.score = 50.0; // Neutral score for new peers
            self.is_reliable = true;
            return;
        }

        let success_rate =
            self.stats.successful_deliveries as f64 / self.stats.messages_relayed as f64;

        // Score factors:
        // - Success rate (70% weight)
        // - Latency (20% weight - lower is better)
        // - Recency (10% weight - recent usage preferred)

        let success_score = success_rate * 70.0;

        let latency_score = if self.stats.avg_latency_ms < 100 {
            20.0
        } else if self.stats.avg_latency_ms < 500 {
            15.0
        } else if self.stats.avg_latency_ms < 1000 {
            10.0
        } else {
            5.0
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age_secs = now.saturating_sub(self.stats.last_used);
        let recency_score = if age_secs < 60 {
            10.0
        } else if age_secs < 300 {
            7.0
        } else if age_secs < 3600 {
            5.0
        } else {
            2.0
        };

        self.score = success_score + latency_score + recency_score;
        self.is_reliable = self.score >= 50.0;
    }
}

/// Tracks reputation of all known relay peers
#[derive(Debug, Clone)]
pub struct ReputationTracker {
    reputations: HashMap<PeerId, RelayReputation>,
}

impl Default for ReputationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ReputationTracker {
    pub fn new() -> Self {
        Self {
            reputations: HashMap::new(),
        }
    }

    /// Record a relay attempt
    pub fn record_relay_attempt(
        &mut self,
        peer_id: PeerId,
        success: bool,
        latency_ms: u64,
        bytes: u64,
    ) {
        let rep = self.reputations.entry(peer_id).or_insert(RelayReputation {
            peer_id,
            stats: RelayStats::default(),
            score: 50.0,
            is_reliable: true,
        });

        rep.stats.messages_relayed += 1;
        rep.stats.bytes_relayed += bytes;

        if success {
            rep.stats.successful_deliveries += 1;
        } else {
            rep.stats.failed_deliveries += 1;
        }

        // Update average latency (moving average)
        rep.stats.avg_latency_ms = (rep.stats.avg_latency_ms + latency_ms) / 2;

        rep.stats.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        rep.calculate_score();
    }

    /// Get best relay peers (sorted by reputation)
    pub fn best_relays(&self, count: usize) -> Vec<PeerId> {
        let mut peers: Vec<_> = self.reputations.values().collect();
        peers.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        peers
            .into_iter()
            .filter(|r| r.is_reliable)
            .take(count)
            .map(|r| r.peer_id)
            .collect()
    }

    /// Get reputation for a specific peer
    pub fn get_reputation(&self, peer_id: &PeerId) -> Option<&RelayReputation> {
        self.reputations.get(peer_id)
    }

    /// Get all reputations
    pub fn all_reputations(&self) -> Vec<RelayReputation> {
        self.reputations.values().cloned().collect()
    }
}

// ============================================================================
// PHASE 6: CONTINUOUS RETRY LOGIC
// ============================================================================

/// Retry strategy for message delivery
#[derive(Debug, Clone)]
pub struct RetryStrategy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to use exponential backoff
    pub use_exponential_backoff: bool,
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: 10, // Never give up easily
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 1.5,
            use_exponential_backoff: true,
        }
    }
}

impl RetryStrategy {
    /// Calculate delay for a given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if !self.use_exponential_backoff {
            return self.initial_delay;
        }

        let delay_ms =
            self.initial_delay.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32);

        let delay = Duration::from_millis(delay_ms as u64);

        delay.min(self.max_delay)
    }

    /// Should we retry after this many attempts?
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}

/// Tracks ongoing delivery attempts
#[derive(Debug, Clone)]
pub struct DeliveryAttempt {
    /// Message ID
    pub message_id: String,
    /// Target peer
    pub target_peer: PeerId,
    /// Attempt number (0-indexed)
    pub attempt: u32,
    /// Paths tried so far (direct or via relays)
    pub paths_tried: Vec<Vec<PeerId>>,
    /// Last attempt timestamp
    pub last_attempt: u64,
    /// Retry strategy
    pub strategy: RetryStrategy,
}

impl DeliveryAttempt {
    pub fn new(message_id: String, target_peer: PeerId) -> Self {
        Self {
            message_id,
            target_peer,
            attempt: 0,
            paths_tried: Vec::new(),
            last_attempt: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            strategy: RetryStrategy::default(),
        }
    }

    /// Get next retry delay
    pub fn next_retry_delay(&self) -> Duration {
        self.strategy.calculate_delay(self.attempt)
    }

    /// Should we retry?
    pub fn should_retry(&self) -> bool {
        self.strategy.should_retry(self.attempt)
    }

    /// Record a failed attempt via a specific path
    pub fn record_failure(&mut self, path: Vec<PeerId>) {
        self.paths_tried.push(path);
        self.attempt += 1;
        self.last_attempt = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Multi-path delivery manager
#[derive(Debug)]
pub struct MultiPathDelivery {
    /// Active delivery attempts
    attempts: HashMap<String, DeliveryAttempt>,
    /// Reputation tracker for selecting best paths
    reputation: ReputationTracker,
}

impl Default for MultiPathDelivery {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiPathDelivery {
    pub fn new() -> Self {
        Self {
            attempts: HashMap::new(),
            reputation: ReputationTracker::new(),
        }
    }

    /// Start a delivery attempt
    pub fn start_delivery(&mut self, message_id: String, target_peer: PeerId) {
        let attempt = DeliveryAttempt::new(message_id.clone(), target_peer);
        self.attempts.insert(message_id, attempt);
    }

    /// Get best paths to try (direct + relay options)
    pub fn get_best_paths(&self, target: &PeerId, count: usize) -> Vec<Vec<PeerId>> {
        let mut paths = Vec::new();

        // Path 1: Direct connection
        paths.push(vec![*target]);

        // Path 2-N: Via best relays
        for relay in self.reputation.best_relays(count.saturating_sub(1)) {
            if relay != *target {
                paths.push(vec![relay, *target]);
            }
        }

        paths
    }

    /// Record delivery success
    pub fn record_success(&mut self, message_id: &str, path: Vec<PeerId>, latency_ms: u64) {
        // Remove from active attempts
        self.attempts.remove(message_id);

        // Update reputation for relays in the path
        if path.len() > 1 {
            for relay in &path[..path.len() - 1] {
                self.reputation
                    .record_relay_attempt(*relay, true, latency_ms, 1024);
            }
        }
    }

    /// Record delivery failure
    pub fn record_failure(&mut self, message_id: &str, path: Vec<PeerId>) {
        if let Some(attempt) = self.attempts.get_mut(message_id) {
            attempt.record_failure(path.clone());

            // Update reputation for relays that failed
            if path.len() > 1 {
                for relay in &path[..path.len() - 1] {
                    self.reputation
                        .record_relay_attempt(*relay, false, 10000, 0);
                }
            }
        }
    }

    /// Get pending delivery attempts
    pub fn pending_attempts(&self) -> Vec<&DeliveryAttempt> {
        self.attempts.values().collect()
    }

    /// Get reputation tracker
    pub fn reputation(&self) -> &ReputationTracker {
        &self.reputation
    }
}

// ============================================================================
// PHASE 4: MESH-BASED DISCOVERY
// ============================================================================

/// Bootstrap capability - any node can help others join the network
#[derive(Debug, Clone)]
pub struct BootstrapCapability {
    /// Peers we know about (potential bootstrap candidates)
    pub known_peers: Vec<PeerId>,
    /// Last time we updated our peer list
    pub last_update: u64,
}

impl Default for BootstrapCapability {
    fn default() -> Self {
        Self::new()
    }
}

impl BootstrapCapability {
    pub fn new() -> Self {
        Self {
            known_peers: Vec::new(),
            last_update: 0,
        }
    }

    /// Add a peer as a potential bootstrap node
    pub fn add_peer(&mut self, peer_id: PeerId) {
        if !self.known_peers.contains(&peer_id) {
            self.known_peers.push(peer_id);
            self.last_update = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    /// Get bootstrap candidates (all stable peers)
    pub fn get_bootstrap_candidates(&self) -> &[PeerId] {
        &self.known_peers
    }

    /// Can this node help others bootstrap?
    pub fn can_bootstrap_others(&self) -> bool {
        !self.known_peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_calculation() {
        let mut rep = RelayReputation {
            peer_id: PeerId::random(),
            stats: RelayStats {
                messages_relayed: 100,
                successful_deliveries: 95,
                failed_deliveries: 5,
                avg_latency_ms: 50,
                ..Default::default()
            },
            score: 0.0,
            is_reliable: false,
        };

        rep.calculate_score();

        assert!(
            rep.score > 80.0,
            "High success rate should yield high score"
        );
        assert!(rep.is_reliable, "Should be marked as reliable");
    }

    #[test]
    fn test_retry_strategy() {
        let strategy = RetryStrategy::default();

        assert_eq!(strategy.calculate_delay(0), Duration::from_millis(100));
        assert!(strategy.calculate_delay(1) > Duration::from_millis(100));
        assert!(strategy.calculate_delay(5) < strategy.max_delay);

        assert!(strategy.should_retry(5));
        assert!(!strategy.should_retry(100));
    }

    #[test]
    fn test_multi_path_delivery() {
        let mut delivery = MultiPathDelivery::new();
        let target = PeerId::random();
        let message_id = "test-message-123".to_string();

        delivery.start_delivery(message_id.clone(), target);

        let paths = delivery.get_best_paths(&target, 3);
        assert!(!paths.is_empty(), "Should provide at least direct path");
        assert_eq!(paths[0], vec![target], "First path should be direct");

        delivery.record_failure(&message_id, vec![target]);

        let pending = delivery.pending_attempts();
        assert_eq!(pending.len(), 1, "Should have one pending attempt");
    }
}
