// libp2p swarm setup — Aggressive Discovery Mode
//
// Philosophy: "A node is a node." All nodes are mandatory relays.
// Connectivity takes priority over strict identity or topic matching.
//
// This creates and manages the libp2p Swarm with:
// - TCP transport (QUIC can be added later)
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
    IronCoreBehaviour, LedgerExchangeRequest, LedgerExchangeResponse, MessageRequest,
    MessageResponse, RelayResponse, SharedPeerEntry,
};
#[cfg(not(target_arch = "wasm32"))]
use super::mesh_routing::{
    advance_route_cursor, BootstrapCapability, MultiPathDelivery, RankedRoute,
};
#[cfg(target_arch = "wasm32")]
use super::multiport::MultiPortConfig;
#[cfg(not(target_arch = "wasm32"))]
use super::multiport::{self, BindResult, MultiPortConfig};
use super::observation::{AddressObserver, ConnectionTracker};
use super::reflection::{AddressReflectionRequest, AddressReflectionService};
use crate::store::relay_custody::RelayCustodyStore;
use anyhow::Result;
use bincode;
use serde::{Deserialize, Serialize};
#[cfg(target_arch = "wasm32")]
use libp2p::Transport;
use libp2p::{identity::Keypair, kad, swarm::SwarmEvent, Multiaddr, PeerId};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::net::SocketAddr;
use std::time::SystemTime;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::mpsc;

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
    for proto in addr.iter() {
        match proto {
            Protocol::Ip4(ip) => {
                if ip.is_loopback() {
                    return false;
                } // 127.x
                if ip.is_unspecified() {
                    return false;
                } // 0.0.0.0
                  // We intentionally allow RFC1918 and CGNAT for local discovery
            }
            Protocol::Ip6(ip) => {
                if ip.is_loopback() {
                    return false;
                } // ::1
                if ip.is_unspecified() {
                    return false;
                } // ::
            }
            Protocol::P2pCircuit => {
                return false;
            } // relay circuits go through relay, not kad
            _ => {}
        }
    }
    true
}

#[cfg(not(target_arch = "wasm32"))]
const ROUTE_ATTEMPT_REASON_INITIAL_SEND: &str = "INITIAL_SEND";
#[cfg(not(target_arch = "wasm32"))]
const ROUTE_ATTEMPT_REASON_RETRY_NEXT: &str = "RETRY_NEXT_CANDIDATE";
#[cfg(not(target_arch = "wasm32"))]
const ROUTE_ATTEMPT_REASON_RETRY_CYCLE: &str = "RETRY_CYCLE_RESTART";
const DELIVERY_CONVERGENCE_TOPIC: &str = "sc-receipt-convergence";
const DELIVERY_CONVERGENCE_PREFIX: &[u8] = b"scm.delivery.convergence.v1:";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeliveryConvergenceMarker {
    relay_message_id: String,
    destination_peer_id: String,
    observed_by_peer_id: String,
    observed_at_ms: u64,
}

impl DeliveryConvergenceMarker {
    fn key(&self) -> String {
        format!(
            "{}::{}",
            self.destination_peer_id, self.relay_message_id
        )
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
fn dispatch_ranked_route(
    swarm: &mut libp2p::Swarm<IronCoreBehaviour>,
    route: &RankedRoute,
    message_id: &str,
    target_peer: PeerId,
    envelope_data: &[u8],
    request_to_message: &mut HashMap<libp2p::request_response::OutboundRequestId, String>,
    pending_relay_requests: &mut HashMap<libp2p::request_response::OutboundRequestId, String>,
) {
    if route.path.len() == 1 {
        let request_id = swarm.behaviour_mut().messaging.send_request(
            &target_peer,
            MessageRequest {
                envelope_data: envelope_data.to_vec(),
            },
        );
        request_to_message.insert(request_id, message_id.to_string());
    } else {
        let relay_peer = route.path[0];
        let relay_request = RelayRequest {
            destination_peer: target_peer.to_bytes(),
            envelope_data: envelope_data.to_vec(),
            message_id: message_id.to_string(),
        };
        let request_id = swarm
            .behaviour_mut()
            .relay
            .send_request(&relay_peer, relay_request);
        pending_relay_requests.insert(request_id, message_id.to_string());
    }
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
            "✅ Delivery convergence applied: message={} destination={} direct_requests={} relay_requests={} dispatches={} custody={} retries_cleared={} pending_cleared={}",
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
            "✅ (wasm) delivery convergence applied: message={} destination={} relay_requests={} dispatches={} custody={} pending_cleared={}",
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
    trigger_reason: &str,
) {
    if !swarm.is_connected(&destination_peer) {
        return;
    }

    let destination_id = destination_peer.to_string();
    let pending = custody_store.pending_for_destination(&destination_id, 64);
    if pending.is_empty() {
        return;
    }

    for custody in pending {
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
            MessageRequest {
                envelope_data: custody.envelope_data.clone(),
            },
        );
        tracing::info!(
            "📦 Dispatching custody {} for relay message {} to {} via {}",
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
}

/// Commands that can be sent to the swarm task
#[derive(Debug)]
pub enum SwarmCommand {
    /// Send a message to a specific peer
    SendMessage {
        peer_id: PeerId,
        envelope_data: Vec<u8>,
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
    /// Get list of connected peers
    GetPeers { reply: mpsc::Sender<Vec<PeerId>> },
    /// Start listening on an address
    Listen {
        addr: Multiaddr,
        reply: mpsc::Sender<Result<Multiaddr, String>>,
    },
    /// Add a known peer address to Kademlia
    AddKadAddress { peer_id: PeerId, addr: Multiaddr },
    /// Subscribe to a Gossipsub topic
    SubscribeTopic { topic: String },
    /// Unsubscribe from a Gossipsub topic
    UnsubscribeTopic { topic: String },
    /// Publish payload to a Gossipsub topic
    PublishTopic { topic: String, data: Vec<u8> },
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
    /// A peer's identity was confirmed (after Identify protocol)
    PeerIdentified {
        peer_id: PeerId,
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
}

/// Handle to communicate with the running swarm task
#[derive(Clone)]
pub struct SwarmHandle {
    command_tx: mpsc::Sender<SwarmCommand>,
}

impl SwarmHandle {
    /// Send an encrypted envelope to a peer
    pub async fn send_message(&self, peer_id: PeerId, envelope_data: Vec<u8>) -> Result<()> {
        let (reply_tx, mut reply_rx) = mpsc::channel(1);
        self.command_tx
            .send(SwarmCommand::SendMessage {
                peer_id,
                envelope_data,
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

    /// Subscribe to a Gossipsub topic
    pub async fn subscribe_topic(&self, topic: String) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::SubscribeTopic { topic })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
    }

    /// Unsubscribe from a Gossipsub topic
    pub async fn unsubscribe_topic(&self, topic: String) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::UnsubscribeTopic { topic })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
    }

    /// Publish data to a Gossipsub topic
    pub async fn publish_topic(&self, topic: String, data: Vec<u8>) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::PublishTopic { topic, data })
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
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

    /// Shut down the swarm
    pub async fn shutdown(&self) -> Result<()> {
        self.command_tx
            .send(SwarmCommand::Shutdown)
            .await
            .map_err(|_| anyhow::anyhow!("Swarm task not running"))
    }
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
    headless: bool,
) -> Result<SwarmHandle> {
    start_swarm_with_config(keypair, listen_addr, event_tx, None, Vec::new(), headless).await
}

/// Build and start the libp2p swarm with custom multi-port configuration.
///
/// `bootstrap_addrs` — Multiaddrs of well-known relay / bootstrap nodes.
/// The swarm will auto-dial these after binding, enabling cross-network
/// peer discovery via Kademlia DHT and relay-circuit connectivity.
pub async fn start_swarm_with_config(
    keypair: Keypair,
    listen_addr: Option<Multiaddr>,
    event_tx: mpsc::Sender<SwarmEvent2>,
    multiport_config: Option<MultiPortConfig>,
    bootstrap_addrs: Vec<Multiaddr>,
    headless: bool,
) -> Result<SwarmHandle> {
    #[cfg(target_arch = "wasm32")]
    let _ = &multiport_config;
    #[cfg(target_arch = "wasm32")]
    let _ = headless;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let local_peer_id = keypair.public().to_peer_id();

        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_quic()
            .with_relay_client(libp2p::noise::Config::new, libp2p::yamux::Config::default)?
            .with_behaviour(|key, relay_client| {
                IronCoreBehaviour::new(key, relay_client, headless)
                    .expect("Failed to create network behaviour")
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(600))
                // 10 min idle (was 5 min)
            })
            .build();

        // Start listening on ports
        let mut bind_results = Vec::new();

        if let Some(config) = multiport_config {
            // Multi-port mode: Try binding to all configured ports
            tracing::info!("Starting multi-port adaptive listening");
            let addresses = multiport::generate_listen_addresses(&config);

            for (addr, port) in addresses {
                match swarm.listen_on(addr.clone()) {
                    Ok(_) => {
                        tracing::info!("✓ Bound to {}", addr);
                        bind_results.push(BindResult::Success { addr, port });
                    }
                    Err(e) => {
                        let error = e.to_string();
                        tracing::warn!("✗ Failed to bind to {} (port {}): {}", addr, port, error);
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
            let addr = listen_addr.unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".parse().unwrap());
            swarm.listen_on(addr)?;
        }

        // Always expose a QUIC listener for NAT traversal and future relay-circuit upgrades.
        if let Ok(quic_addr) = "/ip4/0.0.0.0/udp/0/quic-v1".parse::<Multiaddr>() {
            match swarm.listen_on(quic_addr.clone()) {
                Ok(_) => tracing::info!("✓ Bound QUIC listener {}", quic_addr),
                Err(e) => tracing::warn!("✗ Failed to bind QUIC listener {}: {}", quic_addr, e),
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
            tracing::info!("📡 Subscribed to lobby topic: sc-lobby");
        }

        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&mesh_topic) {
            tracing::warn!("Failed to subscribe to mesh topic: {}", e);
        } else {
            tracing::info!("📡 Subscribed to mesh topic: sc-mesh");
        }

        if let Err(e) = swarm
            .behaviour_mut()
            .gossipsub
            .subscribe(&delivery_convergence_topic)
        {
            tracing::warn!(
                "Failed to subscribe to delivery convergence topic: {}",
                e
            );
        } else {
            tracing::info!(
                "📡 Subscribed to delivery convergence topic: {}",
                DELIVERY_CONVERGENCE_TOPIC
            );
        }

        let (command_tx, mut command_rx) = mpsc::channel::<SwarmCommand>(256);
        let handle = SwarmHandle {
            command_tx: command_tx.clone(),
        };

        // Address reflection service
        let reflection_service = AddressReflectionService::new();

        // Track pending address reflection requests
        let mut pending_reflections: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<String, String>>,
        > = HashMap::new();

        // Track connections and address observations (Phase 1 & 2)
        let mut connection_tracker = ConnectionTracker::new();
        let mut address_observer = AddressObserver::new();

        // Mesh routing components (Phase 3-6)
        let mut multi_path_delivery = MultiPathDelivery::new();
        let mut bootstrap_capability = BootstrapCapability::new();

        // Track pending message deliveries
        let mut pending_messages: HashMap<String, PendingMessage> = HashMap::new();

        // Track outbound request IDs to message IDs for direct sends
        let mut request_to_message: HashMap<libp2p::request_response::OutboundRequestId, String> =
            HashMap::new();

        // Track outbound relay request IDs
        let mut pending_relay_requests: HashMap<
            libp2p::request_response::OutboundRequestId,
            String,
        > = HashMap::new();
        let relay_custody_store = RelayCustodyStore::for_local_peer(&local_peer_id.to_string());
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

        // Track relay peers and their publicly-routable addresses for circuit reservation.
        // When we identify a relay, we save its WAN addrs here and attempt
        // swarm.listen_on(<relay_addr>/p2p-circuit) to register a reservation,
        // which lets the relay dial us back on behalf of other nodes.
        let mut relay_peer_addrs: HashMap<PeerId, Vec<Multiaddr>> = HashMap::new();

        // Track relay reconnect backoff state: (peer_id, attempt_count, next_dial_at)
        let mut relay_reconnect_pending: Vec<(PeerId, u32, std::time::Instant)> = Vec::new();
        let mut seen_delivery_convergence_markers: HashSet<String> = HashSet::new();

        // Auto-dial bootstrap nodes for cross-network discovery
        if !bootstrap_addrs.is_empty() {
            tracing::info!(
                "🌐 Dialing {} bootstrap node(s) for NAT traversal",
                bootstrap_addrs.len()
            );
            for addr in &bootstrap_addrs {
                let stripped_addr: Multiaddr = addr
                    .iter()
                    .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
                    .collect();
                match swarm.dial(stripped_addr.clone()) {
                    Ok(_) => tracing::info!("  ✓ Dialing bootstrap: {}", stripped_addr),
                    Err(e) => {
                        tracing::warn!("  ✗ Failed to dial bootstrap {}: {}", stripped_addr, e)
                    }
                }
            }
        }

        // Spawn the swarm event loop
        tokio::spawn(async move {
            // PHASE 6: Retry interval for failed deliveries
            let mut retry_interval = tokio::time::interval(Duration::from_millis(500));

            // Bootstrap reconnection timer — re-dial bootstrap nodes every 60s
            // to handle network changes and maintain connectivity
            let mut bootstrap_reconnect_interval = tokio::time::interval(Duration::from_secs(60));
            let bootstrap_addrs_clone = bootstrap_addrs;

            // Cover traffic — 1 dummy message/min to mask real traffic patterns
            let mut cover_traffic_interval = tokio::time::interval(Duration::from_secs(60));

            // Relay budget rate-limiting
            let mut relay_budget: u32 = 200;
            let mut relay_count_this_hour: u32 = 0;
            let mut relay_hour_start = std::time::Instant::now();

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

                                    if elapsed >= retry_delay {
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
                                "periodic_pull",
                            );
                        }
                    }

                    // P0.11: Relay reconnect backoff processing
                    _ = relay_reconnect_interval.tick() => {
                        let now = std::time::Instant::now();
                        let mut next_pending = Vec::new();
                        let connected_peers: HashSet<PeerId> = swarm.connected_peers().cloned().collect();

                        for (peer_id, attempts, next_dial) in relay_reconnect_pending.drain(..) {
                            if connected_peers.contains(&peer_id) {
                                // Already connected; drop from pending queue
                                tracing::debug!("✅ Relay {} reconnected successfully", peer_id);
                                continue;
                            }

                            if now >= next_dial {
                                // Time to try dialing!
                                if let Some(addrs) = relay_peer_addrs.get(&peer_id) {
                                    if let Some(addr) = addrs.first() {
                                        tracing::info!(
                                            "🔄 Attempting to re-dial relay {} (Attempt {}): {}",
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
                                                tracing::warn!("⚠️ Re-dial to relay {} failed immediately: {}", peer_id, e);
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
                    // This handles network changes, dropped connections, and roaming
                    _ = bootstrap_reconnect_interval.tick() => {
                        if !bootstrap_addrs_clone.is_empty() {
                            let connected_peers: HashSet<PeerId> = swarm.connected_peers().cloned().collect();
                            for addr in &bootstrap_addrs_clone {
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
                                    match swarm.dial(stripped_addr.clone()) {
                                        Ok(_) => tracing::debug!("🔄 Re-dialing bootstrap: {}", stripped_addr),
                                        Err(e) => tracing::trace!("Bootstrap re-dial {} skipped: {}", stripped_addr, e),
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
                                        // Received a message from a peer
                                        let _ = event_tx.send(SwarmEvent2::MessageReceived {
                                            peer_id: peer,
                                            envelope_data: request.envelope_data,
                                        }).await;

                                        // Send acceptance response
                                        let _ = swarm.behaviour_mut().messaging.send_response(
                                            channel,
                                            MessageResponse { accepted: true, error: None },
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
                                                        "✅ Custody {} delivered to {} (relay message {})",
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
                                                    tracing::info!("✓ Message delivered successfully to {} ({}ms)", pending.target_peer, latency_ms);
                                                    let _ = pending.reply_tx.send(Ok(())).await;
                                                } else {
                                                    // Message rejected, trigger retry
                                                    tracing::warn!("✗ Message rejected by {}: {:?}", pending.target_peer, response.error);
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
                                            "✗ Direct send outbound failure to {}: {}",
                                            pending.target_peer,
                                            error
                                        );
                                        multi_path_delivery
                                            .record_failure(&message_id, vec![pending.target_peer]);
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
                                            .unwrap_or_else(|| "0.0.0.0:0".parse().unwrap());

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

                            // PHASE 3: Relay Protocol Handler — MANDATORY RELAY
                            // All nodes MUST relay. We never refuse a relay request
                            // (except for invalid destination).
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Relay(
                                request_response::Event::Message { peer, message, .. }
                            )) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        tracing::info!("🔄 Relay request from {} for message {}", peer, request.message_id);

                                        // Enforce relay budget — reset counter hourly
                                        if relay_hour_start.elapsed() >= std::time::Duration::from_secs(3600) {
                                            relay_count_this_hour = 0;
                                            relay_hour_start = std::time::Instant::now();
                                        }

                                        // Determine response; channel consumed exactly once at the end
                                        let relay_response = if relay_budget > 0 && relay_count_this_hour >= relay_budget {
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
                                        } else {
                                            relay_count_this_hour += 1;
                                            match PeerId::from_bytes(&request.destination_peer) {
                                                Ok(destination) => {
                                                    let relay_message_id = request.message_id.clone();
                                                    match relay_custody_store.accept_custody(
                                                        peer.to_string(),
                                                        destination.to_string(),
                                                        relay_message_id.clone(),
                                                        request.envelope_data.clone(),
                                                    ) {
                                                        Ok(custody) => {
                                                            if swarm.is_connected(&destination) {
                                                                dispatch_pending_custody_for_peer(
                                                                    &mut swarm,
                                                                    &relay_custody_store,
                                                                    destination,
                                                                    &mut pending_custody_dispatches,
                                                                    "accept_immediate_pull",
                                                                );
                                                            } else {
                                                                tracing::info!(
                                                                    "📦 Accepted custody {} for offline destination {} (relay message {})",
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
                                                    tracing::info!("✓ Message relayed successfully via {} to {} ({}ms)", peer, pending.target_peer, latency_ms);
                                                    let _ = pending.reply_tx.send(Ok(())).await;
                                                } else {
                                                    tracing::warn!("✗ Relay via {} failed: {:?}", peer, response.error);
                                                    multi_path_delivery.record_failure(&message_id, vec![peer, pending.target_peer]);
                                                    pending_messages.insert(message_id, pending);
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
                                            "✗ Relay outbound failure via {} to {}: {}",
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
                                            "📒 Ledger exchange from {}: received {} peer entries (v{})",
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
                                                        tracing::info!("📡 Auto-subscribed to topic from ledger: {}", topic_str);
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
                                            "📒 Ledger exchange response from {}: they learned {} new peers, sent {} back",
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
                                tracing::info!("📡 Peer {} subscribed to topic: {}", peer_id, topic_str);

                                // AUTO-NEGOTIATE: If a peer subscribes to a topic we don't know,
                                // subscribe to it ourselves. "A node is a node."
                                if !subscribed_topics.contains(&topic_str) {
                                    tracing::info!("🔄 Auto-subscribing to discovered topic: {}", topic_str);
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
                                    "📨 Gossipsub message from {} on topic {:?} ({} bytes)",
                                    propagation_source,
                                    message.topic,
                                    message.data.len()
                                );
                                if message.topic.as_str() == DELIVERY_CONVERGENCE_TOPIC {
                                    if let Some(marker) =
                                        decode_delivery_convergence_marker(&message.data)
                                    {
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
                                            "🔭 AutoNAT status: {:?} → {:?}",
                                            old, new
                                        );
                                        // Update NAT status for the application layer.
                                        // This determines whether relay fallback is required.
                                        let status_str = match new {
                                            autonat::NatStatus::Public(addr) => {
                                                tracing::info!("✅ AutoNAT: public reachability confirmed at {}", addr);
                                                format!("public:{}", addr)
                                            }
                                            autonat::NatStatus::Private => {
                                                tracing::info!("🔒 AutoNAT: behind NAT — relay required for inbound");
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
                                        tracing::debug!("AutoNAT outbound probe: {:?}", result);
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Dcutr(event)) => {
                                use libp2p::dcutr;
                                match event {
                                    dcutr::Event { remote_peer_id, result: Ok(num_attempts) } => {
                                        tracing::info!(
                                            "🕳️ DCUtR hole-punch SUCCESS with {} (attempts: {})",
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
                                        let _ = event_tx.send(SwarmEvent2::PeerDiscovered(remote_peer_id)).await;
                                    }
                                    dcutr::Event { remote_peer_id, result: Err(e) } => {
                                        tracing::warn!(
                                            "🕳️ DCUtR hole-punch FAILED with {} — will relay messages instead: {}",
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
                                                "🔄 Relay circuit reservation RENEWED via {}",
                                                relay_peer_id
                                            );
                                        } else {
                                            tracing::info!(
                                                "✅ Relay circuit reservation ACCEPTED via {} — inbound-relayed connections now possible",
                                                relay_peer_id
                                            );
                                        }
                                    }
                                    RelayClientEvent::InboundCircuitEstablished {
                                        src_peer_id,
                                        ..
                                    } => {
                                        tracing::info!(
                                            "🔌 Inbound relay circuit established from {} — peer connected through relay",
                                            src_peer_id
                                        );
                                    }
                                    RelayClientEvent::OutboundCircuitEstablished {
                                        relay_peer_id,
                                        ..
                                    } => {
                                        tracing::info!(
                                            "🔌 Outbound relay circuit established via {} — connected to remote through relay",
                                            relay_peer_id
                                        );
                                    }
                                }
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Ping(event)) => {
                                tracing::trace!("Ping event: {:?}", event);
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Mdns(
                                mdns::Event::Discovered(peers)
                            )) => {
                                for (peer_id, addr) in peers {
                                    tracing::info!("mDNS discovered peer: {} at {}", peer_id, addr);
                                    if is_discoverable_multiaddr(&addr) {
                                        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                    }

                                    bootstrap_capability.add_peer(peer_id);
                                    let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;
                                }
                            }

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
                                tracing::info!(
                                    "🆔 Identified peer {} — agent: {}, protocols: {}, addrs: {}",
                                    peer_id,
                                    info.agent_version,
                                    info.protocols.len(),
                                    info.listen_addrs.len()
                                );
                                // Identity protocol confirms this peer is presently reachable.
                                multi_path_delivery.record_recipient_seen_now(peer_id, peer_id);

                                // Relay-confirmed observation of our externally visible endpoint
                                // as seen by this peer. This gives mobile layers a stable
                                // "what the network sees" signal for publishing connection hints.
                                if let Some(observed_addr) =
                                    ConnectionTracker::extract_socket_addr(&info.observed_addr)
                                {
                                    address_observer.record_observation(peer_id, observed_addr);
                                    tracing::info!(
                                        "🌐 Identify observed address via {}: {}",
                                        peer_id,
                                        observed_addr
                                    );
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
                                        tracing::debug!("🚫 Skipping non-discoverable Kademlia addr for {}: {}", peer_id, addr);
                                    }
                                }

                                // Check if peer advertises relay capability
                                let is_relay = info.agent_version.contains("relay");
                                if is_relay {
                                    tracing::info!("🔄 Peer {} is a relay node", peer_id);
                                    bootstrap_capability.add_peer(peer_id);
                                    multi_path_delivery.add_relay(peer_id);

                                    // P0.5B: Register a circuit relay reservation with this relay.
                                    // Guard: only register ONCE per relay peer — identify fires every 60s
                                    // and without this guard we accumulate unbounded ListenerIds, which
                                    // floods the relay and crowds out real message delivery.
                                    let already_reserved = relay_peer_addrs.contains_key(&peer_id);

                                    if !already_reserved {
                                        let routable_relay_addrs: Vec<Multiaddr> = info.listen_addrs
                                            .iter()
                                            .filter(|a| is_discoverable_multiaddr(a))
                                            .cloned()
                                            .collect();

                                        if !routable_relay_addrs.is_empty() {
                                            relay_peer_addrs.insert(peer_id, routable_relay_addrs.clone());

                                            // Pick the first routable relay address and register a circuit reservation.
                                            // Format: /ip4/<relay-ip>/tcp/<port>/p2p/<relay-peer-id>/p2p-circuit
                                            use libp2p::multiaddr::Protocol;
                                            let relay_circuit_addr = routable_relay_addrs[0]
                                                .clone()
                                                .with(Protocol::P2p(peer_id))
                                                .with(Protocol::P2pCircuit);

                                            tracing::info!(
                                                "📡 Attempting relay circuit reservation via {}: {}",
                                                peer_id, relay_circuit_addr
                                            );
                                            match swarm.listen_on(relay_circuit_addr.clone()) {
                                                Ok(listener_id) => tracing::info!(
                                                    "✅ Relay circuit reservation registered: {:?} via {}",
                                                    listener_id, peer_id
                                                ),
                                                Err(e) => tracing::warn!(
                                                    "⚠️ Could not register relay circuit reservation via {}: {}",
                                                    peer_id, e
                                                ),
                                            }
                                        } else {
                                            tracing::debug!(
                                                "🔄 Relay {} has no WAN-routable addresses yet; \
                                                 will retry reservation after reconnect",
                                                peer_id
                                            );
                                        }
                                    } else {
                                        tracing::debug!(
                                            "📡 Relay circuit already reserved for {} — skipping duplicate",
                                            peer_id
                                        );
                                    }
                                }

                                // Emit event for application layer
                                let _ = event_tx.send(SwarmEvent2::PeerIdentified {
                                    peer_id,
                                    agent_version: info.agent_version.clone(),
                                    listen_addrs: info.listen_addrs.clone(),
                                    protocols: info.protocols.iter().map(|p| p.to_string()).collect(),
                                }).await;
                            }

                            SwarmEvent::NewListenAddr { address, .. } => {
                                tracing::info!("Listening on {}", address);
                                let _ = event_tx.send(SwarmEvent2::ListeningOn(address)).await;
                            }

                            SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
                                tracing::info!(
                                    "🔗 Connected to {} via {} (promiscuous mode — any PeerID accepted)",
                                    peer_id,
                                    endpoint.get_remote_address()
                                );
                                multi_path_delivery.record_recipient_seen_now(peer_id, peer_id);

                                // Track this connection for address observation
                                connection_tracker.add_connection(
                                    peer_id,
                                    endpoint.get_remote_address().clone(),
                                    match endpoint {
                                        libp2p::core::ConnectedPoint::Listener { local_addr, .. } => local_addr.clone(),
                                        libp2p::core::ConnectedPoint::Dialer { .. } => "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
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
                                    "peer_reconnect",
                                );

                                let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;

                                // AUTO LEDGER EXCHANGE: On every new connection, share our
                                // known peers. The application layer will receive
                                // SwarmEvent2::PeerDiscovered and trigger ShareLedger.
                                // This is handled in main.rs to keep swarm.rs agnostic
                                // about the persistent ledger format.
                            }

                            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                tracing::info!("❌ Disconnected from {}", peer_id);
                                connection_tracker.remove_connection(&peer_id);
                                // Allow re-exchange if they reconnect
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

                                // P0.11: If this was a known relay, schedule a reconnect with backoff.
                                // Also clear from relay_peer_addrs so that when reconnection succeeds,
                                // we re-register a fresh circuit reservation (old listener is now dead).
                                // Backoff: 10s → 30s → 60s → 60s (capped).
                                if relay_peer_addrs.remove(&peer_id).is_some() {
                                    tracing::info!(
                                        "🔄 Lost relay peer {}; cleared circuit reservation, scheduling reconnect",
                                        peer_id
                                    );
                                    relay_reconnect_pending.push((peer_id, 0, std::time::Instant::now()));
                                }

                                let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
                            }

                            // Handle outgoing connection errors gracefully — don't panic
                            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                // Downgraded to debug: Kademlia DHT explores many stale addresses
                                // from the routing table; timeouts here are expected churn, not
                                // actionable errors. Relay/identity failures surface at info/warn.
                                if let Some(pid) = peer_id {
                                    tracing::debug!("⚠ Outgoing connection error to {}: {}", pid, error);
                                } else {
                                    tracing::debug!("⚠ Outgoing connection error: {}", error);
                                }
                            }

                            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                                tracing::warn!(
                                    "⚠ Incoming connection error from {} -> {}: {}",
                                    send_back_addr,
                                    local_addr,
                                    error
                                );
                            }

                            _ => {}
                        }
                    }

                    // Process commands from the application layer
                    Some(command) = command_rx.recv() => {
                        match command {
                            SwarmCommand::SendMessage { peer_id, envelope_data, reply } => {
                                // PHASE 6: Multi-path delivery with retry logic
                                let message_id = format!("{}-{}", peer_id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());

                                // Start delivery tracking
                                multi_path_delivery.start_delivery(message_id.clone(), peer_id);

                                let routes = multi_path_delivery.ranked_routes(&peer_id, 3);
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
                                });
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

                            SwarmCommand::GetExternalAddresses { reply } => {
                                let addresses = address_observer.external_addresses().to_vec();
                                let _ = reply.send(addresses).await;
                            }

                            SwarmCommand::Dial { addr, reply } => {
                                tracing::info!("📞 Dialing {} (promiscuous — accepting any PeerID)", addr);
                                match swarm.dial(addr) {
                                    Ok(_) => { let _ = reply.send(Ok(())).await; }
                                    Err(e) => { let _ = reply.send(Err(e.to_string())).await; }
                                }
                            }

                            SwarmCommand::GetPeers { reply } => {
                                let peers: Vec<PeerId> = swarm.connected_peers().cloned().collect();
                                let _ = reply.send(peers).await;
                            }

                            SwarmCommand::Listen { addr, reply } => {
                                match swarm.listen_on(addr) {
                                    Ok(_) => {
                                        let _ = reply.send(Ok("/ip4/0.0.0.0/tcp/0".parse().unwrap())).await;
                                    }
                                    Err(e) => {
                                        let _ = reply.send(Err(e.to_string())).await;
                                    }
                                }
                            }

                            SwarmCommand::AddKadAddress { peer_id, addr } => {
                                if is_discoverable_multiaddr(&addr) {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                            }

                            SwarmCommand::SubscribeTopic { topic } => {
                                if !subscribed_topics.contains(&topic) {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&ident_topic) {
                                        tracing::warn!("Failed to subscribe to topic {}: {}", topic, e);
                                    } else {
                                        tracing::info!("📡 Subscribed to topic: {}", topic);
                                        subscribed_topics.insert(topic);
                                    }
                                }
                            }

                            SwarmCommand::UnsubscribeTopic { topic } => {
                                if subscribed_topics.contains(&topic) {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    if let Err(e) = swarm.behaviour_mut().gossipsub.unsubscribe(&ident_topic) {
                                        tracing::warn!("Failed to unsubscribe from topic {}: {}", topic, e);
                                    } else {
                                        tracing::info!("📡 Unsubscribed from topic: {}", topic);
                                        subscribed_topics.remove(&topic);
                                    }
                                }
                            }

                            SwarmCommand::PublishTopic { topic, data } => {
                                let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(ident_topic, data) {
                                    tracing::warn!("Failed to publish to topic {}: {}", topic, e);
                                } else {
                                    tracing::debug!("Published payload to topic {}", topic);
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
                                        "📒 Sharing ledger with {} ({} entries)",
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
                                    tracing::debug!("📒 Already exchanged ledger with {}, skipping", peer_id);
                                }
                            }

                            SwarmCommand::GetListeners { reply } => {
                    let listeners: Vec<Multiaddr> = swarm.listeners().cloned().collect();
                    let _ = reply.send(listeners).await;
                }
                            SwarmCommand::SetRelayBudget { budget } => {
                                relay_budget = budget;
                                tracing::info!("🔄 Relay budget updated: {} msgs/hour", budget);
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
    {
        use futures::{FutureExt, StreamExt};
        use libp2p::core::{muxing::StreamMuxerBox, upgrade::Version};

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
                IronCoreBehaviour::new(key, relay_client, false)
                    .expect("Failed to create network behaviour")
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(600))
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
            tracing::warn!(
                "Failed to subscribe to delivery convergence topic: {}",
                e
            );
        }

        // Kademlia server mode parity with native.
        swarm
            .behaviour_mut()
            .kademlia
            .set_mode(Some(kad::Mode::Server));

        // Auto-dial bootstrap nodes for internet connectivity.
        if !bootstrap_addrs.is_empty() {
            tracing::info!(
                "🌐 Dialing {} bootstrap node(s) from wasm",
                bootstrap_addrs.len()
            );
            for addr in &bootstrap_addrs {
                let stripped_addr: Multiaddr = addr
                    .iter()
                    .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
                    .collect();
                match swarm.dial(stripped_addr.clone()) {
                    Ok(_) => tracing::info!("  ✓ Dialing bootstrap: {}", stripped_addr),
                    Err(e) => {
                        tracing::warn!("  ✗ Failed to dial bootstrap {}: {}", stripped_addr, e)
                    }
                }
            }
        }

        let (command_tx, mut command_rx) = mpsc::channel::<SwarmCommand>(256);
        let handle = SwarmHandle {
            command_tx: command_tx.clone(),
        };

        let mut pending_direct_replies: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<(), String>>,
        > = HashMap::new();

        let mut pending_reflections: HashMap<
            libp2p::request_response::OutboundRequestId,
            mpsc::Sender<Result<String, String>>,
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
        // `std::time::Instant` panics on wasm32-unknown-unknown; use
        // `js_sys::Date::now()` (f64 ms since epoch) instead.
        let mut relay_hour_start: f64 = js_sys::Date::now();
        let mut last_bootstrap_redial: f64 = js_sys::Date::now();
        let mut last_custody_pull: f64 = js_sys::Date::now();
        let mut seen_delivery_convergence_markers: HashSet<String> = HashSet::new();
        let bootstrap_addrs_clone = bootstrap_addrs;

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
                            SwarmCommand::SendMessage { peer_id, envelope_data, reply } => {
                                let request_id = swarm.behaviour_mut().messaging.send_request(
                                    &peer_id,
                                    MessageRequest { envelope_data },
                                );
                                pending_direct_replies.insert(request_id, reply);
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
                            SwarmCommand::GetExternalAddresses { reply } => {
                                let addresses = address_observer.external_addresses().to_vec();
                                let _ = reply.send(addresses).await;
                            }
                            SwarmCommand::Dial { addr, reply } => {
                                match swarm.dial(addr) {
                                    Ok(_) => { let _ = reply.send(Ok(())).await; }
                                    Err(e) => { let _ = reply.send(Err(e.to_string())).await; }
                                }
                            }
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
                            SwarmCommand::SubscribeTopic { topic } => {
                                if !subscribed_topics.contains(&topic) {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    if swarm.behaviour_mut().gossipsub.subscribe(&ident_topic).is_ok() {
                                        subscribed_topics.insert(topic);
                                    }
                                }
                            }
                            SwarmCommand::UnsubscribeTopic { topic } => {
                                if subscribed_topics.contains(&topic) {
                                    let ident_topic = libp2p::gossipsub::IdentTopic::new(topic.clone());
                                    if swarm.behaviour_mut().gossipsub.unsubscribe(&ident_topic).is_ok() {
                                        subscribed_topics.remove(&topic);
                                    }
                                }
                            }
                            SwarmCommand::PublishTopic { topic, data } => {
                                let ident_topic = libp2p::gossipsub::IdentTopic::new(topic);
                                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(ident_topic, data) {
                                    tracing::warn!("Failed to publish topic payload: {}", e);
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
                            SwarmCommand::SetRelayBudget { budget } => {
                                relay_budget = budget;
                                tracing::info!("🔄 Relay budget updated: {} msgs/hour", budget);
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
                                            let _ = event_tx.send(SwarmEvent2::MessageReceived {
                                                peer_id: peer,
                                                envelope_data: request.envelope_data,
                                            }).await;

                                            let _ = swarm.behaviour_mut().messaging.send_response(
                                                channel,
                                                MessageResponse { accepted: true, error: None },
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
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::AddressReflection(ev)) => {
                                match ev {
                                    request_response::Event::Message { peer, message, .. } => match message {
                                        request_response::Message::Request { request, channel, .. } => {
                                            let observed_addr = connection_tracker
                                                .get_connection(&peer)
                                                .and_then(|conn| ConnectionTracker::extract_socket_addr(&conn.remote_addr))
                                                .unwrap_or_else(|| "0.0.0.0:0".parse().unwrap());

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
                                            if js_sys::Date::now() - relay_hour_start >= 3_600_000.0 {
                                                relay_count_this_hour = 0;
                                                relay_hour_start = js_sys::Date::now();
                                            }

                                            let relay_response = if relay_budget > 0 && relay_count_this_hour >= relay_budget {
                                                RelayResponse {
                                                    accepted: false,
                                                    error: Some("relay_budget_exhausted".to_string()),
                                                    message_id: request.message_id.clone(),
                                                }
                                            } else {
                                                relay_count_this_hour += 1;
                                                match PeerId::from_bytes(&request.destination_peer) {
                                                    Ok(destination) => {
                                                        let relay_message_id = request.message_id.clone();
                                                        match relay_custody_store.accept_custody(
                                                            peer.to_string(),
                                                            destination.to_string(),
                                                            relay_message_id.clone(),
                                                            request.envelope_data.clone(),
                                                        ) {
                                                            Ok(_) => {
                                                                if swarm.is_connected(&destination) {
                                                                    dispatch_pending_custody_for_peer(
                                                                        &mut swarm,
                                                                        &relay_custody_store,
                                                                        destination,
                                                                        &mut pending_custody_dispatches,
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
                                                        let _ = pending.reply_tx.send(Err(
                                                            response.error.unwrap_or_else(|| "relay rejected".to_string())
                                                        )).await;
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

                                let _ = event_tx.send(SwarmEvent2::PeerIdentified {
                                    peer_id,
                                    agent_version: info.agent_version.clone(),
                                    listen_addrs: info.listen_addrs.clone(),
                                    protocols: info.protocols.iter().map(|p| p.to_string()).collect(),
                                }).await;
                            }
                            SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
                                connection_tracker.add_connection(
                                    peer_id,
                                    endpoint.get_remote_address().clone(),
                                    match endpoint {
                                        libp2p::core::ConnectedPoint::Listener { local_addr, .. } => local_addr.clone(),
                                        libp2p::core::ConnectedPoint::Dialer { .. } => "/ip4/0.0.0.0/tcp/0".parse().unwrap(),
                                    },
                                    connection_id.to_string(),
                                );
                                dispatch_pending_custody_for_peer(
                                    &mut swarm,
                                    &relay_custody_store,
                                    peer_id,
                                    &mut pending_custody_dispatches,
                                    "peer_reconnect",
                                );
                                let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;
                            }
                            SwarmEvent::ConnectionClosed { peer_id, .. } => {
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
                                let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
                            }
                            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                // Kademlia churn — expected at debug level
                                if let Some(pid) = peer_id {
                                    tracing::debug!("⚠ Outgoing connection error to {}: {}", pid, error);
                                } else {
                                    tracing::debug!("⚠ Outgoing connection error: {}", error);
                                }
                            }
                            SwarmEvent::IncomingConnectionError { local_addr, send_back_addr, error, .. } => {
                                tracing::warn!(
                                    "⚠ Incoming connection error from {} -> {}: {}",
                                    send_back_addr,
                                    local_addr,
                                    error
                                );
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
                                tracing::trace!(
                                    "Bootstrap re-dial {} skipped: {}",
                                    stripped_addr,
                                    e
                                );
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
#[cfg(not(target_arch = "wasm32"))]
use libp2p::mdns;
use libp2p::{gossipsub, request_response};
