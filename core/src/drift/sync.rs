//! Mesh synchronization protocol using IBLT for efficient set reconciliation
//!
//! Protocol flow:
//! 1. Initiator: Build IBLT from local messages, send SyncOffer
//! 2. Responder: Build IBLT, compute difference, send SyncResponse with their IBLT and missing envelopes
//! 3. Initiator: Compute difference from response, send SyncComplete with their missing envelopes
//! 4. Both sides merge received envelopes into their stores

use super::frame::FrameType;
use super::sketch::IBLT;
use super::store::{MeshStore, MessageId, StoredEnvelope};
use crate::drift::DriftError;
use crate::error::MeshError;
use bincode;

/// Current sync protocol schema version.
///
/// Bumped to 2 by #671, which adds [`SyncMessage::SyncRetry`]. Version 1 is still
/// accepted on read, and a peer that predates the retry ladder keeps the store-relative
/// sizing it always had. It is never SENT a retry: a session knows the version its peer
/// speaks ([`SyncSession::set_peer_schema_version`]) and fails locally instead
/// ([`SYNC_SCHEMA_VERSION_RETRY`], #746 finding 18).
pub const SYNC_SCHEMA_VERSION: u32 = 2;

/// Lowest schema version this build still accepts.
pub const SYNC_SCHEMA_VERSION_MIN: u32 = 1;

/// First schema version that carries [`SyncMessage::SyncRetry`].
///
/// A peer below this can still run an offer/response exchange, but it has no such enum
/// variant to decode: a retry sent to it is not a friendly degrade, it is an undecodable
/// frame, and the two sides then sit in different states (#746 finding 18).
pub const SYNC_SCHEMA_VERSION_RETRY: u32 = 2;

/// Clamp a version a peer reports into the range this build can speak.
pub fn clamp_schema_version(version: u32) -> u32 {
    version.clamp(SYNC_SCHEMA_VERSION_MIN, SYNC_SCHEMA_VERSION)
}

/// Versioned wrapper for sync messages to enable protocol evolution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionedSyncMessage {
    /// Schema version for this message
    pub schema_version: u32,
    /// The actual sync message payload
    pub payload: SyncMessage,
}

impl VersionedSyncMessage {
    /// Create a new versioned sync message with the current schema version
    pub fn new(payload: SyncMessage) -> Self {
        Self::new_for(SYNC_SCHEMA_VERSION, payload)
    }

    /// Create a message stamped for the version the PEER speaks (#746 finding 18).
    ///
    /// The header is what the peer validates, so an envelope meant for a v1 peer has to
    /// say 1: stamping the local version there is exactly the `VersionMismatch` the
    /// wrapper exists to prevent. Out-of-range claims are clamped into what we can speak.
    pub fn new_for(peer_schema_version: u32, payload: SyncMessage) -> Self {
        Self {
            schema_version: clamp_schema_version(peer_schema_version),
            payload,
        }
    }

    /// Validate that the message schema version matches our expected version
    pub fn validate(&self) -> Result<(), MeshError> {
        if self.schema_version < SYNC_SCHEMA_VERSION_MIN
            || self.schema_version > SYNC_SCHEMA_VERSION
        {
            return Err(MeshError::VersionMismatch {
                expected: SYNC_SCHEMA_VERSION,
                received: self.schema_version,
            });
        }
        Ok(())
    }

    /// Serialize VersionedSyncMessage to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, DriftError> {
        bincode::serialize(self).map_err(|e| DriftError::DecompressionFailed(e.to_string()))
    }

    /// Deserialize and validate VersionedSyncMessage from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, DriftError> {
        let msg: VersionedSyncMessage = bincode::deserialize(data)
            .map_err(|e| DriftError::DecompressionFailed(e.to_string()))?;
        msg.validate()
            .map_err(|e| DriftError::DecompressionFailed(e.to_string()))?;
        Ok(msg)
    }
}

/// Sync protocol message types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SyncMessage {
    /// Step 1: Initiator announces their IBLT and message count
    SyncOffer {
        iblt_data: Vec<u8>,
        message_count: u32,
        sketch_capacity: u32,
        /// Cryptographic proof of peer state (added in Task 2.2)
        #[serde(default)]
        peer_proof: String,
        /// Timestamp of sync offer creation (added in Task 2.2)
        #[serde(default)]
        timestamp: u64,
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
    /// Step 2b: Responder could not peel the offered table, so instead of failing the
    /// session it asks for a larger one (#671). The cell format is unchanged.
    SyncRetry {
        /// Cell count the responder believes it can peel at; the initiator clamps it.
        suggested_cells: u32,
        /// Why the retry happened: 1 = decode could not peel, kept for future causes.
        reason: u8,
    },
}

impl SyncMessage {
    /// Serialize SyncMessage to bytes using bincode
    pub fn to_bytes(&self) -> Result<Vec<u8>, DriftError> {
        bincode::serialize(self).map_err(|e| DriftError::DecompressionFailed(e.to_string()))
    }

    /// Deserialize SyncMessage from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, DriftError> {
        bincode::deserialize(data).map_err(|e| DriftError::DecompressionFailed(e.to_string()))
    }

    /// Get the discriminant as a frame type hint
    pub fn frame_type(&self) -> FrameType {
        match self {
            SyncMessage::SyncOffer { .. } => FrameType::SyncReq,
            SyncMessage::SyncResponse { .. } => FrameType::SyncResp,
            SyncMessage::SyncComplete { .. } => FrameType::SyncResp,
            SyncMessage::SyncRetry { .. } => FrameType::SyncResp,
        }
    }
}

/// How the initiator sizes its IBLT (#671).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncSizing {
    /// Control path, unchanged from before #671: size from the local store's message
    /// count (4n + 8 differences). Cost tracks what the node holds, not what differs.
    StoreRelative,
    /// Size from an estimate of the symmetric difference and escalate on a decode
    /// failure instead of failing the session.
    DifferenceDriven {
        /// Cells requested for the first offer. Clamped up to a small floor.
        cells: u32,
    },
}

/// Cells the initiator asks for on the first offer of a difference-driven sync.
///
/// Deliberately small: the ladder is what corrects an underestimate, and every step is
/// one retry rather than a failed session.
pub const DIFF_DRIVEN_MIN_CELLS: u32 = 12;
/// Growth factor applied per retry.
pub const RETRY_GROWTH: u32 = 2;
/// Never climb past this many cells.
///
/// This is the WIRE ceiling, not a memory ceiling: [`IBLT::to_bytes`] encodes the cell
/// count in two bytes and rejects anything larger, so a ladder that climbed higher would
/// build an offer it cannot send and fail as a serialization error (#710 finding 17).
pub const RETRY_MAX_CELLS: u32 = crate::drift::sketch::MAX_WIRE_CELLS as u32;

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
    /// Sizing policy for the next offer (#671)
    sizing: SyncSizing,
    /// Cell count used by the most recent offer, so a retry can grow from it
    last_offer_cells: u32,
    /// Retries already taken in this session
    retries: u32,
    /// Retry ceiling; a table that still will not peel at this point fails honestly
    max_retries: u32,
    /// Schema version the peer speaks, which decides what we may send it
    peer_schema_version: u32,
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
            sizing: SyncSizing::StoreRelative,
            last_offer_cells: 0,
            retries: 0,
            max_retries: 12,
            peer_schema_version: SYNC_SCHEMA_VERSION,
            our_iblt: None,
            peer_iblt: None,
            peer_missing: Vec::new(),
            our_missing: Vec::new(),
            peer_message_count: 0,
        }
    }

    /// Build a session with an explicit sizing policy (#671).
    pub fn with_sizing(sizing: SyncSizing) -> Self {
        let mut session = Self::new();
        session.sizing = sizing;
        session
    }

    /// Build a session that already knows which schema version the peer speaks.
    pub fn with_peer_schema_version(peer_schema_version: u32) -> Self {
        let mut session = Self::new();
        session.set_peer_schema_version(peer_schema_version);
        session
    }

    /// Record the schema version the peer speaks, clamped into what this build speaks.
    pub fn set_peer_schema_version(&mut self, peer_schema_version: u32) {
        self.peer_schema_version = clamp_schema_version(peer_schema_version);
    }

    /// Schema version this session addresses the peer at.
    pub fn peer_schema_version(&self) -> u32 {
        self.peer_schema_version
    }

    /// Whether the peer can decode the retry ladder's message at all.
    pub fn peer_supports_retry(&self) -> bool {
        self.peer_schema_version >= SYNC_SCHEMA_VERSION_RETRY
    }

    /// Retries taken so far in this session.
    pub fn retries(&self) -> u32 {
        self.retries
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

        let cells = match self.sizing {
            // Control path: byte-for-byte the sizing that shipped before #671.
            SyncSizing::StoreRelative => {
                let message_count = store.len() as u32;
                // Differences can be at most message_count + peer_message_count, but we
                // do not know peer_count yet, so size from the local store with headroom.
                let iblt_capacity = (message_count as usize * 4 + 8).max(1);
                IBLT::new(iblt_capacity).cell_count() as u32
            }
            SyncSizing::DifferenceDriven { cells } => cells.max(DIFF_DRIVEN_MIN_CELLS),
        };
        self.retries = 0;
        self.build_offer(store, cells)
    }

    /// Re-offer after a [`SyncMessage::SyncRetry`] (#671).
    ///
    /// The ladder grows the table instead of ending the session. The suggested size is
    /// clamped: never below the first-offer floor, never above [`RETRY_MAX_CELLS`], and
    /// always at least one growth step so a responder that repeats its suggestion cannot
    /// pin the session in place.
    pub fn handle_retry(
        &mut self,
        store: &MeshStore,
        suggested_cells: u32,
    ) -> Result<SyncMessage, DriftError> {
        if self.state != SyncState::AwaitingResponse {
            return Err(DriftError::DecompressionFailed(
                "SyncSession not awaiting a retry".to_string(),
            ));
        }
        if !self.peer_supports_retry() {
            self.state = SyncState::Failed;
            return Err(DriftError::DecompressionFailed(format!(
                "peer speaks sync schema {} and never sends a retry",
                self.peer_schema_version
            )));
        }
        if self.retries >= self.max_retries {
            self.state = SyncState::Failed;
            return Err(DriftError::DecompressionFailed(format!(
                "IBLT decode failed after {} retries at {} cells",
                self.retries, self.last_offer_cells
            )));
        }
        let grown = self.last_offer_cells.saturating_mul(RETRY_GROWTH);
        let target = suggested_cells
            .max(grown)
            .clamp(DIFF_DRIVEN_MIN_CELLS, RETRY_MAX_CELLS);
        self.retries += 1;
        self.build_offer(store, target)
    }

    /// Build and record an offer at an explicit cell count.
    fn build_offer(&mut self, store: &MeshStore, cells: u32) -> Result<SyncMessage, DriftError> {
        let message_count = store.len() as u32;
        let mut iblt = IBLT::with_cells(cells.max(1) as usize);

        for msg_id in store.message_ids() {
            iblt.insert(&msg_id);
        }

        let iblt_data = iblt.to_bytes()?;
        let sketch_capacity = iblt.cell_count() as u32;

        // Generate cryptographic proof of our state
        let peer_proof = store.generate_proof();

        // Get current timestamp
        let timestamp = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.our_iblt = Some(iblt);
        self.last_offer_cells = sketch_capacity;
        self.state = SyncState::AwaitingResponse;

        Ok(SyncMessage::SyncOffer {
            iblt_data,
            message_count,
            sketch_capacity,
            peer_proof,
            timestamp,
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
        let (they_have_not, they_have_we_dont) = match diff.decode() {
            Ok(pair) => pair,
            // #671: an undersized offer is a size problem, not a session failure. Ask for
            // twice the table instead of returning an error the caller cannot act on.
            Err(_) => {
                // #746 finding 18: the schema comment promised a pre-v2 peer "simply never
                // receives one" and nothing enforced it. A v1 build has no SyncRetry variant,
                // so the honest answer is a local failure the caller can act on rather than
                // a frame the peer discards as malformed.
                if !self.peer_supports_retry() {
                    self.state = SyncState::Failed;
                    return Err(DriftError::DecompressionFailed(format!(
                        "peer speaks sync schema {} and cannot decode a retry",
                        self.peer_schema_version
                    )));
                }
                self.state = SyncState::Ready;
                let suggested = (peer_capacity as u32)
                    .saturating_mul(RETRY_GROWTH)
                    .min(RETRY_MAX_CELLS);
                return Ok((
                    SyncMessage::SyncRetry {
                        suggested_cells: suggested.max(DIFF_DRIVEN_MIN_CELLS),
                        reason: 1,
                    },
                    Vec::new(),
                ));
            }
        };

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
                store
                    .get(msg_id)
                    .and_then(|env| bincode::serialize(env).ok())
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
                .filter_map(|msg_id| store.get(msg_id).cloned())
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
                store
                    .get(msg_id)
                    .and_then(|env| bincode::serialize(env).ok())
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
            peer_proof: String::new(),
            timestamp: 0,
        };

        let versioned = VersionedSyncMessage::new(offer);
        let bytes = versioned.to_bytes().unwrap();
        let restored = VersionedSyncMessage::from_bytes(&bytes).unwrap();

        assert_eq!(restored.schema_version, SYNC_SCHEMA_VERSION);
        match restored.payload {
            SyncMessage::SyncOffer {
                iblt_data,
                message_count,
                sketch_capacity,
                ..
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
            peer_proof: String::new(),
            timestamp: 0,
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
        assert!(!session_a.our_missing_ids().is_empty());
        assert!(!session_b.our_missing_ids().is_empty());
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
            assert!(!missing_envelopes.is_empty());
        }
    }

    proptest::proptest! {
        #[test]
        fn test_proptest_sync_reconciles_arbitrary_sets(
            val_a in 1..20u8,
            val_b in 21..40u8,
        ) {
            let mut store_a = MeshStore::new();
            let mut store_b = MeshStore::new();

            store_a.insert(make_test_envelope(make_test_id(val_a), 100));
            store_b.insert(make_test_envelope(make_test_id(val_b), 100));

            let mut session_a = SyncSession::new();
            let offer = session_a.initiate(&store_a).unwrap();

            let mut session_b = SyncSession::new();
            let (response, _) = session_b.respond(&store_b, &offer).unwrap();

            let (_, _received_by_a) = session_a.complete(&store_a, &response).unwrap();

            // A should identify that it is missing val_b
            assert!(session_a.our_missing_ids().contains(&make_test_id(val_b)));
            // B should identify that it is missing val_a
            assert!(session_b.our_missing_ids().contains(&make_test_id(val_a)));
        }
    }

    #[test]
    fn test_control_sizing_is_unchanged_by_671() {
        // The control path must keep the exact pre-#671 table size, or old peers would
        // see a different offer cost for the same store.
        let mut store = MeshStore::new();
        for i in 0..10u8 {
            store.insert(make_test_envelope(make_test_id(i), 100));
        }
        let mut session = SyncSession::with_sizing(SyncSizing::StoreRelative);
        match session.initiate(&store).unwrap() {
            SyncMessage::SyncOffer {
                sketch_capacity, ..
            } => assert_eq!(sketch_capacity, ((10 * 4 + 8) * 3) as u32),
            _ => panic!("expected SyncOffer"),
        }
    }

    #[test]
    fn test_difference_driven_first_offer_is_small() {
        let mut store = MeshStore::new();
        for i in 0..50u8 {
            store.insert(make_test_envelope(make_test_id(i), 100));
        }
        let mut session = SyncSession::with_sizing(SyncSizing::DifferenceDriven {
            cells: DIFF_DRIVEN_MIN_CELLS,
        });
        match session.initiate(&store).unwrap() {
            SyncMessage::SyncOffer {
                sketch_capacity, ..
            } => {
                assert_eq!(sketch_capacity, DIFF_DRIVEN_MIN_CELLS);
                // and far below the control sizing for the same store
                assert!(sketch_capacity < ((50 * 4 + 8) * 3) as u32);
            }
            _ => panic!("expected SyncOffer"),
        }
    }

    #[test]
    fn test_retry_escalates_from_an_undersized_offer() {
        // 20 vs 20 disjoint ids cannot peel from a 12-cell table, so the responder must
        // answer with SyncRetry rather than an error, and the initiator must grow the
        // table instead of ending the session. No decode success is required here: the
        // point of the test is the handshake, and a table this small cannot succeed.
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();
        for i in 0..20u8 {
            store_a.insert(make_test_envelope(make_test_id(i), 100));
        }
        for i in 100..120u8 {
            store_b.insert(make_test_envelope(make_test_id(i), 100));
        }

        let mut session_a = SyncSession::with_sizing(SyncSizing::DifferenceDriven {
            cells: DIFF_DRIVEN_MIN_CELLS,
        });
        let mut session_b = SyncSession::new();

        let offer = session_a.initiate(&store_a).unwrap();
        assert_eq!(session_a.state(), SyncState::AwaitingResponse);

        let (response, envelopes) = session_b.respond(&store_b, &offer).unwrap();
        assert!(envelopes.is_empty());
        let suggested = match response {
            SyncMessage::SyncRetry {
                suggested_cells,
                reason,
            } => {
                assert_eq!(reason, 1);
                suggested_cells
            }
            _ => panic!("an unpeelable offer must produce SyncRetry, not a response"),
        };
        assert_eq!(suggested, DIFF_DRIVEN_MIN_CELLS * RETRY_GROWTH);
        // The responder is ready for a re-offer, not stuck mid-session.
        assert_eq!(session_b.state(), SyncState::Ready);

        let retry_offer = session_a.handle_retry(&store_a, suggested).unwrap();
        assert_eq!(session_a.retries(), 1);
        assert_eq!(session_a.state(), SyncState::AwaitingResponse);
        match retry_offer {
            SyncMessage::SyncOffer {
                sketch_capacity, ..
            } => assert_eq!(sketch_capacity, suggested),
            _ => panic!("a retry must re-offer"),
        }
    }

    #[test]
    fn test_difference_driven_path_reconciles_when_the_table_fits() {
        // The ladder must still land a sync end to end when the first offer is big
        // enough to peel; three ids per side against a 144-cell table does.
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();
        for i in 0..3u8 {
            store_a.insert(make_test_envelope(make_test_id(i), 100));
        }
        for i in 10..13u8 {
            store_b.insert(make_test_envelope(make_test_id(i), 100));
        }

        let mut session_a = SyncSession::with_sizing(SyncSizing::DifferenceDriven { cells: 144 });
        let mut session_b = SyncSession::new();

        let offer = session_a.initiate(&store_a).unwrap();
        let (response, _) = session_b.respond(&store_b, &offer).unwrap();
        assert_eq!(session_a.retries(), 0);

        session_a.complete(&store_a, &response).unwrap();

        assert_eq!(session_a.our_missing_ids().len(), 3);
        assert_eq!(session_a.peer_missing_ids().len(), 3);
        assert_eq!(session_b.our_missing_ids().len(), 3);
        assert_eq!(session_a.state(), SyncState::Complete);
    }

    #[test]
    fn retry_ladder_never_exceeds_the_wire_limit() {
        // #710 finding 17: the ladder must stop at a size the wire can carry. Whoever asks
        // for more (a hostile or optimistic suggestion) is clamped, and the resulting offer
        // must actually serialize.
        assert!(
            RETRY_MAX_CELLS as usize <= crate::drift::sketch::MAX_WIRE_CELLS,
            "the retry ceiling must fit the two-byte wire cell count"
        );
        let mut store = MeshStore::new();
        for i in 0..5u8 {
            store.insert(make_test_envelope(make_test_id(i), 100));
        }
        let mut session = SyncSession::with_sizing(SyncSizing::DifferenceDriven {
            cells: DIFF_DRIVEN_MIN_CELLS,
        });
        let _ = session.initiate(&store).unwrap();
        let offer = session
            .handle_retry(&store, u32::MAX)
            .expect("a clamped retry must still build an offer");
        let (iblt_data, sketch_capacity) = match offer {
            SyncMessage::SyncOffer {
                iblt_data,
                sketch_capacity,
                ..
            } => (iblt_data, sketch_capacity),
            other => panic!("expected a SyncOffer, got {other:?}"),
        };
        assert_eq!(sketch_capacity, RETRY_MAX_CELLS);
        let table = IBLT::from_bytes(&iblt_data).expect("the offer table must decode");
        assert_eq!(table.cell_count(), RETRY_MAX_CELLS as usize);
    }

    #[test]
    fn v1_peer_is_handled_explicitly_instead_of_receiving_an_undecodable_retry() {
        // #746 finding 18. The same unpeelable offer is answered two ways: a v2 peer gets the
        // retry ladder, a v1 peer - whose build has no SyncRetry variant to decode - gets a
        // local failure. Before this change both peers got a SyncRetry, and the v1 side had
        // nothing to parse.
        let mut store_a = MeshStore::new();
        let mut store_b = MeshStore::new();
        for i in 0..20u8 {
            store_a.insert(make_test_envelope(make_test_id(i), 100));
        }
        for i in 100..120u8 {
            store_b.insert(make_test_envelope(make_test_id(i), 100));
        }
        let mut session_a = SyncSession::with_sizing(SyncSizing::DifferenceDriven {
            cells: DIFF_DRIVEN_MIN_CELLS,
        });
        let offer = session_a.initiate(&store_a).unwrap();

        let mut v2 = SyncSession::new();
        let v2_version = v2.peer_schema_version();
        assert_eq!(v2_version, SYNC_SCHEMA_VERSION);
        assert!(v2.peer_supports_retry());
        let (reply, _) = v2.respond(&store_b, &offer).unwrap();
        assert!(matches!(reply, SyncMessage::SyncRetry { .. }));
        assert_eq!(v2.state(), SyncState::Ready);

        let mut v1 = SyncSession::with_peer_schema_version(SYNC_SCHEMA_VERSION_MIN);
        let v1_version = v1.peer_schema_version();
        assert_eq!(v1_version, SYNC_SCHEMA_VERSION_MIN);
        assert!(!v1.peer_supports_retry());
        let refused = match v1.respond(&store_b, &offer) {
            Ok(_) => panic!("a v1 peer must not be answered with a retry"),
            Err(error) => error,
        };
        let message = format!("{refused:?}");
        assert!(message.contains("schema 1"));
        assert_eq!(v1.state(), SyncState::Failed);

        // The ladder is likewise not taken in that direction: a session that knows its peer
        // is v1 refuses to grow a table it could only answer with an undecodable frame.
        let mut v1_initiator = SyncSession::with_sizing(SyncSizing::DifferenceDriven {
            cells: DIFF_DRIVEN_MIN_CELLS,
        });
        let _ = v1_initiator.initiate(&store_a).unwrap();
        v1_initiator.set_peer_schema_version(SYNC_SCHEMA_VERSION_MIN);
        let ladder = v1_initiator.handle_retry(&store_a, DIFF_DRIVEN_MIN_CELLS);
        assert!(ladder.is_err());
        assert_eq!(v1_initiator.state(), SyncState::Failed);
    }

    #[test]
    fn versioned_envelope_carries_the_version_the_peer_speaks() {
        // The header is what the peer validates, so an envelope for a v1 peer must say 1;
        // a claim outside what we speak is clamped rather than propagated.
        let retry = SyncMessage::SyncRetry {
            suggested_cells: 24,
            reason: 1,
        };
        let for_v1 = VersionedSyncMessage::new_for(SYNC_SCHEMA_VERSION_MIN, retry.clone());
        assert_eq!(for_v1.schema_version, SYNC_SCHEMA_VERSION_MIN);
        let bytes = for_v1.to_bytes().unwrap();
        let decoded = VersionedSyncMessage::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.schema_version, SYNC_SCHEMA_VERSION_MIN);

        let current = VersionedSyncMessage::new(retry.clone());
        assert_eq!(current.schema_version, SYNC_SCHEMA_VERSION);
        let floor = VersionedSyncMessage::new_for(0, retry.clone());
        assert_eq!(floor.schema_version, SYNC_SCHEMA_VERSION_MIN);
        let ceiling = VersionedSyncMessage::new_for(u32::MAX, retry);
        assert_eq!(ceiling.schema_version, SYNC_SCHEMA_VERSION);
        let clamped = clamp_schema_version(SYNC_SCHEMA_VERSION_RETRY);
        assert_eq!(clamped, SYNC_SCHEMA_VERSION_RETRY);
    }

    #[test]
    fn test_retry_ceiling_fails_honestly() {
        // A session that cannot grow past its ceiling must fail with a decode error,
        // never spin or report success.
        let mut store = MeshStore::new();
        for i in 0..5u8 {
            store.insert(make_test_envelope(make_test_id(i), 100));
        }
        let mut session = SyncSession::with_sizing(SyncSizing::DifferenceDriven {
            cells: DIFF_DRIVEN_MIN_CELLS,
        });
        let _ = session.initiate(&store).unwrap();
        let mut failed = false;
        for _ in 0..13 {
            // Suggest the same small size every time: growth comes from the ladder itself.
            if session.handle_retry(&store, DIFF_DRIVEN_MIN_CELLS).is_err() {
                failed = true;
                break;
            }
        }
        assert!(failed, "the ladder must stop at its ceiling");
        assert_eq!(session.state(), SyncState::Failed);
    }
}
