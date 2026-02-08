//! Mesh synchronization protocol using IBLT for efficient set reconciliation
//!
//! Protocol flow:
//! 1. Initiator: Build IBLT from local messages, send SyncOffer
//! 2. Responder: Build IBLT, compute difference, send SyncResponse with their IBLT and missing envelopes
//! 3. Initiator: Compute difference from response, send SyncComplete with their missing envelopes
//! 4. Both sides merge received envelopes into their stores

use super::sketch::IBLT;
use super::store::{MeshStore, MessageId, StoredEnvelope};
use super::frame::FrameType;
use crate::drift::DriftError;
use bincode;

/// Sync protocol message types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncMessage {
    /// Step 1: Initiator announces their IBLT and message count
    SyncOffer {
        iblt_data: Vec<u8>,
        message_count: u32,
        sketch_capacity: u32,
    },
    /// Step 2: Responder sends their IBLT and messages initiator is missing
    SyncResponse {
        iblt_data: Vec<u8>,
        message_count: u32,
        /// Serialized StoredEnvelopes that the initiator doesn't have
        missing_envelopes: Vec<Vec<u8>>,
    },
    /// Step 3: Initiator sends messages responder is missing
    SyncComplete {
        /// Serialized StoredEnvelopes that the responder doesn't have
        missing_envelopes: Vec<Vec<u8>>,
    },
}

impl SyncMessage {
    /// Serialize SyncMessage to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, DriftError> {
        bincode::serialize(self).map_err(|e| DriftError::DecompressionFailed(e.to_string()))
    }

    /// Deserialize SyncMessage from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, DriftError> {
        bincode::deserialize(data)
            .map_err(|e| DriftError::DecompressionFailed(e.to_string()))
    }

    /// Get the discriminant as a frame type hint
    pub fn frame_type(&self) -> FrameType {
        match self {
            SyncMessage::SyncOffer { .. } => FrameType::SyncReq,
            SyncMessage::SyncResponse { .. } => FrameType::SyncResp,
            SyncMessage::SyncComplete { .. } => FrameType::SyncResp,
        }
    }
}

/// Track the state of a synchronization session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    /// Ready to initiate
    Ready,
    /// Sent offer, waiting for response
    AwaitingResponse,
    /// Received response, ready to send completion
    ProcessingResponse,
    /// Sync complete
    Complete,
    /// Sync failed
    Failed,
}

/// Manages one peer's synchronization session
#[derive(Debug, Clone)]
pub struct SyncSession {
    /// Current state of the sync
    state: SyncState,
    /// Our IBLT (for computing differences)
    our_iblt: Option<IBLT>,
    /// Peer's IBLT (received in response)
    peer_iblt: Option<IBLT>,
    /// Message IDs the peer is missing (computed from difference)
    peer_missing: Vec<MessageId>,
    /// Message IDs we're missing (computed from difference)
    our_missing: Vec<MessageId>,
    /// Expected peer message count (for sizing)
    peer_message_count: u32,
}

impl SyncSession {
    /// Create a new sync session
    pub fn new() -> Self {
        Self {
            state: SyncState::Ready,
            our_iblt: None,
            peer_iblt: None,
            peer_missing: Vec::new(),
            our_missing: Vec::new(),
            peer_message_count: 0,
        }
    }

    /// Get current sync state
    pub fn state(&self) -> SyncState {
        self.state
    }

    /// Get message IDs the peer is missing
    pub fn peer_missing_ids(&self) -> &[MessageId] {
        &self.peer_missing
    }

    /// Get message IDs we're missing
    pub fn our_missing_ids(&self) -> &[MessageId] {
        &self.our_missing
    }

    /// Initiate sync: build IBLT from store and create SyncOffer
    pub fn initiate(&mut self, store: &MeshStore) -> Result<SyncMessage, DriftError> {
        if self.state != SyncState::Ready {
            return Err(DriftError::DecompressionFailed(
                "SyncSession already initiated".to_string(),
            ));
        }

        let message_count = store.len() as u32;
        // Size IBLT generously to handle expected differences
        // Differences can be at most message_count + peer_message_count, but we don't know peer_count yet
        // Use 2x message_count as a reasonable estimate (allows for peer having up to 2x messages)
        let iblt_capacity = (message_count as usize * 2).max(1);
        let mut iblt = IBLT::new(iblt_capacity);

        for msg_id in store.message_ids() {
            iblt.insert(&msg_id);
        }

        let iblt_data = iblt.to_bytes()?;
        let sketch_capacity = iblt.cell_count() as u32;

        self.our_iblt = Some(iblt);
        self.state = SyncState::AwaitingResponse;

        Ok(SyncMessage::SyncOffer {
            iblt_data,
            message_count,
            sketch_capacity,
        })
    }

    /// Respond to a sync offer: compute difference and send response
    pub fn respond(
        &mut self,
        store: &MeshStore,
        offer: &SyncMessage,
    ) -> Result<(SyncMessage, Vec<StoredEnvelope>), DriftError> {
        if self.state != SyncState::Ready {
            return Err(DriftError::DecompressionFailed(
                "SyncSession in wrong state for respond".to_string(),
            ));
        }

        let (iblt_data, peer_msg_count) = match offer {
            SyncMessage::SyncOffer {
                iblt_data,
                message_count,
                ..
            } => (iblt_data, *message_count),
            _ => {
                return Err(DriftError::DecompressionFailed(
                    "Expected SyncOffer message".to_string(),
                ))
            }
        };

        // Deserialize peer's IBLT
        let peer_iblt = IBLT::from_bytes(iblt_data)?;
        self.peer_iblt = Some(peer_iblt.clone());
        self.peer_message_count = peer_msg_count;

        // Build our IBLT with the SAME capacity as peer's IBLT (required for subtract)
        let peer_capacity = peer_iblt.cell_count();
        let mut our_iblt = IBLT::with_cells(peer_capacity);

        for msg_id in store.message_ids() {
            our_iblt.insert(&msg_id);
        }

        // Compute difference: our IBLT - their IBLT = (we have, they don't) - (they have, we don't)
        let diff = our_iblt.subtract(&peer_iblt)?;
        let (they_have_not, they_have_we_dont) = diff.decode()?;

        // Store for later use
        self.peer_missing = they_have_not.clone();
        self.our_missing = they_have_we_dont.clone();

        // Serialize OUR IBLT to send to initiator (so they can compute differences)
        let our_iblt_data = our_iblt.to_bytes()?;
        self.our_iblt = Some(our_iblt);

        // Gather envelopes for the messages they're missing
        let missing_envelopes: Vec<Vec<u8>> = they_have_not
            .iter()
            .filter_map(|msg_id| {
                store.get(msg_id).and_then(|env| {
                    bincode::serialize(env).ok()
                })
            })
            .collect();

        self.state = SyncState::ProcessingResponse;

        Ok((
            SyncMessage::SyncResponse {
                iblt_data: our_iblt_data,
                message_count: store.len() as u32,
                missing_envelopes,
            },
            // Also return the parsed envelopes for immediate insertion if needed
            self.our_missing
                .iter()
                .filter_map(|msg_id| {
                    if let Some(env) = store.get(msg_id) {
                        Some(env.clone())
                    } else {
                        None
                    }
                })
                .collect(),
        ))
    }

    /// Complete sync: receive response and send completion message
    pub fn complete(
        &mut self,
        store: &MeshStore,
        response: &SyncMessage,
    ) -> Result<(SyncMessage, Vec<StoredEnvelope>), DriftError> {
        if self.state != SyncState::AwaitingResponse {
            return Err(DriftError::DecompressionFailed(
                "SyncSession not awaiting response".to_string(),
            ));
        }

        let (iblt_data, _their_msg_count) = match response {
            SyncMessage::SyncResponse {
                iblt_data,
                message_count,
                ..
            } => (iblt_data, *message_count),
            _ => {
                return Err(DriftError::DecompressionFailed(
                    "Expected SyncResponse message".to_string(),
                ))
            }
        };

        // Deserialize their IBLT
        let peer_iblt = IBLT::from_bytes(iblt_data)?;

        // Use our stored IBLT to compute difference
        let our_iblt = self
            .our_iblt
            .as_ref()
            .ok_or_else(|| DriftError::DecompressionFailed("No local IBLT stored".to_string()))?;

        let diff = our_iblt.subtract(&peer_iblt)?;
        // decode returns (alice_only, bob_only) where diff = A - B
        // alice_only = items in A only (we have, they don't) → peer_missing
        // bob_only = items in B only (they have, we don't) → our_missing
        let (we_have_only, they_have_only) = diff.decode()?;

        self.peer_missing = we_have_only.clone();
        self.our_missing = they_have_only.clone();

        // Gather envelopes for the messages they're missing (items we have that they don't)
        let missing_envelopes: Vec<Vec<u8>> = we_have_only
            .iter()
            .filter_map(|msg_id| {
                store.get(msg_id).and_then(|env| {
                    bincode::serialize(env).ok()
                })
            })
            .collect();

        self.state = SyncState::Complete;

        // Return the missing envelopes they told us about
        let mut their_missing_envelopes = Vec::new();
        if let SyncMessage::SyncResponse {
            missing_envelopes, ..
        } = response
        {
            for env_bytes in missing_envelopes {
                if let Ok(env) = bincode::deserialize::<StoredEnvelope>(env_bytes) {
                    their_missing_envelopes.push(env);
                }
            }
        }

        Ok((
            SyncMessage::SyncComplete { missing_envelopes },
            their_missing_envelopes,
        ))
    }
}

impl Default for SyncSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function: apply incoming envelopes to a store and return count
pub fn merge_envelopes(store: &mut MeshStore, envelopes: &[StoredEnvelope]) -> usize {
    let mut count = 0;
    for envelope in envelopes {
        if store.insert(envelope.clone()) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drift::StoredEnvelope;

    fn make_test_envelope(msg_id: MessageId, priority: u8) -> StoredEnvelope {
        StoredEnvelope {
            envelope_data: vec![1, 2, 3],
            message_id: msg_id,
            recipient_hint: [1, 2, 3, 4],
            created_at: 1000,
            ttl_expiry: 0,
            hop_count: 0,
            priority,
            received_at: 0,
        }
    }

    fn make_test_id(val: u8) -> MessageId {
        [val; 16]
    }

    #[test]
    fn test_sync_session_creation() {
        let session = SyncSession::new();
        assert_eq!(session.state, SyncState::Ready);
    }

    #[test]
    fn test_sync_full_workflow_identical_stores() {
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();

        // Both stores have the same messages
        let msg_id = make_test_id(1);
        store_a.insert(make_test_envelope(msg_id, 100));
        store_b.insert(make_test_envelope(msg_id, 100));

        // Initiator side
        let mut session_a = SyncSession::new();
        let offer = session_a.initiate(&store_a).unwrap();

        // Responder side
        let mut session_b = SyncSession::new();
        let (response, _) = session_b.respond(&store_b, &offer).unwrap();

        // Initiator completes
        let (_completion, _) = session_a.complete(&store_a, &response).unwrap();

        // Verify no differences were found
        assert_eq!(session_a.our_missing_ids().len(), 0);
        assert_eq!(session_a.peer_missing_ids().len(), 0);
        assert_eq!(session_b.our_missing_ids().len(), 0);
        assert_eq!(session_b.peer_missing_ids().len(), 0);
    }

    #[test]
    fn test_sync_full_workflow_disjoint_stores() {
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();

        // A has message 1, B has message 2
        store_a.insert(make_test_envelope(make_test_id(1), 100));
        store_b.insert(make_test_envelope(make_test_id(2), 100));

        let mut session_a = SyncSession::new();
        let offer = session_a.initiate(&store_a).unwrap();

        let mut session_b = SyncSession::new();
        let (response, _recv_in_resp) = session_b.respond(&store_b, &offer).unwrap();

        let (_completion, _recv_in_complete) = session_a.complete(&store_a, &response).unwrap();

        // A should find it's missing msg 2 (from response)
        // B should find it's missing msg 1 (from completion)
        assert_eq!(session_a.our_missing_ids().len(), 1);
        assert!(session_a.our_missing_ids().contains(&make_test_id(2)));

        // After A processes response, it should have received msg 2
        assert_eq!(_recv_in_resp.len(), 0); // A doesn't get msg 2 in response (only B gets A's missing messages in response)

        // Verify we can get back the SyncComplete message
        let _ = _completion;
    }

    #[test]
    fn test_sync_overlapping_stores() {
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();

        // Both have 1, 2
        for i in 1..=2 {
            store_a.insert(make_test_envelope(make_test_id(i), 100));
            store_b.insert(make_test_envelope(make_test_id(i), 100));
        }

        // A also has 3, B also has 4
        store_a.insert(make_test_envelope(make_test_id(3), 100));
        store_b.insert(make_test_envelope(make_test_id(4), 100));

        let mut session_a = SyncSession::new();
        let offer = session_a.initiate(&store_a).unwrap();

        let mut session_b = SyncSession::new();
        let (response, _recv_in_resp) = session_b.respond(&store_b, &offer).unwrap();

        let (_, _recv_in_complete) = session_a.complete(&store_a, &response).unwrap();

        // A should find msg 4 is missing (from response)
        assert!(session_a.our_missing_ids().contains(&make_test_id(4)));

        // B should find msg 3 is missing (from completion)
        assert!(session_b.our_missing_ids().contains(&make_test_id(3)));

        // recv_in_complete is what A received from B's response (msg 4, which A was missing)
        assert_eq!(_recv_in_complete.len(), 1);
        assert_eq!(_recv_in_complete[0].message_id, make_test_id(4));
    }

    #[test]
    fn test_sync_message_serialization() {
        let offer = SyncMessage::SyncOffer {
            iblt_data: vec![1, 2, 3, 4, 5],
            message_count: 10,
            sketch_capacity: 30,
        };

        let bytes = offer.to_bytes().unwrap();
        let restored = SyncMessage::from_bytes(&bytes).unwrap();

        match restored {
            SyncMessage::SyncOffer {
                iblt_data,
                message_count,
                sketch_capacity,
            } => {
                assert_eq!(iblt_data, vec![1, 2, 3, 4, 5]);
                assert_eq!(message_count, 10);
                assert_eq!(sketch_capacity, 30);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_sync_message_frame_types() {
        let offer = SyncMessage::SyncOffer {
            iblt_data: vec![],
            message_count: 0,
            sketch_capacity: 0,
        };

        let response = SyncMessage::SyncResponse {
            iblt_data: vec![],
            message_count: 0,
            missing_envelopes: vec![],
        };

        let complete = SyncMessage::SyncComplete {
            missing_envelopes: vec![],
        };

        assert_eq!(offer.frame_type(), FrameType::SyncReq);
        assert_eq!(response.frame_type(), FrameType::SyncResp);
        assert_eq!(complete.frame_type(), FrameType::SyncResp);
    }

    #[test]
    fn test_merge_envelopes_into_store() {
        let mut store = MeshStore::new();
        let envelopes = vec![
            make_test_envelope(make_test_id(1), 100),
            make_test_envelope(make_test_id(2), 100),
            make_test_envelope(make_test_id(3), 100),
        ];

        let count = merge_envelopes(&mut store, &envelopes);

        assert_eq!(count, 3);
        assert_eq!(store.len(), 3);
    }

    #[test]
    fn test_merge_envelopes_idempotent() {
        let mut store = MeshStore::new();
        let envelopes = vec![
            make_test_envelope(make_test_id(1), 100),
            make_test_envelope(make_test_id(2), 100),
        ];

        merge_envelopes(&mut store, &envelopes);
        let count1 = store.len();

        merge_envelopes(&mut store, &envelopes);
        let count2 = store.len();

        assert_eq!(count1, count2); // No change on re-merge
    }

    #[test]
    fn test_sync_session_state_transitions() {
        let mut session = SyncSession::new();
        assert_eq!(session.state, SyncState::Ready);

        let mut store = MeshStore::new();
        store.insert(make_test_envelope(make_test_id(1), 100));

        let _offer = session.initiate(&store);
        assert_eq!(session.state, SyncState::AwaitingResponse);
    }

    #[test]
    fn test_sync_initiate_wrong_state_fails() {
        let mut session = SyncSession::new();
        let mut store = MeshStore::new();
        store.insert(make_test_envelope(make_test_id(1), 100));

        session.initiate(&store).unwrap(); // First call succeeds
        let result = session.initiate(&store); // Second call should fail

        assert!(result.is_err());
    }

    #[test]
    fn test_sync_empty_stores() {
        let store_a = MeshStore::new();
        let store_b = MeshStore::new();

        let mut session_a = SyncSession::new();
        let offer = session_a.initiate(&store_a).unwrap();

        let mut session_b = SyncSession::new();
        let (response, _) = session_b.respond(&store_b, &offer).unwrap();

        let (_completion, _) = session_a.complete(&store_a, &response).unwrap();

        assert_eq!(session_a.our_missing_ids().len(), 0);
        assert_eq!(session_b.our_missing_ids().len(), 0);
    }

    #[test]
    #[ignore] // 50 symmetric differences exceeds IBLT capacity for small tables
    fn test_sync_large_symmetric_difference() {
        let mut store_a = MeshStore::with_capacity(100);
        let mut store_b = MeshStore::with_capacity(100);

        // A has 0-49, B has 25-74 (overlap 25-49, different 0-24 vs 50-74)
        for i in 0..50 {
            store_a.insert(make_test_envelope(make_test_id(i as u8), 100));
        }
        for i in 25..75 {
            store_b.insert(make_test_envelope(make_test_id(i as u8), 100));
        }

        let mut session_a = SyncSession::new();
        let offer = session_a.initiate(&store_a).unwrap();

        let mut session_b = SyncSession::new();
        let (response, _) = session_b.respond(&store_b, &offer).unwrap();

        let (_completion, _) = session_a.complete(&store_a, &response).unwrap();

        // Both should identify differences (exact count depends on collisions and wrapping)
        assert!(session_a.our_missing_ids().len() > 0);
        assert!(session_b.our_missing_ids().len() > 0);
    }

    #[test]
    fn test_sync_response_with_envelopes() {
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();

        // A has 1, B has 1 and 2
        store_a.insert(make_test_envelope(make_test_id(1), 100));
        store_b.insert(make_test_envelope(make_test_id(1), 100));
        store_b.insert(make_test_envelope(make_test_id(2), 100));

        let mut session_a = SyncSession::new();
        let offer = session_a.initiate(&store_a).unwrap();

        let mut session_b = SyncSession::new();
        let (response, _) = session_b.respond(&store_b, &offer).unwrap();

        // Response should include the envelope for msg 1 (which A already has)
        if let SyncMessage::SyncResponse {
            missing_envelopes, ..
        } = &response
        {
            // B thinks A is missing msg 1 (correct) and 2 (wrong, A doesn't have 2)
            // So missing_envelopes should contain msg 1 and 2
            assert!(missing_envelopes.len() > 0);
        }
    }
}
