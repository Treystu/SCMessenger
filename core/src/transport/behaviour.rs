// Combined NetworkBehaviour for Iron Core
//
// This combines all the libp2p protocols we need:
// - request_response: direct peer-to-peer message delivery
// - address_reflection: sovereign mesh address discovery (replaces STUN)
// - gossipsub: pub/sub for group messaging (future)
// - kademlia: DHT for peer discovery on WAN
// - mdns: peer discovery on LAN
// - identify: exchange peer metadata
// - relay: NAT traversal for mobile nodes

#[cfg(not(target_arch = "wasm32"))]
use libp2p::mdns;
use libp2p::{
    gossipsub, identify, kad,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use std::time::Duration;
use super::reflection::{AddressReflectionRequest, AddressReflectionResponse};

/// The Iron Core network behaviour combining all protocols.
#[derive(NetworkBehaviour)]
pub struct IronCoreBehaviour {
    /// Direct message delivery (request-response pattern)
    pub messaging: request_response::cbor::Behaviour<MessageRequest, MessageResponse>,
    /// Address reflection for sovereign NAT discovery (replaces external STUN)
    pub address_reflection: request_response::cbor::Behaviour<AddressReflectionRequest, AddressReflectionResponse>,
    /// Relay protocol for mesh routing (Phase 3)
    pub relay: request_response::cbor::Behaviour<RelayRequest, RelayResponse>,
    /// Pub/sub for future group messaging
    pub gossipsub: gossipsub::Behaviour,
    /// DHT for WAN peer discovery
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// LAN peer discovery
    #[cfg(not(target_arch = "wasm32"))]
    pub mdns: mdns::tokio::Behaviour,
    /// Peer identification
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

impl IronCoreBehaviour {
    /// Create a new behaviour with the given keypair
    pub fn new(keypair: &libp2p::identity::Keypair) -> anyhow::Result<Self> {
        let peer_id = keypair.public().to_peer_id();

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
        let relay = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/relay/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default().with_request_timeout(Duration::from_secs(30)),
        );

        // Gossipsub for pub/sub (future group messaging)
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| anyhow::anyhow!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Gossipsub error: {}", e))?;

        // Kademlia DHT for peer discovery
        let kademlia = kad::Behaviour::new(peer_id, kad::store::MemoryStore::new(peer_id));

        // mDNS for LAN discovery
        #[cfg(not(target_arch = "wasm32"))]
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Identify protocol
        let identify = identify::Behaviour::new(
            identify::Config::new("/sc/id/1.0.0".to_string(), keypair.public())
                .with_push_listen_addr_updates(true)
                .with_interval(Duration::from_secs(60)),
        );

        Ok(Self {
            messaging,
            address_reflection,
            relay,
            gossipsub,
            kademlia,
            #[cfg(not(target_arch = "wasm32"))]
            mdns,
            identify,
        })
    }
}
