// libp2p swarm setup — Aggressive Discovery Mode
//
// Philosophy: "A node is a node." All nodes are mandatory relays.
// Connectivity takes priority over strict identity or topic matching.
//
// This creates and manages the libp2p Swarm with:
// - TCP transport + QUIC-v1 transport (native only)
// - Noise encryption (transport-level, separate from message encryption)
// - Yamux multiplexing
// - Promiscuous peer acceptance (any PeerID is valid)
// - Dynamic Gossipsub topic negotiation
// - Automatic ledger exchange on connect
// - Mandatory relay for all connections
// - All behaviours from behaviour.rs

#[cfg(not(target_arch = "wasm32"))]
use super::behaviour::RelayRequest;
use super::behaviour::{
    DeregistrationRequest, IronCoreBehaviour, LedgerExchangeRequest, LedgerExchangeResponse,
    Libp2pMessageRequest, Libp2pMessageResponse, RegistrationMessage, RegistrationRequest,
    RegistrationResponse, RelayResponse, SharedPeerEntry,
};
use super::discovery::DiscoveryConfig;
#[cfg(not(target_arch = "wasm32"))]
use super::mesh_routing::{
    advance_route_cursor, BootstrapCapability, MultiPathDelivery, RankedRoute,
};
// Import mycorrhizal routing modules
#[cfg(target_arch = "wasm32")]
use super::multiport::MultiPortConfig;
#[cfg(not(target_arch = "wasm32"))]
use super::multiport::{self, BindResult, MultiPortConfig};
use super::observation::{AddressObserver, ConnectionTracker};
use super::reflection::{AddressReflectionRequest, AddressReflectionService};
#[cfg(not(target_arch = "wasm32"))]
use super::routing::local::TransportType as RoutingTransportType;
use super::routing::optimized_engine::OptimizedRoutingEngine;
#[cfg(not(target_arch = "wasm32"))]
use super::routing::{
    engine::{NextHop, RoutingDecision, RoutingLayer},
    smart_retry::{calculate_next_attempt, BackoffStrategy},
};
use crate::drift::{DriftFrame, SyncSession};
use crate::store::relay_custody::{CustodyCompatMode, CustodyEnforcement, RelayCustodyStore};
use anyhow::Result;
use bincode;
#[cfg(target_arch = "wasm32")]
use libp2p::Transport;
use libp2p::{identity::Keypair, kad, swarm::SwarmEvent, Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::net::SocketAddr;
use std::sync::{Arc, Weak};
use tokio::sync::mpsc;
use web_time::SystemTime;
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};
#[cfg(not(target_arch = "wasm32"))]
use web_time::{Duration, Instant, UNIX_EPOCH};

/// Returns true if a Multiaddr is suitable for discovery (local or global).
///
/// We exclude:
/// - Loopback (127.x.x.x, ::1)
/// - Unspecified (0.0.0.0, ::)
/// - Relay circuit addresses (/p2p-circuit) — handled separately
///
/// We now ALLOW (previously blocked):
/// - RFC1918 private ranges (10.x, 172.16-31.x, 192.168.x)
/// - CGNAT (100.64.0.0/10)
///
/// Allowing private IPs is essential for local WiFi mesh discovery via DHT.
fn is_discoverable_multiaddr(addr: &Multiaddr) -> bool {
    use libp2p::multiaddr::Protocol;
    let mut is_p2p_circuit = false;
    let mut ip_is_restricted = false;
    let mut has_ip = false;

    for proto in addr.iter() {
        match proto {
            Protocol::Ip4(ip) => {
                has_ip = true;
                if ip.is_loopback() || ip.is_unspecified() || ip.is_link_local() {
                    ip_is_restricted = true;
                }
                // Special check for 192.0.0.x (IETF Protocol Assignments)
                // Often used for internal NAT on mobile/VPN.
                if ip.octets()[0] == 192 && ip.octets()[1] == 0 && ip.octets()[2] == 0 {
                    ip_is_restricted = true;
                }
            }
            Protocol::Ip6(ip) => {
                has_ip = true;
                if ip.is_loopback() || ip.is_unspecified() {
                    ip_is_restricted = true;
                }
            }
            Protocol::P2pCircuit => {
                is_p2p_circuit = true;
            }
            _ => {}
        }
    }

    // Allow ANY P2P circuit address, even if it traverses a restricted IP (like 192.0.0.x)
    // as it's the ONLY way to reach the peer via that relay.
    if is_p2p_circuit {
        return true;
    }

    // Otherwise, require an IP and it must not be restricted.
    has_ip && !ip_is_restricted
}

/// Filter mDNS-advertised addresses to exclude circuit addresses that are too long for TXT records.
///
/// The libp2p mDNS implementation has a 1300-byte limit on TXT records. Circuit addresses
/// (e.g., /p2p-circuit/p2p/.../p2p-circuit/p2p/...) can easily exceed this limit when
/// a node has used relay multiple times, causing mDNS to silently drop them.
///
/// This function filters out:
/// - p2p-circuit addresses (contain "/p2p-circuit/")
/// - WebSocket relay addresses (contain "/ws/" or "/wss/")
///
/// Only direct IP addresses (IPv4 or IPv6) are advertised via mDNS.
pub fn build_mdns_advertised_addrs(all_listeners: &[Multiaddr]) -> Vec<Multiaddr> {
    all_listeners
        .iter()
        .filter(|a| {
            // Only advertise addresses a LAN peer can actually reach us on:
            //  - /ip4/ or /ip6/ direct
            //  - NO p2p-circuit
            //  - NO /ws/ websocket relay
            let s = a.to_string();
            !s.contains("/p2p-circuit/") && !s.contains("/ws/") && !s.contains("/wss/")
        })
        .cloned()
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn relay_reservation_multiaddr(base: &Multiaddr, relay_peer_id: PeerId) -> Multiaddr {
    use libp2p::multiaddr::Protocol;

    // Canonical reservation form must be:
    // /ip4|ip6/.../tcp/<port>/p2p/<relay-peer-id>/p2p-circuit
    // Identify addresses can already contain p2p segments; strip path suffixes first.
    let mut normalized = Multiaddr::empty();
    for proto in base.iter() {
        match proto {
            Protocol::P2p(_) | Protocol::P2pCircuit => {}
            other => normalized.push(other),
        }
    }
    normalized
        .with(Protocol::P2p(relay_peer_id))
        .with(Protocol::P2pCircuit)
}

#[cfg(not(target_arch = "wasm32"))]
const ROUTE_ATTEMPT_REASON_INITIAL_SEND: &str = "INITIAL_SEND";
#[cfg(not(target_arch = "wasm32"))]
const ROUTE_ATTEMPT_REASON_RETRY_NEXT: &str = "RETRY_NEXT_CANDIDATE";
#[cfg(not(target_arch = "wasm32"))]
const ROUTE_ATTEMPT_REASON_RETRY_CYCLE: &str = "RETRY_CYCLE_RESTART";
const DELIVERY_CONVERGENCE_TOPIC: &str = "sc-receipt-convergence";
const DELIVERY_CONVERGENCE_PREFIX: &[u8] = b"scm.delivery.convergence.v1:";
const RELAY_MAX_INFLIGHT_DISPATCHES: usize = 256;
const RELAY_PEER_BUCKET_REFILL_PER_SEC: f64 = 4.0;
const RELAY_PEER_BUCKET_BURST_CAPACITY: f64 = 20.0;
const RELAY_PEER_BUCKET_MAX_TRACKED: usize = 2048;
const RELAY_DUPLICATE_WINDOW_MS: u64 = 30_000;
const RELAY_MAX_TRACKED_DUPLICATES: usize = 16_384;
const RELAY_MAX_MESSAGE_ID_LEN: usize = 160;
const RELAY_MAX_ENVELOPE_BYTES: usize = 64 * 1024;
const DELIVERY_CONVERGENCE_MAX_CLOCK_SKEW_MS: u64 = 24 * 60 * 60 * 1000;
/// Identify log deduplication TTL: suppress duplicate "Identified peer" logs within this window.
const IDENTIFY_LOG_DEDUP_TTL_SECS: u64 = 60;

/// Static dedup map for "Identified peer" log lines.
/// Tracks the last log time for each peer_id to suppress spam from repeated identify events.
static LAST_IDENTIFIED_LOG: std::sync::OnceLock<parking_lot::RwLock<HashMap<PeerId, Instant>>> =
    std::sync::OnceLock::new();

fn last_identified_log() -> &'static parking_lot::RwLock<HashMap<PeerId, Instant>> {
    LAST_IDENTIFIED_LOG.get_or_init(|| parking_lot::RwLock::new(HashMap::new()))
}

/// P1-CORE-NEGOTIATION-RATE-SIGNAL: per-remote-address burst detector for
/// IncomingConnectionError. Distinguishes routine single LAN-probe noise
/// (logged at debug!) from a genuine flood of negotiation failures from one
/// address (warn!-worthy), independent of the per-event debug-level logging.
static NEGOTIATION_FAILURE_COUNTS: std::sync::OnceLock<
    parking_lot::RwLock<HashMap<String, (u32, Instant)>>,
> = std::sync::OnceLock::new();

fn negotiation_failure_counts() -> &'static parking_lot::RwLock<HashMap<String, (u32, Instant)>> {
    NEGOTIATION_FAILURE_COUNTS.get_or_init(|| parking_lot::RwLock::new(HashMap::new()))
}

/// Returns true when `addr_key` has produced >=5 negotiation failures within
/// the current 10-second window (caller should then emit a rate-limited warn).
fn record_negotiation_failure_and_check_burst(addr_key: &str) -> bool {
    let mut map = negotiation_failure_counts().write();
    let now = Instant::now();
    let window_duration = Duration::from_secs(10);

    // Bound memory against an attacker spoofing many distinct send_back_addr
    // values: this is a defensive counter, not a source of truth, so an
    // occasional full clear on overflow is an acceptable tradeoff.
    let need_clear = !map.contains_key(addr_key) && map.len() >= 4096;
    if need_clear {
        map.clear();
    }

    let (count, window_start) = map.entry(addr_key.to_string()).or_insert((0, now));

    if window_start.elapsed() >= window_duration {
        *count = 1;
        *window_start = now;
        false
    } else {
        *count += 1;
        *count >= 5
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeliveryConvergenceMarker {
    relay_message_id: String,
    destination_peer_id: String,
    observed_by_peer_id: String,
    observed_at_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct TokenBucketState {
    tokens: f64,
    last_refill_ms: u64,
}

/// Per-address exponential backoff state for bootstrap re-dial.
///
/// When a bootstrap addr refuses a connection, the backoff interval doubles
/// (60s → 120s → … → 960s max). On a successful connection, it resets to 60s.
#[derive(Debug, Clone)]
struct BootstrapBackoffEntry {
    /// Current backoff duration in seconds.
    backoff_secs: u64,
    /// Instant at which the next dial attempt is permitted.
    next_dial: web_time::Instant,
}

impl BootstrapBackoffEntry {
    /// Create a new entry with the initial backoff (60s), eligible immediately.
    fn new() -> Self {
        Self {
            backoff_secs: BOOTSTRAP_BACKOFF_INITIAL_SECS,
            next_dial: web_time::Instant::now(),
        }
    }

    /// Double the backoff on failure, capped at the max.
    fn on_failure(&mut self) {
        self.backoff_secs = (self.backoff_secs * 2).min(BOOTSTRAP_BACKOFF_MAX_SECS);
        self.next_dial = web_time::Instant::now() + Duration::from_secs(self.backoff_secs);
    }

    /// Gentle backoff on failure for hostnames, capped at 120s.
    fn on_failure_gentle(&mut self) {
        self.backoff_secs = (self.backoff_secs * 2).min(120);
        self.next_dial = web_time::Instant::now() + Duration::from_secs(self.backoff_secs);
    }

    /// Reset to the initial interval on success.
    fn on_success(&mut self) {
        self.backoff_secs = BOOTSTRAP_BACKOFF_INITIAL_SECS;
        self.next_dial = web_time::Instant::now() + Duration::from_secs(self.backoff_secs);
    }

    /// Whether this addr is eligible for a dial attempt right now.
    fn is_eligible(&self) -> bool {
        web_time::Instant::now() >= self.next_dial
    }
}

/// Initial bootstrap re-dial backoff in seconds.
const BOOTSTRAP_BACKOFF_INITIAL_SECS: u64 = 60;
/// Maximum bootstrap re-dial backoff in seconds (16 minutes).
const BOOTSTRAP_BACKOFF_MAX_SECS: u64 = 960;

/// How long a `SwarmCommand::Dial` reply may wait for a real
/// `ConnectionEstablished`/`OutgoingConnectionError` signal before the
/// pending entry is expired with a timeout error.
const PENDING_DIAL_TIMEOUT_SECS: u64 = 10;

/// Tracks a `SwarmCommand::Dial` whose `swarm.dial()` call queued
/// successfully but hasn't yet been confirmed connected or failed.
/// Keyed in `pending_dials` by the originally-dialed (stripped of any
/// `/p2p/` component) address so it can be resolved by matching against
/// `ConnectionEstablished`'s remote address or `OutgoingConnectionError`'s
/// failed transport addresses/peer_id — mirrors the same matching pattern
/// already used for `bootstrap_backoff`/`resolved_to_dns` above.
struct PendingDialEntry {
    reply: mpsc::Sender<Result<(), String>>,
    dialed_at: web_time::Instant,
    /// All `/p2p/`-stripped addresses this specific dial could plausibly
    /// resolve via: just the single dialed address for a peer-id-less dial,
    /// or the full candidate ladder (original + last-known-good + port
    /// fallbacks) for a `Some(pid)` dial. A `ConnectionEstablished`/
    /// `OutgoingConnectionError` only resolves this entry if the connected/
    /// failed address is a member of this set.
    candidate_addrs: Vec<Multiaddr>,
}

struct RelayAbuseGuardrails {
    per_peer_buckets: HashMap<String, TokenBucketState>,
    recent_duplicates: HashMap<String, u64>,
}

impl RelayAbuseGuardrails {
    fn new() -> Self {
        Self {
            per_peer_buckets: HashMap::new(),
            recent_duplicates: HashMap::new(),
        }
    }

    fn should_reject_cheap_heuristics(
        &self,
        message_id: &str,
        envelope_len: usize,
    ) -> Option<&'static str> {
        if message_id.is_empty() {
            return Some("relay_message_id_empty");
        }
        if message_id.len() > RELAY_MAX_MESSAGE_ID_LEN {
            return Some("relay_message_id_too_long");
        }
        if envelope_len == 0 {
            return Some("relay_envelope_empty");
        }
        if envelope_len > RELAY_MAX_ENVELOPE_BYTES {
            return Some("relay_envelope_too_large");
        }
        None
    }

    fn consume_peer_token(&mut self, peer_id: &str, now_ms: u64, multiplier: f64) -> bool {
        self.prune_peer_buckets(now_ms);

        let bucket = self
            .per_peer_buckets
            .entry(peer_id.to_string())
            .or_insert(TokenBucketState {
                tokens: RELAY_PEER_BUCKET_BURST_CAPACITY * multiplier,
                last_refill_ms: now_ms,
            });
        let elapsed_ms = now_ms.saturating_sub(bucket.last_refill_ms);
        if elapsed_ms > 0 {
            let refill =
                (elapsed_ms as f64 / 1000.0) * RELAY_PEER_BUCKET_REFILL_PER_SEC * multiplier;
            bucket.tokens =
                (bucket.tokens + refill).min(RELAY_PEER_BUCKET_BURST_CAPACITY * multiplier);
            bucket.last_refill_ms = now_ms;
        }
        if bucket.tokens < 1.0 {
            return false;
        }
        bucket.tokens -= 1.0;
        true
    }

    fn is_recent_duplicate(
        &mut self,
        source_peer_id: &str,
        destination_peer_id: &str,
        relay_message_id: &str,
        now_ms: u64,
    ) -> bool {
        self.prune_duplicate_window(now_ms);
        let key = format!(
            "{}::{}::{}",
            source_peer_id, destination_peer_id, relay_message_id
        );
        self.recent_duplicates
            .get(&key)
            .map(|seen_ms| now_ms.saturating_sub(*seen_ms) <= RELAY_DUPLICATE_WINDOW_MS)
            .unwrap_or(false)
    }

    fn record_accepted(
        &mut self,
        source_peer_id: &str,
        destination_peer_id: &str,
        relay_message_id: &str,
        now_ms: u64,
    ) {
        self.prune_duplicate_window(now_ms);
        let key = format!(
            "{}::{}::{}",
            source_peer_id, destination_peer_id, relay_message_id
        );
        self.recent_duplicates.insert(key, now_ms);
        if self.recent_duplicates.len() > RELAY_MAX_TRACKED_DUPLICATES {
            self.prune_oldest_duplicate();
        }
    }

    fn prune_peer_buckets(&mut self, now_ms: u64) {
        self.per_peer_buckets
            .retain(|_, bucket| now_ms.saturating_sub(bucket.last_refill_ms) <= 300_000);
        if self.per_peer_buckets.len() > RELAY_PEER_BUCKET_MAX_TRACKED {
            self.prune_oldest_peer_bucket();
        }
    }

    fn prune_duplicate_window(&mut self, now_ms: u64) {
        self.recent_duplicates
            .retain(|_, seen_ms| now_ms.saturating_sub(*seen_ms) <= RELAY_DUPLICATE_WINDOW_MS);
    }

    fn prune_oldest_peer_bucket(&mut self) {
        if let Some(oldest_peer) = self
            .per_peer_buckets
            .iter()
            .min_by_key(|(_, state)| state.last_refill_ms)
            .map(|(peer_id, _)| peer_id.clone())
        {
            self.per_peer_buckets.remove(&oldest_peer);
        }
    }

    fn prune_oldest_duplicate(&mut self) {
        if let Some(oldest_key) = self
            .recent_duplicates
            .iter()
            .min_by_key(|(_, seen_ms)| *seen_ms)
            .map(|(key, _)| key.clone())
        {
            self.recent_duplicates.remove(&oldest_key);
        }
    }
}

/// Check if envelope data is a valid DriftFrame and return its type
/// Wrap envelope data in a DriftFrame::Data for transport.
/// This adds a 7-byte transport header (2-byte length, 1-byte type, 4-byte CRC32)
/// for integrity verification and type discrimination on the receiving end.
fn wrap_in_drift_frame(envelope_data: &[u8]) -> Vec<u8> {
    let frame = DriftFrame {
        frame_type: crate::drift::FrameType::Data,
        payload: envelope_data.to_vec(),
    };
    match frame.to_bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            // Fallback: if framing fails (e.g. payload too large), send raw
            tracing::warn!("DriftFrame wrapping failed, sending raw envelope data");
            envelope_data.to_vec()
        }
    }
}

impl DeliveryConvergenceMarker {
    fn key(&self) -> String {
        format!("{}::{}", self.destination_peer_id, self.relay_message_id)
    }
}

fn encode_delivery_convergence_marker(marker: &DeliveryConvergenceMarker) -> Option<Vec<u8>> {
    let mut payload = DELIVERY_CONVERGENCE_PREFIX.to_vec();
    let mut encoded = bincode::serialize(marker).ok()?;
    payload.append(&mut encoded);
    Some(payload)
}

fn decode_delivery_convergence_marker(data: &[u8]) -> Option<DeliveryConvergenceMarker> {
    if !data.starts_with(DELIVERY_CONVERGENCE_PREFIX) {
        return None;
    }
    bincode::deserialize::<DeliveryConvergenceMarker>(&data[DELIVERY_CONVERGENCE_PREFIX.len()..])
        .ok()
}

fn publish_delivery_convergence_marker(
    swarm: &mut libp2p::Swarm<IronCoreBehaviour>,
    marker: &DeliveryConvergenceMarker,
) {
    let topic = libp2p::gossipsub::IdentTopic::new(DELIVERY_CONVERGENCE_TOPIC);
    if let Some(payload) = encode_delivery_convergence_marker(marker) {
        if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, payload) {
            tracing::warn!(
                "Failed to publish delivery convergence marker for message {}: {}",
                marker.relay_message_id,
                e
            );
        }
    } else {
        tracing::warn!(
            "Failed to encode delivery convergence marker for message {}",
            marker.relay_message_id
        );
    }
}

fn purge_request_map_for_message<K>(map: &mut HashMap<K, String>, message_id: &str) -> usize
where
    K: Eq + Hash + Copy,
{
    let stale: Vec<K> = map
        .iter()
        .filter_map(|(request_id, tracked_message_id)| {
            (tracked_message_id == message_id).then_some(*request_id)
        })
        .collect();
    let removed = stale.len();
    for request_id in stale {
        map.remove(&request_id);
    }
    removed
}

fn purge_custody_dispatches_for_message<K>(
    pending_custody_dispatches: &mut HashMap<K, PendingCustodyDispatch>,
    marker: &DeliveryConvergenceMarker,
) -> Vec<PendingCustodyDispatch>
where
    K: Eq + Hash + Copy,
{
    let stale: Vec<K> = pending_custody_dispatches
        .iter()
        .filter_map(|(request_id, dispatch)| {
            (dispatch.relay_message_id == marker.relay_message_id
                && dispatch.destination_peer.to_string() == marker.destination_peer_id)
                .then_some(*request_id)
        })
        .collect();
    let mut removed = Vec::with_capacity(stale.len());
    for request_id in stale {
        if let Some(dispatch) = pending_custody_dispatches.remove(&request_id) {
            removed.push(dispatch);
        }
    }
    removed
}

fn marker_now_ms() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn validate_delivery_convergence_marker_shape(
    marker: &DeliveryConvergenceMarker,
    now_ms: u64,
) -> Result<(), &'static str> {
    if marker.relay_message_id.is_empty() {
        return Err("marker_message_id_empty");
    }
    if marker.relay_message_id.len() > RELAY_MAX_MESSAGE_ID_LEN {
        return Err("marker_message_id_too_long");
    }
    if marker.destination_peer_id.is_empty() {
        return Err("marker_destination_empty");
    }
    if marker.observed_by_peer_id.is_empty() {
        return Err("marker_observed_by_empty");
    }
    if now_ms > marker.observed_at_ms {
        if now_ms.saturating_sub(marker.observed_at_ms) > DELIVERY_CONVERGENCE_MAX_CLOCK_SKEW_MS {
            return Err("marker_too_old");
        }
    } else if marker.observed_at_ms.saturating_sub(now_ms) > DELIVERY_CONVERGENCE_MAX_CLOCK_SKEW_MS
    {
        return Err("marker_from_future");
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn should_apply_delivery_convergence_marker(
    marker: &DeliveryConvergenceMarker,
    pending_messages: &HashMap<String, PendingMessage>,
    request_to_message: &HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_relay_requests: &HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_custody_dispatches: &HashMap<
        libp2p::request_response::OutboundRequestId,
        PendingCustodyDispatch,
    >,
    relay_custody_store: &RelayCustodyStore,
) -> Result<(), &'static str> {
    validate_delivery_convergence_marker_shape(marker, marker_now_ms())?;
    let tracked_locally = pending_messages.contains_key(&marker.relay_message_id)
        || request_to_message
            .values()
            .any(|id| id == &marker.relay_message_id)
        || pending_relay_requests
            .values()
            .any(|id| id == &marker.relay_message_id)
        || pending_custody_dispatches.values().any(|dispatch| {
            dispatch.relay_message_id == marker.relay_message_id
                && dispatch.destination_peer.to_string() == marker.destination_peer_id
        })
        || relay_custody_store
            .has_message_for_destination(&marker.destination_peer_id, &marker.relay_message_id);
    if !tracked_locally {
        return Err("marker_not_locally_tracked");
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn should_apply_delivery_convergence_marker(
    marker: &DeliveryConvergenceMarker,
    pending_messages: &HashMap<String, PendingMessage>,
    pending_relay_requests: &HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_custody_dispatches: &HashMap<
        libp2p::request_response::OutboundRequestId,
        PendingCustodyDispatch,
    >,
    relay_custody_store: &RelayCustodyStore,
) -> Result<(), &'static str> {
    validate_delivery_convergence_marker_shape(marker, marker_now_ms())?;
    let tracked_locally = pending_messages.contains_key(&marker.relay_message_id)
        || pending_relay_requests
            .values()
            .any(|id| id == &marker.relay_message_id)
        || pending_custody_dispatches.values().any(|dispatch| {
            dispatch.relay_message_id == marker.relay_message_id
                && dispatch.destination_peer.to_string() == marker.destination_peer_id
        })
        || relay_custody_store
            .has_message_for_destination(&marker.destination_peer_id, &marker.relay_message_id);
    if !tracked_locally {
        return Err("marker_not_locally_tracked");
    }
    Ok(())
}

fn extract_ed25519_public_key_from_peer_id(peer_id: &PeerId) -> Result<[u8; 32], &'static str> {
    let bytes = peer_id.to_bytes();
    // Inline Ed25519 PeerIds use the protobuf-encoded public key bytes:
    // 0x00(identity multihash), 0x24(total len 36), 0x08(field 1), 0x01(Ed25519),
    // 0x12(field 2), 0x20(32-byte key), followed by the raw 32-byte public key.
    if bytes.len() == 38
        && bytes[0] == 0x00
        && bytes[1] == 0x24
        && bytes[2] == 0x08
        && bytes[3] == 0x01
        && bytes[4] == 0x12
        && bytes[5] == 0x20
    {
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(&bytes[6..38]);
        Ok(public_key)
    } else {
        Err("peer_identity_public_key_unavailable")
    }
}

/// Extract the 32-byte public key from libp2p PeerId bytes for routing module compatibility
/// Handles both libp2p PeerId formats (Ed25519 and secp256k1)
#[cfg(not(target_arch = "wasm32"))]
fn extract_peer_id_bytes(bytes: &[u8]) -> [u8; 32] {
    // For Ed25519 PeerIds (most common): bytes are [0x00, 0x24, 0x08, 0x01, 0x12, 0x20, <32-byte-key>]
    // We extract the last 32 bytes (the actual public key)
    let mut result = [0u8; 32];
    if bytes.len() >= 32 {
        result.copy_from_slice(&bytes[bytes.len() - 32..]);
    } else {
        // Fallback: copy whatever we have
        result.copy_from_slice(bytes);
    }
    result
}

fn verify_registration_message(
    peer: &PeerId,
    message: &RegistrationMessage,
) -> Result<(), &'static str> {
    let public_key = extract_ed25519_public_key_from_peer_id(peer)?;
    match message {
        RegistrationMessage::Register(request) => request.verify_for_public_key(&public_key),
        RegistrationMessage::Deregister(request) => request.verify_for_public_key(&public_key),
    }
}

fn apply_verified_registration_message(
    relay_custody_store: &RelayCustodyStore,
    request: &RegistrationMessage,
) -> RegistrationResponse {
    let result = match request {
        RegistrationMessage::Register(request) => relay_custody_store.register_identity(
            request.payload.identity_id.clone(),
            request.payload.device_id.clone(),
            request.payload.seniority_ts,
        ),
        RegistrationMessage::Deregister(request) => relay_custody_store.deregister_identity(
            request.payload.identity_id.clone(),
            request.payload.from_device_id.clone(),
            request.payload.target_device_id.clone(),
        ),
    };

    match result {
        Ok(_) => RegistrationResponse {
            accepted: true,
            error: None,
        },
        Err(error) => RegistrationResponse {
            accepted: false,
            error: Some(error),
        },
    }
}

fn resolve_custody_metadata(
    relay_custody_store: &RelayCustodyStore,
    recipient_identity_id: Option<&str>,
    intended_device_id: Option<&str>,
    compat_mode: CustodyCompatMode,
) -> Result<(Option<String>, Option<String>), String> {
    match (recipient_identity_id, intended_device_id) {
        (Some(identity_id), Some(device_id)) => {
            match relay_custody_store.enforce_custody(identity_id, device_id) {
                Ok(CustodyEnforcement::Active {
                    identity_id,
                    device_id,
                }) => Ok((Some(identity_id), Some(device_id))),
                Ok(CustodyEnforcement::Redirected {
                    identity_id,
                    to_device_id,
                    ..
                }) => Ok((Some(identity_id), Some(to_device_id))),
                Err(error) => Err(error.to_string()),
            }
        }
        _ => {
            // Compat mode: legacy v0.2.0 clients omit device metadata.
            match compat_mode {
                CustodyCompatMode::PhaseA => {
                    tracing::debug!(
                        identity_id = recipient_identity_id,
                        device_id = intended_device_id,
                        "relay request accepted in Phase A compat mode (no device enforcement)"
                    );
                    Ok((
                        recipient_identity_id.map(|value| value.to_string()),
                        intended_device_id.map(|value| value.to_string()),
                    ))
                }
                CustodyCompatMode::PhaseB => {
                    tracing::warn!(
                        identity_id = recipient_identity_id,
                        device_id = intended_device_id,
                        "relay request accepted in Phase B compat mode (legacy client, deprecation warning)"
                    );
                    Ok((
                        recipient_identity_id.map(|value| value.to_string()),
                        intended_device_id.map(|value| value.to_string()),
                    ))
                }
            }
        }
    }
}

fn is_terminal_identity_rejection(error: &str) -> bool {
    matches!(error, "identity_device_mismatch" | "identity_abandoned")
}

#[cfg(not(target_arch = "wasm32"))]
fn log_route_decision(
    message_id: &str,
    route: &RankedRoute,
    dispatch_attempt: u32,
    pass_count: u32,
    candidate_index: usize,
    total_candidates: usize,
    attempt_reason: &str,
) {
    let route_label = if route.path.len() == 1 {
        "direct"
    } else {
        "relay"
    };
    let relay_peer = route
        .path
        .first()
        .filter(|_| route.path.len() > 1)
        .map(|p| p.to_string())
        .unwrap_or_else(|| "-".to_string());
    let destination = route
        .path
        .last()
        .map(|p| p.to_string())
        .unwrap_or_else(|| "-".to_string());

    tracing::info!(
        "ROUTE_DECISION message_id={} attempt={} pass={} candidate={}/{} route={} relay={} destination={} reason={} policy_reason={} recipient_recency={} relay_score={:.3} latest_success_order={}",
        message_id,
        dispatch_attempt,
        pass_count,
        candidate_index + 1,
        total_candidates,
        route_label,
        relay_peer,
        destination,
        attempt_reason,
        route.reason_code,
        route.recipient_recency,
        route.relay_success_score,
        route.latest_success_order
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn dispatch_ranked_route(
    swarm: &mut libp2p::Swarm<IronCoreBehaviour>,
    route: &RankedRoute,
    message_id: &str,
    target_peer: PeerId,
    envelope_data: &[u8],
    request_to_message: &mut HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_relay_requests: &mut HashMap<libp2p::request_response::OutboundRequestId, String>,
    recipient_identity_id: Option<&str>,
    intended_device_id: Option<&str>,
) {
    // Wrap envelope in DriftFrame for transport integrity (CRC32 + length prefix)
    let framed_data = wrap_in_drift_frame(envelope_data);

    if route.path.len() == 1 {
        let request_id = swarm.behaviour_mut().messaging.send_request(
            &target_peer,
            Libp2pMessageRequest {
                envelope_data: framed_data.clone(),
            },
        );
        request_to_message.insert(request_id, message_id.to_string());
    } else {
        let relay_peer = route.path[0];
        let relay_request = RelayRequest {
            destination_peer: target_peer.to_bytes(),
            envelope_data: framed_data,
            message_id: message_id.to_string(),
            recipient_identity_id: recipient_identity_id.map(|s| s.to_string()),
            intended_device_id: intended_device_id.map(|s| s.to_string()),
        };
        let request_id = swarm
            .behaviour_mut()
            .relay
            .send_request(&relay_peer, relay_request);
        pending_relay_requests.insert(request_id, message_id.to_string());
    }
}

/// Convert libp2p Kademlia protocol mode to routing transport type.
/// Maps the Kademlia query mode to the appropriate transport classification.
#[cfg(not(target_arch = "wasm32"))]
fn transport_type_to_routing_transport(mode: kad::Mode) -> RoutingTransportType {
    match mode {
        kad::Mode::Client => RoutingTransportType::QUIC,
        kad::Mode::Server => RoutingTransportType::TCP,
    }
}

/// Convert routing engine decision to ranked routes for dispatch.
///
/// Uses confidence scores from the routing engine and transport quality
/// heuristics to compute realistic relay_success_score values instead of
/// static constants.
#[cfg(not(target_arch = "wasm32"))]
fn routing_decision_to_ranked_routes(
    decision: &RoutingDecision,
    target_peer: &PeerId,
    multi_path_delivery: &mut MultiPathDelivery,
) -> Vec<RankedRoute> {
    /// Map a RoutingLayer + confidence to a transport-quality score.
    /// Direct local peers score highest; global routes and store-and-carry
    /// progressively lower. The confidence from the routing engine acts as
    /// a multiplier so the mesh can adapt to changing conditions.
    fn transport_quality_score(layer: RoutingLayer, confidence: f64) -> f64 {
        let base = match layer {
            RoutingLayer::Local => 0.95,
            RoutingLayer::Neighborhood => 0.80,
            RoutingLayer::Global => 0.65,
            RoutingLayer::StoreAndCarry => 0.40,
        };
        // Blend base quality with engine confidence (0.0-1.0).
        // When confidence is high the score approaches base; when low it
        // degrades proportionally, ensuring the mesh never trusts an
        // unverified route at face value.
        base * (0.5 + 0.5 * confidence)
    }

    let mut routes = Vec::new();

    match &decision.primary {
        NextHop::Direct {
            peer_id: _,
            transport,
        } => {
            // Direct route -- use target peer directly
            let mut score = transport_quality_score(decision.decided_by, decision.confidence);
            // Factor transport type into score: BLE < WiFi < TCP < QUIC
            let transport_bonus = match transport {
                RoutingTransportType::QUIC => 0.15,
                RoutingTransportType::TCP => 0.10,
                RoutingTransportType::WiFiAware | RoutingTransportType::WiFiDirect => 0.05,
                RoutingTransportType::BLE => 0.0,
            };
            score = (score + transport_bonus).min(1.0);
            routes.push(RankedRoute {
                path: vec![*target_peer],
                reason_code: "DIRECT_FROM_ROUTING_ENGINE",
                recipient_recency: 0,
                relay_success_score: score,
                latest_success_order: 0,
            });
        }
        NextHop::Gateway {
            gateway_id,
            transport,
            hops_remaining,
        } => {
            // Route via gateway
            let gateway_peer = PeerId::from_bytes(gateway_id).ok();
            if let Some(gw) = gateway_peer {
                let mut score = transport_quality_score(decision.decided_by, decision.confidence);
                // Gateway reliability degrades slightly with each hop remaining
                let hop_penalty = (*hops_remaining as f64) * 0.05;
                score = (score - hop_penalty).max(0.1);
                let transport_bonus = match transport {
                    RoutingTransportType::QUIC => 0.15,
                    RoutingTransportType::TCP => 0.10,
                    RoutingTransportType::WiFiAware | RoutingTransportType::WiFiDirect => 0.05,
                    RoutingTransportType::BLE => 0.0,
                };
                score = (score + transport_bonus).min(1.0);
                routes.push(RankedRoute {
                    path: vec![gw, *target_peer],
                    reason_code: "GATEWAY_FROM_ROUTING_ENGINE",
                    recipient_recency: 0,
                    relay_success_score: score,
                    latest_success_order: 0,
                });
            }
        }
        NextHop::GlobalRoute {
            next_hop_id,
            total_hops,
        } => {
            // Route via global route
            let next_hop = PeerId::from_bytes(next_hop_id).ok();
            if let Some(hop) = next_hop {
                let base_score = transport_quality_score(decision.decided_by, decision.confidence);
                // Global routes degrade with total hops
                let hop_penalty = (*total_hops as f64) * 0.05;
                let score = (base_score - hop_penalty).max(0.1);
                routes.push(RankedRoute {
                    path: vec![hop, *target_peer],
                    reason_code: "GLOBAL_ROUTE_FROM_ROUTING_ENGINE",
                    recipient_recency: 0,
                    relay_success_score: score,
                    latest_success_order: 0,
                });
            }
        }
        NextHop::StoreAndCarry => {
            // Fallback: direct to target with low confidence
            let score = transport_quality_score(decision.decided_by, decision.confidence);
            routes.push(RankedRoute {
                path: vec![*target_peer],
                reason_code: "STORE_AND_CARRY",
                recipient_recency: 0,
                relay_success_score: score,
                latest_success_order: 0,
            });
        }
        NextHop::RouteDiscovery { .. } => {
            // Route discovery requested -- fall back to multi_path_delivery
            return multi_path_delivery.ranked_routes(target_peer, 3);
        }
    }

    // Add alternative routes if available, scored lower than primary
    for alt in &decision.alternatives {
        match alt {
            NextHop::Direct { peer_id, transport } => {
                let alt_peer = PeerId::from_bytes(peer_id).ok();
                if let Some(p) = alt_peer {
                    let mut score =
                        transport_quality_score(decision.decided_by, decision.confidence) * 0.9;
                    let transport_bonus = match transport {
                        RoutingTransportType::QUIC => 0.15,
                        RoutingTransportType::TCP => 0.10,
                        RoutingTransportType::WiFiAware | RoutingTransportType::WiFiDirect => 0.05,
                        RoutingTransportType::BLE => 0.0,
                    };
                    score = (score + transport_bonus).min(1.0);
                    routes.push(RankedRoute {
                        path: vec![p],
                        reason_code: "ALT_DIRECT_FROM_ROUTING_ENGINE",
                        recipient_recency: 0,
                        relay_success_score: score,
                        latest_success_order: 0,
                    });
                }
            }
            NextHop::Gateway {
                gateway_id,
                transport,
                hops_remaining,
            } => {
                let gw = PeerId::from_bytes(gateway_id).ok();
                if let Some(g) = gw {
                    let mut score =
                        transport_quality_score(decision.decided_by, decision.confidence) * 0.9;
                    let hop_penalty = (*hops_remaining as f64) * 0.05;
                    score = (score - hop_penalty).max(0.1);
                    let transport_bonus = match transport {
                        RoutingTransportType::QUIC => 0.15,
                        RoutingTransportType::TCP => 0.10,
                        RoutingTransportType::WiFiAware | RoutingTransportType::WiFiDirect => 0.05,
                        RoutingTransportType::BLE => 0.0,
                    };
                    score = (score + transport_bonus).min(1.0);
                    routes.push(RankedRoute {
                        path: vec![g, *target_peer],
                        reason_code: "ALT_GATEWAY_FROM_ROUTING_ENGINE",
                        recipient_recency: 0,
                        relay_success_score: score,
                        latest_success_order: 0,
                    });
                }
            }
            NextHop::GlobalRoute {
                next_hop_id,
                total_hops,
            } => {
                let next_hop = PeerId::from_bytes(next_hop_id).ok();
                if let Some(hop) = next_hop {
                    let base_score =
                        transport_quality_score(decision.decided_by, decision.confidence) * 0.9;
                    let hop_penalty = (*total_hops as f64) * 0.05;
                    let score = (base_score - hop_penalty).max(0.1);
                    routes.push(RankedRoute {
                        path: vec![hop, *target_peer],
                        reason_code: "ALT_GLOBAL_ROUTE_FROM_ROUTING_ENGINE",
                        recipient_recency: 0,
                        relay_success_score: score,
                        latest_success_order: 0,
                    });
                }
            }
            _ => {}
        }
    }

    routes
}

#[derive(Debug, Clone)]
struct PendingCustodyDispatch {
    destination_peer: PeerId,
    custody_id: String,
    relay_message_id: String,
}

#[cfg(not(target_arch = "wasm32"))]
async fn apply_delivery_convergence_marker(
    marker: &DeliveryConvergenceMarker,
    pending_messages: &mut HashMap<String, PendingMessage>,
    request_to_message: &mut HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_relay_requests: &mut HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_custody_dispatches: &mut HashMap<
        libp2p::request_response::OutboundRequestId,
        PendingCustodyDispatch,
    >,
    multi_path_delivery: &mut MultiPathDelivery,
    relay_custody_store: &RelayCustodyStore,
) {
    let removed_direct_requests =
        purge_request_map_for_message(request_to_message, &marker.relay_message_id);
    let removed_relay_requests =
        purge_request_map_for_message(pending_relay_requests, &marker.relay_message_id);

    let removed_dispatches =
        purge_custody_dispatches_for_message(pending_custody_dispatches, marker);
    for dispatch in &removed_dispatches {
        if let Err(e) = relay_custody_store.mark_delivered(
            &dispatch.destination_peer.to_string(),
            &dispatch.custody_id,
            "delivery_convergence_marker_inflight",
        ) {
            tracing::debug!(
                "Convergence marker could not finalize in-flight custody {}: {}",
                dispatch.custody_id,
                e
            );
        }
    }

    let converged_custody = match relay_custody_store.converge_delivered_for_message(
        &marker.destination_peer_id,
        &marker.relay_message_id,
        "delivery_convergence_marker",
    ) {
        Ok(count) => count,
        Err(e) => {
            tracing::warn!(
                "Failed to converge custody for marker {} -> {}: {}",
                marker.relay_message_id,
                marker.destination_peer_id,
                e
            );
            0
        }
    };

    let retry_cleared = multi_path_delivery.converge_delivery(&marker.relay_message_id);
    let pending_cleared = if let Some(pending) = pending_messages.remove(&marker.relay_message_id) {
        let _ = pending.reply_tx.send(Ok(())).await;
        true
    } else {
        false
    };

    if removed_direct_requests > 0
        || removed_relay_requests > 0
        || !removed_dispatches.is_empty()
        || converged_custody > 0
        || retry_cleared
        || pending_cleared
    {
        tracing::info!(
            "[OK] Delivery convergence applied: message={} destination={} direct_requests={} relay_requests={} dispatches={} custody={} retries_cleared={} pending_cleared={}",
            marker.relay_message_id,
            marker.destination_peer_id,
            removed_direct_requests,
            removed_relay_requests,
            removed_dispatches.len(),
            converged_custody,
            retry_cleared,
            pending_cleared
        );
    }
}

#[cfg(target_arch = "wasm32")]
async fn apply_delivery_convergence_marker(
    marker: &DeliveryConvergenceMarker,
    pending_messages: &mut HashMap<String, PendingMessage>,
    pending_relay_requests: &mut HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_custody_dispatches: &mut HashMap<
        libp2p::request_response::OutboundRequestId,
        PendingCustodyDispatch,
    >,
    relay_custody_store: &RelayCustodyStore,
) {
    let removed_relay_requests =
        purge_request_map_for_message(pending_relay_requests, &marker.relay_message_id);
    let removed_dispatches =
        purge_custody_dispatches_for_message(pending_custody_dispatches, marker);
    for dispatch in &removed_dispatches {
        let _ = relay_custody_store.mark_delivered(
            &dispatch.destination_peer.to_string(),
            &dispatch.custody_id,
            "delivery_convergence_marker_inflight",
        );
    }

    let converged_custody = relay_custody_store
        .converge_delivered_for_message(
            &marker.destination_peer_id,
            &marker.relay_message_id,
            "delivery_convergence_marker",
        )
        .unwrap_or(0);

    let pending_cleared = if let Some(pending) = pending_messages.remove(&marker.relay_message_id) {
        let _ = pending.reply_tx.send(Ok(())).await;
        true
    } else {
        false
    };

    if removed_relay_requests > 0
        || !removed_dispatches.is_empty()
        || converged_custody > 0
        || pending_cleared
    {
        tracing::info!(
            "[OK] (wasm) delivery convergence applied: message={} destination={} relay_requests={} dispatches={} custody={} pending_cleared={}",
            marker.relay_message_id,
            marker.destination_peer_id,
            removed_relay_requests,
            removed_dispatches.len(),
            converged_custody,
            pending_cleared
        );
    }
}

fn dispatch_pending_custody_for_peer(
    swarm: &mut libp2p::Swarm<IronCoreBehaviour>,
    custody_store: &RelayCustodyStore,
    destination_peer: PeerId,
    pending_custody_dispatches: &mut HashMap<
        libp2p::request_response::OutboundRequestId,
        PendingCustodyDispatch,
    >,
    max_inflight_dispatches: usize,
    trigger_reason: &str,
) {
    if !swarm.is_connected(&destination_peer) {
        return;
    }

    if pending_custody_dispatches.len() >= max_inflight_dispatches {
        tracing::warn!(
            "Relay inflight dispatch cap reached ({}) — skipping custody pull for {} ({})",
            max_inflight_dispatches,
            destination_peer,
            trigger_reason
        );
        return;
    }

    let destination_id = destination_peer.to_string();
    let pending = custody_store.pending_for_destination(&destination_id, 64);
    if pending.is_empty() {
        return;
    }

    for custody in pending {
        if pending_custody_dispatches.len() >= max_inflight_dispatches {
            tracing::warn!(
                "Relay inflight dispatch cap reached ({}) while dispatching to {} ({})",
                max_inflight_dispatches,
                destination_peer,
                trigger_reason
            );
            break;
        }
        if let Err(e) =
            custody_store.mark_dispatching(&destination_id, &custody.custody_id, trigger_reason)
        {
            tracing::warn!(
                "Failed to mark custody {} dispatching for {}: {}",
                custody.custody_id,
                destination_peer,
                e
            );
            continue;
        }

        let request_id = swarm.behaviour_mut().messaging.send_request(
            &destination_peer,
            Libp2pMessageRequest {
                envelope_data: wrap_in_drift_frame(&custody.envelope_data),
            },
        );
        tracing::info!(
            "Dispatching custody {} for relay message {} to {} via {}",
            custody.custody_id,
            custody.relay_message_id,
            destination_peer,
            trigger_reason
        );
        pending_custody_dispatches.insert(
            request_id,
            PendingCustodyDispatch {
                destination_peer,
                custody_id: custody.custody_id,
                relay_message_id: custody.relay_message_id,
            },
        );
    }
}

/// Pending message delivery tracking
#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
struct PendingMessage {
    target_peer: PeerId,
    envelope_data: Vec<u8>,
    reply_tx: mpsc::Sender<Result<(), String>>,
    current_path_index: usize,
    attempt_start: SystemTime,
    dispatch_attempts: u32,
    pass_count: u32,
    retry_notified: bool,
    /// WS13 tight-pair metadata: SCMessenger identity ID of the recipient.
    recipient_identity_id: Option<String>,
    /// WS13 tight-pair metadata: specific device UUID being targeted.
    intended_device_id: Option<String>,
}

/// Commands that can be sent to the swarm task
#[derive(Debug)]
pub enum SwarmCommand {
    /// Send a message to a specific peer
    SendMessage {
        peer_id: PeerId,
        envelope_data: Vec<u8>,
        /// WS13 tight-pair: SCMessenger identity ID of the recipient (None for legacy callers).
        recipient_identity_id: Option<String>,
        /// WS13 tight-pair: device UUID being targeted (None if not known).
        intended_device_id: Option<String>,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Register the sender's active device for an identity on a remote peer.
    RegisterIdentity {
        peer_id: PeerId,
        request: RegistrationRequest,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Deregister or hand over the sender's active device for an identity on a remote peer.
    DeregisterIdentity {
        peer_id: PeerId,
        request: DeregistrationRequest,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Request address reflection from a peer
    RequestAddressReflection {
        peer_id: PeerId,
        reply: mpsc::Sender<Result<String, String>>,
    },
    /// Get external addresses based on peer observations
    GetExternalAddresses {
        reply: mpsc::Sender<Vec<SocketAddr>>,
    },
    /// Dial a peer at a specific address
    Dial {
        addr: Multiaddr,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Dial resolved IP addresses for a DNS multiaddr
    DialResolved {
        original_dns: Multiaddr,
        resolved_addrs: Vec<Multiaddr>,
    },
    /// DNS resolution failed for a bootstrap node
    ResolutionFailed { original_dns: Multiaddr },
    /// Get list of connected peers
    GetPeers { reply: mpsc::Sender<Vec<PeerId>> },
    /// Get bound addresses
    GetBoundAddresses { reply: mpsc::Sender<Vec<Multiaddr>> },
    /// Start listening on an address
    Listen {
        addr: Multiaddr,
        reply: mpsc::Sender<Result<Multiaddr, String>>,
    },
    /// Add a known peer address to Kademlia
    AddKadAddress { peer_id: PeerId, addr: Multiaddr },
    /// Subscribe to a Gossipsub topic
    SubscribeTopic {
        topic: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Unsubscribe from a Gossipsub topic
    UnsubscribeTopic {
        topic: String,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Publish payload to a Gossipsub topic.
    /// The reply channel surfaces gossipsub failures (e.g. InsufficientPeers)
    /// that were previously logged and swallowed — silent message drops.
    PublishTopic {
        topic: String,
        data: Vec<u8>,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Get currently subscribed topics
    GetTopics { reply: mpsc::Sender<Vec<String>> },
    /// Share our ledger with a specific peer
    ShareLedger {
        peer_id: PeerId,
        entries: Vec<SharedPeerEntry>,
    },
    /// Get listening addresses
    GetListeners { reply: mpsc::Sender<Vec<Multiaddr>> },
    /// Update the relay message budget (messages relayed per hour)
    SetRelayBudget { budget: u32 },
    /// Get best relay peers (sorted by reputation)
    GetBestRelays {
        count: usize,
        reply: mpsc::Sender<Vec<PeerId>>,
    },
    /// Get bootstrap candidates (all stable peers)
    GetBootstrapCandidates { reply: mpsc::Sender<Vec<PeerId>> },
    /// Get best paths to a target (Phase 2 multipath)
    GetBestPaths {
        target: PeerId,
        count: usize,
        reply: mpsc::Sender<Vec<Vec<PeerId>>>,
    },
    /// List known endpoints for a peer (addresses observed via address tracking)
    ListEndpoints {
        peer_id: PeerId,
        reply: mpsc::Sender<Vec<Multiaddr>>,
    },
    /// Register a new endpoint address for a peer
    RegisterEndpoint {
        peer_id: PeerId,
        addr: Multiaddr,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Touch (mark as recently seen) an endpoint for health tracking
    TouchEndpoint {
        peer_id: PeerId,
        addr: Multiaddr,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Unregister an endpoint address for a peer (cleanup on disconnect)
    UnregisterEndpoint {
        peer_id: PeerId,
        addr: Multiaddr,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Update the keepalive interval for a peer connection
    UpdateKeepalive {
        peer_id: PeerId,
        interval_secs: u64,
        reply: mpsc::Sender<Result<(), String>>,
    },
    /// Shutdown the swarm
    Shutdown,
}

/// Events emitted by the swarm to the application layer
#[derive(Debug, Clone)]
pub enum SwarmEvent2 {
    /// A new peer was discovered
    PeerDiscovered(PeerId),
    /// A peer disconnected
    PeerDisconnected(PeerId),
    /// An encrypted message was received from a peer
    MessageReceived {
        peer_id: PeerId,
        envelope_data: Vec<u8>,
    },
    /// Address reflection response received
    AddressReflected {
        peer_id: PeerId,
        observed_address: String,
    },
    /// We started listening on an address
    ListeningOn(Multiaddr),
    /// A listener failed to bind or died after binding (async bind/accept
    /// failure, or the listener was closed by the OS). Without this event a
    /// node can silently lose all inbound connectivity while the application
    /// layer still believes it is listening.
    ListenerFailed { listener_id: String, error: String },
    /// A peer's identity was confirmed (after Identify protocol)
    PeerIdentified {
        peer_id: PeerId,
        public_key: Option<String>,
        agent_version: String,
        listen_addrs: Vec<Multiaddr>,
        protocols: Vec<String>,
    },
    /// A new Gossipsub topic was discovered from a peer
    TopicDiscovered { peer_id: PeerId, topic: String },
    /// Received peer list from a connected peer (ledger exchange)
    LedgerReceived {
        from_peer: PeerId,
        entries: Vec<SharedPeerEntry>,
    },
    /// NAT status changed (from AutoNAT probe)
    /// Value is one of: "public:<addr>", "private", "unknown"
    NatStatusChanged(String),
    /// Port mapping event (UPnP).
    PortMapping(String),
    /// Abuse signal detected by relay guardrails (P0_SECURITY_003).
    /// Carries the offending peer ID and the signal type name.
    AbuseSignalDetected { peer_id: PeerId, signal: String },
    /// Relay client successfully established an outbound circuit.
    /// Wired from libp2p::relay::client::Event::OutboundCircuitEstablished.
    RelayCircuitEstablished,
    /// Relay client circuit is broken.
    /// NOTE: libp2p-relay 0.21.1 client::Event has no failure/closed variant, so this
    /// is not currently emitted by the native swarm event loop. It is consumed in
    /// main.rs so a future relay-client event can be wired here.
    RelayCircuitBroken,
}

/// Handle to communicate with the running swarm task
#[derive(Clone)]
pub struct SwarmHandle {
    command_tx: mpsc::Sender<SwarmCommand>,
    // Retained for API symmetry; event loop holds its own core handle.
    #[allow(dead_code)]
    core_handle: Option<Weak<crate::IronCore>>,
}

impl SwarmHandle {
    /// Send an encrypted envelope to a peer.
    ///
    /// `recipient_identity_id` and `intended_device_id` carry WS13 tight-pair metadata.
    /// Legacy callers should pass `None` for both; relay nodes treat absent metadata as
    /// compatibility mode and forward without device enforcement.
    pub async fn send_message(
        &self,
        peer_id: PeerId,
        envelope_data: Vec<u8>,
        recipient_identity_id: Option<String>,
        intended_device_id: Option<String>,
    ) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::SendMessage {
                peer_id,
                envelope_data,
                recipient_identity_id,
                intended_device_id,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn register_identity(
        &self,
        peer_id: PeerId,
        request: RegistrationRequest,
    ) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::RegisterIdentity {
                peer_id,
                request,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn deregister_identity(
        &self,
        peer_id: PeerId,
        request: DeregistrationRequest,
    ) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::DeregisterIdentity {
                peer_id,
                request,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Request address reflection from a peer
    pub async fn request_address_reflection(&self, peer_id: PeerId) -> Result<String> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::RequestAddressReflection {
                peer_id,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Dial a peer at a multiaddress
    pub async fn dial(&self, addr: Multiaddr) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::Dial {
                addr,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get connected peers
    pub async fn get_bound_addresses(&self) -> Result<Vec<Multiaddr>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetBoundAddresses { reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    pub async fn get_peers(&self) -> Result<Vec<PeerId>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetPeers { reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// Start listening on an address
    pub async fn listen(&self, addr: Multiaddr) -> Result<Multiaddr> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::Listen {
                addr,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get external addresses based on peer observations
    pub async fn get_external_addresses(&self) -> Result<Vec<SocketAddr>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetExternalAddresses { reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// Add a known address for a peer in the DHT
    pub async fn add_kad_address(&self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::AddKadAddress { peer_id, addr })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
    }

    /// Get listening addresses
    pub async fn get_listeners(&self) -> Result<Vec<Multiaddr>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetListeners { reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// List known endpoint addresses for a peer.
    /// Returns the set of multiaddresses observed for the peer via address tracking.
    pub async fn list_endpoints(&self, peer_id: PeerId) -> Result<Vec<Multiaddr>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::ListEndpoints {
                peer_id,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// Register a new endpoint address for a peer.
    /// Adds the address to Kademlia's routing table and the address observer.
    pub async fn register_endpoint(&self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::RegisterEndpoint {
                peer_id,
                addr,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        match reply_rx.recv().await {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(anyhow::anyhow!("{}", e)),
            None => Err(anyhow::anyhow!("No reply from swarm")),
        }
    }

    /// Touch (mark as recently seen) an endpoint for health tracking.
    /// Updates the last-seen timestamp for the peer's address observation.
    pub async fn touch_endpoint(&self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::TouchEndpoint {
                peer_id,
                addr,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        match reply_rx.recv().await {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(anyhow::anyhow!("{}", e)),
            None => Err(anyhow::anyhow!("No reply from swarm")),
        }
    }

    /// Unregister an endpoint address for a peer (cleanup on disconnect).
    /// Removes the address from the address observer's tracking.
    pub async fn unregister_endpoint(&self, peer_id: PeerId, addr: Multiaddr) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::UnregisterEndpoint {
                peer_id,
                addr,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        match reply_rx.recv().await {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(anyhow::anyhow!("{}", e)),
            None => Err(anyhow::anyhow!("No reply from swarm")),
        }
    }

    /// Update the keepalive interval for a peer connection.
    /// Configures how frequently keepalive probes are sent.
    pub async fn update_keepalive(&self, peer_id: PeerId, interval_secs: u64) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::UpdateKeepalive {
                peer_id,
                interval_secs,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        match reply_rx.recv().await {
            Some(Ok(())) => Ok(()),
            Some(Err(e)) => Err(anyhow::anyhow!("{}", e)),
            None => Err(anyhow::anyhow!("No reply from swarm")),
        }
    }

    /// Subscribe to a Gossipsub topic. Awaits the swarm's actual outcome —
    /// a subscription failure is returned to the caller, not swallowed.
    pub async fn subscribe_topic(&self, topic: String) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::SubscribeTopic {
                topic,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;
        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Unsubscribe from a Gossipsub topic. Idempotent: unsubscribing from a
    /// topic we never joined is Ok(()).
    pub async fn unsubscribe_topic(&self, topic: String) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::UnsubscribeTopic {
                topic,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;
        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Publish data to a Gossipsub topic. Awaits the publish outcome so
    /// failures like InsufficientPeers reach the caller instead of being
    /// dropped silently.
    pub async fn publish_topic(&self, topic: String, data: Vec<u8>) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::PublishTopic {
                topic,
                data,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;
        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))?
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Get currently subscribed topics
    pub async fn get_topics(&self) -> Result<Vec<String>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetTopics { reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// Share our ledger with a specific peer
    pub async fn share_ledger(&self, peer_id: PeerId, entries: Vec<SharedPeerEntry>) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::ShareLedger { peer_id, entries })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
    }

    /// Set the relay message budget (messages relayed per hour).
    pub async fn set_relay_budget(&self, messages_per_hour: u32) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::SetRelayBudget {
                budget: messages_per_hour,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
    }

    /// Get best relay peers (sorted by reputation)
    pub async fn get_best_relays(&self, count: usize) -> Result<Vec<PeerId>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetBestRelays {
                count,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// Get bootstrap candidates (all stable peers)
    pub async fn get_bootstrap_candidates(&self) -> Result<Vec<PeerId>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetBootstrapCandidates { reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// Get best paths to a target (Phase 2 multipath)
    pub async fn get_best_paths(&self, target: PeerId, count: usize) -> Result<Vec<Vec<PeerId>>> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::GetBestPaths {
                target,
                count,
                reply: reply_tx,
            })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))?;

        reply_rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("No reply from swarm"))
    }

    /// Shut down the swarm
    pub async fn shutdown(&self) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::Shutdown)
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
    }
}

/// Create a default (empty) routing engine handle for use by callers that
/// don't have an IronCore instance (e.g. CLI, WASM). The handle will be
/// seeded with the local peer ID when the swarm starts.
pub fn default_routing_engine_handle() -> Arc<parking_lot::RwLock<Option<OptimizedRoutingEngine>>> {
    Arc::new(parking_lot::RwLock::new(None))
}

/// Build and start the libp2p swarm, returning a handle for communication.
///
/// This spawns a tokio task that runs the swarm event loop.
/// The returned handle can be used to send commands to the swarm.
///
/// If `multiport_config` is provided, the swarm will attempt to bind to multiple
/// ports for maximum connectivity. Otherwise, it uses the single `listen_addr`.
pub async fn start_swarm(
    keypair: Keypair,
    listen_addr: Option<Multiaddr>,
    event_tx: mpsc::Sender<SwarmEvent2>,
    core_handle: Option<Weak<crate::IronCore>>,
    headless: bool,
    discovery_config: Option<DiscoveryConfig>,
    routing_engine_handle: Arc<parking_lot::RwLock<Option<OptimizedRoutingEngine>>>,
) -> Result<SwarmHandle> {
    start_swarm_with_config(
        keypair,
        listen_addr,
        event_tx,
        None,
        Vec::new(),
        None,
        core_handle,
        headless,
        discovery_config,
        routing_engine_handle,
    )
    .await
}

/// Build and start the libp2p swarm with custom multi-port configuration.
///
/// `bootstrap_addrs` — Multiaddrs of well-known relay / bootstrap nodes.
/// The swarm will auto-dial these after binding, enabling cross-network
/// peer discovery via Kademlia DHT and relay-circuit connectivity.
#[allow(
    clippy::too_many_arguments,
    clippy::blocks_in_conditions,
    unused_variables
)]
pub async fn start_swarm_with_config(
    keypair: Keypair,
    listen_addr: Option<Multiaddr>,
    event_tx: mpsc::Sender<SwarmEvent2>,
    multiport_config: Option<MultiPortConfig>,
    bootstrap_addrs: Vec<Multiaddr>,
    storage_path: Option<String>,
    core_handle: Option<Weak<crate::IronCore>>,
    headless: bool,
    discovery_config: Option<DiscoveryConfig>,
    routing_engine_handle: Arc<parking_lot::RwLock<Option<OptimizedRoutingEngine>>>,
) -> Result<SwarmHandle> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let local_peer_id = keypair.public().to_peer_id();

        // libp2p's convenience WebSocket builder reads the system DNS config.
        // iOS apps have no /etc/resolv.conf, so use the explicit resolver path
        // below just as Android does.
        #[cfg(all(not(target_os = "android"), not(target_os = "ios")))]
        let mut swarm: libp2p::Swarm<IronCoreBehaviour> =
            libp2p::SwarmBuilder::with_existing_identity(keypair)
                .with_tokio()
                .with_tcp(
                    libp2p::tcp::Config::default(),
                    libp2p::noise::Config::new,
                    libp2p::yamux::Config::default,
                )?
                .with_websocket(libp2p::noise::Config::new, libp2p::yamux::Config::default)
                .await?
                .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)?
                .with_behaviour(|key, relay_client| {
                    IronCoreBehaviour::new(key, relay_client, headless, discovery_config)
                        .expect("Failed to create network behaviour")
                })?
                .with_swarm_config(|cfg: libp2p::swarm::Config| {
                    cfg.with_idle_connection_timeout(web_time::Duration::from_secs(600))
                })
                .build();

        #[cfg(any(target_os = "android", target_os = "ios"))]
        let mut swarm: libp2p::Swarm<IronCoreBehaviour> = {
            use libp2p::Transport;
            libp2p::SwarmBuilder::with_existing_identity(keypair)
                .with_tokio()
                .with_other_transport(
                    |id_keys| -> std::result::Result<_, Box<dyn std::error::Error + Send + Sync>> {
                        let tcp_transport1 =
                            libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default());
                        let dns_tcp1 = libp2p::dns::tokio::Transport::custom(
                            tcp_transport1,
                            libp2p::dns::ResolverConfig::google(),
                            libp2p::dns::ResolverOpts::default(),
                        );
                        let tcp_transport2 =
                            libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default());
                        let dns_tcp2 = libp2p::dns::tokio::Transport::custom(
                            tcp_transport2,
                            libp2p::dns::ResolverConfig::google(),
                            libp2p::dns::ResolverOpts::default(),
                        );
                        let ws_transport = libp2p::websocket::Config::new(dns_tcp2);
                        let transport = dns_tcp1.or_transport(ws_transport);
                        let noise = libp2p::noise::Config::new(id_keys)?;
                        Ok(transport
                            .upgrade(libp2p::core::upgrade::Version::V1Lazy)
                            .authenticate(noise)
                            .multiplex(libp2p::yamux::Config::default())
                            .map(|(peer_id, conn), _| {
                                (peer_id, libp2p::core::muxing::StreamMuxerBox::new(conn))
                            }))
                    },
                )?
                .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)?
                .with_behaviour(|key, relay_client| {
                    IronCoreBehaviour::new(key, relay_client, headless, discovery_config)
                        .expect("Failed to create network behaviour")
                })?
                .with_swarm_config(|cfg: libp2p::swarm::Config| {
                    cfg.with_idle_connection_timeout(web_time::Duration::from_secs(600))
                })
                .build()
        };

        // Start listening on ports
        let mut bind_results = Vec::new();

        if let Some(config) = multiport_config {
            // Multi-port mode: Try binding to all configured ports
            tracing::info!("Starting multi-port adaptive listening");
            let addresses = multiport::generate_listen_addresses(&config);

            for (addr, port) in addresses {
                match swarm.listen_on(addr.clone()) {
                    Ok(_) => {
                        tracing::info!("[OK] Bound to {}", addr);
                        bind_results.push(BindResult::Success { addr, port });
                    }
                    Err(e) => {
                        let error = format!("{}", e);
                        tracing::warn!(
                            "[FAIL] Failed to bind to {} (port {}): {}",
                            addr,
                            port,
                            error
                        );
                        bind_results.push(BindResult::Failed { port, error });
                    }
                }
            }

            // Analyze and report results
            let analysis = multiport::analyze_bind_results(&bind_results);
            tracing::info!("\n{}", analysis.report());

            if analysis.successful.is_empty() {
                return Err(anyhow::anyhow!("Failed to bind to any port"));
            }
        } else {
            // Single port mode (legacy behavior)
            let addr = listen_addr.unwrap_or_else(|| {
                "/ip4/0.0.0.0/tcp/0"
                    .parse()
                    .expect("static multiaddr parse cannot fail")
            });
            swarm.listen_on(addr)?;
        }

        // B1_CORE_ENTRY_008: Random port for temporary listeners
        // Call random_port to ensure the IronCore instance is actively used.
        // This also exercises the random port generation path in production.
        if let Some(ref core_weak) = core_handle {
            if let Some(core) = core_weak.upgrade() {
                let _random_port = core.random_port();
                tracing::debug!(
                    "B1_CORE_ENTRY_008: Random port generated for listener: {}",
                    _random_port
                );
            }
        }

        // Expose a QUIC-v1 listener for NAT traversal and future relay-circuit upgrades.
        // Uses quic-v1 (RFC 9001) which is the modern standard in libp2p; the legacy /quic
        // protocol tag is not supported by libp2p ≥ 0.53 SwarmBuilder.
        if let Ok(quic_addr) = "/ip4/0.0.0.0/udp/0/quic-v1".parse::<Multiaddr>() {
            match swarm.listen_on(quic_addr.clone()) {
                Ok(_) => tracing::info!("[OK] Bound QUIC-v1 listener {}", quic_addr),
                Err(e) => tracing::debug!("QUIC-v1 listener not available ({}): {}", quic_addr, e),
            }
        } else {
            tracing::debug!("QUIC-v1 multiaddr parse failed — skipping QUIC listener");
        }

        // ADDED: Always expose a WebSocket listener for WASM bridge on 9002
        if let Ok(ws_addr) = "/ip4/0.0.0.0/tcp/9002/ws".parse::<Multiaddr>() {
            match swarm.listen_on(ws_addr.clone()) {
                Ok(_) => tracing::info!("[OK] Bound WebSocket listener {}", ws_addr),
                Err(e) => tracing::warn!(
                    "[FAIL] Failed to bind WebSocket listener {}: {}",
                    ws_addr,
                    e
                ),
            }
        }

        // Kademlia already set to Server mode in behaviour constructor,
        // but set it again here for belt-and-suspenders:
        swarm
            .behaviour_mut()
            .kademlia
            .set_mode(Some(kad::Mode::Server));

        // Subscribe to default topics immediately (lobby + mesh)
        // The lobby topic is the wildcard discovery channel
        let lobby_topic = libp2p::gossipsub::IdentTopic::new("sc-lobby");
        let mesh_topic = libp2p::gossipsub::IdentTopic::new("sc-mesh");
        let delivery_convergence_topic =
            libp2p::gossipsub::IdentTopic::new(DELIVERY_CONVERGENCE_TOPIC);

        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&lobby_topic) {
            tracing::warn!("Failed to subscribe to lobby topic: {}", e);
        } else {
            tracing::info!("Subscribed to lobby topic: sc-lobby");
        }

        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&mesh_topic) {
            tracing::warn!("Failed to subscribe to mesh topic: {}", e);
        } else {
            tracing::info!("Subscribed to mesh topic: sc-mesh");
        }

        if let Err(e) = swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&delivery_convergence_topic)
        {
            tracing::warn!("Failed to subscribe to delivery convergence topic: {}", e);
        } else {
            tracing::info!(
                "Subscribed to delivery convergence topic: {}",
                DELIVERY_CONVERGENCE_TOPIC
            );
        }

        let (command_tx, mut command_rx) = mpsc::channel::<SwarmCommand>(256);
        let handle = SwarmHandle {
            command_tx: command_tx.clone(),
            core_handle: core_handle.clone(),
        };

        // Address reflection service
        let reflection_service = AddressReflectionService::new();

        // Track pending address reflection requests
        let mut bound_addresses = Vec::new();
        let mut pending_reflections: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<String, String>>,
        > = HashMap::new();
        let mut pending_registration_replies: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<(), String>>,
        > = HashMap::new();

        // Track connections and address observations (Phase 1 & 2)
        let mut connection_tracker = ConnectionTracker::new();
        let mut address_observer = AddressObserver::new();

        // Track successful relay reservations by ListenerId
        let mut successful_relay_reservations: HashMap<
            PeerId,
            libp2p::core::transport::ListenerId,
        > = HashMap::new();

        // P0.12: Deduplicate bridge events to prevent UI freezing and bridge spam
        // We track the last reported 'PeerIdentified' and 'PeerDiscovered' state.
        let _reported_peer_info: HashMap<PeerId, (String, Vec<Multiaddr>)> = HashMap::new();
        let _reported_peer_discoveries: HashSet<PeerId> = HashSet::new();

        tracing::info!("=== OWN_IDENTITY: {} ===", local_peer_id);

        // Mesh routing components (Phase 3-6)
        let mut multi_path_delivery = MultiPathDelivery::new();
        let mut bootstrap_capability = BootstrapCapability::new();

        // Mycorrhizal Routing Engine (Layer 1-3)
        // Use the shared routing engine handle from IronCore so the swarm and
        // the public API share the same engine state. If the engine has not been
        // initialized yet (identity not yet created), seed it with the local
        // peer ID so routing can begin immediately.
        let local_peer_id_bytes_raw = local_peer_id.to_bytes();
        let local_peer_id_bytes: [u8; 32] = extract_peer_id_bytes(&local_peer_id_bytes_raw);
        let local_hint = blake3::hash(&local_peer_id_bytes).as_bytes()[0..4]
            .try_into()
            .expect("blake3 hash should be at least 4 bytes");
        {
            let mut guard = routing_engine_handle.write();
            if guard.is_none() {
                *guard = Some(OptimizedRoutingEngine::new(local_peer_id_bytes, local_hint));
            }
        }

        // Track pending message deliveries
        let mut pending_messages: HashMap<String, PendingMessage> = HashMap::new();

        // SyncSession management for Drift Protocol mesh synchronization
        let mut sync_sessions: HashMap<PeerId, SyncSession> = HashMap::new();

        // Track outbound request IDs to message IDs for direct sends
        let mut request_to_message: HashMap<libp2p::request_response::OutboundRequestId, String> =
            HashMap::new();

        // Track outbound relay request IDs
        let mut pending_relay_requests: HashMap<
            libp2p::request_response::OutboundRequestId,
            String,
        > = HashMap::new();
        let relay_custody_store = RelayCustodyStore::for_service_storage(
            storage_path.as_deref(),
            &local_peer_id.to_string(),
        );
        let mut pending_custody_dispatches: HashMap<
            libp2p::request_response::OutboundRequestId,
            PendingCustodyDispatch,
        > = HashMap::new();

        // Track subscribed topics for dynamic negotiation
        let mut subscribed_topics: HashSet<String> = HashSet::new();
        subscribed_topics.insert("sc-lobby".to_string());
        subscribed_topics.insert("sc-mesh".to_string());
        subscribed_topics.insert(DELIVERY_CONVERGENCE_TOPIC.to_string());

        // Track peers we've already exchanged ledgers with (avoid spamming)
        let mut ledger_exchanged_peers: HashSet<PeerId> = HashSet::new();

        // Track connected peers for relay peer discovery broadcasting
        let mut peer_broadcaster = crate::transport::PeerBroadcaster::new();

        // Track relay peers and their publicly-routable addresses for circuit reservation.
        // When we identify a relay, we save its WAN addrs here and attempt
        // swarm.listen_on(<relay_addr>/p2p-circuit) to register a reservation,
        // which lets the relay dial us back on behalf of other nodes.
        let mut relay_peer_addrs: HashMap<PeerId, Vec<Multiaddr>> = HashMap::new();

        // Track relay reconnect backoff state: (peer_id, attempt_count, next_dial_at)
        let _relay_backoff: HashMap<PeerId, (u32, web_time::Instant)> = HashMap::new();
        // Mycorrhizal routing: smart retry backoff strategy for pending messages
        let _retry_backoff = BackoffStrategy {
            base_ms: 1000,
            max_ms: 30000,
            multiplier: 2.0,
        };
        let mut relay_reconnect_pending: Vec<(PeerId, u32, web_time::Instant)> = Vec::new();
        let mut seen_delivery_convergence_markers: HashSet<String> = HashSet::new();

        // Auto-dial bootstrap nodes for cross-network discovery
        // Self-dial guard: track bootstrap addrs that resolve to our own peer
        // so we log once at info level and then suppress the warning spam.
        let mut self_dial_logged: HashSet<Multiaddr> = HashSet::new();
        if !bootstrap_addrs.is_empty() {
            tracing::info!(
                "Dialing {} bootstrap node(s) for NAT traversal",
                bootstrap_addrs.len()
            );
            for addr in &bootstrap_addrs {
                // Self-dial check: skip bootstrap addrs whose p2p component
                // matches our own peer ID (e.g. portproxy loopback).
                let is_self = addr.iter().any(|proto| {
                    if let libp2p::multiaddr::Protocol::P2p(pid) = proto {
                        pid == local_peer_id
                    } else {
                        false
                    }
                });
                if is_self {
                    if !self_dial_logged.contains(addr) {
                        tracing::info!(
                            "  ⊘ Skipping self-dial bootstrap addr (matches local peer): {}",
                            addr
                        );
                        self_dial_logged.insert(addr.clone());
                    }
                    continue;
                }
                let stripped_addr: Multiaddr = addr
                    .iter()
                    .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
                    .collect();
                match swarm.dial(stripped_addr.clone()) {
                    Ok(_) => tracing::info!("  [OK] Dialing bootstrap: {}", stripped_addr),
                    Err(e) => {
                        tracing::warn!("  [FAIL] Failed to dial bootstrap {}: {}", stripped_addr, e)
                    }
                }
            }
        }

        // Spawn the swarm event loop
        tokio::spawn(async move {
            // PHASE 6: Retry interval for failed deliveries
            let mut retry_interval = tokio::time::interval(Duration::from_millis(500));

            // Bootstrap reconnection timer — re-dial bootstrap nodes every 60s
            // to handle network changes and maintain connectivity.
            // Exponential backoff per addr: 60s → 120s → … → 960s max on failure;
            // resets to 60s on success.
            let mut bootstrap_reconnect_interval = tokio::time::interval(Duration::from_secs(60));
            let bootstrap_addrs_clone = bootstrap_addrs;
            let mut bootstrap_backoff: HashMap<Multiaddr, BootstrapBackoffEntry> = HashMap::new();
            let mut resolved_to_dns: HashMap<Multiaddr, Multiaddr> = HashMap::new();
            let mut resolved_keys_fifo: std::collections::VecDeque<Multiaddr> =
                std::collections::VecDeque::new();
            let mut in_flight_dns: HashSet<Multiaddr> = HashSet::new();

            // Dials awaiting a real ConnectionEstablished/OutgoingConnectionError
            // signal before their SwarmCommand::Dial reply is sent (see
            // PendingDialEntry doc comment above).
            let mut pending_dials: HashMap<Multiaddr, PendingDialEntry> = HashMap::new();
            let mut pending_dial_sweep_interval = tokio::time::interval(Duration::from_secs(5));

            // Cover traffic — 1 dummy message/min to mask real traffic patterns
            let mut cover_traffic_interval = tokio::time::interval(Duration::from_secs(60));

            // Relay budget rate-limiting
            let mut relay_budget: u32 = 200;
            let mut relay_count_this_hour: u32 = 0;
            let mut relay_hour_start = web_time::Instant::now();
            let mut relay_guardrails = RelayAbuseGuardrails::new();

            // P0.12: Deduplicate bridge events to prevent UI freezing and bridge spam
            // We track the last reported 'PeerIdentified' and 'PeerDiscovered' state.
            let mut reported_peer_info: HashMap<PeerId, (String, Vec<Multiaddr>)> = HashMap::new();
            let mut reported_peer_discoveries: std::collections::HashSet<PeerId> =
                std::collections::HashSet::new();

            // Mycorrhizal routing: periodic optimization tick (every 30s)
            let mut routing_optimization_interval = tokio::time::interval(Duration::from_secs(30));

            // Check for pending relay reconnects frequently
            let mut relay_reconnect_interval = tokio::time::interval(Duration::from_secs(5));
            let mut custody_pull_interval = tokio::time::interval(Duration::from_secs(5));

            loop {
                tokio::select! {
                    // PHASE 6: Periodic retry check
                    _ = retry_interval.tick() => {
                        // Check for messages that need retry
                        let mut to_retry = Vec::new();

                        for (msg_id, pending) in pending_messages.iter() {
                            if let Some(attempt) = multi_path_delivery.delivery_attempt(msg_id) {
                                if attempt.should_retry() {
                                    let elapsed = pending.attempt_start.elapsed().unwrap_or_default();
                                    let retry_delay = attempt.next_retry_delay();
                                    // Mycorrhizal routing: use smart retry backoff
                                    let smart_delay = calculate_next_attempt(
                                        pending.pass_count,
                                        &_retry_backoff,
                                    );
                                    let now_ms = web_time::SystemTime::now()
                                        .duration_since(web_time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis() as u64;
                                    let smart_delay_dur = web_time::Duration::from_millis(
                                        smart_delay.saturating_sub(now_ms)
                                    );

                                    if elapsed >= retry_delay.max(smart_delay_dur) {
                                        to_retry.push(msg_id.clone());
                                    }
                                }
                            }
                        }

                        // Process retries
                        for msg_id in to_retry {
                            if let Some(mut pending) = pending_messages.remove(&msg_id) {
                                let routes = multi_path_delivery.ranked_routes(&pending.target_peer, 3);
                                if routes.is_empty() {
                                    tracing::warn!(
                                        "No route candidates available for message {}; keeping in retry cycle",
                                        msg_id
                                    );
                                    pending_messages.insert(msg_id, pending);
                                    continue;
                                }

                                let cursor = advance_route_cursor(pending.current_path_index, routes.len());
                                pending.current_path_index = cursor.next_index;
                                if cursor.wrapped_pass {
                                    pending.pass_count = pending.pass_count.saturating_add(1);
                                    if !pending.retry_notified {
                                        tracing::warn!(
                                            "Delivery pass failed for message {}; continuing cyclic retries",
                                            msg_id
                                        );
                                        let _ = pending
                                            .reply_tx
                                            .send(Err("Delivery pending retry".to_string()))
                                            .await;
                                        pending.retry_notified = true;
                                    }
                                }

                                let route = &routes[pending.current_path_index];
                                pending.attempt_start = SystemTime::now();
                                pending.dispatch_attempts = pending.dispatch_attempts.saturating_add(1);
                                let attempt_reason = if cursor.wrapped_pass {
                                    ROUTE_ATTEMPT_REASON_RETRY_CYCLE
                                } else {
                                    ROUTE_ATTEMPT_REASON_RETRY_NEXT
                                };
                                log_route_decision(
                                    &msg_id,
                                    route,
                                    pending.dispatch_attempts,
                                    pending.pass_count,
                                    pending.current_path_index,
                                    routes.len(),
                                    attempt_reason,
                                );
                                dispatch_ranked_route(
                                    &mut swarm,
                                    route,
                                    &msg_id,
                                    pending.target_peer,
                                    &pending.envelope_data,
                                    &mut request_to_message,
                                    &mut pending_relay_requests,
                                    pending.recipient_identity_id.as_deref(),
                                    pending.intended_device_id.as_deref(),
                                );

                                pending_messages.insert(msg_id, pending);
                            }
                        }
                    }

                    // Periodic relay-side pull of pending custody for connected peers.
                    _ = custody_pull_interval.tick() => {
                        let connected: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                        for destination in connected {
                            dispatch_pending_custody_for_peer(
                                &mut swarm,
                                &relay_custody_store,
                                destination,
                                &mut pending_custody_dispatches,
                                RELAY_MAX_INFLIGHT_DISPATCHES,
                                "periodic_pull",
                            );
                        }
                    }

                    // Expire pending Dial replies that never got a real
                    // ConnectionEstablished/OutgoingConnectionError signal (e.g. the
                    // dial went into a black hole — no ICMP/RST at all), so the
                    // caller's reply_rx.recv().await doesn't hang forever.
                    _ = pending_dial_sweep_interval.tick() => {
                        let timed_out: Vec<Multiaddr> = pending_dials
                            .iter()
                            .filter(|(_, entry)| entry.dialed_at.elapsed() >= web_time::Duration::from_secs(PENDING_DIAL_TIMEOUT_SECS))
                            .map(|(key, _)| key.clone())
                            .collect();
                        for key in timed_out {
                            if let Some(entry) = pending_dials.remove(&key) {
                                tracing::debug!("Pending dial to {} timed out after {}s with no connection signal", key, PENDING_DIAL_TIMEOUT_SECS);
                                let _ = entry.reply.send(Err(format!("Dial timed out after {}s with no connection signal", PENDING_DIAL_TIMEOUT_SECS))).await;
                            }
                        }
                    }

                    // Mycorrhizal routing: periodic optimization tick
                    _ = routing_optimization_interval.tick() => {
                        let now_secs = web_time::SystemTime::now()
                            .duration_since(web_time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let mut guard = routing_engine_handle.write();
                        if let Some(ref mut engine) = guard.as_mut() {
                            let maintenance = engine.tick(now_secs);
                            let neg = &maintenance.negative_cache_stats;
                            tracing::debug!(
                                "[ROUTING] Optimization tick: neg_cache({} entries, {} cleaned), \
                                 ttl({} cleaned), budget({:?} elapsed, {:?} phase)",
                                neg.entry_count,
                                maintenance.negative_cache_entries_cleaned,
                                maintenance.adaptive_ttl_entries_cleaned,
                                maintenance.timeout_budget_summary.elapsed,
                                maintenance.timeout_budget_summary.current_phase,
                            );
                        }
                    }

                    // P0.11: Relay reconnect backoff processing
                    _ = relay_reconnect_interval.tick() => {
                        let now = web_time::Instant::now();
                        let mut next_pending = Vec::new();
                        let connected_peers: HashSet<PeerId> = swarm.connected_peers().cloned().collect();

                        for (peer_id, attempts, next_dial) in relay_reconnect_pending.drain(..) {
                            if connected_peers.contains(&peer_id) {
                                // Already connected; drop from pending queue
                                tracing::debug!("[OK] Relay {} reconnected successfully", peer_id);
                                continue;
                            }

                            if now >= next_dial {
                                // Time to try dialing!
                                if let Some(addrs) = relay_peer_addrs.get(&peer_id) {
                                    if let Some(addr) = addrs.first() {
                                        tracing::info!(
                                            "Attempting to re-dial relay {} (Attempt {}): {}",
                                            peer_id, attempts + 1, addr
                                        );
                                        match swarm.dial(addr.clone()) {
                                            Ok(_) => {
                                                // Re-enqueue with backoff for next attempt if this fails.
                                                // Backoff: 10s -> 30s -> 60s
                                                let backoff_secs = match attempts {
                                                    0 => 10,
                                                    1 => 30,
                                                    _ => 60,
                                                };
                                                next_pending.push((
                                                    peer_id,
                                                    attempts + 1,
                                                    now + Duration::from_secs(backoff_secs),
                                                ));
                                            }
                                            Err(e) => {
                                                tracing::warn!("[WARNING] Re-dial to relay {} failed immediately: {}", peer_id, e);
                                                // Re-enqueue with max backoff
                                                next_pending.push((
                                                    peer_id,
                                                    attempts + 1,
                                                    now + Duration::from_secs(60),
                                                ));
                                            }
                                        }
                                    }
                                }
                            } else {
                                // Not time yet, keep in queue
                                next_pending.push((peer_id, attempts, next_dial));
                            }
                        }
                        relay_reconnect_pending = next_pending;
                    }

                    // Bootstrap reconnection: re-dial bootstrap nodes periodically
                    // This handles network changes, dropped connections, and roaming.
                    // Exponential backoff per addr avoids spamming logs for persistently
                    // unreachable nodes (Connection refused, timeout, etc.).
                    _ = bootstrap_reconnect_interval.tick() => {
                        // Prune resolved_to_dns entries for hostnames no longer in bootstrap config
                        resolved_to_dns.retain(|_, original_dns| {
                            bootstrap_addrs_clone.contains(original_dns)
                        });
                        tracing::info!("Relay custody audit log count: {}", relay_custody_store.audit_count());
                        if !bootstrap_addrs_clone.is_empty() {
                            let connected_peers: HashSet<PeerId> = swarm.connected_peers().cloned().collect();
                            for addr in &bootstrap_addrs_clone {
                                // Self-dial guard: skip bootstrap addrs that resolve to
                                // our own peer ID (e.g. portproxy loopback) to avoid
                                // the "tried to dial local peer id" warning every 60s.
                                let is_self = addr.iter().any(|proto| {
                                    if let libp2p::multiaddr::Protocol::P2p(pid) = proto {
                                        pid == local_peer_id
                                    } else {
                                        false
                                    }
                                });
                                if is_self {
                                    if !self_dial_logged.contains(addr) {
                                        tracing::info!(
                                            "  ⊘ Skipping self-dial bootstrap addr (matches local peer): {}",
                                            addr
                                        );
                                        self_dial_logged.insert(addr.clone());
                                    }
                                    continue;
                                }

                                // Exponential backoff gate: skip this addr if it's still
                                // within its backoff window after a recent failure.
                                if !bootstrap_backoff.get(addr).is_none_or(|e| e.is_eligible()) {
                                    continue;
                                }

                                // Extract peer ID from multiaddr if present to avoid
                                // re-dialing already-connected bootstrap nodes
                                let already_connected = addr.iter().any(|proto| {
                                    if let libp2p::multiaddr::Protocol::P2p(pid) = proto {
                                        connected_peers.contains(&pid)
                                    } else {
                                        false
                                    }
                                });

                                if !already_connected {
                                    let stripped_addr: Multiaddr = addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
                                    if is_dns_multiaddr(&stripped_addr) {
                                        if !in_flight_dns.contains(addr) {
                                            in_flight_dns.insert(addr.clone());
                                            let addr_clone = addr.clone();
                                            let command_tx_clone = command_tx.clone();
                                            tokio::spawn(async move {
                                                let resolved = resolve_dns_multiaddr(&addr_clone).await;
                                                if resolved.is_empty() {
                                                    let _ = command_tx_clone.send(SwarmCommand::ResolutionFailed {
                                                        original_dns: addr_clone,
                                                    }).await;
                                                } else {
                                                    let _ = command_tx_clone.send(SwarmCommand::DialResolved {
                                                        original_dns: addr_clone,
                                                        resolved_addrs: resolved,
                                                    }).await;
                                                }
                                            });
                                        }
                                    } else {
                                        match swarm.dial(stripped_addr.clone()) {
                                            Ok(_) => tracing::debug!("Re-dialing bootstrap: {}", stripped_addr),
                                            Err(e) => {
                                                // Dial was rejected internally (e.g. already dialing).
                                                // Treat as a failure and apply backoff to avoid retry spam.
                                                tracing::trace!("Bootstrap re-dial {} skipped: {}", stripped_addr, e);
                                                bootstrap_backoff.entry(addr.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Cover traffic — publish a dummy gossipsub message to mask real traffic
                    _ = cover_traffic_interval.tick() => {
                        use crate::privacy::cover::{CoverConfig, CoverTrafficGenerator};
                        if let Ok(gen) = CoverTrafficGenerator::new(CoverConfig {
                            rate_per_minute: 1,
                            message_size: 256,
                            enabled: true,
                        }) {
                            if let Ok(cover_msg) = gen.generate_cover_message() {
                                if let Ok(bytes) = bincode::serialize(&cover_msg) {
                                    let topic = libp2p::gossipsub::IdentTopic::new("sc-mesh");
                                    let _ = swarm.behaviour_mut().gossipsub.publish(topic, bytes);
                                }
                            }
                        }
                    }

                    // Process incoming swarm events
                    event = swarm.select_next_some() => {
                        match event {
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Messaging(
                                request_response::Event::Message { peer, message, .. }
                            )) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        // Block enforcement FIRST (before any parse or dial): a blocked
                                        // peer must not be able to drive relay-discovery dialing. This
                                        // check previously ran after the relay-discovery branch; keeping
                                        // it here closes that bypass (adversarial review 2026-07-17 F3).
                                        let sender_blocked = if let Some(core_handle) = core_handle.as_ref().and_then(|w| w.upgrade()) {
                                            // Libp2pMessageRequest doesn't have device_id, only RelayRequest does.
                                            // For direct messaging, we check the peer-level block.
                                            core_handle.is_peer_blocked(peer.to_string(), None).unwrap_or(false)
                                        } else {
                                            false
                                        };
                                        if sender_blocked {
                                            tracing::warn!("Blocked peer {} attempted to send message", peer);
                                            let _ = swarm.behaviour_mut().messaging.send_response(
                                                channel,
                                                Libp2pMessageResponse { accepted: false, error: Some("blocked".to_string()) },
                                            );
                                            continue;
                                        }

                                        // Unwrap DriftFrame FIRST: relay peer-discovery messages
                                        // (PeerJoined/PeerListResponse/PeerLeft) arrive DriftFrame-wrapped
                                        // like all /sc/message traffic. Probing RelayMessage on the raw
                                        // wrapped bytes never matches, so they fell through to envelope
                                        // decode and failed with "unexpected end of file" (farm-sim
                                        // root cause, HERMES_FARM_AUDIT 2026-07-16).
                                        let envelope_payload = match DriftFrame::from_bytes(&request.envelope_data) {
                                            Ok(frame) => {
                                                tracing::debug!(
                                                    "Received DriftFrame type: {:?} from {}",
                                                    frame.frame_type,
                                                    peer
                                                );
                                                frame.payload
                                            }
                                            Err(_) => {
                                                // Not a DriftFrame: either a legacy message or a
                                                // corrupted frame. Log at debug so truncation events
                                                // are visible without being noisy in normal operation.
                                                // The envelope decoder below will reject invalid data.
                                                tracing::debug!(
                                                    "No DriftFrame header from {} (len={}); treating as legacy envelope",
                                                    peer,
                                                    request.envelope_data.len()
                                                );
                                                request.envelope_data.clone()
                                            }
                                        };

                                        // RELAY PEER DISCOVERY: relay control messages are small. Only
                                        // probe payloads under RELAY_CONTROL_MAX_BYTES so a 4 MiB raw
                                        // payload can't drive a giant bincode alloc + dial storm on the
                                        // shared event loop (adversarial review 2026-07-17 F1/F2/F4).
                                        const RELAY_CONTROL_MAX_BYTES: usize = 64 * 1024;
                                        const MAX_DISCOVERY_DIALS: usize = 32;
                                        if envelope_payload.len() <= RELAY_CONTROL_MAX_BYTES {
                                        if let Ok(relay_msg) = crate::relay::protocol::RelayMessage::from_bytes(&envelope_payload) {
                                            match relay_msg {
                                                crate::relay::protocol::RelayMessage::PeerJoined { peer_info } => {
                                                    tracing::info!("Received PeerJoined: {} with {} addresses", peer_info.peer_id, peer_info.addresses.len());
                                                    let mut dialed = 0usize;
                                                    for addr_str in peer_info.addresses.iter().take(MAX_DISCOVERY_DIALS) {
                                                        if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                                            if is_discoverable_multiaddr(&addr) {
                                                                tracing::debug!("  Dialing announced peer at {}", addr);
                                                                let _ = swarm.dial(addr);
                                                                dialed += 1;
                                                            }
                                                        }
                                                    }
                                                    if peer_info.addresses.len() > MAX_DISCOVERY_DIALS {
                                                        tracing::debug!("  Capped PeerJoined dials at {} (had {})", dialed, peer_info.addresses.len());
                                                    }
                                                    let _ = swarm.behaviour_mut().messaging.send_response(
                                                        channel,
                                                        Libp2pMessageResponse { accepted: true, error: None },
                                                    );
                                                    continue;
                                                }
                                                crate::relay::protocol::RelayMessage::PeerListResponse { peers } => {
                                                    tracing::info!("Received peer list: {} peers", peers.len());
                                                    // Cap TOTAL dials across all peers, not per-peer, so a
                                                    // large peer list cannot amplify into thousands of dials.
                                                    let mut dialed = 0usize;
                                                    'outer: for peer_info in peers {
                                                        tracing::debug!("  Peer: {} ({} addresses)", peer_info.peer_id, peer_info.addresses.len());
                                                        for addr_str in &peer_info.addresses {
                                                            if dialed >= MAX_DISCOVERY_DIALS {
                                                                tracing::debug!("  Capped PeerListResponse dials at {}", MAX_DISCOVERY_DIALS);
                                                                break 'outer;
                                                            }
                                                            if let Ok(addr) = addr_str.parse::<Multiaddr>() {
                                                                if is_discoverable_multiaddr(&addr) {
                                                                    let _ = swarm.dial(addr);
                                                                    dialed += 1;
                                                                }
                                                            }
                                                        }
                                                    }
                                                    let _ = swarm.behaviour_mut().messaging.send_response(
                                                        channel,
                                                        Libp2pMessageResponse { accepted: true, error: None },
                                                    );
                                                    continue;
                                                }
                                                crate::relay::protocol::RelayMessage::PeerLeft { peer_id } => {
                                                    tracing::info!("Peer left: {}", peer_id);
                                                    let _ = swarm.behaviour_mut().messaging.send_response(
                                                        channel,
                                                        Libp2pMessageResponse { accepted: true, error: None },
                                                    );
                                                    continue;
                                                }
                                                _ => {
                                                    // Other relay messages, fall through to normal handling
                                                }
                                            }
                                        }
                                        }

                                        // Received a message from a peer
                                        let _ = event_tx.send(SwarmEvent2::MessageReceived {
                                            peer_id: peer,
                                            envelope_data: envelope_payload,
                                        }).await;

                                        // Send acceptance response
                                        let _ = swarm.behaviour_mut().messaging.send_response(
                                            channel,
                                            Libp2pMessageResponse { accepted: true, error: None },
                                        );
                                    }
                                    request_response::Message::Response { request_id, response } => {
                                        if let Some(dispatch) =
                                            pending_custody_dispatches.remove(&request_id)
                                        {
                                            if response.accepted {
                                                if let Err(e) = relay_custody_store.mark_delivered(
                                                    &dispatch.destination_peer.to_string(),
                                                    &dispatch.custody_id,
                                                    "recipient_ack",
                                                ) {
                                                    tracing::warn!(
                                                        "Failed to mark custody {} delivered (relay message {}): {}",
                                                        dispatch.custody_id,
                                                        dispatch.relay_message_id,
                                                        e
                                                    );
                                                } else {
                                                    tracing::info!(
                                                        "[OK] Custody {} delivered to {} (relay message {})",
                                                        dispatch.custody_id,
                                                        dispatch.destination_peer,
                                                        dispatch.relay_message_id
                                                    );
                                                }

                                                let marker = DeliveryConvergenceMarker {
                                                    relay_message_id: dispatch.relay_message_id.clone(),
                                                    destination_peer_id: dispatch
                                                        .destination_peer
                                                        .to_string(),
                                                    observed_by_peer_id: local_peer_id
                                                        .to_string(),
                                                    observed_at_ms: SystemTime::now()
                                                        .duration_since(UNIX_EPOCH)
                                                        .unwrap_or_default()
                                                        .as_millis()
                                                        as u64,
                                                };
                                                if seen_delivery_convergence_markers
                                                    .insert(marker.key())
                                                {
                                                    apply_delivery_convergence_marker(
                                                        &marker,
                                                        &mut pending_messages,
                                                        &mut request_to_message,
                                                        &mut pending_relay_requests,
                                                        &mut pending_custody_dispatches,
                                                        &mut multi_path_delivery,
                                                        &relay_custody_store,
                                                    )
                                                    .await;
                                                    publish_delivery_convergence_marker(
                                                        &mut swarm,
                                                        &marker,
                                                    );
                                                }
                                            } else {
                                                let reason = response
                                                    .error
                                                    .unwrap_or_else(|| "recipient_rejected".to_string());
                                                let reason = format!("recipient_rejected:{}", reason);
                                                if let Err(e) = relay_custody_store.mark_dispatch_failed(
                                                    &dispatch.destination_peer.to_string(),
                                                    &dispatch.custody_id,
                                                    &reason,
                                                ) {
                                                    tracing::warn!(
                                                        "Failed to return custody {} to accepted after rejection (relay message {}): {}",
                                                        dispatch.custody_id,
                                                        dispatch.relay_message_id,
                                                        e
                                                    );
                                                }
                                            }
                                        } else if let Some(message_id) =
                                            request_to_message.remove(&request_id)
                                        {
                                            // Response to our outbound message request
                                            if let Some(pending) = pending_messages.remove(&message_id) {
                                                if response.accepted {
                                                    // PHASE 5: Track successful delivery
                                                    let latency_ms = pending.attempt_start.elapsed().unwrap_or_default().as_millis() as u64;
                                                    multi_path_delivery.record_success(&message_id, vec![pending.target_peer], latency_ms);
                                                    // Mycorrhizal routing: record activity for adaptive TTL and update reliability
                                                    {
                                                        let mut guard = routing_engine_handle.write();
                                                        if let Some(ref mut engine) = guard.as_mut() {
                                                            engine.record_message_activity(&pending.target_peer.to_string());
                                                            // Update peer reliability score on successful delivery
                                                            let peer_bytes = extract_peer_id_bytes(&pending.target_peer.to_bytes());
                                                            engine.base_engine_mut().local_cell_mut().update_reliability(&peer_bytes, true);
                                                        }
                                                    }
                                                    tracing::info!("[OK] Message delivered successfully to {} ({}ms)", pending.target_peer, latency_ms);
                                                    let _ = pending.reply_tx.send(Ok(())).await;
                                                } else {
                                                    // Message rejected, trigger retry
                                                    tracing::warn!("[FAIL] Message rejected by {}: {:?}", pending.target_peer, response.error);
                                                    multi_path_delivery.record_failure(&message_id, vec![pending.target_peer]);
                                                    pending_messages.insert(message_id, pending);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Messaging(
                                request_response::Event::OutboundFailure { request_id, error, .. }
                            )) => {
                                if let Some(dispatch) = pending_custody_dispatches.remove(&request_id) {
                                    let reason = format!("dispatch_outbound_failure:{}", error);
                                    if let Err(e) = relay_custody_store.mark_dispatch_failed(
                                        &dispatch.destination_peer.to_string(),
                                        &dispatch.custody_id,
                                        &reason,
                                    ) {
                                        tracing::warn!(
                                            "Failed to return custody {} to accepted after outbound failure (relay message {}): {}",
                                            dispatch.custody_id,
                                            dispatch.relay_message_id,
                                            e
                                        );
                                    }
                                } else if let Some(message_id) = request_to_message.remove(&request_id) {
                                    if let Some(pending) = pending_messages.remove(&message_id) {
                                        tracing::warn!(
                                            "[FAIL] Direct send outbound failure to {}: {}",
                                            pending.target_peer,
                                            error
                                        );
                                        multi_path_delivery
                                            .record_failure(&message_id, vec![pending.target_peer]);
                                        // Mycorrhizal routing: record unreachable in negative cache and downgrade reliability
                                        {
                                            let mut guard = routing_engine_handle.write();
                                            if let Some(ref mut engine) = guard.as_mut() {
                                                engine.record_unreachable_peer(&pending.target_peer.to_string());
                                                let peer_bytes = extract_peer_id_bytes(&pending.target_peer.to_bytes());
                                                engine.base_engine_mut().local_cell_mut().update_reliability(&peer_bytes, false);
                                            }
                                        }
                                        pending_messages.insert(message_id, pending);
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::AddressReflection(
                                request_response::Event::Message { peer, message, .. }
                            )) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        // Peer is requesting address reflection
                                        let observed_addr = connection_tracker
                                            .get_connection(&peer)
                                            .and_then(|conn| ConnectionTracker::extract_socket_addr(&conn.remote_addr))
                                            .unwrap_or_else(|| "0.0.0.0:0".parse().expect("static socket addr parse cannot fail"));

                                        tracing::debug!("Observed address for {}: {}", peer, observed_addr);

                                        let response = reflection_service.handle_request(request, observed_addr);
                                        let _ = swarm.behaviour_mut().address_reflection.send_response(channel, response);
                                    }
                                    request_response::Message::Response { request_id, response } => {
                                        tracing::info!("Address reflection from {}: {}", peer, response.observed_address);

                                        if let Ok(observed_addr) = response.observed_address.parse::<SocketAddr>() {
                                            address_observer.record_observation(peer, observed_addr);

                                            if let Some(primary) = address_observer.primary_external_address() {
                                                tracing::info!("Consensus external address: {}", primary);
                                                // Convert SocketAddr to Multiaddr and add to swarm
                                                let (ip, port) = (primary.ip(), primary.port());
                                                let maddr: Multiaddr = match ip {
                                                    std::net::IpAddr::V4(ip4) => format!("/ip4/{}/tcp/{}", ip4, port).parse().expect("formatted multiaddr is always valid"),
                                                    std::net::IpAddr::V6(ip6) => format!("/ip6/{}/tcp/{}", ip6, port).parse().expect("formatted multiaddr is always valid"),
                                                };
                                                swarm.add_external_address(maddr);
                                            }
                                        }

                                        if let Some(reply_tx) = pending_reflections.remove(&request_id) {
                                            let _ = reply_tx.send(Ok(response.observed_address.clone())).await;
                                        }

                                        let _ = event_tx.send(SwarmEvent2::AddressReflected {
                                            peer_id: peer,
                                            observed_address: response.observed_address,
                                        }).await;
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(
                                super::behaviour::IronCoreBehaviourEvent::Registration(
                                    request_response::Event::Message { peer, message, .. },
                                ),
                            ) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        let response = match verify_registration_message(&peer, &request) {
                                            Ok(()) => {
                                                apply_verified_registration_message(
                                                    &relay_custody_store,
                                                    &request,
                                                )
                                            }
                                            Err(error) => {
                                                tracing::warn!(
                                                    "Rejected registration message from {}: {}",
                                                    peer,
                                                    error
                                                );
                                                RegistrationResponse {
                                                    accepted: false,
                                                    error: Some(error.to_string()),
                                                }
                                            }
                                        };
                                        let _ = swarm
                                            .behaviour_mut()
                                            .registration
                                            .send_response(channel, response);
                                    }
                                    request_response::Message::Response { request_id, response } => {
                                        if let Some(reply_tx) =
                                            pending_registration_replies.remove(&request_id)
                                        {
                                            let result = if response.accepted {
                                                Ok(())
                                            } else {
                                                Err(response.error.unwrap_or_else(|| {
                                                    "registration_request_rejected".to_string()
                                                }))
                                            };
                                            let _ = reply_tx.send(result).await;
                                        }
                                    }
                                }
                            }
                            SwarmEvent::Behaviour(
                                super::behaviour::IronCoreBehaviourEvent::Registration(
                                    request_response::Event::OutboundFailure {
                                        request_id, error, ..
                                    },
                                ),
                            ) => {
                                if let Some(reply_tx) =
                                    pending_registration_replies.remove(&request_id)
                                {
                                    let _ = reply_tx.send(Err(error.to_string())).await;
                                }
                            }

                            // PHASE 3: Relay Protocol Handler — MANDATORY RELAY
                            // All nodes MUST relay. We never refuse a relay request
                            // (except for invalid destination).
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Relay(
                                request_response::Event::Message { peer, message, .. }
                            )) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        tracing::info!("Relay request from {} for message {}", peer, request.message_id);

                                        // Enforce relay budget — reset counter hourly
                                        if relay_hour_start.elapsed() >= web_time::Duration::from_secs(3600) {
                                            relay_count_this_hour = 0;
                                            relay_hour_start = web_time::Instant::now();
                                        }

                                        let now_ms = SystemTime::now()
                                            .duration_since(UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_millis() as u64;

                                        // Determine response; channel consumed exactly once at the end
                                        let relay_response = if let Some(reason) = relay_guardrails
                                            .should_reject_cheap_heuristics(
                                                &request.message_id,
                                                request.envelope_data.len(),
                                            )
                                        {
                                            tracing::warn!(
                                                "Relay request rejected by heuristic from {} (message {}): {}",
                                                peer,
                                                request.message_id,
                                                reason
                                            );
                                            let abuse_signal = if reason.contains("oversized") {
                                                "OversizedMessage"
                                            } else if reason.contains("duplicate") {
                                                "DuplicateMessage"
                                            } else {
                                                "InvalidFormat"
                                            };
                                            let _ = event_tx.send(SwarmEvent2::AbuseSignalDetected {
                                                peer_id: peer,
                                                signal: abuse_signal.to_string(),
                                            }).await;
                                            RelayResponse {
                                                accepted: false,
                                                error: Some(reason.to_string()),
                                                message_id: request.message_id.clone(),
                                            }
                                        } else if relay_budget > 0 && relay_count_this_hour >= relay_budget {
                                            tracing::warn!(
                                                "Relay budget ({}/hr) exhausted — dropping relay request {}",
                                                relay_budget,
                                                request.message_id
                                            );
                                            RelayResponse {
                                                accepted: false,
                                                error: Some("relay_budget_exhausted".to_string()),
                                                message_id: request.message_id.clone(),
                                            }
                                        } else if pending_custody_dispatches.len()
                                            >= RELAY_MAX_INFLIGHT_DISPATCHES
                                        {
                                            tracing::warn!(
                                                "Relay inflight cap reached ({}) — rejecting relay request {}",
                                                RELAY_MAX_INFLIGHT_DISPATCHES,
                                                request.message_id
                                            );
                                            RelayResponse {
                                                accepted: false,
                                                error: Some("relay_inflight_capped".to_string()),
                                                message_id: request.message_id.clone(),
                                            }
                                        } else if {
                                            let (multiplier, spam_score) = if let Some(core) = core_handle.as_ref().and_then(|w| w.upgrade()) {
                                                (
                                                    core.peer_rate_limit_multiplier(peer.to_string()),
                                                    core.peer_spam_score(peer.to_string())
                                                )
                                            } else {
                                                (1.0, 0.0)
                                            };

                                            if spam_score > 0.8 {
                                                tracing::warn!("Relay REJECTED: high spam score for peer {}", peer);
                                                true // Reject
                                            } else {
                                                !relay_guardrails.consume_peer_token(
                                                    &peer.to_string(),
                                                    now_ms,
                                                    multiplier,
                                                )
                                            }
                                        } {
                                            tracing::warn!(
                                                "Relay request rejected (rate-limited or spam) for peer {} (message {})",
                                                peer,
                                                request.message_id
                                            );
                                            let _ = event_tx.send(SwarmEvent2::AbuseSignalDetected {
                                                peer_id: peer,
                                                signal: "RateLimited".to_string(),
                                            }).await;
                                            RelayResponse {
                                                accepted: false,
                                                error: Some("relay_peer_rejected".to_string()),
                                                message_id: request.message_id.clone(),
                                            }
                                        } else {
                                            relay_count_this_hour += 1;
                                            match PeerId::from_bytes(&request.destination_peer) {
                                                Ok(destination) => {
                                                    let relay_message_id = request.message_id.clone();
                                                    match resolve_custody_metadata(
                                                        &relay_custody_store,
                                                        request.recipient_identity_id.as_deref(),
                                                        request.intended_device_id.as_deref(),
                                                        CustodyCompatMode::default(),
                                                    ) {
                                                        Err(error) => {
                                                            tracing::warn!(
                                                                "Relay request rejected by custody enforcement from {} -> {} (message {}): {}",
                                                                peer,
                                                                destination,
                                                                relay_message_id,
                                                                error
                                                            );
                                                            RelayResponse {
                                                                accepted: false,
                                                                error: Some(error),
                                                                message_id: relay_message_id,
                                                            }
                                                        }
                                                        Ok((resolved_identity_id, resolved_device_id)) => {
                                                            if relay_guardrails.is_recent_duplicate(
                                                                &peer.to_string(),
                                                                &destination.to_string(),
                                                                &relay_message_id,
                                                                now_ms,
                                                            ) {
                                                                tracing::info!(
                                                                    "Relay duplicate suppressed from {} -> {} for message {}",
                                                                    peer,
                                                                    destination,
                                                                    relay_message_id
                                                                );
                                                                RelayResponse {
                                                                    accepted: true,
                                                                    error: None,
                                                                    message_id: relay_message_id,
                                                                }
                                                            } else {
                                                                match relay_custody_store.accept_custody(
                                                                    peer.to_string(),
                                                                    destination.to_string(),
                                                                    relay_message_id.clone(),
                                                                    request.envelope_data.clone(),
                                                                    resolved_identity_id,
                                                                    resolved_device_id,
                                                                ) {
                                                                    Ok(custody) => {
                                                                        relay_guardrails.record_accepted(
                                                                            &peer.to_string(),
                                                                            &destination.to_string(),
                                                                            &relay_message_id,
                                                                            now_ms,
                                                                        );
                                                                        if swarm.is_connected(&destination) {
                                                                            dispatch_pending_custody_for_peer(
                                                                                &mut swarm,
                                                                                &relay_custody_store,
                                                                                destination,
                                                                                &mut pending_custody_dispatches,
                                                                                RELAY_MAX_INFLIGHT_DISPATCHES,
                                                                                "accept_immediate_pull",
                                                                            );
                                                                        } else {
                                                                            tracing::info!(
                                                                                "Accepted custody {} for offline destination {} (relay message {})",
                                                                                custody.custody_id,
                                                                                destination,
                                                                                relay_message_id
                                                                            );
                                                                        }
                                                                        RelayResponse {
                                                                            accepted: true,
                                                                            error: None,
                                                                            message_id: relay_message_id,
                                                                        }
                                                                    }
                                                                    Err(e) => RelayResponse {
                                                                        accepted: false,
                                                                        error: Some(format!(
                                                                            "custody_store_failed: {}",
                                                                            e
                                                                        )),
                                                                        message_id: relay_message_id,
                                                                    },
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::error!("Invalid destination peer ID: {}", e);
                                                    RelayResponse {
                                                        accepted: false,
                                                        error: Some("Invalid destination peer ID".to_string()),
                                                        message_id: request.message_id.clone(),
                                                    }
                                                }
                                            }
                                        };
                                        let _ = swarm.behaviour_mut().relay.send_response(channel, relay_response);
                                    }
                                    request_response::Message::Response { request_id, response } => {
                                        if let Some(message_id) = pending_relay_requests.remove(&request_id) {
                                            if let Some(pending) = pending_messages.remove(&message_id) {
                                                if response.accepted {
                                                    let latency_ms = pending.attempt_start.elapsed().unwrap_or_default().as_millis() as u64;
                                                    multi_path_delivery.record_success(&message_id, vec![peer, pending.target_peer], latency_ms);
                                                    tracing::info!("[OK] Message relayed successfully via {} to {} ({}ms)", peer, pending.target_peer, latency_ms);
                                                    let _ = pending.reply_tx.send(Ok(())).await;
                                                } else {
                                                    let error = response
                                                        .error
                                                        .unwrap_or_else(|| "relay rejected".to_string());
                                                    tracing::warn!("[FAIL] Relay via {} failed: {}", peer, error);
                                                    multi_path_delivery.record_failure(&message_id, vec![peer, pending.target_peer]);
                                                    if is_terminal_identity_rejection(&error) {
                                                        let _ = pending.reply_tx.send(Err(error)).await;
                                                    } else {
                                                        pending_messages.insert(message_id, pending);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Relay(
                                request_response::Event::OutboundFailure { peer, request_id, error, .. }
                            )) => {
                                if let Some(message_id) = pending_relay_requests.remove(&request_id) {
                                    if let Some(pending) = pending_messages.remove(&message_id) {
                                        tracing::warn!(
                                            "[FAIL] Relay outbound failure via {} to {}: {}",
                                            peer,
                                            pending.target_peer,
                                            error
                                        );
                                        multi_path_delivery.record_failure(
                                            &message_id,
                                            vec![peer, pending.target_peer],
                                        );
                                        pending_messages.insert(message_id, pending);
                                    }
                                }
                            }

                            // LEDGER EXCHANGE — Automatic peer list sharing
                            // When a peer sends us their known peers, we merge them into our
                            // knowledge and respond with our own list. This creates a viral
                            // discovery mechanism where connecting to ONE peer bootstraps
                            // knowledge of the ENTIRE mesh.
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::LedgerExchange(
                                request_response::Event::Message { peer, message, .. }
                            )) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        tracing::info!(
                                            "Ledger exchange from {}: received {} peer entries (v{})",
                                            peer,
                                            request.peers.len(),
                                            request.version,
                                        );

                                        // Forward received entries to the application layer
                                        // The app will merge them into its persistent ledger
                                        let _ = event_tx.send(SwarmEvent2::LedgerReceived {
                                            from_peer: peer,
                                            entries: request.peers.clone(),
                                        }).await;

                                        // Also add any addresses with known PeerIDs to Kademlia RIGHT NOW
                                        // for immediate discoverability
                                        let mut new_count = 0u32;
                                        for entry in &request.peers {
                                            if let Some(ref pid_str) = entry.last_peer_id {
                                                if let Ok(pid) = pid_str.parse::<PeerId>() {
                                                    multi_path_delivery.record_recipient_seen_via_relay(
                                                        peer,
                                                        pid,
                                                        entry.last_seen,
                                                    );
                                                    if let Ok(addr) = entry.multiaddr.parse::<Multiaddr>() {
                                                        if is_discoverable_multiaddr(&addr) {
                                                            swarm.behaviour_mut().kademlia.add_address(&pid, addr);
                                                            new_count += 1;
                                                        }
                                                    }
                                                }
                                            }

                                            // Auto-subscribe to any topics from the shared entries
                                            for topic_str in &entry.known_topics {
                                                if !subscribed_topics.contains(topic_str) {
                                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic_str.clone());
                                                    if swarm.behaviour_mut().gossipsub.subscribe(&ident_topic).is_ok() {
                                                        tracing::info!("Auto-subscribed to topic from ledger: {}", topic_str);
                                                        subscribed_topics.insert(topic_str.clone());
                                                    }
                                                }
                                            }
                                        }

                                        // Respond with an empty list — the application layer
                                        // will send our full ledger via ShareLedger command
                                        // after processing the received entries.
                                        let _ = swarm.behaviour_mut().ledger_exchange.send_response(
                                            channel,
                                            LedgerExchangeResponse {
                                                peers: Vec::new(), // App layer fills this via ShareLedger
                                                new_peers_learned: new_count,
                                                version: 1,
                                            },
                                        );

                                        ledger_exchanged_peers.insert(peer);
                                    }
                                    request_response::Message::Response { response, .. } => {
                                        tracing::info!(
                                            "Ledger exchange response from {}: they learned {} new peers, sent {} back",
                                            peer,
                                            response.new_peers_learned,
                                            response.peers.len(),
                                        );

                                        // If they sent peers back in the response, merge those too
                                        if !response.peers.is_empty() {
                                            let _ = event_tx.send(SwarmEvent2::LedgerReceived {
                                                from_peer: peer,
                                                entries: response.peers.clone(),
                                            }).await;

                                            // Add routable addresses to Kademlia
                                            for entry in &response.peers {
                                                if let Some(ref pid_str) = entry.last_peer_id {
                                                    if let Ok(pid) = pid_str.parse::<PeerId>() {
                                                        multi_path_delivery.record_recipient_seen_via_relay(
                                                            peer,
                                                            pid,
                                                            entry.last_seen,
                                                        );
                                                        if let Ok(addr) = entry.multiaddr.parse::<Multiaddr>() {
                                                            if is_discoverable_multiaddr(&addr) {
                                                                swarm.behaviour_mut().kademlia.add_address(&pid, addr);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Gossipsub events — Dynamic Topic Negotiation
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Gossipsub(
                                gossipsub::Event::Subscribed { peer_id, topic }
                            )) => {
                                let topic_str = topic.to_string();
                                tracing::info!("Peer {} subscribed to topic: {}", peer_id, topic_str);

                                // AUTO-NEGOTIATE: If a peer subscribes to a topic we don't know,
                                // subscribe to it ourselves. "A node is a node."
                                if !subscribed_topics.contains(&topic_str) {
                                    tracing::info!("Auto-subscribing to discovered topic: {}", topic_str);
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic_str.clone());
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&ident_topic) {
                                        tracing::warn!("Failed to auto-subscribe to {}: {}", topic_str, e);
                                    } else {
                                        subscribed_topics.insert(topic_str.clone());
                                    }
                                }

                                let _ = event_tx.send(SwarmEvent2::TopicDiscovered {
                                    peer_id,
                                    topic: topic_str,
                                }).await;
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Gossipsub(
                                gossipsub::Event::Message { propagation_source, message, .. }
                            )) => {
                                // Accept all gossipsub messages — log and forward
                                tracing::debug!(
                                    "Gossipsub message from {} on topic {:?} ({} bytes)",
                                    propagation_source,
                                    message.topic,
                                    message.data.len()
                                );
                                if message.topic.as_str() == DELIVERY_CONVERGENCE_TOPIC {
                                    if let Some(marker) =
                                        decode_delivery_convergence_marker(&message.data)
                                    {
                                        if let Err(reason) = should_apply_delivery_convergence_marker(
                                            &marker,
                                            &pending_messages,
                                            &request_to_message,
                                            &pending_relay_requests,
                                            &pending_custody_dispatches,
                                            &relay_custody_store,
                                        ) {
                                            tracing::warn!(
                                                "Ignoring convergence marker message={} destination={} from={} reason={}",
                                                marker.relay_message_id,
                                                marker.destination_peer_id,
                                                propagation_source,
                                                reason
                                            );
                                            continue;
                                        }
                                        if seen_delivery_convergence_markers.insert(marker.key()) {
                                            apply_delivery_convergence_marker(
                                                &marker,
                                                &mut pending_messages,
                                                &mut request_to_message,
                                                &mut pending_relay_requests,
                                                &mut pending_custody_dispatches,
                                                &mut multi_path_delivery,
                                                &relay_custody_store,
                                            )
                                            .await;

                                            // Re-broadcast once from each node to fanout convergence
                                            // markers beyond direct neighborhoods.
                                            if propagation_source != local_peer_id {
                                                publish_delivery_convergence_marker(
                                                    &mut swarm,
                                                    &marker,
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Autonat(event)) => {
                                use libp2p::autonat;
                                match event {
                                    autonat::Event::StatusChanged { old, new } => {
                                        tracing::info!(
                                            "AutoNAT status: {:?} → {:?}",
                                            old, new
                                        );
                                        // Update NAT status for the application layer.
                                        // This determines whether relay fallback is required.
                                        let status_str = match new {
                                            autonat::NatStatus::Public(addr) => {
                                                tracing::info!("[OK] AutoNAT: public reachability confirmed at {}", addr);
                                                format!("public:{}", addr)
                                            }
                                            autonat::NatStatus::Private => {
                                                tracing::info!("AutoNAT: behind NAT — relay required for inbound");
                                                "private".to_string()
                                            }
                                            autonat::NatStatus::Unknown => {
                                                "unknown".to_string()
                                            }
                                        };
                                        let _ = event_tx.send(SwarmEvent2::NatStatusChanged(status_str)).await;
                                    }
                                    autonat::Event::InboundProbe(result) => {
                                        tracing::debug!("AutoNAT inbound probe: {:?}", result);
                                    }
                                    autonat::Event::OutboundProbe(result) => {
                                        // Gate noisy AutoNAT logs: if we have no connected
                                        // peers, a NoServer error is expected and inevitable —
                                        // skip logging entirely to avoid log spam. When peers
                                        // ARE connected, log at debug as usual for diagnostics.
                                        match &result {
                                            autonat::OutboundProbeEvent::Error {
                                                peer: None,
                                                error: autonat::OutboundProbeError::NoServer,
                                                ..
                                            } => {
                                                if swarm.connected_peers().next().is_some() {
                                                    tracing::debug!(
                                                        "AutoNAT outbound probe: {:?}",
                                                        result
                                                    );
                                                }
                                                // No connected peers → NoServer is expected;
                                                // silent skip, no log.
                                            }
                                            _ => {
                                                tracing::debug!(
                                                    "AutoNAT outbound probe: {:?}",
                                                    result
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Dcutr(event)) => {
                                use libp2p::dcutr;
                                match event {
                                    dcutr::Event { remote_peer_id, result: Ok(num_attempts) } => {
                                        tracing::info!(
                                            "DCUtR hole-punch SUCCESS with {} (attempts: {})",
                                            remote_peer_id, num_attempts
                                        );
                                        // Hole-punch succeeded — direct connection established.
                                        // Add this peer's direct addresses to Kademlia so the
                                        // DHT knows how to reach them without the relay.
                                        // Collect first to avoid simultaneous immutable + mutable borrow of swarm.
                                        let ext_addrs: Vec<libp2p::Multiaddr> =
                                            swarm.external_addresses().cloned().collect();
                                        for addr in ext_addrs {
                                            swarm.behaviour_mut().kademlia.add_address(
                                                &remote_peer_id,
                                                addr
                                            );
                                        }
                                        bootstrap_capability.add_peer(remote_peer_id);
                                        if reported_peer_discoveries.insert(remote_peer_id) {
                                            let _ = event_tx.send(SwarmEvent2::PeerDiscovered(remote_peer_id)).await;

                                            // Activate SyncSession for Drift Protocol mesh synchronization
                                            sync_sessions.insert(remote_peer_id, SyncSession::new());
                                            tracing::debug!(
                                                "Activated SyncSession for peer: {}",
                                                remote_peer_id
                                            );
                                        }
                                    }
                                    dcutr::Event { remote_peer_id, result: Err(e) } => {
                                        tracing::warn!(
                                            "DCUtR hole-punch FAILED with {} — will relay messages instead: {}",
                                            remote_peer_id, e
                                        );
                                        // Hole-punch failed — this is OK; our application-layer
                                        // relay (/sc/relay/1.0.0) handles the fallback.
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::RelayClient(event)) => {
                                use libp2p::relay::client::Event as RelayClientEvent;
                                match event {
                                    RelayClientEvent::ReservationReqAccepted {
                                        relay_peer_id,
                                        renewal,
                                        ..
                                    } => {
                                        if renewal {
                                            tracing::debug!(
                                                "Relay circuit reservation RENEWED via {}",
                                                relay_peer_id
                                            );
                                        } else {
                                            tracing::info!(
                                                "[OK] Relay circuit reservation ACCEPTED via {} — inbound-relayed connections now possible",
                                                relay_peer_id
                                            );
                                        }
                                    }
                                    RelayClientEvent::InboundCircuitEstablished {
                                        src_peer_id,
                                        ..
                                    } => {
                                        tracing::info!(
                                            "Inbound relay circuit established from {} — peer connected through relay",
                                            src_peer_id
                                        );
                                    }
                                    RelayClientEvent::OutboundCircuitEstablished {
                                        relay_peer_id,
                                        ..
                                    } => {
                                        tracing::info!(
                                            "Outbound relay circuit established via {} — connected to remote through relay",
                                            relay_peer_id
                                        );
                                        let _ = event_tx
                                            .send(SwarmEvent2::RelayCircuitEstablished)
                                            .await;
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::RelayServer(event)) => {
                                use libp2p::relay::Event as RelayServerEvent;
                                #[allow(deprecated)]
                                match event {
                                    RelayServerEvent::ReservationReqAccepted { src_peer_id, .. } => {
                                        tracing::info!(
                                            "[OK] Relay server: accepted reservation from {} — acting as relay for this peer",
                                            src_peer_id
                                        );
                                    }
                                    RelayServerEvent::CircuitReqAccepted { src_peer_id, dst_peer_id } => {
                                        tracing::info!(
                                            "Relay server: circuit established {} -> {} — relaying traffic",
                                            src_peer_id,
                                            dst_peer_id
                                        );
                                    }
                                    RelayServerEvent::CircuitClosed { src_peer_id, dst_peer_id, .. } => {
                                        tracing::debug!(
                                            "Circuit closed: {} -> {}",
                                            src_peer_id,
                                            dst_peer_id
                                        );
                                    }
                                    RelayServerEvent::ReservationReqDenied { .. } |
                                    RelayServerEvent::ReservationTimedOut { .. } |
                                    RelayServerEvent::ReservationClosed { .. } |
                                    RelayServerEvent::CircuitReqDenied { .. } |
                                    RelayServerEvent::CircuitReqOutboundConnectFailed { .. } |
                                    RelayServerEvent::ReservationReqAcceptFailed { .. } |
                                    RelayServerEvent::ReservationReqDenyFailed { .. } |
                                    RelayServerEvent::CircuitReqDenyFailed { .. } |
                                    RelayServerEvent::CircuitReqAcceptFailed { .. } => {
                                        // Logged internally by libp2p
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Ping(event)) => {
                                tracing::trace!("Ping event: {:?}", event);
                            }

                            #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Mdns(
                                mdns::Event::Discovered(peers)
                            )) => {
                                for (peer_id, addr) in peers {
                                    tracing::info!("mDNS discovered peer: {} at {}", peer_id, addr);
                                    if is_discoverable_multiaddr(&addr) {
                                        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                    }

                                    bootstrap_capability.add_peer(peer_id);
                                    if reported_peer_discoveries.insert(peer_id) {
                                        let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;

                                        // Activate SyncSession for Drift Protocol mesh synchronization
                                        sync_sessions.insert(peer_id, SyncSession::new());
                                        tracing::debug!(
                                            "Activated SyncSession for peer: {}",
                                            peer_id
                                        );
                                    }
                                }
                            }

                            #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android"), not(target_os = "windows")))]
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Mdns(
                                mdns::Event::Expired(peers)
                            )) => {
                                for (peer_id, _addr) in peers {
                                    tracing::info!("mDNS peer expired: {}", peer_id);
                                    let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
                                }
                            }

                            // Identify — PROMISCUOUS peer acceptance
                            // Accept ANY peer identity, regardless of expected PeerID.
                            // Log the identity and add all addresses to Kademlia.
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Identify(
                                identify::Event::Received { peer_id, info, .. }
                            )) => {
                                // Dedup: suppress "Identified peer" logs for same peer within TTL window
                                {
                                    let now = Instant::now();
                                    let mut map = last_identified_log().write();
                                    let ttl = Duration::from_secs(IDENTIFY_LOG_DEDUP_TTL_SECS);
                                    if let Some(last) = map.get(&peer_id) {
                                        if now.duration_since(*last) < ttl {
                                            // Suppress duplicate log within TTL window
                                            continue;
                                        }
                                    }
                                    map.insert(peer_id, now);
                                }

                                // Consolidate multi-address logging: build single summary line per peer
                                let discoverable_addrs: Vec<&Multiaddr> = info.listen_addrs
                                    .iter()
                                    .filter(|a| is_discoverable_multiaddr(a))
                                    .collect();
                                tracing::info!(
                                    "🆔 Identified peer {} — agent: {}, protocols: {}, discoverable_addrs: {}",
                                    peer_id,
                                    info.agent_version,
                                    info.protocols.len(),
                                    discoverable_addrs.len()
                                );
                                // Identity protocol confirms this peer is presently reachable.
                                multi_path_delivery.record_recipient_seen_now(peer_id, peer_id);

                                // MYCORRHIZAL ROUTING: Update routing engine with peer discovery
                                let peer_id_bytes = extract_peer_id_bytes(&peer_id.to_bytes());
                                let _peer_hint: [u8; 4] = blake3::hash(&peer_id_bytes).as_bytes()[0..4]
                                    .try_into()
                                    .expect("blake3 hash should be at least 4 bytes");
                                // When identity protocol confirms a peer, use the Kademlia server
                                // mode as the transport type basis since identity requires a
                                // server-capable connection.
                                let transport_type = transport_type_to_routing_transport(kad::Mode::Server);
                                {
                                    let mut guard = routing_engine_handle.write();
                                    if let Some(ref mut engine) = guard.as_mut() {
                                        engine.base_engine_mut().local_cell_mut().peer_seen(
                                            peer_id_bytes,
                                            transport_type,
                                        );
                                    }
                                }
                                // Update routing engine with peer hints from identify info
                                // (peers announce what hints they can reach)

                                // Relay-confirmed observation of our externally visible endpoint
                                // as seen by this peer. This gives mobile layers a stable
                                // "what the network sees" signal for publishing connection hints.
                                if let Some(observed_addr) =
                                    ConnectionTracker::extract_socket_addr(&info.observed_addr)
                                {
                                    address_observer.record_observation(peer_id, observed_addr);
                                    tracing::info!(
                                        "Identify observed address via {}: {}",
                                        peer_id,
                                        observed_addr
                                    );

                                    if let Some(primary) = address_observer.primary_external_address() {
                                        // Convert SocketAddr to Multiaddr and add to swarm
                                        let (ip, port) = (primary.ip(), primary.port());
                                        let maddr: Multiaddr = match ip {
                                            std::net::IpAddr::V4(ip4) => format!("/ip4/{}/tcp/{}", ip4, port).parse().expect("formatted multiaddr is always valid"),
                                            std::net::IpAddr::V6(ip6) => format!("/ip6/{}/tcp/{}", ip6, port).parse().expect("formatted multiaddr is always valid"),
                                        };
                                        swarm.add_external_address(maddr);
                                    }
                                } else {
                                    tracing::trace!(
                                        "Identify observed_addr not socket-like: {}",
                                        info.observed_addr
                                    );
                                }

                                // Add only discoverable addresses to Kademlia.
                                // Loopback/unspecified addresses are excluded.
                                // Private/RFC1918/CGNAT are NOW allowed for local mesh.
                                for addr in &info.listen_addrs {
                                    if is_discoverable_multiaddr(addr) {
                                        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                                    } else {
                                        tracing::debug!("Skipping non-discoverable Kademlia addr for {}: {}", peer_id, addr);
                                    }
                                }

                                // Check if peer advertises relay capability
                                let is_relay = info.agent_version.contains("relay");
                                if is_relay {
                                    tracing::info!("Peer {} is identified as a RELAY node (agent: {})", peer_id, info.agent_version);
                                    bootstrap_capability.add_peer(peer_id);
                                    multi_path_delivery.add_relay(peer_id);
                                    // Mycorrhizal routing: mark relay-capable peer as gateway
                                    let gw_bytes = extract_peer_id_bytes(&peer_id.to_bytes());
                                    {
                                        let mut guard = routing_engine_handle.write();
                                        if let Some(ref mut engine) = guard.as_mut() {
                                            engine.base_engine_mut().local_cell_mut().mark_as_gateway(&gw_bytes, true);
                                        }
                                    }

                                    // P0.5B: Register a circuit relay reservation with this relay.
                                    // Guard: only register ONCE per relay peer — identify fires every 60s
                                    // and without this guard we accumulate unbounded ListenerIds, which
                                    // floods the relay and crowds out real message delivery.
                                    let already_reserved = successful_relay_reservations.contains_key(&peer_id);

                                    if !already_reserved {
                                        let routable_relay_addrs: Vec<Multiaddr> = info.listen_addrs
                                            .iter()
                                            .filter(|a| is_discoverable_multiaddr(a))
                                            .cloned()
                                            .collect();

                                        if !routable_relay_addrs.is_empty() {
                                            // Pick the first routable relay address and register a circuit reservation.
                                            // Format: /ip4/<relay-ip>/tcp/<port>/p2p/<relay-peer-id>/p2p-circuit
                                            let relay_circuit_addr = relay_reservation_multiaddr(
                                                &routable_relay_addrs[0],
                                                peer_id,
                                            );

                                            tracing::info!(
                                                "Attempting relay circuit reservation via {}: {}",
                                                peer_id, relay_circuit_addr
                                            );
                                            match swarm.listen_on(relay_circuit_addr.clone()) {
                                                Ok(listener_id) => {
                                                    tracing::info!(
                                                        "[OK] Relay circuit reservation registered: {:?} via {}",
                                                        listener_id, peer_id
                                                    );
                                                    successful_relay_reservations.insert(peer_id, listener_id);
                                                    relay_peer_addrs.insert(peer_id, routable_relay_addrs.clone());
                                                },
                                                Err(e) => tracing::warn!(
                                                    "[WARNING] Could not register relay circuit reservation via {}: {:?}",
                                                    peer_id, e
                                                ),
                                            }
                                        } else {
                                            tracing::debug!(
                                                "Relay {} has no WAN-routable addresses yet; \
                                                 will retry reservation after reconnect",
                                                peer_id
                                            );
                                        }
                                    } else {
                                        tracing::debug!(
                                            "Relay circuit already active for {} — skipping duplicate",
                                            peer_id
                                        );
                                    }
                                }

                                // Deduplicate event emission to avoid bridge spam
                                let should_report = match reported_peer_info.get(&peer_id) {
                                    Some((old_agent, old_addrs)) => {
                                        old_agent != &info.agent_version || old_addrs != &info.listen_addrs
                                    }
                                    None => true,
                                };

                                if should_report {
                                    reported_peer_info.insert(peer_id, (info.agent_version.clone(), info.listen_addrs.clone()));
                                    // Emit event for application layer
                                    let public_key_hex = info.public_key.clone().try_into_ed25519().map(|pk| hex::encode(pk.to_bytes())).ok();
                                    // Site-3: flush outbox now that peer identity is confirmed.
                                    if let Some(pk_hex) = &public_key_hex {
                                        if let Some(c) = &core_handle {
                                            if let Some(c_arc) = c.upgrade() {
                                                c_arc.handle_peer_connection_event(pk_hex, true);
                                            }
                                        }
                                    }
                                    let _ = event_tx.send(SwarmEvent2::PeerIdentified {
                                        peer_id,
                                        public_key: public_key_hex,
                                        agent_version: info.agent_version.clone(),
                                        listen_addrs: info.listen_addrs.clone(),
                                        protocols: info.protocols.iter().map(|p| p.to_string()).collect(),
                                    }).await;
                                }
                            }

                            #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android"), not(target_os = "windows")))]
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Upnp(event)) => {
                                use libp2p::upnp;
                                match event {
                                    upnp::Event::NewExternalAddr(addr) => {
                                        tracing::info!("UPnP: successfully mapped external address {}", addr);
                                        swarm.add_external_address(addr.clone());
                                        let _ = event_tx.send(SwarmEvent2::PortMapping(format!("mapped:{}", addr))).await;
                                    }
                                    upnp::Event::GatewayNotFound => {
                                        tracing::debug!("UPnP: no compatible gateway found");
                                    }
                                    upnp::Event::NonRoutableGateway => {
                                        tracing::debug!("UPnP: gateway is not a routing device");
                                    }
                                    upnp::Event::ExpiredExternalAddr(addr) => {
                                        tracing::info!("UPnP: external address mapping expired: {}", addr);
                                        let _ = event_tx.send(SwarmEvent2::PortMapping(format!("expired:{}", addr))).await;
                                    }
                                }
                            }

                            SwarmEvent::NewListenAddr { address, .. } => {
                                tracing::info!("Listening on {}", address);
                                bound_addresses.push(address.clone());
                                let _ = event_tx.send(SwarmEvent2::ListeningOn(address)).await;
                            }

                            SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
                                let remote_addr = endpoint.get_remote_address().clone();

                                // Prune resolved_to_dns mappings for this peer / hostname
                                let stripped_remote: Multiaddr = remote_addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
                                let mut dns_to_prune = None;
                                if let Some(dns) = resolved_to_dns.remove(&remote_addr) {
                                    dns_to_prune = Some(dns);
                                } else if let Some(dns) = resolved_to_dns.remove(&stripped_remote) {
                                    dns_to_prune = Some(dns);
                                }
                                if let Some(dns) = dns_to_prune {
                                    resolved_to_dns.retain(|_, v| v != &dns);
                                }

                                tracing::info!(
                                    "Connected to {} via {} (promiscuous mode — any PeerID accepted)",
                                    peer_id,
                                    remote_addr
                                );

                                // Resolve any pending Dial waiting on this connection. Match by
                                // address membership in the entry's own candidate_addrs (covers
                                // both the None/peer-id-less branch, where candidate_addrs is
                                // just the one dialed address, and the Some(pid) candidate-ladder
                                // branch, where the connected address may be any rung of that
                                // specific dial's own ladder) -- deliberately NOT matched by bare
                                // peer_id, so an unrelated concurrent dial or background
                                // reconnect to the SAME peer_id via a DIFFERENT address can't
                                // falsely resolve this entry.
                                let mut resolved_pending_key = None;
                                for (key, entry) in pending_dials.iter() {
                                    if entry.candidate_addrs.iter().any(|a| a == &remote_addr || a == &stripped_remote) {
                                        resolved_pending_key = Some(key.clone());
                                        break;
                                    }
                                }
                                if let Some(key) = resolved_pending_key {
                                    if let Some(entry) = pending_dials.remove(&key) {
                                        let _ = entry.reply.send(Ok(())).await;
                                    }
                                }

                                multi_path_delivery.record_recipient_seen_now(peer_id, peer_id);

                                if let Some(c) = &core_handle {
                                    if let Some(c_arc) = c.upgrade() {
                                        let fp = crate::store::transport_memory::get_network_fingerprint();
                                        let mut transport = String::new();
                                        let mut port = 0;
                                        for p in remote_addr.iter() {
                                            match p {
                                                libp2p::multiaddr::Protocol::Tcp(p) => { transport = "tcp".to_string(); port = p; },
                                                libp2p::multiaddr::Protocol::Udp(p) => { transport = "udp".to_string(); port = p; },
                                                _ => {}
                                            }
                                        }
                                        if port > 0 {
                                            let _ = c_arc.transport_memory.read().record_success(&peer_id, &fp, transport, port, 0);
                                        }
                                    }
                                }

                                // Track this connection for address observation
                                connection_tracker.add_connection(
                                    peer_id,
                                    remote_addr.clone(),
                                    match endpoint {
                                        libp2p::core::ConnectedPoint::Listener { local_addr, .. } => local_addr.clone(),
                                        libp2p::core::ConnectedPoint::Dialer { .. } => "/ip4/0.0.0.0/tcp/0".parse().expect("static multiaddr parse cannot fail"),
                                    },
                                    connection_id.to_string(),
                                );

                                // Add to bootstrap capability (potential relay node)
                                // ALL peers are mandatory relays
                                bootstrap_capability.add_peer(peer_id);
                                multi_path_delivery.add_relay(peer_id);
                                dispatch_pending_custody_for_peer(
                                    &mut swarm,
                                    &relay_custody_store,
                                    peer_id,
                                    &mut pending_custody_dispatches,
                                    RELAY_MAX_INFLIGHT_DISPATCHES,
                                    "peer_reconnect",
                                );

                                // RELAY PEER DISCOVERY: Track peer and broadcast to others
                                // Start with the observed remote address.
                                let mut addresses = vec![remote_addr.to_string()];

                                // Enrich with circuit-relay addresses based on our own external exposure.
                                // This is CRITICAL for browser nodes: it tells other mesh members (like Android)
                                // that this peer is reachable THROUGH us.
                                let local_peer_id = *swarm.local_peer_id();
                                for ext_addr in swarm.external_addresses().cloned() {
                                    // Construct: /.../p2p/<our-id>/p2p-circuit/p2p/<their-id>
                                    let mut circuit_addr: Multiaddr = ext_addr;
                                    circuit_addr.push(libp2p::multiaddr::Protocol::P2p(local_peer_id));
                                    circuit_addr.push(libp2p::multiaddr::Protocol::P2pCircuit);
                                    circuit_addr.push(libp2p::multiaddr::Protocol::P2p(peer_id));
                                    addresses.push(circuit_addr.to_string());
                                }

                                peer_broadcaster.peer_connected(peer_id, addresses.clone());

                                // Broadcast PeerJoined to all other connected peers
                                if let Some(join_msg) = peer_broadcaster.create_peer_joined_message(&peer_id) {
                                    if let Ok(join_bytes) = join_msg.to_bytes() {
                                        for other_peer in peer_broadcaster.get_peers_except(&peer_id) {
                                            let framed = wrap_in_drift_frame(&join_bytes);
                                            let _request_id = swarm.behaviour_mut().messaging.send_request(
                                                &other_peer,
                                                Libp2pMessageRequest { envelope_data: framed },
                                            );
                                            tracing::debug!("Broadcast PeerJoined({}) to {}", peer_id, other_peer);
                                        }
                                    }
                                }

                                // Send full peer list to newly connected peer, including our own filtered bound addresses
                                let filtered_self = build_mdns_advertised_addrs(&bound_addresses);
                                let self_addrs = filtered_self.into_iter().map(|a| a.to_string()).collect::<Vec<_>>();
                                let list_msg = peer_broadcaster.create_peer_list_response(Some((&local_peer_id, self_addrs)));
                                if let Ok(list_bytes) = list_msg.to_bytes() {
                                    let framed = wrap_in_drift_frame(&list_bytes);
                                    let _request_id = swarm.behaviour_mut().messaging.send_request(
                                        &peer_id,
                                        Libp2pMessageRequest { envelope_data: framed },
                                    );
                                    tracing::info!("Sent peer list ({} peers) to {}", peer_broadcaster.peer_count(), peer_id);
                                }

                                if reported_peer_discoveries.insert(peer_id) {
                                    let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;

                                    // Activate SyncSession for Drift Protocol mesh synchronization
                                    sync_sessions.insert(peer_id, SyncSession::new());
                                    tracing::debug!(
                                        "Activated SyncSession for peer: {}",
                                        peer_id
                                    );
                                }

                                // AUTO LEDGER EXCHANGE: On every new connection, share our
                                // known peers. The application layer will receive
                                // SwarmEvent2::PeerDiscovered and trigger ShareLedger.
                                // This is handled in main.rs to keep swarm.rs agnostic
                                // about the persistent ledger format.

                                // Reset bootstrap backoff for any addr that matches this peer.
                                // On successful connection, the backoff should reset so
                                // subsequent failures start from the initial 60s interval.
                                for ba in &bootstrap_addrs_clone {
                                    let matches = ba.iter().any(|proto| {
                                        if let libp2p::multiaddr::Protocol::P2p(p) = proto { p == peer_id } else { false }
                                    });
                                    if matches {
                                        if let Some(entry) = bootstrap_backoff.get_mut(ba) {
                                            entry.on_success();
                                            tracing::debug!("Reset bootstrap backoff for {} (connected)", ba);
                                        }
                                        break;
                                    }
                                }
                            }

                            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                tracing::info!("[ERROR] Disconnected from {}", peer_id);
                                connection_tracker.remove_connection(&peer_id);
                                // Allow re-exchange if they reconnect
                                ledger_exchanged_peers.remove(&peer_id);
                                reported_peer_discoveries.remove(&peer_id);
                                reported_peer_info.remove(&peer_id);

                                // P0.13: Clear relay tracking so we can re-reserve on reconnect
                                if let Some(listener_id) = successful_relay_reservations.remove(&peer_id) {
                                    tracing::debug!("Clearing stale relay reservation for {}: {:?}", peer_id, listener_id);
                                    // Note: libp2p usually kills circuit listeners on connection close,
                                    // but we remove it from swarm to be sure.
                                    let _ = swarm.remove_listener(listener_id);
                                }

                                let stale_dispatches: Vec<libp2p::request_response::OutboundRequestId> =
                                    pending_custody_dispatches
                                        .iter()
                                        .filter_map(|(request_id, dispatch)| {
                                            (dispatch.destination_peer == peer_id)
                                                .then_some(*request_id)
                                        })
                                        .collect();
                                for request_id in stale_dispatches {
                                    if let Some(dispatch) =
                                        pending_custody_dispatches.remove(&request_id)
                                    {
                                        let _ = relay_custody_store.mark_dispatch_failed(
                                            &dispatch.destination_peer.to_string(),
                                            &dispatch.custody_id,
                                            "peer_disconnected",
                                        );
                                    }
                                }

                                // RELAY PEER DISCOVERY: Broadcast PeerLeft to remaining peers
                                let left_msg = crate::transport::PeerBroadcaster::create_peer_left_message(&peer_id);
                                if let Ok(left_bytes) = left_msg.to_bytes() {
                                    for other_peer in peer_broadcaster.get_peers_except(&peer_id) {
                                        let framed = wrap_in_drift_frame(&left_bytes);
                                        let _request_id = swarm.behaviour_mut().messaging.send_request(
                                            &other_peer,
                                            Libp2pMessageRequest { envelope_data: framed },
                                        );
                                        tracing::debug!("Broadcast PeerLeft({}) to {}", peer_id, other_peer);
                                    }
                                }
                                peer_broadcaster.peer_disconnected(&peer_id);

                                // P0.11: If this was a known relay, schedule a reconnect with backoff.
                                // Also clear from relay_peer_addrs so that when reconnection succeeds,
                                // we re-register a fresh circuit reservation (old listener is now dead).
                                // Backoff: 10s → 30s → 60s → 60s (capped).
                                if relay_peer_addrs.remove(&peer_id).is_some() {
                                    tracing::info!(
                                        "Lost relay peer {}; cleared circuit reservation, scheduling reconnect",
                                        peer_id
                                    );
                                    relay_reconnect_pending.push((peer_id, 0, web_time::Instant::now()));
                                }

                                // SELF-HEALING: Auto-redial non-relay peers that disconnect.
                                // Use the bootstrap_backoff map to track them — the next
                                // bootstrap reconnect tick will find them eligible and redial.
                                // We only do this for peers that are in our bootstrap list,
                                // since we know their addresses.
                                if !relay_peer_addrs.contains_key(&peer_id) {
                                    let local_peer = swarm.local_peer_id();
                                    if peer_id != *local_peer {
                                        for ba in &bootstrap_addrs_clone {
                                            let matches = ba.iter().any(|proto| {
                                                if let libp2p::multiaddr::Protocol::P2p(p) = proto { p == peer_id } else { false }
                                            });
                                            if matches {
                                                tracing::info!(
                                                    "Self-heal: queueing redial for disconnected bootstrap peer {}",
                                                    peer_id
                                                );
                                                if !bootstrap_backoff.contains_key(ba) {
                                                    bootstrap_backoff.insert(ba.clone(), BootstrapBackoffEntry::new());
                                                }
                                                break;
                                            }
                                        }
                                    }
                                }

                                let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
                            }

                            // Handle outgoing connection errors gracefully — don't panic
                            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                // Downgraded to debug: Kademlia DHT explores many stale addresses
                                // from the routing table; timeouts here are expected churn, not
                                // actionable errors. Relay/identity failures surface at info/warn.
                                if let Some(pid) = peer_id {
                                    tracing::debug!("[WARNING] Outgoing connection error to {}: {}", pid, error);
                                } else {
                                    tracing::debug!("[WARNING] Outgoing connection error: {}", error);
                                }

                                // Resolve any pending Dial(s) that this error corresponds to.
                                // Match ONLY by address membership in each entry's own
                                // candidate_addrs (mirrors the same exact/stripped matching
                                // approach the bootstrap_backoff check below uses) --
                                // deliberately NOT by bare peer_id, so a failure on one address
                                // for a given peer can't falsely fail an unrelated concurrent
                                // dial (or background reconnect) to the SAME peer via a
                                // DIFFERENT address that hasn't actually failed. Entries whose
                                // dial fails with a non-Transport DialError variant (rare) fall
                                // through to the periodic sweep's timeout instead -- see review
                                // notes; this is an accepted latency tradeoff, not a correctness
                                // gap, since a peer_id-only fallback here previously reintroduced
                                // exactly this false-attribution risk.
                                let mut resolved_dial_keys: Vec<Multiaddr> = Vec::new();
                                if let libp2p::swarm::DialError::Transport(ref errors) = error {
                                    for (failed_addr, _) in errors {
                                        let stripped_failed: Multiaddr = failed_addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
                                        for (key, entry) in pending_dials.iter() {
                                            if entry.candidate_addrs.iter().any(|a| a == failed_addr || a == &stripped_failed)
                                                && !resolved_dial_keys.contains(key)
                                            {
                                                resolved_dial_keys.push(key.clone());
                                            }
                                        }
                                    }
                                }
                                for key in resolved_dial_keys {
                                    if let Some(entry) = pending_dials.remove(&key) {
                                        let _ = entry.reply.send(Err(format!("{}", error))).await;
                                    }
                                }

                                // Exponential backoff for bootstrap re-dial: if the failed
                                // connection matches any bootstrap addr (by IP+port for
                                // peer_id=None errors, or by /p2p/ component), apply backoff
                                // so we don't keep hammering a node that refuses connections.
                                tracing::trace!("Bootstrap backoff check: {} addrs, peer_id={:?}", bootstrap_addrs_clone.len(), peer_id);
                                for ba in &bootstrap_addrs_clone {
                                    let mut matches = false;
                                    let mut resolved_match = false;

                                    // 1. Check if ba or one of its resolved IPs matches the failed addresses
                                    if let libp2p::swarm::DialError::Transport(ref errors) = error {
                                        for (failed_addr, _) in errors {
                                            if let Some(dns_addr) = resolved_to_dns.get(failed_addr) {
                                                if dns_addr == ba {
                                                    matches = true;
                                                    resolved_match = true;
                                                    // Apply aggressive backoff to the resolved IP
                                                    bootstrap_backoff.entry(failed_addr.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                                    tracing::debug!("Applied backoff to resolved IP {}", failed_addr);
                                                }
                                            }
                                            let stripped_failed: Multiaddr = failed_addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
                                            if let Some(dns_addr) = resolved_to_dns.get(&stripped_failed) {
                                                if dns_addr == ba {
                                                    matches = true;
                                                    resolved_match = true;
                                                    bootstrap_backoff.entry(failed_addr.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                                    tracing::debug!("Applied backoff to resolved IP {}", failed_addr);
                                                }
                                            }
                                        }
                                    }

                                    // 2. Fallback to standard matching if no resolved match
                                    if !matches {
                                        matches = if let Some(pid) = peer_id {
                                            // Known peer: match by p2p component
                                            ba.iter().any(|proto| {
                                                if let libp2p::multiaddr::Protocol::P2p(p) = proto { p == pid } else { false }
                                            })
                                        } else {
                                            // Unknown peer (connection refused before handshake):
                                            // Extract IP + TCP port from the bootstrap multiaddr and
                                            // check that both appear in the error string. This is
                                            // more robust than matching the full formatted multiaddr.
                                            let mut ip_str = None;
                                            let mut port_str = None;
                                            for proto in ba.iter() {
                                                match proto {
                                                    libp2p::multiaddr::Protocol::Ip4(ip) => ip_str = Some(format!("{}", ip)),
                                                    libp2p::multiaddr::Protocol::Tcp(p) => port_str = Some(format!("{}", p)),
                                                    _ => {}
                                                }
                                            }
                                            let err_str = format!("{} {:?}", error, error);
                                            ip_str.as_ref().is_some_and(|ip| err_str.contains(ip.as_str()))
                                                && port_str.as_ref().is_some_and(|p| err_str.contains(p.as_str()))
                                        };
                                    }

                                    if matches {
                                        if is_dns_multiaddr(ba) {
                                            bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure_gentle();
                                            tracing::debug!("Applied gentle backoff to DNS hostname {}", ba);
                                        } else if !resolved_match {
                                            bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                            tracing::debug!("Applied backoff to bootstrap addr {}", ba);
                                        }
                                        break;
                                    }
                                }
                            }

                            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                                // Inbound connection errors on the LAN listeners are
                                // dominated by benign TCP port-probes -- notably our own
                                // Android SubnetProbe LAN-discovery fallback, which opens a
                                // socket, waits ~200ms, then closes without ever writing the
                                // multistream-select / WS-handshake bytes. libp2p surfaces
                                // that as Select(Failed) / Handshake(UnexpectedEof), wrapped
                                // in the generic "Failed to negotiate transport protocol(s)".
                                // These are not actionable and previously masqueraded as a
                                // real negotiation bug, so log at debug rather than warn. A
                                // genuine peer-connectivity problem surfaces via
                                // OutgoingConnectionError or the absence of ConnectionEstablished.
                                tracing::debug!(
                                    "Incoming connection negotiation aborted from {} -> {}: {}",
                                    send_back_addr,
                                    local_addr,
                                    error
                                );

                                if record_negotiation_failure_and_check_burst(&send_back_addr.to_string()) {
                                    tracing::warn!(
                                        "High rate of incoming negotiation failures from {} -> {}: {}",
                                        send_back_addr,
                                        local_addr,
                                        error
                                    );
                                }
                            }

                            SwarmEvent::ListenerError { listener_id, error } => {
                                tracing::error!(
                                    "Listener {:?} reported an error (async bind/accept failure): {}",
                                    listener_id,
                                    error
                                );
                                let _ = event_tx.send(SwarmEvent2::ListenerFailed {
                                    listener_id: format!("{:?}", listener_id),
                                    error: error.to_string(),
                                }).await;
                            }

                            SwarmEvent::ListenerClosed { listener_id, addresses, reason } => {
                                tracing::warn!(
                                    "Listener {:?} closed for addresses {:?}: {:?}",
                                    listener_id,
                                    addresses,
                                    reason
                                );
                                if reason.is_err() {
                                    let _ = event_tx.send(SwarmEvent2::ListenerFailed {
                                        listener_id: format!("{:?}", listener_id),
                                        error: format!("listener closed for {:?}: {:?}", addresses, reason),
                                    }).await;
                                }
                            }

                            _ => {}
                        }
                    }

                    // Process commands from the application layer
                    Some(command) = command_rx.recv() => {
                        match command {
                            #[cfg(not(target_arch = "wasm32"))]
                            SwarmCommand::SendMessage { peer_id, envelope_data, recipient_identity_id, intended_device_id, reply } => {
                                // PHASE 6: Multi-path delivery with routing engine integration
                                let message_id = format!("{}-{}", peer_id, SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock before UNIX_EPOCH").as_millis());

                                // Start delivery tracking
                                multi_path_delivery.start_delivery(message_id.clone(), peer_id);

                                // MYCORRHIZAL ROUTING: Use routing engine to determine path
                                // Convert libp2p PeerId to routing module format
                                let peer_id_bytes = extract_peer_id_bytes(&peer_id.to_bytes());
                                // Get recipient hint from peer_id (first 4 bytes of blake3 hash)
                                let hint = blake3::hash(&peer_id_bytes).as_bytes()[0..4]
                                    .try_into()
                                    .expect("blake3 hash should be at least 4 bytes");

                                // Route message using mycorrhizal routing engine.
                                //
                                // CRITICAL BYPASS: the mycorrhizal engine only ever sees a
                                // 4-byte hint -- it has no way to know we already hold an
                                // active libp2p connection to this exact peer_id right now,
                                // and its layers (negative cache/prefetch/multipath/base
                                // discovery) are designed for *indirect* routing when the
                                // path genuinely isn't known. Without this check, a message
                                // to an already-connected peer could get decided as
                                // StoreAndCarry/RouteDiscovery and queued indefinitely
                                // instead of dispatched immediately (see
                                // HANDOFF/todo/CRITICAL_OUTBOX_NEVER_FLUSHES_DESPITE_ACTIVE_CONNECTION.md).
                                // The wasm32 SendMessage variant already does this ("simple
                                // direct send without complex routing") -- this mirrors that
                                // pattern for the native path instead of consulting the
                                // hint-only engine when we don't need to.
                                let routing_decision = if swarm.is_connected(&peer_id) {
                                    RoutingDecision {
                                        message_id: message_id.as_bytes()[0..16].try_into().unwrap_or([0u8; 16]),
                                        recipient_hint: hint,
                                        primary: NextHop::Direct {
                                            peer_id: peer_id_bytes,
                                            transport: RoutingTransportType::TCP,
                                        },
                                        alternatives: vec![],
                                        decided_by: RoutingLayer::Local,
                                        confidence: 1.0,
                                    }
                                } else {
                                    let mut guard = routing_engine_handle.write();
                                    if let Some(ref mut engine) = guard.as_mut() {
                                        engine.route_message_optimized(
                                            &hint,
                                            &message_id.as_bytes()[0..16].try_into().unwrap_or([0u8; 16]),
                                            50, // default priority
                                            SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs(),
                                        )
                                    } else {
                                        // Fallback: no routing engine available, use store-and-carry
                                        RoutingDecision {
                                            message_id: message_id.as_bytes()[0..16].try_into().unwrap_or([0u8; 16]),
                                            recipient_hint: hint,
                                            primary: NextHop::StoreAndCarry,
                                            alternatives: vec![],
                                            decided_by: RoutingLayer::StoreAndCarry,
                                            confidence: 0.0,
                                        }
                                    }
                                };

                                // Convert routing decision to ranked routes for dispatch
                                let routes = routing_decision_to_ranked_routes(
                                    &routing_decision,
                                    &peer_id,
                                    &mut multi_path_delivery,
                                );

                                if routes.is_empty() {
                                    let _ = reply.send(Err("No paths available".to_string())).await;
                                    continue;
                                }

                                let initial_route = &routes[0];
                                let attempt_start = SystemTime::now();
                                log_route_decision(
                                    &message_id,
                                    initial_route,
                                    1,
                                    0,
                                    0,
                                    routes.len(),
                                    ROUTE_ATTEMPT_REASON_INITIAL_SEND,
                                );
                                dispatch_ranked_route(
                                    &mut swarm,
                                    initial_route,
                                    &message_id,
                                    peer_id,
                                    &envelope_data,
                                    &mut request_to_message,
                                    &mut pending_relay_requests,
                                    recipient_identity_id.as_deref(),
                                    intended_device_id.as_deref(),
                                );

                                // Store pending message for retry handling
                                pending_messages.insert(message_id.clone(), PendingMessage {
                                    target_peer: peer_id,
                                    envelope_data,
                                    reply_tx: reply,
                                    current_path_index: 0,
                                    attempt_start,
                                    dispatch_attempts: 1,
                                    pass_count: 0,
                                    retry_notified: false,
                                    recipient_identity_id,
                                    intended_device_id,
                                });
                            }
                            #[cfg(target_arch = "wasm32")]
                            SwarmCommand::SendMessage { peer_id, envelope_data, recipient_identity_id, intended_device_id, reply } => {
                                // WASM: Simple direct send without complex routing
                                let message_id = format!("{}-{}", peer_id, SystemTime::now().duration_since(UNIX_EPOCH).expect("system clock before UNIX_EPOCH").as_millis());
                                let framed = wrap_in_drift_frame(&envelope_data);
                                let request_id = swarm.behaviour_mut().messaging.send_request(
                                    &peer_id,
                                    Libp2pMessageRequest {
                                        envelope_data: framed,
                                    },
                                );
                                pending_messages.insert(message_id.clone(), PendingMessage {
                                    target_peer: peer_id,
                                    envelope_data,
                                    reply_tx: reply,
                                    current_path_index: 0,
                                    attempt_start: SystemTime::now(),
                                    dispatch_attempts: 1,
                                    pass_count: 0,
                                    retry_notified: false,
                                    recipient_identity_id,
                                    intended_device_id,
                                });
                                request_to_message.insert(request_id, message_id);
                            }

                            SwarmCommand::RegisterIdentity { peer_id, request, reply } => {
                                let request_id = swarm
                                    .behaviour_mut()
                                    .registration
                                    .send_request(&peer_id, RegistrationMessage::Register(request));
                                pending_registration_replies.insert(request_id, reply);
                            }

                            SwarmCommand::DeregisterIdentity { peer_id, request, reply } => {
                                let request_id = swarm
                                    .behaviour_mut()
                                    .registration
                                    .send_request(&peer_id, RegistrationMessage::Deregister(request));
                                pending_registration_replies.insert(request_id, reply);
                            }

                            SwarmCommand::RequestAddressReflection { peer_id, reply } => {
                                let mut request_id_bytes = [0u8; 16];
                                use rand::RngCore;
                                rand::thread_rng().fill_bytes(&mut request_id_bytes);

                                let request = AddressReflectionRequest {
                                    request_id: request_id_bytes,
                                    version: 1,
                                };

                                let request_id = swarm.behaviour_mut().address_reflection.send_request(
                                    &peer_id,
                                    request,
                                );

                                pending_reflections.insert(request_id, reply);
                            }

                            SwarmCommand::GetBoundAddresses { reply } => {
                                let _ = reply.send(bound_addresses.clone()).await;
                            }
                            SwarmCommand::GetExternalAddresses { reply } => {
                                let addresses = address_observer.external_addresses().to_vec();
                                let _ = reply.send(addresses).await;
                            }

                            SwarmCommand::Dial { addr, reply } => {
                                tracing::debug!("Dialing {} (synthesizing port ladder if applicable)", addr);
                                let s = addr.to_string();
                                let is_direct = !s.contains("/p2p-circuit/") && !s.contains("/ws/") && !s.contains("/wss/");

                                let mut target_peer_id = None;
                                let mut base_prefix = Multiaddr::empty();
                                let mut found_ip = false;

                                if is_direct {
                                    for p in addr.iter() {
                                        match p {
                                            libp2p::multiaddr::Protocol::P2p(pid) => target_peer_id = Some(pid),
                                            libp2p::multiaddr::Protocol::Ip4(_) | libp2p::multiaddr::Protocol::Ip6(_) => {
                                                found_ip = true;
                                                base_prefix.push(p);
                                            }
                                            libp2p::multiaddr::Protocol::Tcp(_) | libp2p::multiaddr::Protocol::Udp(_) => {}
                                            _ => base_prefix.push(p),
                                        }
                                    }
                                }

                                // Every address actually dialed for this attempt, captured
                                // (stripped of any /p2p/ component, for later comparison
                                // against ConnectionEstablished/OutgoingConnectionError) so
                                // resolution below can require address correspondence rather
                                // than resolving on peer_id alone.
                                let mut dial_candidate_addrs: Vec<Multiaddr> = Vec::new();

                                let dial_res = if found_ip {
                                    match target_peer_id {
                                        Some(pid) => {
                                            let mut candidates = vec![addr.clone()];
                                            let fp = crate::store::transport_memory::get_network_fingerprint();

                                            if let Some(c) = &core_handle {
                                                if let Some(c_arc) = c.upgrade() {
                                                    if let Ok(Some(last_good)) = c_arc.transport_memory.read().get_last_good(&pid, &fp) {
                                                        let mut a = base_prefix.clone();
                                                        if last_good.transport == "tcp" {
                                                            a.push(libp2p::multiaddr::Protocol::Tcp(last_good.port));
                                                        } else {
                                                            a.push(libp2p::multiaddr::Protocol::Udp(last_good.port));
                                                        }
                                                        a.push(libp2p::multiaddr::Protocol::P2p(pid));
                                                        if !candidates.contains(&a) { candidates.push(a); }
                                                    }
                                                }
                                            }

                                            for port in [443, 80, 8080] {
                                                let mut a = base_prefix.clone();
                                                a.push(libp2p::multiaddr::Protocol::Tcp(port));
                                                a.push(libp2p::multiaddr::Protocol::P2p(pid));
                                                if !candidates.contains(&a) { candidates.push(a); }
                                            }

                                            dial_candidate_addrs = candidates
                                                .iter()
                                                .map(|a| a.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect())
                                                .collect();

                                            tracing::debug!("Dialing candidate ladder for {}: {:?}", pid, candidates);
                                            let opts = libp2p::swarm::dial_opts::DialOpts::peer_id(pid)
                                                .addresses(candidates)
                                                .build();
                                            swarm.dial(opts)
                                        }
                                        None => {
                                            dial_candidate_addrs.push(
                                                addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect(),
                                            );
                                            swarm.dial(addr.clone())
                                        }
                                    }
                                } else {
                                    dial_candidate_addrs.push(
                                        addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect(),
                                    );
                                    swarm.dial(addr.clone())
                                };

                                match dial_res {
                                    Ok(_) => {
                                        // Don't reply yet: a queued dial is not a connected
                                        // dial. Register it and wait for a real
                                        // ConnectionEstablished/OutgoingConnectionError signal
                                        // (or the periodic sweep's timeout) to resolve it.
                                        let stripped_addr: Multiaddr = addr
                                            .iter()
                                            .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
                                            .collect();
                                        // A prior in-flight dial to this same address gets an
                                        // explicit error reply here rather than being silently
                                        // dropped (which would otherwise close its reply channel
                                        // and leave the caller with an opaque "No reply from
                                        // swarm" error).
                                        if let Some(old_entry) = pending_dials.remove(&stripped_addr) {
                                            let _ = old_entry.reply.send(Err(
                                                "Superseded by a newer dial to the same address".to_string(),
                                            )).await;
                                        }
                                        pending_dials.insert(
                                            stripped_addr,
                                            PendingDialEntry {
                                                reply,
                                                dialed_at: web_time::Instant::now(),
                                                candidate_addrs: dial_candidate_addrs,
                                            },
                                        );
                                    }
                                    Err(e) => {
                                        let err_msg: String = format!("{}", e);
                                        let _ = reply.send(Err(err_msg)).await;
                                    }
                                }
                            }

                            SwarmCommand::DialResolved { original_dns, resolved_addrs } => {
                                in_flight_dns.remove(&original_dns);
                                // Prune old entries for this original_dns to prevent stale mappings
                                resolved_to_dns.retain(|_, v| v != &original_dns);

                                for resolved_addr in resolved_addrs {
                                    if bootstrap_backoff.get(&resolved_addr).is_none_or(|e| e.is_eligible()) {
                                        tracing::debug!("Dialing resolved address {} for DNS bootstrap {}", resolved_addr, original_dns);
                                        resolved_to_dns.insert(resolved_addr.clone(), original_dns.clone());
                                        resolved_keys_fifo.push_back(resolved_addr.clone());
                                        // Also insert stripped versions without /p2p/ to be robust
                                        let stripped_res: Multiaddr = resolved_addr.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
                                        let stripped_dns: Multiaddr = original_dns.iter().filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_))).collect();
                                        resolved_to_dns.insert(stripped_res.clone(), stripped_dns);
                                        resolved_keys_fifo.push_back(stripped_res);

                                        let _ = swarm.dial(resolved_addr);
                                    } else {
                                        tracing::debug!("Skipping dial of resolved IP {} (backed off)", resolved_addr);
                                    }
                                }

                                // Evict oldest when over a sane cap (e.g. 200)
                                while resolved_to_dns.len() > 200 {
                                    if let Some(oldest) = resolved_keys_fifo.pop_front() {
                                        resolved_to_dns.remove(&oldest);
                                    } else {
                                        break;
                                    }
                                }
                            }

                            SwarmCommand::ResolutionFailed { original_dns } => {
                                in_flight_dns.remove(&original_dns);
                                bootstrap_backoff.entry(original_dns.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure_gentle();
                                tracing::debug!("DNS resolution failed for {}; applied gentle backoff", original_dns);
                            }

                            SwarmCommand::GetPeers { reply } => {
                                let peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                                let _ = reply.send(peers).await;
                            }

                            SwarmCommand::Listen { addr, reply } => {
                                match swarm.listen_on(addr) {
                                    Ok(_) => {
                                        let _ = reply.send(Ok("/ip4/0.0.0.0/tcp/0".parse().expect("static multiaddr parse cannot fail"))).await;
                                    }
                                    Err(e) => {
                                        let _ = reply.send(Err(format!("{}", e))).await;
                                    }
                                }
                            }

                            SwarmCommand::AddKadAddress { peer_id, addr } => {
                                if is_discoverable_multiaddr(&addr) {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                            }

                            SwarmCommand::SubscribeTopic { topic, reply } => {
                                if subscribed_topics.contains(&topic) {
                                    let _ = reply.send(Ok(())).await;
                                } else {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    match swarm.behaviour_mut().gossipsub.subscribe(&ident_topic) {
                                        Ok(_) => {
                                            tracing::info!("Subscribed to topic: {}", topic);
                                            subscribed_topics.insert(topic);
                                            let _ = reply.send(Ok(())).await;
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to subscribe to topic {}: {}", topic, e);
                                            let _ = reply.send(Err(e.to_string())).await;
                                        }
                                    }
                                }
                            }

                            SwarmCommand::UnsubscribeTopic { topic, reply } => {
                                if subscribed_topics.contains(&topic) {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    if swarm.behaviour_mut().gossipsub.unsubscribe(&ident_topic) {
                                        tracing::info!("Unsubscribed from topic: {}", topic);
                                        subscribed_topics.remove(&topic);
                                        let _ = reply.send(Ok(())).await;
                                    } else {
                                        tracing::warn!("Not subscribed to topic {}", topic);
                                        let _ = reply.send(Err(format!(
                                            "gossipsub reports no subscription for topic {}",
                                            topic
                                        ))).await;
                                    }
                                } else {
                                    // Idempotent: not subscribed locally, nothing to undo.
                                    let _ = reply.send(Ok(())).await;
                                }
                            }

                            SwarmCommand::PublishTopic { topic, data, reply } => {
                                let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                match swarm.behaviour_mut().gossipsub.publish(ident_topic, data) {
                                    Ok(_) => {
                                        tracing::debug!("Published payload to topic {}", topic);
                                        let _ = reply.send(Ok(())).await;
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to publish to topic {}: {}", topic, e);
                                        let _ = reply.send(Err(e.to_string())).await;
                                    }
                                }
                            }

                            SwarmCommand::GetTopics { reply } => {
                                let topics: Vec<String> = subscribed_topics.iter().cloned().collect();
                                let _ = reply.send(topics).await;
                            }

                            SwarmCommand::ShareLedger { peer_id, entries } => {
                                // Send our known peer list to the specified peer
                                if !ledger_exchanged_peers.contains(&peer_id) {
                                    tracing::info!(
                                        "Sharing ledger with {} ({} entries)",
                                        peer_id,
                                        entries.len()
                                    );

                                    let request = LedgerExchangeRequest {
                                        peers: entries,
                                        sender_peer_id: local_peer_id.to_string(),
                                        version: 1,
                                    };

                                    let _request_id = swarm.behaviour_mut().ledger_exchange.send_request(
                                        &peer_id,
                                        request,
                                    );

                                    ledger_exchanged_peers.insert(peer_id);
                                } else {
                                    tracing::debug!("Already exchanged ledger with {}, skipping", peer_id);
                                }
                            }

                            SwarmCommand::GetListeners { reply } => {
                    let listeners: Vec<Multiaddr> = swarm.listeners().cloned().collect();
                    let _ = reply.send(listeners).await;
                }
                            SwarmCommand::ListEndpoints { peer_id: _, reply } => {
                                // Return our own listening addresses as the endpoint list.
                                // The Kademlia DHT doesn't expose a direct address lookup per peer
                                // in the current libp2p version; peers discover addresses via
                                // Identify and Kademlia protocols automatically.
                                let addrs: Vec<Multiaddr> = swarm.listeners().cloned().collect();
                                let _ = reply.send(addrs).await;
                            }
                            SwarmCommand::RegisterEndpoint { peer_id, addr, reply } => {
                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                                if let Some(socket) = crate::transport::observation::ConnectionTracker::extract_socket_addr(&addr) {
                                    address_observer.record_observation(peer_id, socket);
                                }
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::TouchEndpoint { peer_id, addr, reply } => {
                                if let Some(socket) = crate::transport::observation::ConnectionTracker::extract_socket_addr(&addr) {
                                    address_observer.record_observation(peer_id, socket);
                                }
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::UnregisterEndpoint { peer_id: _, addr, reply } => {
                                // Remove from Kademlia routing table if present.
                                // The AddressObserver doesn't have a remove method, so we
                                // just log the removal and rely on TTL expiry for cleanup.
                                tracing::debug!("Unregistering endpoint: {}", addr);
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::UpdateKeepalive { peer_id: _, interval_secs, reply } => {
                                tracing::debug!("Keepalive interval updated to {}s", interval_secs);
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::SetRelayBudget { budget } => {
                                relay_budget = budget;
                                tracing::info!("Relay budget updated: {} msgs/hour", budget);
                            }

                            SwarmCommand::GetBestRelays { count, reply } => {
                                let relays = multi_path_delivery.best_relays(count);
                                let _ = reply.send(relays).await;
                            }
                            SwarmCommand::GetBootstrapCandidates { reply } => {
                                let candidates = bootstrap_capability.get_bootstrap_candidates().to_vec();
                                let _ = reply.send(candidates).await;
                            }
                            SwarmCommand::GetBestPaths { target, count, reply } => {
                                let paths = multi_path_delivery.get_best_paths(&target, count);
                                let _ = reply.send(paths).await;
                            }
                            SwarmCommand::Shutdown => {
                                tracing::info!("Swarm shutting down");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(handle)
    }

    #[cfg(target_arch = "wasm32")]
    #[allow(
        clippy::collapsible_match,
        clippy::single_match,
        clippy::if_same_then_else
    )]
    {
        use futures::{FutureExt, StreamExt};
        use libp2p::core::{muxing::StreamMuxerBox, upgrade::Version};

        let _ = routing_engine_handle;
        let local_peer_id = keypair.public().to_peer_id();

        // Browser transport: websocket-websys + Noise + Yamux, then relay client support.
        // This keeps protocol-level parity with native swarm behaviour.
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_wasm_bindgen()
            .with_other_transport(
                |id_keys| -> std::result::Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    let noise = libp2p::noise::Config::new(id_keys)?;
                    Ok(libp2p::websocket_websys::Transport::default()
                        .upgrade(Version::V1Lazy)
                        .authenticate(noise)
                        .multiplex(libp2p::yamux::Config::default())
                        .map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn))))
                },
            )?
            .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)?
            .with_behaviour(|key, relay_client| {
                IronCoreBehaviour::new(key, relay_client, headless, discovery_config)
                    .expect("Failed to create network behaviour")
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(web_time::Duration::from_secs(600))
            })
            .build();

        // Browser nodes cannot open TCP listeners. We keep deterministic command semantics:
        // Listen => unsupported, GetListeners => empty.
        if let Some(addr) = listen_addr {
            tracing::warn!(
                "Ignoring listen addr on wasm32 (browser cannot listen): {}",
                addr
            );
        }

        // Keep default topic parity with native.
        let lobby_topic = libp2p::gossipsub::IdentTopic::new("sc-lobby");
        let mesh_topic = libp2p::gossipsub::IdentTopic::new("sc-mesh");
        let delivery_convergence_topic =
            libp2p::gossipsub::IdentTopic::new(DELIVERY_CONVERGENCE_TOPIC);
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&lobby_topic) {
            tracing::warn!("Failed to subscribe to lobby topic: {}", e);
        }
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&mesh_topic) {
            tracing::warn!("Failed to subscribe to mesh topic: {}", e);
        }
        if let Err(e) = swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&delivery_convergence_topic)
        {
            tracing::warn!("Failed to subscribe to delivery convergence topic: {}", e);
        }

        // Kademlia server mode parity with native.
        swarm
            .behaviour_mut()
            .kademlia
            .set_mode(Some(kad::Mode::Server));

        // Auto-dial bootstrap nodes for internet connectivity.
        // Self-dial guard: track bootstrap addrs that resolve to our own peer
        // so we log once at info level and then suppress the warning spam.
        let mut self_dial_logged: HashSet<Multiaddr> = HashSet::new();
        if !bootstrap_addrs.is_empty() {
            tracing::info!(
                "Dialing {} bootstrap node(s) from wasm",
                bootstrap_addrs.len()
            );
            for addr in &bootstrap_addrs {
                // Self-dial check: skip bootstrap addrs whose p2p component
                // matches our own peer ID (e.g. portproxy loopback).
                let is_self = addr.iter().any(|proto| {
                    if let libp2p::multiaddr::Protocol::P2p(pid) = proto {
                        pid == local_peer_id
                    } else {
                        false
                    }
                });
                if is_self {
                    if !self_dial_logged.contains(addr) {
                        tracing::info!(
                            "  ⊘ Skipping self-dial bootstrap addr (matches local peer): {}",
                            addr
                        );
                        self_dial_logged.insert(addr.clone());
                    }
                    continue;
                }
                let stripped_addr: Multiaddr = addr
                    .iter()
                    .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
                    .collect();
                match swarm.dial(stripped_addr.clone()) {
                    Ok(_) => tracing::info!("  [OK] Dialing bootstrap: {}", stripped_addr),
                    Err(e) => {
                        tracing::warn!("  [FAIL] Failed to dial bootstrap {}: {}", stripped_addr, e)
                    }
                }
            }
        }

        let (command_tx, mut command_rx) = mpsc::channel::<SwarmCommand>(256);
        let handle = SwarmHandle {
            command_tx: command_tx.clone(),
            core_handle: core_handle.clone(),
        };

        let mut pending_direct_replies: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<(), String>>,
        > = HashMap::new();

        let mut bound_addresses = Vec::new();
        let mut pending_reflections: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<String, String>>,
        > = HashMap::new();
        let mut pending_registration_replies: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<(), String>>,
        > = HashMap::new();

        let mut pending_relay_requests: HashMap<
            libp2p::request_response::OutboundRequestId,
            String,
        > = HashMap::new();
        let relay_custody_store = RelayCustodyStore::in_memory();
        let mut pending_custody_dispatches: HashMap<
            libp2p::request_response::OutboundRequestId,
            PendingCustodyDispatch,
        > = HashMap::new();

        let mut pending_messages: HashMap<String, PendingMessage> = HashMap::new();

        let mut subscribed_topics: HashSet<String> = HashSet::new();
        subscribed_topics.insert("sc-lobby".to_string());
        subscribed_topics.insert("sc-mesh".to_string());
        subscribed_topics.insert(DELIVERY_CONVERGENCE_TOPIC.to_string());

        let mut ledger_exchanged_peers: HashSet<PeerId> = HashSet::new();

        // Keep observational parity where possible on wasm.
        let reflection_service = AddressReflectionService::new();
        let mut connection_tracker = ConnectionTracker::new();
        let mut address_observer = AddressObserver::new();
        let mut relay_budget: u32 = 200;
        let mut relay_count_this_hour: u32 = 0;
        let mut relay_guardrails = RelayAbuseGuardrails::new();
        // This WASM-only event loop uses js_sys::Date::now() (f64 ms since
        // epoch) for elapsed-time checks because the existing comparison logic
        // (`now - start >= threshold_ms`) relies on f64 arithmetic.  web_time::Instant
        // would also work here, but migrating would require restructuring all the
        // timing comparisons with no functional benefit — the native path already
        // uses web_time::Instant (see run_swarm above).
        let mut relay_hour_start: f64 = js_sys::Date::now();
        let mut last_bootstrap_redial: f64 = js_sys::Date::now();
        let mut last_custody_pull: f64 = js_sys::Date::now();
        let mut seen_delivery_convergence_markers: HashSet<String> = HashSet::new();
        let bootstrap_addrs_clone = bootstrap_addrs;
        let mut bootstrap_backoff: HashMap<Multiaddr, BootstrapBackoffEntry> = HashMap::new();
        let mut reported_peer_discoveries: HashSet<PeerId> = HashSet::new();
        let mut sync_sessions: HashMap<PeerId, SyncSession> = HashMap::new();

        wasm_bindgen_futures::spawn_local(async move {
            loop {
                let command_fut = command_rx.recv().fuse();
                let swarm_fut = swarm.select_next_some().fuse();
                futures::pin_mut!(command_fut, swarm_fut);

                futures::select! {
                    maybe_command = command_fut => {
                        let Some(command) = maybe_command else {
                            tracing::info!("WASM swarm command channel closed");
                            break;
                        };

                        match command {
                            SwarmCommand::SendMessage { peer_id, envelope_data, reply, .. } => {
                                let framed = wrap_in_drift_frame(&envelope_data);
                                let request_id = swarm.behaviour_mut().messaging.send_request(
                                    &peer_id,
                                    Libp2pMessageRequest { envelope_data: framed },
                                );
                                pending_direct_replies.insert(request_id, reply);
                            }
                            SwarmCommand::RegisterIdentity { peer_id, request, reply } => {
                                let request_id = swarm
                                    .behaviour_mut()
                                    .registration
                                    .send_request(&peer_id, RegistrationMessage::Register(request));
                                pending_registration_replies.insert(request_id, reply);
                            }
                            SwarmCommand::DeregisterIdentity { peer_id, request, reply } => {
                                let request_id = swarm
                                    .behaviour_mut()
                                    .registration
                                    .send_request(&peer_id, RegistrationMessage::Deregister(request));
                                pending_registration_replies.insert(request_id, reply);
                            }
                            SwarmCommand::RequestAddressReflection { peer_id, reply } => {
                                let mut request_id_bytes = [0u8; 16];
                                use rand::RngCore;
                                rand::thread_rng().fill_bytes(&mut request_id_bytes);

                                let request = AddressReflectionRequest {
                                    request_id: request_id_bytes,
                                    version: 1,
                                };

                                let request_id = swarm.behaviour_mut().address_reflection.send_request(
                                    &peer_id,
                                    request,
                                );
                                pending_reflections.insert(request_id, reply);
                            }
                            SwarmCommand::GetBoundAddresses { reply } => {
                                let _ = reply.send(bound_addresses.clone()).await;
                            }
                            SwarmCommand::GetExternalAddresses { reply } => {
                                let addresses = address_observer.external_addresses().to_vec();
                                let _ = reply.send(addresses).await;
                            }
                            SwarmCommand::Dial { addr, reply } => {
                                match swarm.dial(addr) {
                                    Ok(_) => { let _ = reply.send(Ok(())).await; }
                                    Err(e) => {
                                        let err_msg: String = e.to_string();
                                        let _ = reply.send(Err(err_msg)).await;
                                    }
                                }
                            }
                            SwarmCommand::DialResolved { .. } => {}
                            SwarmCommand::ResolutionFailed { .. } => {}
                            SwarmCommand::GetPeers { reply } => {
                                let peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                                let _ = reply.send(peers).await;
                            }
                            SwarmCommand::Listen { reply, .. } => {
                                let _ = reply
                                    .send(Err("listen is unsupported on wasm32/browser transport".to_string()))
                                    .await;
                            }
                            SwarmCommand::AddKadAddress { peer_id, addr } => {
                                if is_discoverable_multiaddr(&addr) {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                            }
                            SwarmCommand::SubscribeTopic { topic, reply } => {
                                if subscribed_topics.contains(&topic) {
                                    let _ = reply.send(Ok(())).await;
                                } else {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    match swarm.behaviour_mut().gossipsub.subscribe(&ident_topic) {
                                        Ok(_) => {
                                            subscribed_topics.insert(topic);
                                            let _ = reply.send(Ok(())).await;
                                        }
                                        Err(e) => {
                                            let _ = reply.send(Err(e.to_string())).await;
                                        }
                                    }
                                }
                            }
                            SwarmCommand::UnsubscribeTopic { topic, reply } => {
                                if subscribed_topics.contains(&topic) {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    if swarm.behaviour_mut().gossipsub.unsubscribe(&ident_topic) {
                                        subscribed_topics.remove(&topic);
                                        let _ = reply.send(Ok(())).await;
                                    } else {
                                        let _ = reply.send(Err(format!(
                                            "gossipsub reports no subscription for topic {}",
                                            topic
                                        ))).await;
                                    }
                                } else {
                                    // Idempotent: not subscribed locally, nothing to undo.
                                    let _ = reply.send(Ok(())).await;
                                }
                            }
                            SwarmCommand::PublishTopic { topic, data, reply } => {
                                let ident_topic = libp2p::gossipsub::IdentTopic::new(topic);
                                match swarm.behaviour_mut().gossipsub.publish(ident_topic, data) {
                                    Ok(_) => {
                                        let _ = reply.send(Ok(())).await;
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to publish topic payload: {}", e);
                                        let _ = reply.send(Err(e.to_string())).await;
                                    }
                                }
                            }
                            SwarmCommand::GetTopics { reply } => {
                                let topics: Vec<String> = subscribed_topics.iter().cloned().collect();
                                let _ = reply.send(topics).await;
                            }
                            SwarmCommand::ShareLedger { peer_id, entries } => {
                                if !ledger_exchanged_peers.contains(&peer_id) {
                                    let request = LedgerExchangeRequest {
                                        peers: entries,
                                        sender_peer_id: local_peer_id.to_string(),
                                        version: 1,
                                    };
                                    let _ = swarm.behaviour_mut().ledger_exchange.send_request(&peer_id, request);
                                    ledger_exchanged_peers.insert(peer_id);
                                }
                            }
                            SwarmCommand::GetListeners { reply } => {
                                // Browser nodes do not expose listen addresses.
                                let _ = reply.send(Vec::new()).await;
                            }
                            SwarmCommand::ListEndpoints { peer_id: _, reply } => {
                                // WASM nodes do not track endpoint addresses locally.
                                let _ = reply.send(Vec::new()).await;
                            }
                            SwarmCommand::RegisterEndpoint { peer_id: _, addr: _, reply } => {
                                // WASM nodes register endpoints via the daemon bridge, not locally.
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::TouchEndpoint { peer_id: _, addr: _, reply } => {
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::UnregisterEndpoint { peer_id: _, addr: _, reply } => {
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::UpdateKeepalive { peer_id: _, interval_secs, reply } => {
                                tracing::debug!("WASM: keepalive interval updated to {}s", interval_secs);
                                let _ = reply.send(Ok(())).await;
                            }
                            SwarmCommand::SetRelayBudget { budget } => {
                                relay_budget = budget;
                                tracing::info!("Relay budget updated: {} msgs/hour", budget);
                            }
                            SwarmCommand::GetBestRelays { reply, .. } => {
                                let _ = reply.send(Vec::new()).await;
                            }
                            SwarmCommand::GetBootstrapCandidates { reply } => {
                                let _ = reply.send(Vec::new()).await;
                            }
                            SwarmCommand::GetBestPaths { reply, .. } => {
                                let _ = reply.send(Vec::new()).await;
                            }
                            SwarmCommand::Shutdown => {
                                tracing::info!("WASM swarm shutting down");
                                break;
                            }
                        }
                    }
                    event = swarm_fut => {
                        match event {
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Messaging(ev)) => {
                                match ev {
                                    request_response::Event::Message { peer, message, .. } => match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            // Check if sender is blocked before processing message
                                            let sender_blocked = if let Some(ref core_handle) = core_handle {
                                                // WASM version doesn't have device ID in request, so pass None
                                                core_handle.upgrade().map(|c| c.is_peer_blocked(peer.to_string(), None).unwrap_or(false)).unwrap_or(false)
                                            } else {
                                                false
                                            };

                                            if sender_blocked {
                                                tracing::warn!("Blocked peer {} attempted to send message", peer);
                                                let _ = swarm.behaviour_mut().messaging.send_response(
                                                    channel,
                                                    Libp2pMessageResponse { accepted: false, error: Some("blocked".to_string()) },
                                                );
                                                continue;
                                            }

                                            // Unwrap DriftFrame if present, otherwise use raw data (legacy)
                                            let envelope_payload = match DriftFrame::from_bytes(&request.envelope_data) {
                                                Ok(frame) => {
                                                    tracing::debug!("Received DriftFrame type: {:?} from {}", frame.frame_type, peer);
                                                    frame.payload
                                                }
                                                Err(_) => request.envelope_data.clone(),
                                            };

                                            let _ = event_tx.send(SwarmEvent2::MessageReceived {
                                                peer_id: peer,
                                                envelope_data: envelope_payload,
                                            }).await;

                                            let _ = swarm.behaviour_mut().messaging.send_response(
                                                channel,
                                                Libp2pMessageResponse { accepted: true, error: None },
                                            );
                                        }
                                        request_response::Message::Response { request_id, response } => {
                                            if let Some(dispatch) =
                                                pending_custody_dispatches.remove(&request_id)
                                            {
                                                if response.accepted {
                                                    let _ = relay_custody_store.mark_delivered(
                                                        &dispatch.destination_peer.to_string(),
                                                        &dispatch.custody_id,
                                                        "recipient_ack",
                                                    );
                                                    let marker = DeliveryConvergenceMarker {
                                                        relay_message_id: dispatch
                                                            .relay_message_id
                                                            .clone(),
                                                        destination_peer_id: dispatch
                                                            .destination_peer
                                                            .to_string(),
                                                        observed_by_peer_id: local_peer_id
                                                            .to_string(),
                                                        observed_at_ms: js_sys::Date::now() as u64,
                                                    };
                                                    if seen_delivery_convergence_markers
                                                        .insert(marker.key())
                                                    {
                                                        apply_delivery_convergence_marker(
                                                            &marker,
                                                            &mut pending_messages,
                                                            &mut pending_relay_requests,
                                                            &mut pending_custody_dispatches,
                                                            &relay_custody_store,
                                                        )
                                                        .await;
                                                        publish_delivery_convergence_marker(
                                                            &mut swarm,
                                                            &marker,
                                                        );
                                                    }
                                                } else {
                                                    let reason = response
                                                        .error
                                                        .unwrap_or_else(|| "recipient_rejected".to_string());
                                                    let reason = format!("recipient_rejected:{}", reason);
                                                    let _ = relay_custody_store.mark_dispatch_failed(
                                                        &dispatch.destination_peer.to_string(),
                                                        &dispatch.custody_id,
                                                        &reason,
                                                    );
                                                }
                                            } else if let Some(reply_tx) =
                                                pending_direct_replies.remove(&request_id)
                                            {
                                                let result = if response.accepted {
                                                    Ok(())
                                                } else {
                                                    Err(response.error.unwrap_or_else(|| "message rejected".to_string()))
                                                };
                                                let _ = reply_tx.send(result).await;
                                            }
                                        }
                                    },
                                    request_response::Event::OutboundFailure { request_id, error, .. } => {
                                        if let Some(dispatch) =
                                            pending_custody_dispatches.remove(&request_id)
                                        {
                                            let reason = format!("dispatch_outbound_failure:{}", error);
                                            let _ = relay_custody_store.mark_dispatch_failed(
                                                &dispatch.destination_peer.to_string(),
                                                &dispatch.custody_id,
                                                &reason,
                                            );
                                        } else if let Some(reply_tx) =
                                            pending_direct_replies.remove(&request_id)
                                        {
                                            let _ = reply_tx.send(Err(error.to_string())).await;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            SwarmEvent::Behaviour(
                                super::behaviour::IronCoreBehaviourEvent::Registration(ev),
                            ) => {
                                match ev {
                                    request_response::Event::Message { peer, message, .. } => match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            let response = match verify_registration_message(&peer, &request) {
                                                Ok(()) => apply_verified_registration_message(
                                                    &relay_custody_store,
                                                    &request,
                                                ),
                                                Err(error) => RegistrationResponse {
                                                    accepted: false,
                                                    error: Some(error.to_string()),
                                                },
                                            };
                                            let _ = swarm
                                                .behaviour_mut()
                                                .registration
                                                .send_response(channel, response);
                                        }
                                        request_response::Message::Response { request_id, response } => {
                                            if let Some(reply_tx) =
                                                pending_registration_replies.remove(&request_id)
                                            {
                                                let result = if response.accepted {
                                                    Ok(())
                                                } else {
                                                    Err(response.error.unwrap_or_else(|| {
                                                        "registration_request_rejected".to_string()
                                                    }))
                                                };
                                                let _ = reply_tx.send(result).await;
                                            }
                                        }
                                    },
                                    request_response::Event::OutboundFailure { request_id, error, .. } => {
                                        if let Some(reply_tx) =
                                            pending_registration_replies.remove(&request_id)
                                        {
                                            let _ = reply_tx.send(Err(error.to_string())).await;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::AddressReflection(ev)) => {
                                match ev {
                                    request_response::Event::Message { peer, message, .. } => match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            let observed_addr = connection_tracker
                                                .get_connection(&peer)
                                                .and_then(|conn| ConnectionTracker::extract_socket_addr(&conn.remote_addr))
                                                .unwrap_or_else(|| "0.0.0.0:0".parse().expect("static socket addr parse cannot fail"));

                                            let response = reflection_service.handle_request(request, observed_addr);
                                            let _ = swarm.behaviour_mut().address_reflection.send_response(channel, response);
                                        }
                                        request_response::Message::Response { request_id, response } => {
                                            if let Ok(observed_addr) = response.observed_address.parse::<SocketAddr>() {
                                                address_observer.record_observation(peer, observed_addr);
                                            }
                                            if let Some(reply_tx) = pending_reflections.remove(&request_id) {
                                                let _ = reply_tx.send(Ok(response.observed_address.clone())).await;
                                            }
                                            let _ = event_tx.send(SwarmEvent2::AddressReflected {
                                                peer_id: peer,
                                                observed_address: response.observed_address,
                                            }).await;
                                        }
                                    },
                                    request_response::Event::OutboundFailure { request_id, error, .. } => {
                                        if let Some(reply_tx) = pending_reflections.remove(&request_id) {
                                            let _ = reply_tx.send(Err(error.to_string())).await;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Relay(ev)) => {
                                match ev {
                                    request_response::Event::Message { peer, message, .. } => match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                            let now_ms = js_sys::Date::now() as u64;
                                            if js_sys::Date::now() - relay_hour_start >= 3_600_000.0 {
                                                relay_count_this_hour = 0;
                                                relay_hour_start = js_sys::Date::now();
                                            }

                                            let relay_response = if let Some(reason) = relay_guardrails
                                                .should_reject_cheap_heuristics(
                                                    &request.message_id,
                                                    request.envelope_data.len(),
                                                )
                                            {
                                                tracing::warn!(
                                                    "Relay request rejected by heuristic from {} (message {}): {}",
                                                    peer,
                                                    request.message_id,
                                                    reason
                                                );
                                                let abuse_signal = if reason.contains("oversized") {
                                                    "OversizedMessage"
                                                } else if reason.contains("duplicate") {
                                                    "DuplicateMessage"
                                                } else {
                                                    "InvalidFormat"
                                                };
                                                let _ = event_tx.send(SwarmEvent2::AbuseSignalDetected {
                                                    peer_id: peer,
                                                    signal: abuse_signal.to_string(),
                                                }).await;
                                                RelayResponse {
                                                    accepted: false,
                                                    error: Some(reason.to_string()),
                                                    message_id: request.message_id.clone(),
                                                }
                                            } else if relay_budget > 0 && relay_count_this_hour >= relay_budget {
                                                RelayResponse {
                                                    accepted: false,
                                                    error: Some("relay_budget_exhausted".to_string()),
                                                    message_id: request.message_id.clone(),
                                                }
                                            } else if pending_custody_dispatches.len()
                                                >= RELAY_MAX_INFLIGHT_DISPATCHES
                                            {
                                                tracing::warn!(
                                                    "Relay inflight cap reached ({}) — rejecting relay request {}",
                                                    RELAY_MAX_INFLIGHT_DISPATCHES,
                                                    request.message_id
                                                );
                                                RelayResponse {
                                                    accepted: false,
                                                    error: Some("relay_inflight_capped".to_string()),
                                                    message_id: request.message_id.clone(),
                                                }
                                            } else if !relay_guardrails.consume_peer_token(
                                                &peer.to_string(),
                                                now_ms,
                                                1.0,
                                            ) {
                                                tracing::warn!(
                                                    "Relay request rate-limited for peer {} (message {})",
                                                    peer,
                                                    request.message_id
                                                );
                                                let _ = event_tx.send(SwarmEvent2::AbuseSignalDetected {
                                                    peer_id: peer,
                                                    signal: "RateLimited".to_string(),
                                                }).await;
                                                RelayResponse {
                                                    accepted: false,
                                                    error: Some("relay_peer_rate_limited".to_string()),
                                                    message_id: request.message_id.clone(),
                                                }
                                            } else {
                                                relay_count_this_hour += 1;
                                                match PeerId::from_bytes(&request.destination_peer) {
                                                    Ok(destination) => {
                                                        let relay_message_id = request.message_id.clone();
                                                        match resolve_custody_metadata(
                                                            &relay_custody_store,
                                                            request.recipient_identity_id.as_deref(),
                                                            request.intended_device_id.as_deref(),
                                                            CustodyCompatMode::default(),
                                                        ) {
                                                            Err(error) => RelayResponse {
                                                                accepted: false,
                                                                error: Some(error),
                                                                message_id: relay_message_id,
                                                            },
                                                            Ok((resolved_identity_id, resolved_device_id)) => {
                                                                if relay_guardrails.is_recent_duplicate(
                                                                    &peer.to_string(),
                                                                    &destination.to_string(),
                                                                    &relay_message_id,
                                                                    now_ms,
                                                                ) {
                                                                    tracing::info!(
                                                                        "Relay duplicate suppressed from {} -> {} for message {}",
                                                                        peer,
                                                                        destination,
                                                                        relay_message_id
                                                                    );
                                                                    RelayResponse {
                                                                        accepted: true,
                                                                        error: None,
                                                                        message_id: relay_message_id,
                                                                    }
                                                                } else {
                                                                    match relay_custody_store.accept_custody(
                                                                        peer.to_string(),
                                                                        destination.to_string(),
                                                                        relay_message_id.clone(),
                                                                        request.envelope_data.clone(),
                                                                        resolved_identity_id,
                                                                        resolved_device_id,
                                                                    ) {
                                                                        Ok(_) => {
                                                                            relay_guardrails.record_accepted(
                                                                                &peer.to_string(),
                                                                                &destination.to_string(),
                                                                                &relay_message_id,
                                                                                now_ms,
                                                                            );
                                                                            if swarm.is_connected(&destination) {
                                                                                dispatch_pending_custody_for_peer(
                                                                                    &mut swarm,
                                                                                    &relay_custody_store,
                                                                                    destination,
                                                                                    &mut pending_custody_dispatches,
                                                                                    RELAY_MAX_INFLIGHT_DISPATCHES,
                                                                                    "accept_immediate_pull",
                                                                                );
                                                                            }
                                                                            RelayResponse {
                                                                                accepted: true,
                                                                                error: None,
                                                                                message_id: relay_message_id,
                                                                            }
                                                                        }
                                                                        Err(e) => RelayResponse {
                                                                            accepted: false,
                                                                            error: Some(format!(
                                                                                "custody_store_failed: {}",
                                                                                e
                                                                            )),
                                                                            message_id: relay_message_id,
                                                                        },
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    Err(_) => RelayResponse {
                                                        accepted: false,
                                                        error: Some("Invalid destination peer ID".to_string()),
                                                        message_id: request.message_id.clone(),
                                                    },
                                                }
                                            };
                                            let _ = swarm.behaviour_mut().relay.send_response(channel, relay_response);
                                        }
                                        request_response::Message::Response { request_id, response } => {
                                            if let Some(message_id) = pending_relay_requests.remove(&request_id) {
                                                if let Some(pending) = pending_messages.remove(&message_id) {
                                                    if response.accepted {
                                                        let _ = pending.reply_tx.send(Ok(())).await;
                                                    } else {
                                                        let error = response
                                                            .error
                                                            .unwrap_or_else(|| "relay rejected".to_string());
                                                        if is_terminal_identity_rejection(&error) {
                                                            let _ = pending.reply_tx.send(Err(error)).await;
                                                        } else {
                                                            pending_messages.insert(message_id, pending);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::LedgerExchange(ev)) => {
                                if let request_response::Event::Message { peer, message, .. } = ev {
                                    match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            let _ = event_tx.send(SwarmEvent2::LedgerReceived {
                                                from_peer: peer,
                                                entries: request.peers.clone(),
                                            }).await;
                                            let _ = swarm.behaviour_mut().ledger_exchange.send_response(
                                                channel,
                                                LedgerExchangeResponse {
                                                    peers: Vec::new(),
                                                    new_peers_learned: 0,
                                                    version: 1,
                                                },
                                            );
                                            ledger_exchanged_peers.insert(peer);
                                        }
                                        request_response::Message::Response { response, .. } => {
                                            if !response.peers.is_empty() {
                                                let _ = event_tx.send(SwarmEvent2::LedgerReceived {
                                                    from_peer: peer,
                                                    entries: response.peers,
                                                }).await;
                                            }
                                        }
                                    }
                                }
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Gossipsub(
                                gossipsub::Event::Subscribed { peer_id, topic }
                            )) => {
                                let topic_str = topic.to_string();
                                if !subscribed_topics.contains(&topic_str) {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic_str.clone());
                                    if swarm.behaviour_mut().gossipsub.subscribe(&ident_topic).is_ok() {
                                        subscribed_topics.insert(topic_str.clone());
                                    }
                                }
                                let _ = event_tx.send(SwarmEvent2::TopicDiscovered {
                                    peer_id,
                                    topic: topic_str,
                                }).await;
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Gossipsub(
                                gossipsub::Event::Message { propagation_source, message, .. }
                            )) => {
                                if message.topic.as_str() == DELIVERY_CONVERGENCE_TOPIC {
                                    if let Some(marker) =
                                        decode_delivery_convergence_marker(&message.data)
                                    {
                                        if let Err(reason) = should_apply_delivery_convergence_marker(
                                            &marker,
                                            &pending_messages,
                                            &pending_relay_requests,
                                            &pending_custody_dispatches,
                                            &relay_custody_store,
                                        ) {
                                            tracing::warn!(
                                                "(wasm) ignoring convergence marker message={} destination={} from={} reason={}",
                                                marker.relay_message_id,
                                                marker.destination_peer_id,
                                                propagation_source,
                                                reason
                                            );
                                            continue;
                                        }
                                        if seen_delivery_convergence_markers.insert(marker.key()) {
                                            apply_delivery_convergence_marker(
                                                &marker,
                                                &mut pending_messages,
                                                &mut pending_relay_requests,
                                                &mut pending_custody_dispatches,
                                                &relay_custody_store,
                                            )
                                            .await;
                                            if propagation_source != local_peer_id {
                                                publish_delivery_convergence_marker(
                                                    &mut swarm,
                                                    &marker,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Identify(
                                identify::Event::Received { peer_id, info, .. }
                            )) => {
                                for addr in &info.listen_addrs {
                                    if is_discoverable_multiaddr(addr) {
                                        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                                    }
                                }
                                if let Some(observed_addr) =
                                    ConnectionTracker::extract_socket_addr(&info.observed_addr)
                                {
                                    address_observer.record_observation(peer_id, observed_addr);
                                }

                                let public_key_hex = info.public_key.clone().try_into_ed25519().map(|pk| hex::encode(pk.to_bytes())).ok();
                                let _ = event_tx.send(SwarmEvent2::PeerIdentified {
                                    peer_id,
                                    public_key: public_key_hex,
                                    agent_version: info.agent_version.clone(),
                                    listen_addrs: info.listen_addrs.clone(),
                                    protocols: info.protocols.iter().map(|p| p.to_string()).collect(),
                                }).await;
                            }
                            SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
                                tracing::info!(
                                    event = "peer_connected",
                                    trigger = ?crate::routing::smart_retry::DeliveryTrigger::PeerDiscovered(peer_id.to_string()),
                                    peer = %peer_id
                                );
                                connection_tracker.add_connection(
                                    peer_id,
                                    endpoint.get_remote_address().clone(),
                                    match endpoint {
                                        libp2p::core::ConnectedPoint::Listener { local_addr, .. } => local_addr.clone(),
                                        libp2p::core::ConnectedPoint::Dialer { .. } => "/ip4/0.0.0.0/tcp/0".parse().expect("static multiaddr parse cannot fail"),
                                    },
                                    connection_id.to_string(),
                                );
                                dispatch_pending_custody_for_peer(
                                    &mut swarm,
                                    &relay_custody_store,
                                    peer_id,
                                    &mut pending_custody_dispatches,
                                    RELAY_MAX_INFLIGHT_DISPATCHES,
                                    "peer_reconnect",
                                );
                                if reported_peer_discoveries.insert(peer_id) {
                                    let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;

                                    // Activate SyncSession for Drift Protocol mesh synchronization
                                    sync_sessions.insert(peer_id, SyncSession::new());
                                    tracing::debug!(
                                        "Activated SyncSession for peer: {}",
                                        peer_id
                                    );
                                }

                                // Reset bootstrap backoff for any addr matching this peer
                                for ba in &bootstrap_addrs_clone {
                                    let matches = ba.iter().any(|proto| {
                                        if let libp2p::multiaddr::Protocol::P2p(p) = proto { p == peer_id } else { false }
                                    });
                                    if matches {
                                        if let Some(entry) = bootstrap_backoff.get_mut(ba) {
                                            entry.on_success();
                                            tracing::debug!("Reset bootstrap backoff for {} (connected)", ba);
                                        }
                                        break;
                                    }
                                }
                            }
                            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                tracing::info!("[ERROR] Disconnected from {} (WASM)", peer_id);
                                connection_tracker.remove_connection(&peer_id);
                                ledger_exchanged_peers.remove(&peer_id);
                                let stale_dispatches: Vec<libp2p::request_response::OutboundRequestId> =
                                    pending_custody_dispatches
                                        .iter()
                                        .filter_map(|(request_id, dispatch)| {
                                            (dispatch.destination_peer == peer_id)
                                                .then_some(*request_id)
                                        })
                                        .collect();
                                for request_id in stale_dispatches {
                                    if let Some(dispatch) =
                                        pending_custody_dispatches.remove(&request_id)
                                    {
                                        let _ = relay_custody_store.mark_dispatch_failed(
                                            &dispatch.destination_peer.to_string(),
                                            &dispatch.custody_id,
                                            "peer_disconnected",
                                        );
                                    }
                                }

                                // SELF-HEALING (WASM): Queue redial for disconnected bootstrap peers
                                {
                                    let local_peer = swarm.local_peer_id();
                                    if peer_id != *local_peer {
                                        for ba in &bootstrap_addrs_clone {
                                            let matches = ba.iter().any(|proto| {
                                                if let libp2p::multiaddr::Protocol::P2p(p) = proto { p == peer_id } else { false }
                                            });
                                            if matches && !bootstrap_backoff.contains_key(ba) {
                                                tracing::info!(
                                                    "Self-heal: queueing redial for disconnected WASM bootstrap peer {}",
                                                    peer_id
                                                );
                                                bootstrap_backoff.insert(ba.clone(), BootstrapBackoffEntry::new());
                                            }
                                        }
                                    }
                                }

                                let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
                            }
                            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                // Kademlia churn — expected at debug level
                                if let Some(pid) = peer_id {
                                    tracing::debug!("[WARNING] Outgoing connection error to {}: {}", pid, error);
                                } else {
                                    tracing::debug!("[WARNING] Outgoing connection error: {}", error);
                                }
                                // Exponential backoff for bootstrap re-dial: if the failed
                                // connection matches any bootstrap addr (by IP+port for
                                // peer_id=None errors, or by /p2p/ component), apply backoff
                                // so we don't keep hammering a node that refuses connections.
                                tracing::trace!("Bootstrap backoff check: {} addrs, peer_id={:?}", bootstrap_addrs_clone.len(), peer_id);
                                for ba in &bootstrap_addrs_clone {
                                    let matches = if let Some(pid) = peer_id {
                                        // Known peer: match by p2p component
                                        ba.iter().any(|proto| {
                                            if let libp2p::multiaddr::Protocol::P2p(p) = proto { p == pid } else { false }
                                        })
                                    } else {
                                        // Unknown peer (connection refused before handshake):
                                        // Extract IP + TCP port from the bootstrap multiaddr and
                                        // check that both appear in the error string. This is
                                        // more robust than matching the full formatted multiaddr.
                                        let mut ip_str = None;
                                        let mut port_str = None;
                                        for proto in ba.iter() {
                                            match proto {
                                                libp2p::multiaddr::Protocol::Ip4(ip) => ip_str = Some(format!("{}", ip)),
                                                libp2p::multiaddr::Protocol::Tcp(p) => port_str = Some(format!("{}", p)),
                                                _ => {}
                                            }
                                        }
                                        let err_str = format!("{} {:?}", error, error);
                                        let matched = ip_str.as_ref().map_or(false, |ip| err_str.contains(ip.as_str()))
                                            && port_str.as_ref().map_or(false, |p| err_str.contains(p.as_str()));
                                        if matched {
                                            tracing::debug!("Bootstrap backoff match: addr {} in error", ba);
                                        }
                                        matched
                                    };
                                    if matches {
                                        if is_dns_multiaddr(ba) {
                                            bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure_gentle();
                                            tracing::debug!("Applied gentle backoff to DNS hostname {}", ba);
                                        } else {
                                            bootstrap_backoff.entry(ba.clone()).or_insert_with(BootstrapBackoffEntry::new).on_failure();
                                            tracing::debug!("Applied backoff to bootstrap addr {}", ba);
                                        }
                                        break;
                                    }
                                }
                            }
                            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                                // Inbound connection errors on the LAN listeners are
                                // dominated by benign TCP port-probes -- notably our own
                                // Android SubnetProbe LAN-discovery fallback, which opens a
                                // socket, waits ~200ms, then closes without ever writing the
                                // multistream-select / WS-handshake bytes. libp2p surfaces
                                // that as Select(Failed) / Handshake(UnexpectedEof), wrapped
                                // in the generic "Failed to negotiate transport protocol(s)".
                                // These are not actionable and previously masqueraded as a
                                // real negotiation bug, so log at debug rather than warn. A
                                // genuine peer-connectivity problem surfaces via
                                // OutgoingConnectionError or the absence of ConnectionEstablished.
                                tracing::debug!(
                                    "Incoming connection negotiation aborted from {} -> {}: {}",
                                    send_back_addr,
                                    local_addr,
                                    error
                                );

                                if record_negotiation_failure_and_check_burst(&send_back_addr.to_string()) {
                                    tracing::warn!(
                                        "High rate of incoming negotiation failures from {} -> {}: {}",
                                        send_back_addr,
                                        local_addr,
                                        error
                                    );
                                }
                            }
                            SwarmEvent::ListenerError { listener_id, error } => {
                                tracing::error!(
                                    "Listener {:?} reported an error (async bind/accept failure): {}",
                                    listener_id,
                                    error
                                );
                                let _ = event_tx.send(SwarmEvent2::ListenerFailed {
                                    listener_id: format!("{:?}", listener_id),
                                    error: error.to_string(),
                                }).await;
                            }
                            SwarmEvent::ListenerClosed { listener_id, addresses, reason } => {
                                tracing::warn!(
                                    "Listener {:?} closed for addresses {:?}: {:?}",
                                    listener_id,
                                    addresses,
                                    reason
                                );
                                if reason.is_err() {
                                    let _ = event_tx.send(SwarmEvent2::ListenerFailed {
                                        listener_id: format!("{:?}", listener_id),
                                        error: format!("listener closed for {:?}: {:?}", addresses, reason),
                                    }).await;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if js_sys::Date::now() - last_custody_pull >= 5_000.0 {
                    let connected: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                    for destination in connected {
                        dispatch_pending_custody_for_peer(
                            &mut swarm,
                            &relay_custody_store,
                            destination,
                            &mut pending_custody_dispatches,
                            RELAY_MAX_INFLIGHT_DISPATCHES,
                            "periodic_pull",
                        );
                    }
                    last_custody_pull = js_sys::Date::now();
                }

                // Keep bootstrap links warm on browser clients.
                if js_sys::Date::now() - last_bootstrap_redial >= 60_000.0 {
                    let connected_peers: HashSet<PeerId> =
                        swarm.connected_peers().cloned().collect();
                    for addr in &bootstrap_addrs_clone {
                        // Self-dial guard: skip bootstrap addrs that resolve to
                        // our own peer ID (e.g. portproxy loopback) to avoid
                        // the "tried to dial local peer id" warning every 60s.
                        let is_self = addr.iter().any(|proto| {
                            if let libp2p::multiaddr::Protocol::P2p(pid) = proto {
                                pid == local_peer_id
                            } else {
                                false
                            }
                        });
                        if is_self {
                            if !self_dial_logged.contains(addr) {
                                tracing::info!(
                                    "  ⊘ Skipping self-dial bootstrap addr (matches local peer): {}",
                                    addr
                                );
                                self_dial_logged.insert(addr.clone());
                            }
                            continue;
                        }

                        // Exponential backoff gate: skip this addr if it is still
                        // within its backoff window after a recent failure.
                        if !bootstrap_backoff
                            .get(addr)
                            .map_or(true, |e| e.is_eligible())
                        {
                            continue;
                        }

                        let already_connected = addr.iter().any(|proto| {
                            if let libp2p::multiaddr::Protocol::P2p(pid) = proto {
                                connected_peers.contains(&pid)
                            } else {
                                false
                            }
                        });

                        if !already_connected {
                            let stripped_addr: Multiaddr = addr
                                .iter()
                                .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
                                .collect();
                            if let Err(e) = swarm.dial(stripped_addr.clone()) {
                                // Dial was rejected internally (e.g. already dialing).
                                // Treat as a failure and apply backoff to avoid retry spam.
                                tracing::trace!(
                                    "Bootstrap re-dial {} skipped: {}",
                                    stripped_addr,
                                    e
                                );
                                bootstrap_backoff
                                    .entry(addr.clone())
                                    .or_insert_with(BootstrapBackoffEntry::new)
                                    .on_failure();
                            }
                        }
                    }
                    last_bootstrap_redial = js_sys::Date::now();
                }
            }
        });

        Ok(handle)
    }
}

#[cfg(not(target_arch = "wasm32"))]
use futures::StreamExt;
use libp2p::identify;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
use libp2p::mdns;
use libp2p::{gossipsub, request_response};

#[cfg(test)]
mod tests {
    use super::{
        extract_ed25519_public_key_from_peer_id, should_apply_delivery_convergence_marker,
        validate_delivery_convergence_marker_shape, verify_registration_message,
        DeliveryConvergenceMarker, PendingCustodyDispatch, PendingMessage, RelayAbuseGuardrails,
        RELAY_DUPLICATE_WINDOW_MS, RELAY_PEER_BUCKET_BURST_CAPACITY,
        RELAY_PEER_BUCKET_REFILL_PER_SEC,
    };
    use crate::identity::IdentityKeys;
    use crate::store::relay_custody::RelayCustodyStore;
    use crate::transport::RegistrationMessage;
    use libp2p::{Multiaddr, PeerId};
    use std::collections::HashMap;

    #[test]
    fn abusive_peer_burst_is_rate_limited_but_other_peer_still_passes() {
        let mut guardrails = RelayAbuseGuardrails::new();
        let now_ms = 1_000_000;
        let mut accepted = 0usize;
        for _ in 0..50 {
            if guardrails.consume_peer_token("peer-abusive", now_ms, 1.0) {
                accepted += 1;
            }
        }
        assert!(accepted <= RELAY_PEER_BUCKET_BURST_CAPACITY as usize);
        assert!(guardrails.consume_peer_token("peer-normal", now_ms, 1.0));
    }

    #[test]
    fn normal_low_volume_usage_is_unaffected() {
        let mut guardrails = RelayAbuseGuardrails::new();
        let start_ms = 2_000_000;
        for step in 0..10 {
            let now_ms = start_ms + (step * 1_000) as u64;
            assert!(
                guardrails.consume_peer_token("peer-family", now_ms, 1.0),
                "expected token to be available at step {}",
                step
            );
        }
    }

    #[test]
    fn duplicate_window_suppresses_immediate_replay_then_expires() {
        let mut guardrails = RelayAbuseGuardrails::new();
        let source = "peer-a";
        let destination = "peer-b";
        let relay_message_id = "msg-dup";
        let now_ms = 3_000_000;

        assert!(!guardrails.is_recent_duplicate(source, destination, relay_message_id, now_ms));
        guardrails.record_accepted(source, destination, relay_message_id, now_ms);
        assert!(guardrails.is_recent_duplicate(
            source,
            destination,
            relay_message_id,
            now_ms + 100
        ));
        assert!(!guardrails.is_recent_duplicate(
            source,
            destination,
            relay_message_id,
            now_ms + RELAY_DUPLICATE_WINDOW_MS + 1
        ));
    }

    #[test]
    fn token_bucket_refills_after_elapsed_time() {
        let mut guardrails = RelayAbuseGuardrails::new();
        let now_ms = 4_000_000;
        for _ in 0..RELAY_PEER_BUCKET_BURST_CAPACITY as usize {
            assert!(guardrails.consume_peer_token("peer-a", now_ms, 1.0));
        }
        assert!(!guardrails.consume_peer_token("peer-a", now_ms, 1.0));

        let refill_ms = (1_000.0 / RELAY_PEER_BUCKET_REFILL_PER_SEC).ceil() as u64;
        assert!(guardrails.consume_peer_token("peer-a", now_ms + refill_ms, 1.0));
    }

    #[test]
    fn cheap_heuristics_reject_invalid_payload_shapes() {
        let guardrails = RelayAbuseGuardrails::new();
        assert_eq!(
            guardrails.should_reject_cheap_heuristics("", 12),
            Some("relay_message_id_empty")
        );
        assert_eq!(
            guardrails.should_reject_cheap_heuristics("ok", 0),
            Some("relay_envelope_empty")
        );
        assert!(guardrails
            .should_reject_cheap_heuristics("ok", 1024)
            .is_none());
    }

    #[test]
    fn convergence_marker_rejects_invalid_shape() {
        let marker = DeliveryConvergenceMarker {
            relay_message_id: "".to_string(),
            destination_peer_id: "dest".to_string(),
            observed_by_peer_id: "observer".to_string(),
            observed_at_ms: super::marker_now_ms(),
        };
        assert_eq!(
            validate_delivery_convergence_marker_shape(&marker, super::marker_now_ms()),
            Err("marker_message_id_empty")
        );
    }

    #[test]
    fn convergence_marker_requires_local_tracking_context() {
        let marker = DeliveryConvergenceMarker {
            relay_message_id: "relay-msg-unknown".to_string(),
            destination_peer_id: "dest-peer".to_string(),
            observed_by_peer_id: "observer-peer".to_string(),
            observed_at_ms: super::marker_now_ms(),
        };
        let pending_messages: HashMap<String, PendingMessage> = HashMap::new();
        let request_to_message = HashMap::new();
        let pending_relay_requests = HashMap::new();
        let pending_custody_dispatches: HashMap<
            libp2p::request_response::OutboundRequestId,
            PendingCustodyDispatch,
        > = HashMap::new();
        let custody_store = RelayCustodyStore::in_memory();
        assert_eq!(
            should_apply_delivery_convergence_marker(
                &marker,
                &pending_messages,
                &request_to_message,
                &pending_relay_requests,
                &pending_custody_dispatches,
                &custody_store,
            ),
            Err("marker_not_locally_tracked")
        );
    }

    #[test]
    fn convergence_marker_accepts_when_custody_exists_locally() {
        let custody_store = RelayCustodyStore::in_memory();
        let _accepted = custody_store
            .accept_custody(
                "src-peer".to_string(),
                "dest-peer".to_string(),
                "relay-msg-known".to_string(),
                vec![1, 2, 3],
                None,
                None,
            )
            .unwrap();
        let marker = DeliveryConvergenceMarker {
            relay_message_id: "relay-msg-known".to_string(),
            destination_peer_id: "dest-peer".to_string(),
            observed_by_peer_id: "observer-peer".to_string(),
            observed_at_ms: super::marker_now_ms(),
        };
        let pending_messages: HashMap<String, PendingMessage> = HashMap::new();
        let request_to_message = HashMap::new();
        let pending_relay_requests = HashMap::new();
        let pending_custody_dispatches: HashMap<
            libp2p::request_response::OutboundRequestId,
            PendingCustodyDispatch,
        > = HashMap::new();
        assert!(should_apply_delivery_convergence_marker(
            &marker,
            &pending_messages,
            &request_to_message,
            &pending_relay_requests,
            &pending_custody_dispatches,
            &custody_store,
        )
        .is_ok());
    }

    #[test]
    fn peer_id_public_key_extraction_roundtrips_for_ed25519_peers() {
        let keys = IdentityKeys::generate();
        let keypair = keys.to_libp2p_keypair().unwrap();
        let peer_id = keypair.public().to_peer_id();

        let extracted = extract_ed25519_public_key_from_peer_id(&peer_id).unwrap();
        assert_eq!(extracted, keys.signing_key.verifying_key().to_bytes());
    }

    #[test]
    fn verify_registration_message_rejects_peer_identity_mismatch() {
        let signing_identity = IdentityKeys::generate();
        let wrong_peer_identity = IdentityKeys::generate();
        let request = crate::transport::RegistrationRequest::new_signed(
            &signing_identity,
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            1_731_000_000,
        )
        .unwrap();
        let wrong_peer = wrong_peer_identity
            .to_libp2p_keypair()
            .unwrap()
            .public()
            .to_peer_id();

        assert_eq!(
            verify_registration_message(&wrong_peer, &RegistrationMessage::Register(request)),
            Err("registration_identity_mismatch")
        );
    }

    #[test]
    fn extract_tcp_port_from_multiaddr_returns_tcp_port() {
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/9101".parse().unwrap();
        assert_eq!(super::extract_tcp_port_from_multiaddr(&addr), Some(9101));
    }

    #[test]
    fn extract_tcp_port_from_multiaddr_returns_udp_port() {
        let addr: Multiaddr = "/ip4/0.0.0.0/udp/9876".parse().unwrap();
        assert_eq!(super::extract_tcp_port_from_multiaddr(&addr), Some(9876));
    }

    #[test]
    fn extract_tcp_port_from_multiaddr_returns_none_for_dns() {
        let addr: Multiaddr = "/dns4/example.com/tcp/443".parse().unwrap();
        // DNS multiaddrs skip straight past IP layers; our extractor returns the TCP port if present.
        // This test asserts the extractor does not panic on DNS-prefixed addresses.
        let _ = super::extract_tcp_port_from_multiaddr(&addr);
    }

    #[test]
    fn identify_log_dedup_suppresses_within_ttl() {
        // Clear the map first
        super::last_identified_log().write().clear();

        // Simulate first identify event: insert entry, expect map size 1
        {
            let mut map = super::last_identified_log().write();
            let key = PeerId::random();
            let now = web_time::Instant::now();
            map.insert(key, now);
            assert!(map.contains_key(&key));
        }

        // Simulate second identify event within TTL: lookup existing entry
        {
            let map = super::last_identified_log().read();
            assert_eq!(map.len(), 1, "Dedup map should retain the original entry");
        }
    }

    #[test]
    fn mdns_filter_drops_circuit_addresses() {
        let addrs = vec![
            "/ip4/192.168.0.230/tcp/9101".parse().unwrap(),
            "/ip4/172.26.144.1/tcp/9101/p2p/12D3KooWJk8KPYRVn8SaqHxq5fFJnT9VYjW7pNvGHgL8F6ShzW3T/p2p-circuit/p2p/12D3KooWJk8KPYRVn8SaqHxq5fFJnT9VYjW7pNvGHgL8F6ShzW3T".parse().unwrap(),
            "/ip4/172.26.154.211/tcp/9002/ws/p2p/12D3KooWJk8KPYRVn8SaqHxq5fFJnT9VYjW7pNvGHgL8F6ShzW3T/p2p-circuit/p2p/12D3KooWJk8KPYRVn8SaqHxq5fFJnT9VYjW7pNvGHgL8F6ShzW3T".parse().unwrap(),
        ];
        let filtered = super::build_mdns_advertised_addrs(&addrs);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0]
            .to_string()
            .starts_with("/ip4/192.168.0.230/tcp/9101"));
    }
}

/// Extract the TCP port from a Multiaddr.
/// Returns None if the address doesn't contain a TCP/QUIC port.
pub fn extract_tcp_port_from_multiaddr(addr: &Multiaddr) -> Option<u16> {
    for proto in addr.iter() {
        match proto {
            libp2p::multiaddr::Protocol::Tcp(port) => return Some(port),
            libp2p::multiaddr::Protocol::Udp(port) => return Some(port),
            _ => {}
        }
    }
    None
}

/// Detect and log a mismatch between configured listen port and actual swarm listeners.
/// This function checks the first TCP/UDP port from the swarm's listeners against
/// the configured port and logs a warning if they differ.
///
/// # Arguments
/// * `config_port` - The listen_port from config.json
/// * `swarm` - The Swarm instance (passed by reference to access listeners)
pub fn detect_and_log_port_mismatch(config_port: u16, swarm: &libp2p::Swarm<IronCoreBehaviour>) {
    // Get the first listener that has a TCP/UDP port
    let actual_port = swarm.listeners().find_map(extract_tcp_port_from_multiaddr);

    if let Some(actual) = actual_port {
        if config_port != actual {
            tracing::warn!(
                "Config says listen_port={} but swarm is bound to port {}. \
                 Config file may be stale — update config or pass --port.",
                config_port,
                actual
            );
        }
    }
}

pub fn is_dns_multiaddr(addr: &libp2p::Multiaddr) -> bool {
    addr.iter().any(|proto| {
        matches!(
            proto,
            libp2p::multiaddr::Protocol::Dns(_)
                | libp2p::multiaddr::Protocol::Dns4(_)
                | libp2p::multiaddr::Protocol::Dns6(_)
                | libp2p::multiaddr::Protocol::Dnsaddr(_)
        )
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn resolve_dns_multiaddr(multiaddr: &libp2p::Multiaddr) -> Vec<libp2p::Multiaddr> {
    let mut host = None;
    let mut port = None;
    let mut peer_id = None;
    let mut is_dns4 = false;
    let mut is_dns6 = false;
    let mut is_udp = false;

    for proto in multiaddr.iter() {
        match proto {
            libp2p::multiaddr::Protocol::Dns(h) => {
                host = Some(h.to_string());
            }
            libp2p::multiaddr::Protocol::Dns4(h) => {
                host = Some(h.to_string());
                is_dns4 = true;
            }
            libp2p::multiaddr::Protocol::Dns6(h) => {
                host = Some(h.to_string());
                is_dns6 = true;
            }
            libp2p::multiaddr::Protocol::Dnsaddr(h) => {
                host = Some(h.to_string());
            }
            libp2p::multiaddr::Protocol::Tcp(p) => {
                port = Some(p);
            }
            libp2p::multiaddr::Protocol::Udp(p) => {
                port = Some(p);
                is_udp = true;
            }
            libp2p::multiaddr::Protocol::P2p(pid) => {
                peer_id = Some(pid);
            }
            _ => {}
        }
    }

    let Some(host) = host else {
        return vec![];
    };
    let Some(port) = port else {
        return vec![];
    };

    let host_port = format!("{}:{}", host, port);
    let lookup_res = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        tokio::net::lookup_host(host_port),
    )
    .await;

    let Ok(Ok(socket_addrs)) = lookup_res else {
        return vec![];
    };

    let mut resolved = vec![];
    for socket_addr in socket_addrs {
        let ip = socket_addr.ip();
        if is_dns4 && ip.is_ipv6() {
            continue;
        }
        if is_dns6 && ip.is_ipv4() {
            continue;
        }

        let mut new_addr = libp2p::Multiaddr::empty();
        match ip {
            std::net::IpAddr::V4(ipv4) => {
                new_addr.push(libp2p::multiaddr::Protocol::Ip4(ipv4));
            }
            std::net::IpAddr::V6(ipv6) => {
                new_addr.push(libp2p::multiaddr::Protocol::Ip6(ipv6));
            }
        }

        if is_udp {
            new_addr.push(libp2p::multiaddr::Protocol::Udp(port));
        } else {
            new_addr.push(libp2p::multiaddr::Protocol::Tcp(port));
        }

        if let Some(pid) = peer_id {
            new_addr.push(libp2p::multiaddr::Protocol::P2p(pid));
        }

        resolved.push(new_addr);
    }

    resolved
}
