//! P0_SECURITY_003: Automatic blocking based on reputation thresholds.
//!
//! Implements threshold-based automatic blocking with:
//! 1. Configurable reputation score thresholds for auto-block
//! 2. Configurable spam confidence thresholds for auto-block
//! 3. Audit trail for all automatic blocking actions
//! 4. Manual override capability (whitelist/exempt)
//! 5. Evidence-preserving blocking (messages still route/relay)

use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::store::blocked::BlockedManager;
use crate::transport::reputation::ReputationScore;
use crate::abuse::reputation::EnhancedAbuseReputationManager;

/// Configuration for automatic blocking behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBlockConfig {
    /// Reputation score below which a peer is automatically blocked (0-100)
    pub reputation_threshold: f64,
    /// Spam confidence above which a peer is automatically blocked (0.0-1.0)
    pub spam_confidence_threshold: f64,
    /// Whether automatic blocking is enabled
    pub enabled: bool,
    /// Peers exempt from automatic blocking (whitelist)
    pub exempt_peer_ids: Vec<String>,
    /// Maximum number of audit log entries to retain
    pub max_audit_entries: usize,
}

impl Default for AutoBlockConfig {
    fn default() -> Self {
        Self {
            reputation_threshold: 10.0, // Block when reputation drops below 10 (abusive range)
            spam_confidence_threshold: 0.8, // Block when 80%+ spam confidence
            enabled: true,
            exempt_peer_ids: Vec::new(),
            max_audit_entries: 1024,
        }
    }
}

/// Reason a peer was automatically blocked
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoBlockReason {
    /// Peer reputation dropped below threshold
    LowReputation,
    /// Peer spam confidence exceeded threshold
    HighSpamConfidence,
    /// Both reputation and spam confidence triggered
    CombinedAbuse,
}

/// Audit entry for an automatic blocking action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBlockAuditEntry {
    /// Peer that was blocked
    pub peer_id: String,
    /// Reason for the block
    pub reason: AutoBlockReason,
    /// Reputation score at the time of blocking
    pub reputation_score: f64,
    /// Spam confidence at the time of blocking
    pub spam_confidence: f64,
    /// Unix timestamp (seconds) when the block occurred
    pub timestamp_secs: u64,
    /// Whether this was an auto-block or manual override
    pub was_automatic: bool,
}

/// Result of an auto-block evaluation
#[derive(Debug, Clone)]
pub struct AutoBlockResult {
    /// Whether the peer should be blocked
    pub should_block: bool,
    /// Reason for the block (if applicable)
    pub reason: Option<AutoBlockReason>,
    /// Current reputation score
    pub reputation_score: f64,
    /// Current spam confidence
    pub spam_confidence: f64,
}

/// Automatic blocking engine that evaluates peers against thresholds
/// and blocks them via the BlockedManager when criteria are met.
pub struct AutoBlockEngine {
    config: RwLock<AutoBlockConfig>,
    blocked_manager: Arc<BlockedManager>,
    reputation_manager: Arc<EnhancedAbuseReputationManager>,
    audit_log: RwLock<Vec<AutoBlockAuditEntry>>,
}

impl AutoBlockEngine {
    pub fn new(
        config: AutoBlockConfig,
        blocked_manager: Arc<BlockedManager>,
        reputation_manager: Arc<EnhancedAbuseReputationManager>,
    ) -> Self {
        Self {
            config: RwLock::new(config),
            blocked_manager,
            reputation_manager,
            audit_log: RwLock::new(Vec::new()),
        }
    }

    /// Evaluate whether a peer should be automatically blocked.
    /// Returns the evaluation result. Does NOT perform the block — call `enforce_block` to act.
    pub fn evaluate(&self, peer_id: &str) -> AutoBlockResult {
        let config = self.config.read();

        if !config.enabled {
            return AutoBlockResult {
                should_block: false,
                reason: None,
                reputation_score: ReputationScore::NEUTRAL,
                spam_confidence: 0.0,
            };
        }

        // Check exemption list
        if config.exempt_peer_ids.iter().any(|p| p == peer_id) {
            return AutoBlockResult {
                should_block: false,
                reason: None,
                reputation_score: self.reputation_manager.get_score(peer_id).value(),
                spam_confidence: 0.0,
            };
        }

        let enhanced = self.reputation_manager.get_enhanced_score(peer_id);
        let rep_value = enhanced.base_score.value();
        let spam_conf = enhanced.spam_confidence;

        let low_rep = rep_value < config.reputation_threshold;
        let high_spam = spam_conf >= config.spam_confidence_threshold;

        let (should_block, reason) = if low_rep && high_spam {
            (true, Some(AutoBlockReason::CombinedAbuse))
        } else if low_rep {
            (true, Some(AutoBlockReason::LowReputation))
        } else if high_spam {
            (true, Some(AutoBlockReason::HighSpamConfidence))
        } else {
            (false, None)
        };

        AutoBlockResult {
            should_block,
            reason,
            reputation_score: rep_value,
            spam_confidence: spam_conf,
        }
    }

    /// Evaluate and, if criteria are met, perform the automatic block.
    /// Records the action in the audit log.
    pub fn evaluate_and_block(&self, peer_id: &str) -> Result<bool, crate::IronCoreError> {
        let result = self.evaluate(peer_id);

        if !result.should_block {
            return Ok(false);
        }

        let reason = result.reason.unwrap_or(AutoBlockReason::CombinedAbuse);

        // Perform the block (evidence-preserving: not block_and_delete)
        let blocked = crate::store::blocked::BlockedIdentity::new(peer_id.to_string());
        self.blocked_manager.block(blocked)?;

        // Record in audit log
        let entry = AutoBlockAuditEntry {
            peer_id: peer_id.to_string(),
            reason,
            reputation_score: result.reputation_score,
            spam_confidence: result.spam_confidence,
            timestamp_secs: current_epoch_secs(),
            was_automatic: true,
        };

        let mut audit = self.audit_log.write();
        audit.push(entry);
        if audit.len() > self.config.read().max_audit_entries {
            let remove_count = audit.len() - self.config.read().max_audit_entries;
            audit.drain(0..remove_count);
        }

        Ok(true)
    }

    /// Add a peer to the exemption whitelist
    pub fn exempt_peer(&self, peer_id: String) {
        self.config.write().exempt_peer_ids.push(peer_id);
    }

    /// Remove a peer from the exemption whitelist
    pub fn unexempt_peer(&self, peer_id: &str) {
        self.config.write().exempt_peer_ids.retain(|p| p != peer_id);
    }

    /// Check if a peer is exempt from auto-blocking
    pub fn is_exempt(&self, peer_id: &str) -> bool {
        self.config.read().exempt_peer_ids.iter().any(|p| p == peer_id)
    }

    /// Get the audit log entries
    pub fn audit_log(&self) -> Vec<AutoBlockAuditEntry> {
        self.audit_log.read().clone()
    }

    /// Update the auto-block configuration
    pub fn update_config(&self, config: AutoBlockConfig) {
        *self.config.write() = config;
    }

    /// Get current configuration
    pub fn config(&self) -> AutoBlockConfig {
        self.config.read().clone()
    }

    /// Batch-evaluate all tracked peers and auto-block those meeting criteria.
    /// Returns the number of peers newly blocked.
    pub fn evaluate_all_tracked(&self) -> Result<usize, crate::IronCoreError> {
        let reputations = self.reputation_manager.all_enhanced_scores();
        let mut blocked_count = 0;

        for (peer_id, _score) in &reputations {
            if self.evaluate_and_block(peer_id)? {
                blocked_count += 1;
            }
        }

        Ok(blocked_count)
    }
}

fn current_epoch_secs() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;
    use crate::store::contacts::ContactManager;
    use crate::abuse::spam_detection::{SpamDetectionConfig, SpamDetectionEngine};

    fn make_engine() -> AutoBlockEngine {
        let backend = Arc::new(MemoryStorage::new());
        let contacts = Arc::new(ContactManager::new(backend.clone()));
        let blocked = Arc::new(BlockedManager::new(backend));
        let spam_detector = SpamDetectionEngine::new(
            SpamDetectionConfig::default(),
            contacts.clone(),
            blocked.clone(),
        );
        let reputation_mgr = Arc::new(EnhancedAbuseReputationManager::new(1000, spam_detector));
        AutoBlockEngine::new(
            AutoBlockConfig::default(),
            blocked,
            reputation_mgr,
        )
    }

    #[test]
    fn test_default_config() {
        let config = AutoBlockConfig::default();
        assert!(config.enabled);
        assert_eq!(config.reputation_threshold, 10.0);
        assert!((config.spam_confidence_threshold - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_exempt_peer_not_blocked() {
        let engine = make_engine();
        engine.exempt_peer("good_peer".to_string());
        assert!(engine.is_exempt("good_peer"));
        let result = engine.evaluate("good_peer");
        assert!(!result.should_block);
    }

    #[test]
    fn test_unexempt_peer() {
        let engine = make_engine();
        engine.exempt_peer("peer1".to_string());
        engine.unexempt_peer("peer1");
        assert!(!engine.is_exempt("peer1"));
    }

    #[test]
    fn test_audit_log_records_block() {
        let engine = make_engine();
        // Force a block by degrading reputation far below threshold
        for _ in 0..20 {
            engine.reputation_manager.record_signal(
                "bad_peer",
                crate::transport::reputation::AbuseSignal::RateLimited,
            );
        }
        let blocked = engine.evaluate_and_block("bad_peer").unwrap();
        assert!(blocked);
        let audit = engine.audit_log();
        assert_eq!(audit.len(), 1);
        assert_eq!(audit[0].peer_id, "bad_peer");
        assert!(audit[0].was_automatic);
    }

    #[test]
    fn test_disabled_auto_block() {
        let engine = make_engine();
        let mut config = AutoBlockConfig::default();
        config.enabled = false;
        engine.update_config(config);
        let result = engine.evaluate("any_peer");
        assert!(!result.should_block);
    }

    #[test]
    fn test_neutral_peer_not_blocked() {
        let engine = make_engine();
        let result = engine.evaluate("new_peer");
        // New peer should have neutral reputation, not triggering block
        assert!(!result.should_block);
    }
}