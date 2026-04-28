//! Enhanced reputation system with spam detection integration.
//!
//! Extends the existing reputation system with:
//! 1. Community-based spam detection signals
//! 2. Evidence-preserving blocking (messages still route/relay)
//! 3. Sophisticated abuse pattern recognition
//! 4. Automatic blocking integration

use std::sync::Arc;
use crate::transport::reputation::{AbuseReputationManager, AbuseSignal, ReputationScore};
use crate::abuse::spam_detection::{SpamDetectionEngine, SpamSignal};

/// Enhanced abuse reputation manager with spam detection
pub struct EnhancedAbuseReputationManager {
    base_manager: AbuseReputationManager,
    spam_detector: SpamDetectionEngine,
}

impl EnhancedAbuseReputationManager {
    pub fn new(
        max_tracked_peers: usize,
        spam_detector: SpamDetectionEngine,
    ) -> Self {
        Self {
            base_manager: AbuseReputationManager::new(max_tracked_peers),
            spam_detector,
        }
    }

    /// Create with persistent storage backend.
    /// Reputation data will be loaded from storage and persisted across sessions.
    pub fn with_backend(
        max_tracked_peers: usize,
        spam_detector: SpamDetectionEngine,
        backend: Arc<dyn crate::store::backend::StorageBackend>,
    ) -> Self {
        Self {
            base_manager: AbuseReputationManager::with_backend(max_tracked_peers, backend),
            spam_detector,
        }
    }

    /// Apply time-based reputation decay to all tracked peers.
    pub fn apply_decay(&self) {
        self.base_manager.apply_decay();
    }

    /// Flush reputation data to persistent storage.
    pub fn flush_to_storage(&self) {
        self.base_manager.flush_to_storage();
    }

    /// Record an abuse signal and update reputation score.
    /// Additionally checks for spam patterns when duplicate/invalid signals occur.
    pub fn record_signal(&self, peer_id: &str, signal: AbuseSignal) -> ReputationScore {
        let score = self.base_manager.record_signal(peer_id, signal);

        // Check for spam patterns when certain signals occur
        if matches!(signal, AbuseSignal::DuplicateMessage | AbuseSignal::InvalidFormat) {
            let spam_result = self.spam_detector.detect_spam(peer_id);
            if spam_result.is_spam {
                self.base_manager.record_signal(
                    peer_id,
                    AbuseSignal::InvalidFormat,
                );
            }
        }

        score
    }

    /// Record a spam-specific signal and translate it to base reputation signals.
    pub fn record_spam_signal(&self, peer_id: &str, signal: SpamSignal) {
        // Record in spam detector for heuristic tracking
        self.spam_detector.record_spam_signal(peer_id, signal);

        // Map spam signals to base abuse signals
        match signal {
            SpamSignal::CommunityBlocked => {
                // Handled by the spam detector; no additional base signal needed
            },
            SpamSignal::ContentPattern => {
                self.base_manager.record_signal(peer_id, AbuseSignal::InvalidFormat);
            },
            SpamSignal::Flooding => {
                self.base_manager.record_signal(peer_id, AbuseSignal::RateLimited);
            },
            SpamSignal::MassDistribution => {
                self.base_manager.record_signal(peer_id, AbuseSignal::DuplicateMessage);
            },
            SpamSignal::ColdContactSpam => {
                self.base_manager.record_signal(peer_id, AbuseSignal::InvalidDestination);
            },
        }
    }

    /// Record an outbound message for spam heuristic tracking.
    /// Delegates to the spam detection engine.
    pub fn record_outbound_message(&self, peer_id: &str, envelope_data: &[u8], is_to_contact: bool) {
        self.spam_detector.record_outbound_message(peer_id, envelope_data, is_to_contact);
    }

    /// Get enhanced reputation score that includes spam detection
    pub fn get_enhanced_score(&self, peer_id: &str) -> EnhancedReputationScore {
        let base_score = self.base_manager.get_score(peer_id);
        let spam_score = self.spam_detector.spam_score(peer_id);

        EnhancedReputationScore {
            base_score,
            spam_confidence: spam_score,
            is_community_flagged: spam_score > 0.5,
        }
    }

    /// Get the reputation score for a peer
    pub fn get_score(&self, peer_id: &str) -> ReputationScore {
        self.base_manager.get_score(peer_id)
    }

    /// Get the rate limit multiplier for a peer
    pub fn rate_limit_multiplier(&self, peer_id: &str) -> f64 {
        self.base_manager.rate_limit_multiplier(peer_id)
    }

    /// Get all enhanced reputation entries (for auto-block batch evaluation)
    pub fn all_enhanced_scores(&self) -> Vec<(String, EnhancedReputationScore)> {
        let base_reputations = self.base_manager.all_reputations();
        base_reputations
            .into_iter()
            .map(|(peer_id, base_score)| {
                let spam_score = self.spam_detector.spam_score(&peer_id);
                (
                    peer_id,
                    EnhancedReputationScore {
                        base_score,
                        spam_confidence: spam_score,
                        is_community_flagged: spam_score > 0.5,
                    },
                )
            })
            .collect()
    }

    /// Get a reference to the spam detector for direct access
    pub fn spam_detector(&self) -> &SpamDetectionEngine {
        &self.spam_detector
    }

    /// Get a reference to the base reputation manager
    pub fn base_manager(&self) -> &AbuseReputationManager {
        &self.base_manager
    }
}

/// Enhanced reputation score that includes spam detection information
#[derive(Debug, Clone)]
pub struct EnhancedReputationScore {
    /// Base reputation score from abuse signals
    pub base_score: ReputationScore,
    /// Confidence level that this peer is spam (0.0 to 1.0)
    pub spam_confidence: f64,
    /// Whether this peer is flagged by community spam detection
    pub is_community_flagged: bool,
}

impl EnhancedReputationScore {
    /// Overall trust score combining base reputation and spam detection.
    /// Max 80% reduction for high spam confidence.
    pub fn overall_score(&self) -> f64 {
        let base_value = self.base_score.value();
        base_value * (1.0 - self.spam_confidence * 0.8)
    }

    /// Whether this peer should be treated with caution
    pub fn is_suspicious(&self) -> bool {
        self.base_score.is_suspicious() || self.is_community_flagged
    }

    /// Whether this peer should be heavily restricted
    pub fn is_abusive(&self) -> bool {
        self.base_score.is_abusive() || self.spam_confidence > 0.8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::store::backend::MemoryStorage;
    use crate::store::contacts::ContactManager;
    use crate::store::blocked::BlockedManager;
    use crate::abuse::spam_detection::SpamDetectionConfig;

    fn make_manager() -> EnhancedAbuseReputationManager {
        let backend = Arc::new(MemoryStorage::new());
        let contacts = Arc::new(ContactManager::new(backend.clone()));
        let blocked = Arc::new(BlockedManager::new(backend));
        let spam_detector = SpamDetectionEngine::new(
            SpamDetectionConfig::default(),
            contacts,
            blocked,
        );
        EnhancedAbuseReputationManager::new(1000, spam_detector)
    }

    #[test]
    fn test_neutral_peer_has_neutral_score() {
        let mgr = make_manager();
        let score = mgr.get_score("new_peer");
        assert_eq!(score.value(), ReputationScore::NEUTRAL);
    }

    #[test]
    fn test_positive_signals_increase_score() {
        let mgr = make_manager();
        for _ in 0..10 {
            mgr.record_signal("good_peer", AbuseSignal::SuccessfulDelivery);
        }
        let score = mgr.get_score("good_peer");
        assert!(score.value() > 50.0);
    }

    #[test]
    fn test_negative_signals_decrease_score() {
        let mgr = make_manager();
        for _ in 0..10 {
            mgr.record_signal("bad_peer", AbuseSignal::RateLimited);
        }
        let score = mgr.get_score("bad_peer");
        assert!(score.value() < 50.0);
    }

    #[test]
    fn test_enhanced_score_combines_base_and_spam() {
        let mgr = make_manager();
        let enhanced = mgr.get_enhanced_score("new_peer");
        assert_eq!(enhanced.base_score.value(), 50.0);
        // No contacts means no spam detection, so spam_confidence should be 0
        assert_eq!(enhanced.spam_confidence, 0.0);
    }

    #[test]
    fn test_spam_signal_recording() {
        let mgr = make_manager();
        mgr.record_spam_signal("spammer", SpamSignal::Flooding);
        let score = mgr.get_score("spammer");
        // Flooding maps to RateLimited, which decreases score
        assert!(score.value() < 50.0);
    }

    #[test]
    fn test_all_enhanced_scores() {
        let mgr = make_manager();
        mgr.record_signal("peer1", AbuseSignal::RateLimited);
        mgr.record_signal("peer2", AbuseSignal::SuccessfulDelivery);
        let all = mgr.all_enhanced_scores();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_outbound_message_tracking() {
        let mgr = make_manager();
        mgr.record_outbound_message("sender", b"hello", true);
        mgr.record_outbound_message("sender", b"hello", false);
        // Should not panic and should track via spam detector
        let score = mgr.get_score("sender");
        assert_eq!(score.value(), 50.0); // No reputation change from tracking
    }
}