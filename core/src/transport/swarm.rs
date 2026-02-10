// libp2p swarm setup — the actual running network node
//
// This creates and manages the libp2p Swarm with:
// - TCP + QUIC transports
// - Noise encryption (transport-level, separate from message encryption)
// - Yamux multiplexing
// - All behaviours from behaviour.rs

use super::behaviour::{IronCoreBehaviour, MessageRequest, MessageResponse, RelayRequest, RelayResponse};
use super::reflection::{AddressReflectionRequest, AddressReflectionService};
use super::observation::{AddressObserver, ConnectionTracker};
use super::multiport::{self, MultiPortConfig, BindResult};
use super::mesh_routing::{MultiPathDelivery, BootstrapCapability};
use anyhow::Result;
use libp2p::{identity::Keypair, kad, swarm::SwarmEvent, Multiaddr, PeerId};
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Pending message delivery tracking
#[derive(Debug)]
struct PendingMessage {
    message_id: String,
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
    start_swarm_with_config(keypair, listen_addr, event_tx, None).await
}

/// Build and start the libp2p swarm with custom multi-port configuration.
pub async fn start_swarm_with_config(
    keypair: Keypair,
    listen_addr: Option<Multiaddr>,
    event_tx: mpsc::Sender<SwarmEvent2>,
    multiport_config: Option<MultiPortConfig>,
) -> Result<SwarmHandle> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_behaviour(|key| {
                IronCoreBehaviour::new(key).expect("Failed to create network behaviour")
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(300))
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

        // Set Kademlia to server mode (so we can be found)
        swarm
            .behaviour_mut()
            .kademlia
            .set_mode(Some(kad::Mode::Server));

        let (command_tx, mut command_rx) = mpsc::channel::<SwarmCommand>(256);
        let handle = SwarmHandle {
            command_tx: command_tx.clone(),
        };

        // Address reflection service
        let reflection_service = AddressReflectionService::new();

        // Track pending address reflection requests
        let mut pending_reflections: HashMap<libp2p::request_response::OutboundRequestId, mpsc::Sender<Result<String, String>>> = HashMap::new();

        // Track connections and address observations (Phase 1 & 2)
        let mut connection_tracker = ConnectionTracker::new();
        let mut address_observer = AddressObserver::new();

        // Mesh routing components (Phase 3-6)
        let mut multi_path_delivery = MultiPathDelivery::new();
        let mut bootstrap_capability = BootstrapCapability::new();

        // Track pending message deliveries
        let mut pending_messages: HashMap<String, PendingMessage> = HashMap::new();

        // Track outbound request IDs to message IDs for direct sends
        let mut request_to_message: HashMap<libp2p::request_response::OutboundRequestId, String> = HashMap::new();

        // Track outbound relay request IDs
        let mut pending_relay_requests: HashMap<libp2p::request_response::OutboundRequestId, String> = HashMap::new();

        // Spawn the swarm event loop
        tokio::spawn(async move {
            // PHASE 6: Retry interval for failed deliveries
            let mut retry_interval = tokio::time::interval(Duration::from_millis(500));

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
                                        // We observe their address from the connection endpoint

                                        // Get the remote address from our connection tracker
                                        let observed_addr = connection_tracker
                                            .get_connection(&peer)
                                            .and_then(|conn| ConnectionTracker::extract_socket_addr(&conn.remote_addr))
                                            .unwrap_or_else(|| "0.0.0.0:0".parse().unwrap());

                                        tracing::debug!("Observed address for {}: {}", peer, observed_addr);

                                        // Create response using the service
                                        let response = reflection_service.handle_request(request, observed_addr);

                                        // Send response back
                                        let _ = swarm.behaviour_mut().address_reflection.send_response(channel, response);
                                    }
                                    request_response::Message::Response { request_id, response } => {
                                        // We received a reflection response from a peer
                                        tracing::info!("Address reflection from {}: {}", peer, response.observed_address);

                                        // Parse and record the observation
                                        if let Ok(observed_addr) = response.observed_address.parse::<SocketAddr>() {
                                            address_observer.record_observation(peer, observed_addr);

                                            // Log consensus
                                            if let Some(primary) = address_observer.primary_external_address() {
                                                tracing::info!("Consensus external address: {}", primary);
                                            }
                                        }

                                        // Reply to pending request
                                        if let Some(reply_tx) = pending_reflections.remove(&request_id) {
                                            let _ = reply_tx.send(Ok(response.observed_address.clone())).await;
                                        }

                                        // Also emit event for application layer
                                        let _ = event_tx.send(SwarmEvent2::AddressReflected {
                                            peer_id: peer,
                                            observed_address: response.observed_address,
                                        }).await;
                                    }
                                }
                            }

                            // PHASE 3: Relay Protocol Handler
                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Relay(
                                request_response::Event::Message { peer, message, .. }
                            )) => {
                                match message {
                                    request_response::Message::Request { request, channel, .. } => {
                                        // PHASE 3: Peer is asking us to relay a message
                                        tracing::info!("Relay request from {} for message {}", peer, request.message_id);

                                        // Parse destination peer
                                        match PeerId::from_bytes(&request.destination_peer) {
                                            Ok(destination) => {
                                                // Check if we're connected to the destination
                                                if swarm.is_connected(&destination) {
                                                    // Forward the message
                                                    let _forward_id = swarm.behaviour_mut().messaging.send_request(
                                                        &destination,
                                                        MessageRequest { envelope_data: request.envelope_data },
                                                    );

                                                    // Send acceptance response
                                                    let _ = swarm.behaviour_mut().relay.send_response(
                                                        channel,
                                                        RelayResponse {
                                                            accepted: true,
                                                            error: None,
                                                            message_id: request.message_id.clone(),
                                                        },
                                                    );

                                                    tracing::info!("✓ Relaying message {} to {}", request.message_id, destination);
                                                } else {
                                                    // Not connected to destination
                                                    let _ = swarm.behaviour_mut().relay.send_response(
                                                        channel,
                                                        RelayResponse {
                                                            accepted: false,
                                                            error: Some("Destination not connected".to_string()),
                                                            message_id: request.message_id,
                                                        },
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Invalid destination peer ID: {}", e);
                                                let _ = swarm.behaviour_mut().relay.send_response(
                                                    channel,
                                                    RelayResponse {
                                                        accepted: false,
                                                        error: Some("Invalid destination peer ID".to_string()),
                                                        message_id: request.message_id,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                    request_response::Message::Response { request_id, response } => {
                                        // Response to our relay request
                                        if let Some(message_id) = pending_relay_requests.remove(&request_id) {
                                            if let Some(pending) = pending_messages.remove(&message_id) {
                                                if response.accepted {
                                                    // PHASE 5: Track successful relay delivery
                                                    let latency_ms = pending.attempt_start.elapsed().unwrap_or_default().as_millis() as u64;
                                                    multi_path_delivery.record_success(&message_id, vec![peer, pending.target_peer], latency_ms);
                                                    tracing::info!("✓ Message relayed successfully via {} to {} ({}ms)", peer, pending.target_peer, latency_ms);
                                                    let _ = pending.reply_tx.send(Ok(())).await;
                                                } else {
                                                    // Relay failed, try next path
                                                    tracing::warn!("✗ Relay via {} failed: {:?}", peer, response.error);
                                                    multi_path_delivery.record_failure(&message_id, vec![peer, pending.target_peer]);

                                                    // Try next path
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

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Mdns(
                                mdns::Event::Discovered(peers)
                            )) => {
                                for (peer_id, addr) in peers {
                                    tracing::info!("mDNS discovered peer: {} at {}", peer_id, addr);
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);

                                    // PHASE 4: Add to bootstrap capability
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

                            SwarmEvent::Behaviour(super::behaviour::IronCoreBehaviourEvent::Identify(
                                identify::Event::Received { peer_id, info, .. }
                            )) => {
                                tracing::debug!("Identified peer {} with {} addresses", peer_id, info.listen_addrs.len());
                                for addr in info.listen_addrs {
                                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                                }
                            }

                            SwarmEvent::NewListenAddr { address, .. } => {
                                tracing::info!("Listening on {}", address);
                                let _ = event_tx.send(SwarmEvent2::ListeningOn(address)).await;
                            }

                            SwarmEvent::ConnectionEstablished { peer_id, endpoint, connection_id, .. } => {
                                tracing::info!("Connected to {} via {}", peer_id, endpoint.get_remote_address());

                                // Track this connection for address observation
                                connection_tracker.add_connection(
                                    peer_id,
                                    endpoint.get_remote_address().clone(),
                                    endpoint.get_local_address().clone(),
                                    connection_id.to_string(),
                                );

                                // PHASE 4: Add to bootstrap capability (potential relay node)
                                bootstrap_capability.add_peer(peer_id);

                                let _ = event_tx.send(SwarmEvent2::PeerDiscovered(peer_id)).await;
                            }

                            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                tracing::info!("Disconnected from {}", peer_id);
                                connection_tracker.remove_connection(&peer_id);
                                let _ = event_tx.send(SwarmEvent2::PeerDisconnected(peer_id)).await;
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
                                    message_id: message_id.clone(),
                                    target_peer: peer_id,
                                    envelope_data,
                                    reply_tx: reply,
                                    current_path_index: 0,
                                    attempt_start: SystemTime::now(),
                                });
                            }

                            SwarmCommand::RequestAddressReflection { peer_id, reply } => {
                                // Generate a unique request ID
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

                                // Store reply channel for when response arrives
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
        anyhow::bail!("WASM transport not yet implemented");
    }
}

use futures::StreamExt;
use libp2p::identify;
#[cfg(not(target_arch = "wasm32"))]
use libp2p::mdns;
use libp2p::request_response;
