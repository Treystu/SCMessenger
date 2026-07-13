// core/src/crypto/session_manager.rs
use super::ratchet::{Chain, RatchetKey, RatchetSession};
use crate::store::backend::StorageBackend;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use zeroize::Zeroize;

/// Manages ratchet sessions for multiple peer conversations.
pub struct RatchetSessionManager {
    sessions: HashMap<String, RatchetSession>,
    backend: Option<Arc<dyn StorageBackend>>,
}

impl Default for RatchetSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RatchetSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            backend: None,
        }
    }

    /// Create a manager backed by persistent storage.
    pub fn with_backend(backend: Arc<dyn StorageBackend>) -> Self {
        Self {
            sessions: HashMap::new(),
            backend: Some(backend),
        }
    }

    /// Save all sessions to the persistent backend.
    pub fn save(&self) -> Result<()> {
        if let Some(backend) = &self.backend {
            let mut json = self.serialize_sessions()?;
            backend
                .put(b"ratchet_sessions_v1", json.as_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to save ratchet sessions: {}", e))?;
            // Zeroize the JSON string containing all session secrets
            use zeroize::Zeroize;
            json.zeroize();
        }
        Ok(())
    }

    /// Load sessions from the persistent backend.
    pub fn load(&mut self) -> Result<()> {
        if let Some(backend) = &self.backend {
            if let Some(bytes) = backend
                .get(b"ratchet_sessions_v1")
                .map_err(|e| anyhow::anyhow!("Failed to load ratchet sessions: {}", e))?
            {
                let json = String::from_utf8(bytes)
                    .map_err(|e| anyhow::anyhow!("Invalid ratchet session encoding: {}", e))?;
                self.deserialize_sessions(&json)?;
            }
        }
        Ok(())
    }

    /// Get or create a ratchet session for a peer (as sender).
    pub fn get_or_create_session(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        their_identity_public_x25519: &X25519PublicKey,
    ) -> Result<&mut RatchetSession> {
        if !self.sessions.contains_key(peer_id) {
            let session =
                RatchetSession::init_as_sender(our_signing_key, their_identity_public_x25519)?;
            self.sessions.insert(peer_id.to_string(), session);
        }
        // SAFETY: We just inserted the session above if it didn't exist
        Ok(self
            .sessions
            .get_mut(peer_id)
            .expect("session just inserted"))
    }

    /// Create a receiver session with the sender's identity key.
    pub fn create_receiver_session(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        sender_identity_public_x25519: &X25519PublicKey,
    ) -> Result<&mut RatchetSession> {
        let session =
            RatchetSession::init_as_receiver(our_signing_key, sender_identity_public_x25519)?;
        self.sessions.insert(peer_id.to_string(), session);
        // SAFETY: We just inserted the session above
        Ok(self
            .sessions
            .get_mut(peer_id)
            .expect("session just inserted"))
    }

    /// Get or create a hybrid ratchet session for a peer (as sender).
    pub fn get_or_create_session_hybrid(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        our_bundle: &crate::identity::PublicKeyBundle,
        their_bundle: &crate::identity::PublicKeyBundle,
    ) -> Result<&mut RatchetSession> {
        if !self.sessions.contains_key(peer_id) {
            let (suite, hash) = crate::crypto::negotiation::negotiate_suite(
                &our_bundle.supported_suites,
                &their_bundle.supported_suites,
                &our_bundle.ed25519_public,
                &their_bundle.ed25519_public,
            )?;

            let session = if suite == 0x02 {
                RatchetSession::init_as_sender_hybrid(our_signing_key, their_bundle, hash)?
            } else {
                let their_x25519 =
                    crate::crypto::encrypt::ed25519_public_to_x25519(&their_bundle.ed25519_public)?;
                let mut s = RatchetSession::init_as_sender(our_signing_key, &their_x25519)?;
                s.negotiated_suite = Some(suite);
                s.transcript_hash = Some(hash);
                s
            };

            self.sessions.insert(peer_id.to_string(), session);
        }
        Ok(self
            .sessions
            .get_mut(peer_id)
            .expect("session just inserted"))
    }

    /// Create a receiver hybrid session.
    pub fn create_receiver_session_hybrid(
        &mut self,
        peer_id: &str,
        our_signing_key: &ed25519_dalek::SigningKey,
        our_x25519_secret: &x25519_dalek::StaticSecret,
        our_mlkem_keypair: &crate::crypto::pq::MlKem768KeyPair,
        our_bundle: &crate::identity::PublicKeyBundle,
        their_bundle: &crate::identity::PublicKeyBundle,
        hct_opt: Option<&crate::crypto::pq::hybrid::HybridCiphertext>,
    ) -> Result<&mut RatchetSession> {
        let (suite, hash) = crate::crypto::negotiation::negotiate_suite(
            &their_bundle.supported_suites, // Initiator's suites
            &our_bundle.supported_suites,   // Responder's suites
            &their_bundle.ed25519_public,   // Initiator's ed25519
            &our_bundle.ed25519_public,     // Responder's ed25519
        )?;

        let sender_x25519 = x25519_dalek::PublicKey::from(their_bundle.x25519_public);
        let session = if suite == 0x02 {
            if let Some(hct) = hct_opt {
                RatchetSession::init_as_receiver_hybrid(
                    our_signing_key,
                    our_x25519_secret,
                    our_mlkem_keypair,
                    their_bundle,
                    hct,
                    hash,
                )?
            } else {
                anyhow::bail!("Suite 0x02 requires hybrid ciphertext for receiver initialization");
            }
        } else {
            let mut s = RatchetSession::init_as_receiver(our_signing_key, &sender_x25519)?;
            s.negotiated_suite = Some(suite);
            s.transcript_hash = Some(hash);
            s
        };

        self.sessions.insert(peer_id.to_string(), session);
        Ok(self
            .sessions
            .get_mut(peer_id)
            .expect("session just inserted"))
    }

    /// Get an existing session for a peer (returns None if no session exists).
    pub fn get_session(&self, peer_id: &str) -> Option<&RatchetSession> {
        self.sessions.get(peer_id)
    }

    /// Get a mutable session for a peer.
    pub fn get_session_mut(&mut self, peer_id: &str) -> Option<&mut RatchetSession> {
        self.sessions.get_mut(peer_id)
    }

    /// Remove a session (e.g., on peer disconnect or session timeout).
    pub fn remove_session(&mut self, peer_id: &str) {
        self.sessions.remove(peer_id);
    }

    /// Number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Check if a ratchet session exists for a peer.
    pub fn has_session(&self, peer_id: &str) -> bool {
        self.sessions.contains_key(peer_id)
    }

    /// Serialize all sessions to JSON for persistence across app restarts.
    pub fn serialize_sessions(&self) -> Result<String> {
        let serializable: HashMap<String, SerializableRatchetSession> = self
            .sessions
            .iter()
            .map(|(k, v)| (k.clone(), SerializableRatchetSession::from_session(v)))
            .collect();
        serde_json::to_string(&serializable)
            .map_err(|e| anyhow::anyhow!("Failed to serialize ratchet sessions: {}", e))
    }

    /// Deserialize sessions from JSON and merge into the manager.
    /// Existing sessions for the same peer_id are NOT overwritten.
    /// Per-entry conversion failures are silently skipped; this is meant
    /// for best-effort startup loads (`load()`). Use
    /// `deserialize_sessions_strict` for contexts (e.g. backup import)
    /// that must fail on any corrupted entry instead of dropping it.
    pub fn deserialize_sessions(&mut self, json: &str) -> Result<()> {
        let map: HashMap<String, SerializableRatchetSession> = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ratchet sessions: {}", e))?;

        for (peer_id, serializable) in map {
            if self.sessions.contains_key(&peer_id) {
                continue; // Don't overwrite existing in-memory sessions
            }
            if let Ok(session) = serializable.into_session() {
                self.sessions.insert(peer_id, session);
            }
        }
        Ok(())
    }

    /// Like `deserialize_sessions`, but aborts on the first per-entry
    /// conversion failure instead of skipping it. Use where a corrupted
    /// entry must fail the whole operation, e.g. validating and restoring
    /// a backup.
    pub fn deserialize_sessions_strict(&mut self, json: &str) -> Result<()> {
        let map: HashMap<String, SerializableRatchetSession> = serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize ratchet sessions: {}", e))?;

        let mut decoded = Vec::new();

        for (peer_id, serializable) in map {
            if self.sessions.contains_key(&peer_id) {
                continue; // Don't overwrite existing in-memory sessions
            }
            let session = serializable.into_session()?;
            decoded.push((peer_id, session));
        }

        for (peer_id, session) in decoded {
            self.sessions.insert(peer_id, session);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Session serialization (for persistence across app restarts)
// ---------------------------------------------------------------------------

/// Serializable representation of a `RatchetSession`.
///
/// This is the wire/disk format. Secret keys are stored as hex strings.
/// When loading, the caller must ensure the storage is encrypted at rest.
#[derive(Serialize, Deserialize)]
pub struct SerializableRatchetSession {
    /// Our DH ratchet secret key (hex-encoded X25519 static secret bytes)
    pub our_dh_secret_hex: String,
    /// Our DH ratchet public key (hex-encoded)
    pub our_dh_public_hex: String,
    /// Their DH ratchet public key (hex-encoded), if known
    pub their_dh_public_hex: Option<String>,
    /// Root key (hex-encoded)
    pub root_key_hex: String,
    /// Sending chain key (hex-encoded) and index, if initialized
    pub sending_chain: Option<ChainState>,
    /// Receiving chain key (hex-encoded) and index, if initialized
    pub receiving_chain: Option<ChainState>,
    /// DH ratchet step count
    pub dh_step_count: u32,
    /// Whether session is fully initialized
    pub initialized: bool,
    /// Whether identity secret is still held (for first DH step)
    pub has_identity_secret: bool,
    /// Identity secret (hex-encoded), only present before first DH ratchet
    pub identity_secret_hex: Option<String>,
    /// The negotiated cryptographic suite (from PQC-04)
    pub negotiated_suite: Option<u8>,
    /// The transcript hash binding the session to the negotiation (hex-encoded, from PQC-04)
    pub transcript_hash_hex: Option<String>,
    #[serde(default)]
    pub peer_confirmed: bool,
    pub bootstrap_hct_ephemeral_hex: Option<String>,
    pub bootstrap_hct_mlkem_hex: Option<String>,
}

/// Serializable chain state.
#[derive(Serialize, Deserialize)]
pub struct ChainState {
    pub chain_key_hex: String,
    pub index: u32,
}

impl SerializableRatchetSession {
    /// Create a serializable snapshot from a live session.
    pub fn from_session(session: &RatchetSession) -> Self {
        Self {
            our_dh_secret_hex: hex::encode(session.our_dh_secret_bytes()),
            our_dh_public_hex: hex::encode(session.our_public_key()),
            their_dh_public_hex: session.their_public_key().map(hex::encode),
            root_key_hex: hex::encode(session.root_key_bytes()),
            sending_chain: session
                .sending_chain_state()
                .map(|(key, index)| ChainState {
                    chain_key_hex: hex::encode(key),
                    index,
                }),
            receiving_chain: session
                .receiving_chain_state()
                .map(|(key, index)| ChainState {
                    chain_key_hex: hex::encode(key),
                    index,
                }),
            dh_step_count: session.dh_step_count(),
            initialized: session.is_initialized(),
            has_identity_secret: session.has_identity_secret(),
            identity_secret_hex: session.identity_secret_bytes().map(hex::encode),
            negotiated_suite: session.negotiated_suite,
            transcript_hash_hex: session.transcript_hash.map(hex::encode),
            peer_confirmed: session.peer_confirmed,
            bootstrap_hct_ephemeral_hex: session
                .bootstrap_hct
                .as_ref()
                .map(|h| hex::encode(h.x25519_ephemeral_public)),
            bootstrap_hct_mlkem_hex: session
                .bootstrap_hct
                .as_ref()
                .map(|h| hex::encode(&h.mlkem_ciphertext)),
        }
    }

    /// Reconstruct a live session from a serialized snapshot.
    pub fn into_session(self) -> Result<RatchetSession> {
        let mut our_dh_secret_bytes = hex::decode(&self.our_dh_secret_hex)
            .map_err(|e| anyhow::anyhow!("Invalid our_dh_secret_hex: {}", e))?;
        if our_dh_secret_bytes.len() != 32 {
            bail!("our_dh_secret must be 32 bytes");
        }
        let mut secret_arr = [0u8; 32];
        secret_arr.copy_from_slice(&our_dh_secret_bytes);
        our_dh_secret_bytes.zeroize();
        let our_dh_secret = X25519StaticSecret::from(secret_arr);
        secret_arr.zeroize();

        let our_dh_public_bytes = hex::decode(&self.our_dh_public_hex)
            .map_err(|e| anyhow::anyhow!("Invalid our_dh_public_hex: {}", e))?;
        if our_dh_public_bytes.len() != 32 {
            bail!("our_dh_public must be 32 bytes");
        }
        let mut pub_arr = [0u8; 32];
        pub_arr.copy_from_slice(&our_dh_public_bytes);
        let our_dh_public = X25519PublicKey::from(pub_arr);

        let their_dh_public = match &self.their_dh_public_hex {
            Some(hex_str) => {
                let bytes = hex::decode(hex_str)
                    .map_err(|e| anyhow::anyhow!("Invalid their_dh_public_hex: {}", e))?;
                if bytes.len() != 32 {
                    bail!("their_dh_public must be 32 bytes");
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(X25519PublicKey::from(arr))
            }
            None => None,
        };

        let mut root_key_bytes = hex::decode(&self.root_key_hex)
            .map_err(|e| anyhow::anyhow!("Invalid root_key_hex: {}", e))?;
        if root_key_bytes.len() != 32 {
            bail!("root_key must be 32 bytes");
        }
        let mut rk_arr = [0u8; 32];
        rk_arr.copy_from_slice(&root_key_bytes);
        root_key_bytes.zeroize();
        let root_key = RatchetKey::from_bytes(rk_arr);

        let sending_chain = self
            .sending_chain
            .map(|cs| -> anyhow::Result<Chain> {
                let mut ck = [0u8; 32];
                hex::decode_to_slice(&cs.chain_key_hex, &mut ck)
                    .map_err(|e| anyhow::anyhow!("Corrupt sending chain_key_hex in DB: {}", e))?;
                Ok(Chain::new_with_index(RatchetKey::from_bytes(ck), cs.index))
            })
            .transpose()?;

        let receiving_chain = self
            .receiving_chain
            .map(|cs| -> anyhow::Result<Chain> {
                let mut ck = [0u8; 32];
                hex::decode_to_slice(&cs.chain_key_hex, &mut ck)
                    .map_err(|e| anyhow::anyhow!("Corrupt receiving chain_key_hex in DB: {}", e))?;
                Ok(Chain::new_with_index(RatchetKey::from_bytes(ck), cs.index))
            })
            .transpose()?;

        let our_identity_secret = if let Some(ref hex_str) = self.identity_secret_hex {
            let mut bytes = hex::decode(hex_str)
                .map_err(|e| anyhow::anyhow!("Invalid identity_secret_hex: {}", e))?;
            if bytes.len() != 32 {
                bail!("identity_secret must be 32 bytes");
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            bytes.zeroize();
            let secret = X25519StaticSecret::from(arr);
            arr.zeroize();
            Some(secret)
        } else {
            None
        };

        let transcript_hash = if let Some(ref hex_str) = self.transcript_hash_hex {
            let bytes = hex::decode(hex_str)
                .map_err(|e| anyhow::anyhow!("Invalid transcript_hash_hex: {}", e))?;
            if bytes.len() != 32 {
                bail!("transcript_hash must be 32 bytes");
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            Some(arr)
        } else {
            None
        };

        let bootstrap_hct = if let (Some(e_hex), Some(m_hex)) = (
            self.bootstrap_hct_ephemeral_hex,
            self.bootstrap_hct_mlkem_hex,
        ) {
            let e_bytes = hex::decode(e_hex).map_err(|e| anyhow::anyhow!("bad eph: {}", e))?;
            let m_bytes = hex::decode(m_hex).map_err(|e| anyhow::anyhow!("bad mlkem: {}", e))?;
            if e_bytes.len() != 32 || m_bytes.len() != 1088 {
                anyhow::bail!("bad hct lengths");
            }
            let mut e_arr = [0u8; 32];
            e_arr.copy_from_slice(&e_bytes);
            Some(crate::crypto::pq::hybrid::HybridCiphertext {
                x25519_ephemeral_public: e_arr,
                mlkem_ciphertext: m_bytes,
            })
        } else {
            None
        };

        Ok(RatchetSession::reconstruct(
            our_dh_secret,
            our_dh_public,
            their_dh_public,
            root_key,
            sending_chain,
            receiving_chain,
            self.dh_step_count,
            self.initialized,
            our_identity_secret,
            self.negotiated_suite,
            transcript_hash,
            self.peer_confirmed,
            bootstrap_hct,
            None, // pq_our_keypair
            None, // pq_prev_keypair
            None, // pq_their_encaps_key
            None, // pq_pending_ct
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::backend::MemoryStorage;
    use ed25519_dalek::SigningKey;
    use rand::RngCore;

    fn generate_signing_key() -> SigningKey {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        SigningKey::from_bytes(&bytes)
    }

    #[test]
    fn test_manager_persistence_roundtrip() {
        let backend = Arc::new(MemoryStorage::new());
        let mut manager = RatchetSessionManager::with_backend(backend.clone());

        let our_key = generate_signing_key();
        let their_pub = X25519PublicKey::from([1u8; 32]);
        let peer_id = "peer-1";

        // Create a session
        manager
            .get_or_create_session(peer_id, &our_key, &their_pub)
            .unwrap();
        assert_eq!(manager.session_count(), 1);

        // Save
        manager.save().unwrap();

        // Create a new manager with same backend
        let mut manager2 = RatchetSessionManager::with_backend(backend);
        assert_eq!(manager2.session_count(), 0);

        // Load
        manager2.load().unwrap();
        assert_eq!(manager2.session_count(), 1);
        assert!(manager2.get_session(peer_id).is_some());
    }

    #[test]
    fn test_deserialize_sessions_strict_rejects_corrupted_entry() {
        let mut manager = RatchetSessionManager::new();
        let our_key = generate_signing_key();
        let their_pub = X25519PublicKey::from([1u8; 32]);

        manager
            .get_or_create_session("peer-good", &our_key, &their_pub)
            .unwrap();
        let mut good_serializable: HashMap<String, SerializableRatchetSession> = manager
            .sessions
            .iter()
            .map(|(k, v)| (k.clone(), SerializableRatchetSession::from_session(v)))
            .collect();

        // Corrupt the one entry's hex field so `into_session()` fails.
        let mut corrupted = good_serializable.remove("peer-good").unwrap();
        corrupted.our_dh_secret_hex = "not-hex".to_string();
        let mut map = HashMap::new();
        map.insert("peer-corrupted".to_string(), corrupted);
        let json = serde_json::to_string(&map).unwrap();

        // Lenient path: silently skips the corrupted entry.
        let mut lenient = RatchetSessionManager::new();
        lenient.deserialize_sessions(&json).unwrap();
        assert_eq!(lenient.session_count(), 0);

        // Strict path: fails instead of dropping the entry.
        let mut strict = RatchetSessionManager::new();
        assert!(strict.deserialize_sessions_strict(&json).is_err());
        assert_eq!(strict.session_count(), 0);
    }
}
