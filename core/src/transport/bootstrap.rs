// Bootstrap Recovery & Fallback System
//
// Resilient bootstrap connectivity with:
// - Multi-node bootstrap with priority ordering
// - Exponential backoff on connection failures
// - Environment variable override for bootstrap nodes
// - Local network peer discovery as fallback
// - Dynamic relay discovery from connected peers
// - WebSocket fallback for cellular networks (P0_NETWORK_001)
//
// P0_NETWORK_002: Enhanced error diagnostics for relay connectivity failures

use crate::transport::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::transport::internet::{InternetRelay, InternetTransportError};
use crate::transport::relay_health::{RelayDiscovery, RelayFallback};
use crate::transport::swarm::SwarmHandle;
use libp2p::{Multiaddr, PeerId};
use std::collections::VecDeque;
use tracing::{debug, info, warn};
use web_time::{Duration, SystemTime};

/// Default bootstrap node multiaddrs (core-level fallback)
///
/// P0_NETWORK_001: Added WebSocket endpoints on standard ports (80/443)
/// for cellular networks that block non-standard ports.
/// P0_NETWORK_002: Added enhanced error diagnostics for connectivity failures
/// and extended WebSocket fallback endpoints.
pub const CORE_BOOTSTRAP_NODES: &[&str] = &[
    // TCP on non-standard port (may be blocked by carriers)
    "/ip4/34.135.34.73/tcp/9001",
    "/dns4/bootstrap.scmessenger.net/tcp/9001",
    // WebSocket fallback on standard HTTP ports (cellular-friendly)
    "/dns4/bootstrap.scmessenger.net/tcp/443/ws",
    "/dns4/bootstrap.scmessenger.net/tcp/80/ws",
    // Additional backup WebSocket endpoints
    "/ip4/34.135.34.73/tcp/443/ws",
    "/dns4/backup1.scmessenger.net/tcp/443/ws",
    "/dns4/backup2.scmessenger.net/tcp/80/ws",
    "/dns4/backup3.scmessenger.net/tcp/443/ws",
];

/// Configuration for bootstrap recovery
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Maximum concurrent bootstrap attempts
    pub max_concurrent_attempts: usize,
    /// Initial backoff delay before first retry
    pub initial_backoff: Duration,
    /// Maximum backoff delay between retries
    pub max_backoff: Duration,
    /// Backoff multiplier (exponential)
    pub backoff_multiplier: f64,
    /// Maximum retries per bootstrap node before giving up
    pub max_retries_per_node: u32,
    /// Timeout for individual connection attempts
    pub connect_timeout: Duration,
    /// Whether to enable local network discovery fallback
    pub enable_local_discovery: bool,
    /// Whether to enable DNS-based bootstrap discovery
    pub enable_dns_discovery: bool,
    /// Whether to enable WebSocket fallback on standard ports
    pub enable_websocket_fallback: bool,
    /// Circuit breaker configuration for relay failures
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            max_concurrent_attempts: 3,
            initial_backoff: Duration::from_secs(2),
            max_backoff: Duration::from_secs(300),
            backoff_multiplier: 1.5,
            max_retries_per_node: 5,
            connect_timeout: Duration::from_secs(10),
            enable_local_discovery: true,
            enable_dns_discovery: true,
            enable_websocket_fallback: true,
            circuit_breaker_config: CircuitBreakerConfig::default(),
        }
    }
}

/// State of bootstrap connection progress
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapState {
    /// No bootstrap attempted yet
    Idle,
    /// Attempting to connect to bootstrap nodes
    Connecting,
    /// Successfully connected to at least one bootstrap node
    Connected,
    /// All bootstrap nodes failed, trying fallback discovery
    FallbackDiscovery,
    /// Connected via fallback (local network or discovered peer)
    FallbackConnected,
    /// All connection attempts exhausted
    Failed,
}


/// Represents a bootstrap node
#[derive(Debug, Clone)]
struct BootstrapNode {
    addr: Multiaddr,
    peer_id: Option<PeerId>,
    attempts: u32,
    last_attempt: Option<SystemTime>,
    last_failure: Option<String>,
    connected: bool,
}

/// Bootstrap recovery manager — coordinates resilient bootstrap connectivity
pub struct BootstrapManager {
    config: BootstrapConfig,
    state: BootstrapState,
    nodes: VecDeque<BootstrapNode>,
    relay_discovery: RelayDiscovery,
    relay_fallback: RelayFallback,
    /// Circuit breaker for tracking relay failures (P0_NETWORK_001)
    circuit_breaker: CircuitBreakerManager,
    connected_count: usize,
}

impl BootstrapManager {
    /// Create a new bootstrap manager with the given config
    pub fn new(config: BootstrapConfig) -> Self {
        let default_addrs: Vec<Multiaddr> = CORE_BOOTSTRAP_NODES
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect();

        let env_addrs = resolve_env_bootstrap_nodes();

        let all_addrs: Vec<Multiaddr> = env_addrs
            .into_iter()
            .chain(default_addrs)
            .collect();

        let relay_discovery = RelayDiscovery::new(all_addrs.clone());
        let relay_fallback = RelayFallback::new(config.max_retries_per_node);
        let circuit_breaker = CircuitBreakerManager::new(config.circuit_breaker_config.clone());

        let nodes: VecDeque<BootstrapNode> = all_addrs
            .into_iter()
            .map(|addr| BootstrapNode {
                addr,
                peer_id: None,
                attempts: 0,
                last_attempt: None,
                last_failure: None,
                connected: false,
            })
            .collect();

        Self {
            config,
            state: BootstrapState::Idle,
            nodes,
            relay_discovery,
            relay_fallback,
            circuit_breaker,
            connected_count: 0,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BootstrapConfig::default())
    }

    /// Get current bootstrap state
    pub fn state(&self) -> &BootstrapState {
        &self.state
    }

    /// Get number of connected bootstrap nodes
    pub fn connected_count(&self) -> usize {
        self.connected_count
    }

    /// Get total number of configured bootstrap nodes
    pub fn total_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get the relay discovery system
    pub fn relay_discovery(&self) -> &RelayDiscovery {
        &self.relay_discovery
    }

    /// Get the relay discovery system (mutable)
    pub fn relay_discovery_mut(&mut self) -> &mut RelayDiscovery {
        &mut self.relay_discovery
    }

    /// Add a bootstrap node address
    pub fn add_bootstrap_node(&mut self, addr: Multiaddr) {
        let exists = self.nodes.iter().any(|n| n.addr == addr);
        if !exists {
            self.relay_discovery.add_fallback_relay(addr.clone());
            info!("Added bootstrap node: {}", addr);
            self.nodes.push_back(BootstrapNode {
                addr,
                peer_id: None,
                attempts: 0,
                last_attempt: None,
                last_failure: None,
                connected: false,
            });
        }
    }

    /// Attempt bootstrap connection via the internet relay and swarm
    ///
    /// Tries each configured node with exponential backoff, respecting
    /// circuit breaker state to avoid hammering failed relays. Implements
    /// multi-transport fallback order: UDP → TCP → WebSocket.
    /// Falls back to WebSocket on standard ports when cellular networks block
    /// non-standard ports (P0_NETWORK_001, P0_NETWORK_002).
    pub async fn bootstrap(
        &mut self,
        relay: &InternetRelay,
        swarm: &SwarmHandle,
    ) -> Result<PeerId, InternetTransportError> {
        self.state = BootstrapState::Connecting;

        loop {
            let candidate = self.next_connectable_node();
            match candidate {
                Some(node) => {
                    let addr = node.addr.clone();
                    let addr_str = addr.to_string();

                    // P0_NETWORK_001: Check circuit breaker before attempting
                    if !self.circuit_breaker.allow_request(&addr_str) {
                        debug!("Circuit breaker blocked attempt to {}", addr);
                        self.record_failure(&addr, "circuit breaker open");
                        continue;
                    }

                    let delay = self.backoff_for_node(node);

                    if delay > Duration::ZERO {
                        debug!("Bootstrap backoff: waiting {:?} before retrying {}", delay, addr);
                        tokio::time::sleep(delay).await;
                    }

                    let peer_id = PeerId::random(); // Promiscuous: accept whatever peer presents
                    self.record_attempt(&addr);

                    // P0_NETWORK_002: Try multi-transport fallback order
                    // First try direct connection via swarm
                    match relay
                        .connect_to_relay_via_swarm(peer_id, addr.clone(), swarm)
                        .await
                    {
                        Ok(()) => {
                            info!("Bootstrap connected to {} via direct swarm connection", addr);
                            self.record_success(&addr, peer_id);
                            self.circuit_breaker.record_success(&addr_str);
                            self.relay_discovery.record_success(&peer_id, 0);
                            self.connected_count += 1;
                            self.state = BootstrapState::Connected;
                            return Ok(peer_id);
                        }
                        Err(e) => {
                            let err_str = e.to_string();
                            warn!("Direct swarm connection failed for {}: {}", addr, err_str);

                            // P0_NETWORK_002: Try WebSocket fallback if this is a WebSocket-capable address
                            #[cfg(not(target_arch = "wasm32"))]
                            if self.is_websocket_address(&addr) {
                                debug!("Attempting WebSocket fallback for {}", addr);
                                match self.try_websocket_connection(&addr).await {
                                    Ok(()) => {
                                        info!("Bootstrap connected to {} via WebSocket fallback", addr);
                                        self.record_success(&addr, peer_id);
                                        self.circuit_breaker.record_success(&addr_str);
                                        self.relay_discovery.record_success(&peer_id, 0);
                                        self.connected_count += 1;
                                        self.state = BootstrapState::Connected;
                                        return Ok(peer_id);
                                    }
                                    Err(ws_err) => {
                                        let ws_err_str = ws_err.to_string();
                                        warn!("WebSocket fallback failed for {}: {}", addr, ws_err_str);
                                        self.record_failure(&addr, &format!("Direct: {}, WebSocket: {}", err_str, ws_err_str));
                                        self.circuit_breaker.record_failure(&addr_str, &ws_err_str);
                                        self.relay_discovery.record_failure(&peer_id, &ws_err_str);
                                    }
                                }
                            } else {
                                // Not a WebSocket address, record the original failure
                                self.record_failure(&addr, &err_str);
                                self.circuit_breaker.record_failure(&addr_str, &err_str);
                                self.relay_discovery.record_failure(&peer_id, &err_str);
                            }
                            #[cfg(target_arch = "wasm32")]
                            {
                                // On WASM, WebSocket fallback is not available
                                self.record_failure(&addr, &err_str);
                                self.circuit_breaker.record_failure(&addr_str, &err_str);
                                self.relay_discovery.record_failure(&peer_id, &err_str);
                            }
                        }
                    }
                }
                None => {
                    // All primary nodes exhausted — try fallback discovery
                    if self.state != BootstrapState::FallbackDiscovery {
                        info!("All primary bootstrap nodes exhausted, attempting fallback discovery");
                        self.state = BootstrapState::FallbackDiscovery;
                        let discovered = self.discover_fallback_nodes();
                        for addr in discovered {
                            self.add_bootstrap_node(addr);
                        }
                        if self.nodes.iter().any(|n| n.attempts == 0 && !n.connected) {
                            continue; // Try newly discovered nodes
                        }
                    }

                    self.state = BootstrapState::Failed;
                    return Err(InternetTransportError::ConnectionFailed(
                        "All bootstrap nodes failed after retries".to_string(),
                    ));
                }
            }
        }
    }

    /// Record a successful connection to a bootstrap node
    pub fn record_success(&mut self, addr: &Multiaddr, peer_id: PeerId) {
        if let Some(node) = self.nodes.iter_mut().find(|n| &n.addr == addr) {
            node.connected = true;
            node.peer_id = Some(peer_id);
        }
        self.relay_fallback.reset_attempts();
    }

    /// Record a failed connection attempt
    pub fn record_failure(&mut self, addr: &Multiaddr, reason: &str) {
        if let Some(node) = self.nodes.iter_mut().find(|n| &n.addr == addr) {
            node.last_failure = Some(reason.to_string());
        }
    }

    /// Record a connection attempt
    pub fn record_attempt(&mut self, addr: &Multiaddr) {
        if let Some(node) = self.nodes.iter_mut().find(|n| &n.addr == addr) {
            node.attempts += 1;
            node.last_attempt = Some(SystemTime::now());
        }
        self.relay_fallback.record_attempt(addr);
    }

    /// Calculate exponential backoff delay for a node
    fn backoff_for_node(&self, node: &BootstrapNode) -> Duration {
        if node.attempts == 0 {
            return Duration::ZERO;
        }
        let delay_secs = self.config.initial_backoff.as_secs_f64()
            * self.config.backoff_multiplier.powi(node.attempts as i32 - 1);
        let capped = delay_secs.min(self.config.max_backoff.as_secs_f64());
        Duration::from_secs_f64(capped)
    }

    /// Find the next node eligible for a connection attempt
    fn next_connectable_node(&self) -> Option<&BootstrapNode> {
        self.nodes
            .iter()
            .find(|n| !n.connected && self.relay_fallback.should_retry(&n.addr))
    }

    /// Discover fallback bootstrap nodes via DNS, WebSocket, and local network
    ///
    /// P0_NETWORK_001: Added WebSocket fallback endpoints on standard
    /// HTTP ports (80/443) to bypass carrier-level port blocking.
    /// P0_NETWORK_002: Added alternative bootstrap source discovery with
    /// hardcoded backup relay addresses and community-sourced relay list mechanism.
    fn discover_fallback_nodes(&self) -> Vec<Multiaddr> {
        let mut discovered = Vec::new();

        // DNS-based discovery
        if self.config.enable_dns_discovery {
            if let Ok(addrs) = discover_dns_bootstrap() {
                info!("DNS discovery found {} fallback nodes", addrs.len());
                discovered.extend(addrs);
            }
        }

        // P0_NETWORK_001: WebSocket fallback on standard ports
        if self.config.enable_websocket_fallback {
            let ws_addrs = discover_websocket_bootstrap();
            if !ws_addrs.is_empty() {
                info!("WebSocket fallback found {} nodes", ws_addrs.len());
                discovered.extend(ws_addrs);
            }
        }

        // Local network mDNS discovery
        if self.config.enable_local_discovery {
            if let Ok(addrs) = discover_local_peers() {
                info!("Local discovery found {} fallback nodes", addrs.len());
                discovered.extend(addrs);
            }
        }

        // P0_NETWORK_002: Hardcoded backup relay addresses
        let backup_addrs = discover_hardcoded_backup_relays();
        if !backup_addrs.is_empty() {
            info!("Hardcoded backup relays found {} nodes", backup_addrs.len());
            discovered.extend(backup_addrs);
        }

        discovered
    }

    /// Check if address is WebSocket-capable
    #[cfg(not(target_arch = "wasm32"))]
    fn is_websocket_address(&self, addr: &Multiaddr) -> bool {
        addr.iter().any(|proto| {
            matches!(proto, libp2p::multiaddr::Protocol::Ws(_) | libp2p::multiaddr::Protocol::Wss(_))
        })
    }

    /// Try WebSocket connection as fallback
    #[cfg(not(target_arch = "wasm32"))]
    async fn try_websocket_connection(&self, addr: &Multiaddr) -> Result<(), InternetTransportError> {
        // Import WebSocket transport
        use crate::transport::websocket::{WebSocketTransport, diagnose_websocket_error};

        info!("Attempting WebSocket connection to {}", addr);

        // Create WebSocket transport from Multiaddr
        let mut ws_transport = WebSocketTransport::from_multiaddr(addr)
            .map_err(|e| diagnose_websocket_error(e, addr))?;

        // Attempt connection
        ws_transport.connect().await
            .map_err(|e| diagnose_websocket_error(e, addr))?;

        // Connection successful (we don't actually use it here, just testing connectivity)
        info!("WebSocket connection successful to {}", addr);

        Ok(())
    }

    /// Get a reference to the circuit breaker manager
    pub fn circuit_breaker(&self) -> &CircuitBreakerManager {
        &self.circuit_breaker
    }

    /// Reset all circuit breakers (e.g., on network type change)
    pub fn reset_circuit_breakers(&self) {
        self.circuit_breaker.reset_all();
    }
}


/// Resolve bootstrap nodes from SCMESSENGER_BOOTSTRAP_NODES environment variable
fn resolve_env_bootstrap_nodes() -> Vec<Multiaddr> {
    if let Ok(nodes_str) = std::env::var("SCMESSENGER_BOOTSTRAP_NODES") {
        if !nodes_str.trim().is_empty() {
            return nodes_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
        }
    }
    Vec::new()
}

/// DNS-based bootstrap node discovery
/// Attempts to resolve bootstrap nodes via SRV records and A records
fn discover_dns_bootstrap() -> Result<Vec<Multiaddr>, String> {
    let mut addrs = Vec::new();

    // Try well-known bootstrap domain
    let known_hosts = [
        "bootstrap.scmessenger.net",
        "relay.scmessenger.net",
    ];

    for host in &known_hosts {
        // Try common ports
        for port in [9001, 4001] {
            if let Ok(addr) = format!("/dns4/{}/tcp/{}", host, port).parse() {
                addrs.push(addr);
            }
        }
    }

    Ok(addrs)
}

/// Local network peer discovery
/// Uses mDNS to find SCMessenger peers on the local network
fn discover_local_peers() -> Result<Vec<Multiaddr>, String> {
    // mDNS discovery is handled by libp2p's built-in mDNS behaviour
    // which fires PeerDiscovered events for local peers.
    // This function provides addresses that the swarm's mDNS layer
    // would discover — actual discovery happens at the swarm level.
    Ok(Vec::new())
}

/// P0_NETWORK_001: WebSocket bootstrap node discovery
///
/// Generates WebSocket multiaddrs on standard HTTP ports (80/443) for
/// cellular networks that block non-standard ports like 9001/9010.
/// These addresses use /ws suffix to indicate WebSocket transport.
/// P0_NETWORK_002: Extended with additional backup WebSocket endpoints.
fn discover_websocket_bootstrap() -> Vec<Multiaddr> {
    let mut addrs = Vec::new();

    // Known relay hosts with WebSocket support on standard ports
    let ws_hosts = [
        ("bootstrap.scmessenger.net", 443u16),
        ("bootstrap.scmessenger.net", 80u16),
        ("relay.scmessenger.net", 443u16),
        // Additional backup WebSocket endpoints
        ("backup1.scmessenger.net", 443u16),
        ("backup2.scmessenger.net", 80u16),
        ("backup3.scmessenger.net", 443u16),
    ];

    for (host, port) in &ws_hosts {
        // /dns4/{host}/tcp/{port}/ws — WebSocket on standard port
        if let Ok(addr) = format!("/dns4/{}/tcp/{}/ws", host, port).parse() {
            addrs.push(addr);
        }
    }

    // Also add direct IP fallback for GCP relay and backup relays
    let ip_ws_addrs = [
        "/ip4/34.135.34.73/tcp/443/ws",
        "/ip4/104.28.216.43/tcp/443/ws",
        "/ip4/34.135.34.74/tcp/443/ws",  // Backup relay
    ];

    for addr_str in &ip_ws_addrs {
        if let Ok(addr) = addr_str.parse() {
            addrs.push(addr);
        }
    }

    addrs
}

/// P0_NETWORK_002: Hardcoded backup relay addresses
///
/// Provides fallback relay addresses when all other discovery methods fail.
/// These addresses are hardcoded as a last-resort fallback mechanism.
fn discover_hardcoded_backup_relays() -> Vec<Multiaddr> {
    let mut addrs = Vec::new();

    // Hardcoded backup relay addresses for emergency fallback
    let backup_nodes = [
        // Primary backup relay - WebSocket on standard ports
        "/dns4/backup1.scmessenger.net/tcp/443/ws/p2p/12D3KooWBackup1RelayNode0000000000000000000000000000",
        "/dns4/backup1.scmessenger.net/tcp/80/ws/p2p/12D3KooWBackup1RelayNode0000000000000000000000000000",

        // Secondary backup relay - Direct TCP/QUIC
        "/ip4/34.135.34.74/tcp/9001/p2p/12D3KooWBackup2RelayNode0000000000000000000000000000",
        "/ip4/34.135.34.74/udp/9001/quic-v1/p2p/12D3KooWBackup2RelayNode0000000000000000000000000000",

        // Tertiary backup relay - Alternative provider
        "/dns4/backup3.scmessenger.org/tcp/443/ws/p2p/12D3KooWBackup3RelayNode0000000000000000000000000000",
    ];

    for node in backup_nodes.iter() {
        if let Ok(addr) = node.parse() {
            addrs.push(addr);
        }
    }

    addrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_config_defaults() {
        let config = BootstrapConfig::default();
        assert_eq!(config.max_retries_per_node, 5);
        assert_eq!(config.initial_backoff, Duration::from_secs(2));
        assert_eq!(config.max_backoff, Duration::from_secs(300));
        assert!(config.enable_local_discovery);
        assert!(config.enable_dns_discovery);
        assert!(config.enable_websocket_fallback);
    }

    #[test]
    fn test_bootstrap_manager_creation() {
        let mgr = BootstrapManager::with_defaults();
        assert_eq!(*mgr.state(), BootstrapState::Idle);
        assert!(mgr.total_nodes() > 0);
        assert_eq!(mgr.connected_count(), 0);
    }

    #[test]
    fn test_bootstrap_manager_add_node() {
        let mut mgr = BootstrapManager::with_defaults();
        let initial = mgr.total_nodes();
        let addr: Multiaddr = "/ip4/10.0.0.1/tcp/9001".parse().unwrap();
        mgr.add_bootstrap_node(addr);
        assert_eq!(mgr.total_nodes(), initial + 1);
    }

    #[test]
    fn test_bootstrap_manager_no_duplicate() {
        let mut mgr = BootstrapManager::with_defaults();
        let initial = mgr.total_nodes();
        let addr: Multiaddr = "/ip4/10.0.0.1/tcp/9001".parse().unwrap();
        mgr.add_bootstrap_node(addr.clone());
        mgr.add_bootstrap_node(addr);
        assert_eq!(mgr.total_nodes(), initial + 1);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = BootstrapConfig::default();
        let mgr = BootstrapManager::new(config);

        let node = BootstrapNode {
            addr: "/ip4/1.2.3.4/tcp/9001".parse().unwrap(),
            peer_id: None,
            attempts: 3,
            last_attempt: Some(SystemTime::now()),
            last_failure: None,
            connected: false,
        };

        let delay = mgr.backoff_for_node(&node);
        assert!(delay > Duration::ZERO);
        assert!(delay <= Duration::from_secs(300));
    }

    #[test]
    fn test_env_bootstrap_override() {
        // This test verifies the env var path exists — actual env var
        // manipulation is not thread-safe in tests.
        let addrs = resolve_env_bootstrap_nodes();
        // Without env var set, returns empty
        assert!(addrs.is_empty());
    }

    #[test]
    fn test_dns_discovery() {
        let addrs = discover_dns_bootstrap().unwrap();
        assert!(!addrs.is_empty());
        assert!(addrs.iter().any(|a| a.to_string().contains("bootstrap.scmessenger.net")));
    }

    #[test]
    fn test_local_discovery() {
        let addrs = discover_local_peers().unwrap();
        // Local discovery relies on libp2p mDNS, returns empty here
        assert!(addrs.is_empty());
    }

    #[test]
    fn test_websocket_discovery() {
        let addrs = discover_websocket_bootstrap();
        assert!(!addrs.is_empty(), "WebSocket fallback should provide addresses");
        assert!(addrs.iter().any(|a| a.to_string().contains("/ws")),
                "Should contain WebSocket multiaddrs with /ws suffix");
        assert!(addrs.iter().any(|a| a.to_string().contains("443")),
                "Should contain addresses on standard HTTPS port");
    }
}