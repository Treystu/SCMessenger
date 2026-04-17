//! P0_SECURITY_003: Anti-abuse controls implementation.
//!
//! Comprehensive anti-abuse system including:
//! - Community-based spam detection with heuristic analysis
//! - Enhanced reputation scoring with spam integration
//! - Automatic blocking based on reputation thresholds
//! - Evidence-preserving blocking (messages still route/relay)
//! - Audit logging for all blocking actions
//! - Integration with relay and routing systems

pub mod auto_block;
pub mod reputation;
pub mod spam_detection;

pub use auto_block::{AutoBlockConfig, AutoBlockEngine, AutoBlockAuditEntry, AutoBlockReason, AutoBlockResult};
pub use reputation::{EnhancedAbuseReputationManager, EnhancedReputationScore};
pub use spam_detection::{SpamDetectionEngine, SpamDetectionConfig, SpamDetectionResult, SpamSignal};