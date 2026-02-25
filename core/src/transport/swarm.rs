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
use super::mesh_routing::{BootstrapCapability, MultiPathDelivery};
#[cfg(target_arch = "wasm32")]
use super::multiport::MultiPortConfig;
#[cfg(not(target_arch = "wasm32"))]
use super::multiport::{self, BindResult, MultiPortConfig};
use super::observation::{AddressObserver, ConnectionTracker};
use super::reflection::{AddressReflectionRequest, AddressReflectionService};
use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use bincode;
#[cfg(target_arch = "wasm32")]
use libp2p::Transport;
use libp2p::{identity::Keypair, kad, swarm::SwarmEvent, Multiaddr, PeerId};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::time::SystemTime;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::mpsc;

/// Pending message delivery tracking
#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
struct PendingMessage {
    target_peer: PeerId,
    envelope_data: Vec<u8>,
    reply_tx: mpsc::Sender<Result<(), String>>,
    current_path_index: usize,
    attempt_start: SystemTime,
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
) -> Result<SwarmHandle> {
    start_swarm_with_config(keypair, listen_addr, event_tx, None, Vec::new()).await
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
) -> Result<SwarmHandle> {
    #[cfg(target_arch = "wasm32")]
    let _ = &multiport_config;

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
                IronCoreBehaviour::new(key, relay_client)
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

        // Track subscribed topics for dynamic negotiation
        let mut subscribed_topics: HashSet<String> = HashSet::new();
        subscribed_topics.insert("sc-lobby".to_string());
        subscribed_topics.insert("sc-mesh".to_string());

        // Track peers we've already exchanged ledgers with (avoid spamming)
        let mut ledger_exchanged_peers: HashSet<PeerId> = HashSet::new();

        // Auto-dial bootstrap nodes for cross-network discovery
        if !bootstrap_addrs.is_empty() {
            tracing::info!(
                "🌐 Dialing {} bootstrap node(s) for NAT traversal",
                bootstrap_addrs.len()
            );
            for addr in &bootstrap_addrs {
                match swarm.dial(addr.clone()) {
                    Ok(_) => tracing::info!("  ✓ Dialing bootstrap: {}", addr),
                    Err(e) => tracing::warn!("  ✗ Failed to dial bootstrap {}: {}", addr, e),
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

            loop {
                tokio::select! {
                    // PHASE 6: Periodic retry check
                    _ = retry_interval.tick() => {
                        // Check for messages that need retry
                        let mut to_retry = Vec::new();

                        for (msg_id, pending) in pending_messages.iter() {
                            if let Some(attempt) = multi_path_delivery.pending_attempts().iter().find(|a| &a.message_id == msg_id) {
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
                                pending.current_path_index += 1;
                                let paths = multi_path_delivery.get_best_paths(&pending.target_peer, 3);

                                if pending.current_path_index < paths.len() {
                                    let path = &paths[pending.current_path_index];
                                    tracing::info!("RETRY: Attempting delivery via path {:?}", path);

                                    pending.attempt_start = SystemTime::now();

                                    if path.len() == 1 {
                                        // Direct retry
                                        let request_id = swarm.behaviour_mut().messaging.send_request(
                                            &pending.target_peer,
                                            MessageRequest { envelope_data: pending.envelope_data.clone() },
                                        );
                                        request_to_message.insert(request_id, msg_id.clone());
                                    } else {
                                        // Relay retry
                                        let relay_peer = path[0];
                                        let destination_peer_bytes = pending.target_peer.to_bytes();

                                        let relay_request = RelayRequest {
                                            destination_peer: destination_peer_bytes,
                                            envelope_data: pending.envelope_data.clone(),
                                            message_id: msg_id.clone(),
                                        };

                                        let request_id = swarm.behaviour_mut().relay.send_request(
                                            &relay_peer,
                                            relay_request,
                                        );
                                        pending_relay_requests.insert(request_id, msg_id.clone());
                                    }

                                    pending_messages.insert(msg_id, pending);
                                } else {
                                    // All paths exhausted
                                    tracing::error!("All delivery paths exhausted for message {}", msg_id);
                                    let _ = pending.reply_tx.send(Err("All delivery paths exhausted".to_string())).await;
                                }
                            }
                        }
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
                                    match swarm.dial(addr.clone()) {
                                        Ok(_) => tracing::debug!("🔄 Re-dialing bootstrap: {}", addr),
                                        Err(e) => tracing::trace!("Bootstrap re-dial {} skipped: {}", addr, e),
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
                                        // Response to our outbound message request
                                        if let Some(message_id) = request_to_message.remove(&request_id) {
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

                                                    // Try next path
                                                    let paths = multi_path_delivery.get_best_paths(&pending.target_peer, 3);
                                                    if pending.current_path_index + 1 < paths.len() {
                                                        // Retry with next path will be handled by retry task
                                                        pending_messages.insert(message_id, pending);
                                                    } else {
                                                        let _ = pending.reply_tx.send(Err(response.error.unwrap_or("Rejected".to_string()))).await;
                                                    }
                                                }
                                            }
                                        }
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
                                                    if swarm.is_connected(&destination) {
                                                        let _forward_id = swarm.behaviour_mut().messaging.send_request(
                                                            &destination,
                                                            MessageRequest { envelope_data: request.envelope_data },
                                                        );
                                                        tracing::info!("✓ Relaying message {} to {}", request.message_id, destination);
                                                        RelayResponse {
                                                            accepted: true,
                                                            error: None,
                                                            message_id: request.message_id.clone(),
                                                        }
                                                    } else {
                                                        tracing::warn!("⚠ Destination {} not connected, relay cannot proceed", destination);
                                                        RelayResponse {
                                                            accepted: false,
                                                            error: Some("Destination not connected".to_string()),
                                                            message_id: request.message_id.clone(),
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
                                                    tracing::info!("✓ Message relayed successfully via {} to {} ({}ms)", peer, pending.target_peer, latency_ms);
                                                    let _ = pending.reply_tx.send(Ok(())).await;
                                                } else {
                                                    tracing::warn!("✗ Relay via {} failed: {:?}", peer, response.error);
                                                    multi_path_delivery.record_failure(&message_id, vec![peer, pending.target_peer]);

                                                    let paths = multi_path_delivery.get_best_paths(&pending.target_peer, 3);
                                                    if pending.current_path_index + 1 < paths.len() {
                                                        pending_messages.insert(message_id, pending);
                                                    } else {
                                                        let _ = pending.reply_tx.send(Err(response.error.unwrap_or("All paths failed".to_string()))).await;
                                                    }
                                                }
                                            }
                                        }
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
                                                    if let Ok(addr) = entry.multiaddr.parse::<Multiaddr>() {
                                                        swarm.behaviour_mut().kademlia.add_address(&pid, addr);
                                                        new_count += 1;
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

                                            // Add to Kademlia immediately
                                            for entry in &response.peers {
                                                if let Some(ref pid_str) = entry.last_peer_id {
                                                    if let Ok(pid) = pid_str.parse::<PeerId>() {
                                                        if let Ok(addr) = entry.multiaddr.parse::<Multiaddr>() {
                                                            swarm.behaviour_mut().kademlia.add_address(&pid, addr);
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
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Autonat(event)) => {
                                tracing::debug!("AutoNAT event: {:?}", event);
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Dcutr(event)) => {
                                tracing::debug!("DCUtR event: {:?}", event);
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::RelayClient(event)) => {
                                tracing::debug!("Relay client event: {:?}", event);
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Ping(event)) => {
                                tracing::trace!("Ping event: {:?}", event);
                            }

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Mdns(
                                mdns::Event::Discovered(peers)
                            )) => {
                                for (peer_id, addr) in peers {
                                    tracing::info!("mDNS discovered peer: {} at {}", peer_id, addr);
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);

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

                                // Add ALL reported addresses to Kademlia — no filtering
                                for addr in &info.listen_addrs {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
                                }

                                // Check if peer advertises relay capability
                                let is_relay = info.agent_version.contains("relay");
                                if is_relay {
                                    tracing::info!("🔄 Peer {} is a relay node", peer_id);
                                    bootstrap_capability.add_peer(peer_id);
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
                                let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
                            }

                            // Handle outgoing connection errors gracefully — don't panic
                            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                if let Some(pid) = peer_id {
                                    tracing::warn!("⚠ Outgoing connection error to {}: {}", pid, error);
                                } else {
                                    tracing::warn!("⚠ Outgoing connection error: {}", error);
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

                                // Get best paths (direct + relay options)
                                let paths = multi_path_delivery.get_best_paths(&peer_id, 3);

                                if paths.is_empty() {
                                    let _ = reply.send(Err("No paths available".to_string())).await;
                                    continue;
                                }

                                // Try first path (direct or via relay)
                                let path = &paths[0];
                                tracing::info!("Attempting delivery via path: {:?}", path);

                                if path.len() == 1 {
                                    // Direct send
                                    let request_id = swarm.behaviour_mut().messaging.send_request(
                                        &peer_id,
                                        MessageRequest { envelope_data: envelope_data.clone() },
                                    );
                                    request_to_message.insert(request_id, message_id.clone());
                                } else {
                                    // Relay via intermediate peer
                                    let relay_peer = path[0];
                                    let destination_peer_bytes = peer_id.to_bytes();

                                    let relay_request = RelayRequest {
                                        destination_peer: destination_peer_bytes,
                                        envelope_data: envelope_data.clone(),
                                        message_id: message_id.clone(),
                                    };

                                    let request_id = swarm.behaviour_mut().relay.send_request(
                                        &relay_peer,
                                        relay_request,
                                    );
                                    pending_relay_requests.insert(request_id, message_id.clone());
                                }

                                // Store pending message for retry handling
                                pending_messages.insert(message_id.clone(), PendingMessage {
                                    target_peer: peer_id,
                                    envelope_data,
                                    reply_tx: reply,
                                    current_path_index: 0,
                                    attempt_start: SystemTime::now(),
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
                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
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
                IronCoreBehaviour::new(key, relay_client)
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
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&lobby_topic) {
            tracing::warn!("Failed to subscribe to lobby topic: {}", e);
        }
        if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&mesh_topic) {
            tracing::warn!("Failed to subscribe to mesh topic: {}", e);
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
                match swarm.dial(addr.clone()) {
                    Ok(_) => tracing::info!("  ✓ Dialing bootstrap: {}", addr),
                    Err(e) => tracing::warn!("  ✗ Failed to dial bootstrap {}: {}", addr, e),
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

        let mut pending_messages: HashMap<String, PendingMessage> = HashMap::new();

        let mut subscribed_topics: HashSet<String> = HashSet::new();
        subscribed_topics.insert("sc-lobby".to_string());
        subscribed_topics.insert("sc-mesh".to_string());

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
                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
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
                                            if let Some(reply_tx) = pending_direct_replies.remove(&request_id) {
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
                                        if let Some(reply_tx) = pending_direct_replies.remove(&request_id) {
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
                                    request_response::Event::Message { peer: _, message, .. } => match message {
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
                                                        if swarm.is_connected(&destination) {
                                                            let _ = swarm.behaviour_mut().messaging.send_request(
                                                                &destination,
                                                                MessageRequest { envelope_data: request.envelope_data },
                                                            );
                                                            RelayResponse {
                                                                accepted: true,
                                                                error: None,
                                                                message_id: request.message_id.clone(),
                                                            }
                                                        } else {
                                                            RelayResponse {
                                                                accepted: false,
                                                                error: Some("Destination not connected".to_string()),
                                                                message_id: request.message_id.clone(),
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
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Identify(
                                identify::Event::Received { peer_id, info, .. }
                            )) => {
                                for addr in &info.listen_addrs {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
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
                                let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;
                            }
                            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                connection_tracker.remove_connection(&peer_id);
                                ledger_exchanged_peers.remove(&peer_id);
                                let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
                            }
                            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                if let Some(pid) = peer_id {
                                    tracing::warn!("⚠ Outgoing connection error to {}: {}", pid, error);
                                } else {
                                    tracing::warn!("⚠ Outgoing connection error: {}", error);
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
                            if let Err(e) = swarm.dial(addr.clone()) {
                                tracing::trace!("Bootstrap re-dial {} skipped: {}", addr, e);
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
