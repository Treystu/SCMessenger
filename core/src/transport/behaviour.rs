// Combined NetworkBehaviour for Iron Core
//
// This combines all the libp2p protocols we need:
// - request_response: direct peer-to-peer message delivery
// - gossipsub: pub/sub for group messaging (future)
// - kademlia: DHT for peer discovery on WAN
// - mdns: peer discovery on LAN (optional based on DiscoveryConfig)
// - identify: exchange peer metadata (optional based on DiscoveryConfig)
// - relay: NAT traversal for mobile nodes

use crate::transport::discovery::DiscoveryConfig;
use libp2p::{
    gossipsub, identify, kad, mdns,
    request_response::{self, ProtocolSupport},
    swarm::{behaviour::toggle::Toggle, NetworkBehaviour},
    StreamProtocol,
};
use std::time::Duration;

/// The Iron Core network behaviour combining all protocols.
///
/// mDNS and Identify are optional based on the DiscoveryConfig.
#[derive(NetworkBehaviour)]
pub struct IronCoreBehaviour {
    /// Direct message delivery (request-response pattern)
    pub messaging: request_response::cbor::Behaviour<MessageRequest, MessageResponse>,
    /// Pub/sub for future group messaging
    pub gossipsub: gossipsub::Behaviour,
    /// DHT for WAN peer discovery
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// LAN peer discovery (optional, based on DiscoveryConfig)
    pub mdns: Toggle<mdns::tokio::Behaviour>,
    /// Peer identification (optional, based on DiscoveryConfig)
    pub identify: Toggle<identify::Behaviour>,
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

impl IronCoreBehaviour {
    /// Create a new behaviour with the given keypair and discovery configuration
    pub fn new(
        keypair: &libp2p::identity::Keypair,
        discovery_config: &DiscoveryConfig,
    ) -> anyhow::Result<Self> {
        let peer_id = keypair.public().to_peer_id();

        // Request-response for direct messaging
        let messaging = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/sc/message/1.0.0"),
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

        // mDNS for LAN discovery (optional based on discovery mode)
        let mdns = if discovery_config.mode.allows_mdns() {
            tracing::info!("Discovery: mDNS enabled");
            Toggle::from(Some(mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                peer_id,
            )?))
        } else {
            tracing::info!("Discovery: mDNS disabled (mode: {:?})", discovery_config.mode);
            Toggle::from(None)
        };

        // Identify protocol (optional based on discovery mode)
        let identify = if discovery_config.mode.allows_identify() {
            tracing::info!("Discovery: Identify protocol enabled");
            let identify_config = if discovery_config.mode.advertises_identify() {
                identify::Config::new("/sc/id/1.0.0".to_string(), keypair.public())
                    .with_push_listen_addr_updates(true)
                    .with_interval(Duration::from_secs(60))
            } else {
                // DarkBLE: disable advertising of own addresses to unknown peers
                identify::Config::new("/sc/id/1.0.0".to_string(), keypair.public())
                    .with_push_listen_addr_updates(false)
                    .with_interval(Duration::from_secs(300)) // Less frequent probes
            };
            Toggle::from(Some(identify::Behaviour::new(identify_config)))
        } else {
            tracing::info!(
                "Discovery: Identify protocol disabled (mode: {:?})",
                discovery_config.mode
            );
            Toggle::from(None)
        };

        Ok(Self {
            messaging,
            gossipsub,
            kademlia,
            mdns,
            identify,
        })
    }
}
