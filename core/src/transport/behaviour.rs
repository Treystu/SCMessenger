// Combined NetworkBehaviour for Iron Core — Aggressive Discovery Mode
//
// Philosophy: "A node is a node." All nodes are mandatory relays.
//
// This combines all the libp2p protocols we need:
// - request_response: direct peer-to-peer message delivery
// - address_reflection: sovereign mesh address discovery (replaces STUN)
// - gossipsub: pub/sub — PERMISSIVE mode for dynamic topic negotiation
// - kademlia: DHT for peer discovery on WAN
// - mdns: peer discovery on LAN
// - identify: exchange peer metadata (advertises relay capability)
// - relay: NAT traversal — all nodes are mandatory relays
// - ledger_exchange: automatic peer list sharing for aggressive discovery

use super::reflection::{AddressReflectionRequest, AddressReflectionResponse};
#[cfg(not(target_arch = "wasm32"))]
use libp2p::mdns;
use libp2p::{
    autonat, dcutr, gossipsub, identify, kad, ping, relay,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use std::time::Duration;

/// The Iron Core network behaviour combining all protocols.
#[derive(NetworkBehaviour)]
pub struct IronCoreBehaviour {
    /// Circuit Relay v2 client for relay reservations and relayed dials.
    pub relay_client: relay::client::Behaviour,
    /// Direct connection upgrade through relay (hole punching).
    pub dcutr: dcutr::Behaviour,
    /// NAT status probing via observed reachability.
    pub autonat: autonat::Behaviour,
    /// Keepalive and round-trip telemetry.
    pub ping: ping::Behaviour,
    /// Direct message delivery (request-response pattern)
    pub messaging: request_response::cbor::Behaviour<MessageRequest, MessageResponse>,
    /// Address reflection for sovereign NAT discovery (replaces external STUN)
    pub address_reflection:
        request_response::cbor::Behaviour<AddressReflectionRequest, AddressReflectionResponse>,
    /// Relay protocol for mesh routing (Phase 3)
    pub relay: request_response::cbor::Behaviour<RelayRequest, RelayResponse>,
    /// Ledger exchange — peers share their known peer lists on connect
    pub ledger_exchange:
        request_response::cbor::Behaviour<LedgerExchangeRequest, LedgerExchangeResponse>,
    /// Pub/sub for group messaging — PERMISSIVE mode for topic auto-negotiation
    pub gossipsub: gossipsub::Behaviour,
    /// DHT for WAN peer discovery
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// LAN peer discovery
    #[cfg(not(target_arch = "wasm32"))]
    pub mdns: mdns::tokio::Behaviour,
    /// Peer identification — advertises relay capability
    pub identify: identify::Behaviour,
}

/// A message request sent to a peer
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageRequest {
    /// Serialized Envelope bytes
    pub envelope_data: Vec<u8>,
}

/// A response to a message request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageResponse {
    /// Whether the message was accepted
    pub accepted: bool,
    /// Optional error message
    pub error: Option<String>,
}

/// A relay request (asking a peer to forward a message to another peer)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayRequest {
    /// The final destination peer ID (serialized)
    pub destination_peer: Vec<u8>,
    /// Serialized Envelope bytes to relay
    pub envelope_data: Vec<u8>,
    /// Unique message ID for tracking
    pub message_id: String,
}

/// Response to a relay request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelayResponse {
    /// Whether the relay was accepted
    pub accepted: bool,
    /// Optional error message
    pub error: Option<String>,
    /// Message ID being acknowledged
    pub message_id: String,
}

/// A shared peer entry for ledger exchange.
/// Stripped-down version of ledger data suitable for wire transfer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SharedPeerEntry {
    /// The multiaddr (transport address only, no /p2p/ suffix)
    pub multiaddr: String,
    /// Last known PeerID at this address (if any)
    pub last_peer_id: Option<String>,
    /// Unix timestamp of last successful connection
    pub last_seen: u64,
    /// Gossipsub topics this peer was subscribed to
    pub known_topics: Vec<String>,
}

/// Ledger exchange request — sent automatically on new connection.
/// "Here are all the peers I know about. Tell me yours."
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerExchangeRequest {
    /// Our known peers (shared generously)
    pub peers: Vec<SharedPeerEntry>,
    /// Our own PeerID (so the remote can record us)
    pub sender_peer_id: String,
    /// Protocol version for forward compatibility
    pub version: u32,
}

/// Ledger exchange response — reciprocal sharing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LedgerExchangeResponse {
    /// Their known peers (shared back)
    pub peers: Vec<SharedPeerEntry>,
    /// Number of new peers they learned from our request
    pub new_peers_learned: u32,
    /// Protocol version
    pub version: u32,
}

impl IronCoreBehaviour {
    /// Create a new behaviour with the given keypair.
    ///
    /// Key design decisions for Aggressive Discovery:
    /// - Gossipsub uses PERMISSIVE validation (accept messages from any topic)
    /// - Identify advertises this node as a relay
    /// - Kademlia set to Server mode by default
    /// - Ledger exchange for automatic peer list sharing
    /// - All timeouts are generous to survive flaky networks
    pub fn new(
        keypair: &libp2p::identity::Keypair,
        relay_client: relay::client::Behaviour,
        headless: bool,
    ) -> anyhow::Result<Self> {
        let peer_id = keypair.public().to_peer_id();
        let dcutr = dcutr::Behaviour::new(peer_id);
        let autonat = autonat::Behaviour::new(peer_id, autonat::Config::default());
        let ping = ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(15))
                .with_timeout(Duration::from_secs(20)),
        );

        // Request-response for direct messaging
        let messaging = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/message/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(30)),
        );

        // Request-response for address reflection (sovereign NAT discovery)
        let address_reflection = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/address-reflection/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(10)),
        );

        // Request-response for relay (mesh routing - Phase 3)
        // All nodes are mandatory relays - always Full support
        let relay = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/relay/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(60)), // Generous timeout for relay
        );

        // Ledger exchange — automatic peer list sharing
        // When two peers connect, they exchange their known peer lists.
        // This dramatically accelerates mesh discovery.
        let ledger_exchange = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/ledger-exchange/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(30)),
        );

        // Gossipsub for pub/sub — PERMISSIVE mode for aggressive discovery
        //
        // ValidationMode::Permissive means:
        //   - Accept messages even if we don't have the signing key
        //   - Accept messages from topics we're not explicitly subscribed to
        //   - Log anomalies but don't drop connections
        //
        // This enables dynamic topic negotiation: when a peer advertises
        // a different topic, we can see it and auto-subscribe.
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(5)) // Faster heartbeat for quicker discovery
            .validation_mode(gossipsub::ValidationMode::Permissive) // PERMISSIVE: accept everything
            .mesh_outbound_min(1) // Must be <= mesh_n_low
            .mesh_n_low(1) // Accept mesh with just 1 peer
            .mesh_n(3) // Target 3 peers in mesh
            .mesh_n_high(12) // Allow up to 12
            .gossip_lazy(3) // Gossip to at least 3 non-mesh peers
            .history_length(5) // Keep 5 heartbeat windows of message history
            .history_gossip(3) // Gossip about last 3 windows
            .build()
            .map_err(|e| anyhow::anyhow!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Gossipsub error: {}", e))?;

        // Kademlia DHT for peer discovery
        let mut kademlia = kad::Behaviour::new(peer_id, kad::store::MemoryStore::new(peer_id));
        // Set server mode immediately — we want to be discoverable
        kademlia.set_mode(Some(kad::Mode::Server));

        // mDNS for LAN discovery
        #[cfg(not(target_arch = "wasm32"))]
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Identify protocol — advertise this node as a relay
        //
        // agent_version includes "relay" to signal we're a mandatory relay.
        // We also distinguish "headless" (infrastructure) vs "full" (human) nodes.
        // push_listen_addr_updates ensures peers learn our addresses quickly.
        let type_str = if headless { "headless" } else { "full" };
        let identify = identify::Behaviour::new(
            identify::Config::new("/sc/id/1.0.0".to_string(), keypair.public())
                .with_push_listen_addr_updates(true)
                .with_interval(Duration::from_secs(30)) // More frequent than before (was 60s)
                .with_agent_version(format!("scmessenger/0.1.0/{}/relay/{}", type_str, peer_id)),
        );

        Ok(Self {
            relay_client,
            dcutr,
            autonat,
            ping,
            messaging,
            address_reflection,
            relay,
            ledger_exchange,
            gossipsub,
            kademlia,
            #[cfg(not(target_arch = "wasm32"))]
            mdns,
            identify,
        })
    }
}
