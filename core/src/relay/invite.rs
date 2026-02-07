//! Invite System — cryptographic invites with web-of-trust tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Invite system errors
#[derive(Debug, Error)]
pub enum InviteError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Signature verification failed")]
    VerificationFailed,
    #[error("Token expired")]
    TokenExpired,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Invalid inviter")]
    InvalidInviter,
}

/// Cryptographic invite token signed by an inviter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteToken {
    /// ID of the inviter (who sent the invite)
    pub inviter_id: String,
    /// Inviter's Ed25519 public key (for verification)
    pub inviter_public_key: Vec<u8>,
    /// ID of the invitee (who can use this invite)
    pub invitee_id: String,
    /// Unix timestamp when token was created
    pub created_at: u64,
    /// Unix timestamp when token expires
    pub expires_at: u64,
    /// Ed25519 signature over the token data
    pub signature: Vec<u8>,
    /// Optional metadata/purpose
    pub metadata: Option<String>,
}

impl InviteToken {
    /// Create a new unsigned invite token
    pub fn new(
        inviter_id: String,
        inviter_public_key: Vec<u8>,
        invitee_id: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            inviter_id,
            inviter_public_key,
            invitee_id,
            created_at: now,
            expires_at: now + 30 * 24 * 3600, // 30 days default
            signature: Vec::new(),
            metadata: None,
        }
    }

    /// Set custom expiry duration
    pub fn with_expiry(mut self, duration_secs: u64) -> Self {
        self.expires_at = self.created_at + duration_secs;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set signature
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = signature;
        self
    }

    /// Check if token is still valid
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now < self.expires_at && !self.signature.is_empty()
    }

    /// Get data to be signed (everything except signature)
    pub fn get_signable_data(&self) -> Result<Vec<u8>, InviteError> {
        let temp = Self {
            inviter_id: self.inviter_id.clone(),
            inviter_public_key: self.inviter_public_key.clone(),
            invitee_id: self.invitee_id.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            signature: Vec::new(),
            metadata: self.metadata.clone(),
        };

        bincode::serialize(&temp)
            .map_err(|e| InviteError::SerializationError(e.to_string()))
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, InviteError> {
        bincode::serialize(self).map_err(|e| InviteError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, InviteError> {
        bincode::deserialize(bytes).map_err(|e| InviteError::SerializationError(e.to_string()))
    }
}

/// Tracks the web-of-trust chain (who invited whom)
#[derive(Debug, Clone)]
pub struct InviteChain {
    /// Invite ID -> (inviter_id, invitee_id, timestamp)
    invites: HashMap<String, (String, String, u64)>,
}

impl InviteChain {
    /// Create a new invite chain tracker
    pub fn new() -> Self {
        Self {
            invites: HashMap::new(),
        }
    }

    /// Record an invite relationship
    pub fn record_invite(&mut self, invite_id: String, inviter_id: String, invitee_id: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.invites.insert(invite_id, (inviter_id, invitee_id, now));
    }

    /// Get who invited a specific person
    pub fn get_inviter(&self, invitee_id: &str) -> Option<String> {
        for (_, (inviter_id, invited_id, _)) in &self.invites {
            if invited_id == invitee_id {
                return Some(inviter_id.clone());
            }
        }
        None
    }

    /// Get all people invited by a specific inviter
    pub fn get_invitees(&self, inviter_id: &str) -> Vec<String> {
        self.invites
            .values()
            .filter_map(|(iid, invitee_id, _)| {
                if iid == inviter_id {
                    Some(invitee_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Build the trust chain from a root node
    pub fn get_trust_chain(&self, node_id: &str) -> Vec<String> {
        let mut chain = vec![node_id.to_string()];

        let mut current = node_id.to_string();
        while let Some(inviter) = self.get_inviter(&current) {
            chain.push(inviter.clone());
            current = inviter;
        }

        chain
    }

    /// Get number of degrees of separation from root
    pub fn distance_from_root(&self, node_id: &str) -> u32 {
        (self.get_trust_chain(node_id).len() as u32).saturating_sub(1)
    }

    /// Get total number of invites tracked
    pub fn invite_count(&self) -> usize {
        self.invites.len()
    }

    /// Clear all invite records
    pub fn clear(&mut self) {
        self.invites.clear();
    }

    /// Get direct invitations from a person (not recursive)
    pub fn get_direct_invitations(&self, person_id: &str) -> Vec<(String, u64)> {
        self.invites
            .values()
            .filter_map(|(inviter_id, invitee_id, timestamp)| {
                if inviter_id == person_id {
                    Some((invitee_id.clone(), *timestamp))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for InviteChain {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level invite system
pub struct InviteSystem {
    /// Our node ID
    our_id: String,
    /// Our Ed25519 public key
    our_public_key: Vec<u8>,
    /// Tracking invite chain (web of trust)
    chain: InviteChain,
}

impl InviteSystem {
    /// Create a new invite system
    pub fn new(node_id: String, public_key: Vec<u8>) -> Self {
        Self {
            our_id: node_id,
            our_public_key: public_key,
            chain: InviteChain::new(),
        }
    }

    /// Create an invite token for another peer
    pub fn create_invite_token(&self, invitee_id: String) -> InviteToken {
        InviteToken::new(
            self.our_id.clone(),
            self.our_public_key.clone(),
            invitee_id,
        )
    }

    /// Record that we invited someone
    pub fn record_invitation(&mut self, invitee_id: String) {
        let invite_id = format!("{}_{}", self.our_id, invitee_id);
        self.chain
            .record_invite(invite_id, self.our_id.clone(), invitee_id);
    }

    /// Get the trust chain for a peer
    pub fn get_trust_chain(&self, peer_id: &str) -> Vec<String> {
        self.chain.get_trust_chain(peer_id)
    }

    /// Get our direct invitees
    pub fn get_invitees(&self) -> Vec<String> {
        self.chain.get_invitees(&self.our_id)
    }

    /// Get who invited us
    pub fn get_inviter(&self) -> Option<String> {
        self.chain.get_inviter(&self.our_id)
    }

    /// Check if we're directly connected to a peer in the trust graph
    pub fn is_direct_connection(&self, peer_id: &str) -> bool {
        self.chain.get_invitees(&self.our_id).contains(&peer_id.to_string())
            || self.chain.get_inviter(&self.our_id).as_deref() == Some(peer_id)
    }

    /// Get all connected peers in our trust network
    pub fn get_connected_peers(&self) -> Vec<String> {
        let mut peers = self.chain.get_invitees(&self.our_id);

        if let Some(inviter) = self.chain.get_inviter(&self.our_id) {
            peers.push(inviter);
        }

        peers
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_token() -> InviteToken {
        InviteToken::new(
            "alice".to_string(),
            vec![1, 2, 3, 4, 5],
            "bob".to_string(),
        )
    }

    #[test]
    fn test_invite_token_creation() {
        let token = test_token();
        assert_eq!(token.inviter_id, "alice");
        assert_eq!(token.invitee_id, "bob");
        assert!(token.expires_at > token.created_at);
    }

    #[test]
    fn test_invite_token_with_expiry() {
        let token = test_token().with_expiry(3600);
        assert_eq!(token.expires_at - token.created_at, 3600);
    }

    #[test]
    fn test_invite_token_with_metadata() {
        let token = test_token().with_metadata("group-1".to_string());
        assert_eq!(token.metadata, Some("group-1".to_string()));
    }

    #[test]
    fn test_invite_token_validity() {
        let mut token = test_token();
        assert!(!token.is_valid()); // No signature yet

        token = token.with_signature(vec![1, 2, 3]);
        assert!(token.is_valid());
    }

    #[test]
    fn test_invite_token_expiry_check() {
        let token = test_token()
            .with_signature(vec![1, 2, 3])
            .with_expiry(0);

        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(!token.is_valid());
    }

    #[test]
    fn test_invite_token_serialization() {
        let token = test_token().with_signature(vec![1, 2, 3]);
        let bytes = token.to_bytes().expect("Failed to serialize");
        let restored = InviteToken::from_bytes(&bytes).expect("Failed to deserialize");

        assert_eq!(token.inviter_id, restored.inviter_id);
        assert_eq!(token.invitee_id, restored.invitee_id);
        assert_eq!(token.signature, restored.signature);
    }

    #[test]
    fn test_invite_chain_creation() {
        let chain = InviteChain::new();
        assert_eq!(chain.invite_count(), 0);
    }

    #[test]
    fn test_record_invite() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());

        assert_eq!(chain.invite_count(), 1);
        assert_eq!(chain.get_inviter("bob"), Some("alice".to_string()));
    }

    #[test]
    fn test_get_invitees() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite("inv2".to_string(), "alice".to_string(), "charlie".to_string());

        let invitees = chain.get_invitees("alice");
        assert_eq!(invitees.len(), 2);
        assert!(invitees.contains(&"bob".to_string()));
        assert!(invitees.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_get_trust_chain() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite("inv2".to_string(), "bob".to_string(), "charlie".to_string());

        let trust_chain = chain.get_trust_chain("charlie");
        assert_eq!(trust_chain[0], "charlie");
        assert_eq!(trust_chain[1], "bob");
        assert_eq!(trust_chain[2], "alice");
    }

    #[test]
    fn test_distance_from_root() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite("inv2".to_string(), "bob".to_string(), "charlie".to_string());
        chain.record_invite("inv3".to_string(), "charlie".to_string(), "diana".to_string());

        assert_eq!(chain.distance_from_root("alice"), 0);
        assert_eq!(chain.distance_from_root("bob"), 1);
        assert_eq!(chain.distance_from_root("charlie"), 2);
        assert_eq!(chain.distance_from_root("diana"), 3);
    }

    #[test]
    fn test_get_direct_invitations() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        chain.record_invite("inv2".to_string(), "alice".to_string(), "charlie".to_string());

        let direct = chain.get_direct_invitations("alice");
        assert_eq!(direct.len(), 2);
    }

    #[test]
    fn test_invite_system_creation() {
        let system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        assert_eq!(system.our_id, "alice");
    }

    #[test]
    fn test_create_invite_token() {
        let system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        let token = system.create_invite_token("bob".to_string());

        assert_eq!(token.inviter_id, "alice");
        assert_eq!(token.invitee_id, "bob");
    }

    #[test]
    fn test_record_invitation() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system.record_invitation("bob".to_string());

        let invitees = system.get_invitees();
        assert!(invitees.contains(&"bob".to_string()));
    }

    #[test]
    fn test_get_inviter() {
        let _system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);

        // Simulate being invited by alice
        let mut other_system = InviteSystem::new("bob".to_string(), vec![4, 5, 6]);
        other_system.chain.record_invite(
            "inv1".to_string(),
            "alice".to_string(),
            "bob".to_string(),
        );

        assert_eq!(other_system.get_inviter(), Some("alice".to_string()));
    }

    #[test]
    fn test_is_direct_connection() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system.record_invitation("bob".to_string());

        assert!(system.is_direct_connection("bob"));
        assert!(!system.is_direct_connection("charlie"));
    }

    #[test]
    fn test_get_connected_peers() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system.record_invitation("bob".to_string());
        system.record_invitation("charlie".to_string());

        let peers = system.get_connected_peers();
        assert_eq!(peers.len(), 2);
        assert!(peers.contains(&"bob".to_string()));
        assert!(peers.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_get_trust_chain_via_system() {
        let mut system = InviteSystem::new("alice".to_string(), vec![1, 2, 3]);
        system.chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        system.chain.record_invite("inv2".to_string(), "bob".to_string(), "charlie".to_string());

        let trust_chain = system.get_trust_chain("charlie");
        assert_eq!(trust_chain.len(), 3);
    }

    #[test]
    fn test_chain_clear() {
        let mut chain = InviteChain::new();
        chain.record_invite("inv1".to_string(), "alice".to_string(), "bob".to_string());
        assert_eq!(chain.invite_count(), 1);

        chain.clear();
        assert_eq!(chain.invite_count(), 0);
    }
}
