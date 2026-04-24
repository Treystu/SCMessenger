//! Peer reputation tracking (Phase 2 API stub)
//!
//! Placeholder implementation for the ReputationTracker used by integration tests.
//! Full implementation will be delivered in Phase 2.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Tracks peer reputation scores for routing decisions
#[derive(Debug, Clone)]
pub struct ReputationTracker {
    scores: HashMap<u64, i64>,
    decay_half_life: Duration,
}

impl Default for ReputationTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ReputationTracker {
    /// Create a new reputation tracker with default settings
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            decay_half_life: Duration::from_secs(3600),
        }
    }

    /// Record a positive interaction with a peer
    pub fn record_success(&mut self, peer_id: u64) {
        let entry = self.scores.entry(peer_id).or_insert(0);
        *entry = entry.saturating_add(10);
    }

    /// Record a negative interaction with a peer
    pub fn record_failure(&mut self, peer_id: u64) {
        let entry = self.scores.entry(peer_id).or_insert(0);
        *entry = entry.saturating_sub(5);
    }

    /// Get the reputation score for a peer (higher is better)
    pub fn score(&self, peer_id: u64) -> i64 {
        self.scores.get(&peer_id).copied().unwrap_or(0)
    }

    /// Apply time-based decay to all scores
    pub fn apply_decay(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for score in self.scores.values_mut() {
            if *score > 0 {
                let factor = (now % 3600) as i64;
                *score = score.saturating_sub(factor);
            }
        }
    }

    /// Remove all scores below the threshold
    pub fn prune_below(&mut self, threshold: i64) {
        self.scores.retain(|_, score| *score >= threshold);
    }

    /// Get the number of tracked peers
    pub fn len(&self) -> usize {
        self.scores.len()
    }

    /// Check if the tracker is empty
    pub fn is_empty(&self) -> bool {
        self.scores.is_empty()
    }
}