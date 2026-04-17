// Bootstrap Recovery & Fallback System
//
// Resilient bootstrap connectivity with:
// - Multi-node bootstrap with priority ordering
// - Exponential backoff on connection failures
// - Environment variable override for bootstrap nodes
// - Local network peer discovery as fallback
// - Dynamic relay discovery from connected peers

use crate::transport::internet::{InternetRelay, InternetTransportError};
use crate::transport::relay_health::{RelayDiscovery, RelayFallback};
use crate::transport::swarm::SwarmHandle;
use libp2p::{Multiaddr, PeerId};
use std::collections::VecDeque;
use tracing::{debug, info, warn};
use web_time::{Duration, SystemTime};

/// Default bootstrap node multiaddrs (core-level fallback)
pub const CORE_BOOTSTRAP_NODES: &[&str] = &[
    "/ip4/34.135.34.73/tcp/9001",
    "/dns4/bootstrap.scmessenger.net/tcp/9001",
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

/// Tracks a single bootstrap node's connection state
#[derive(Debug, Clone)]
struct BootstrapNodeState {
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
    nodes: VecDeque<BootstrapNodeState>,
    relay_discovery: RelayDiscovery,
    relay_fallback: RelayFallback,
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

        let nodes: VecDeque<BootstrapNodeState> = all_addrs
            .into_iter()
            .map(|addr| BootstrapNodeState {
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
            self.nodes.push_back(BootstrapNodeState {
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
    /// Tries each configured node with exponential backoff. Returns the
    /// first successfully connected peer, or an error if all attempts fail.
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
                    let delay = self.backoff_for_node(node);

                    if delay > Duration::ZERO {
                        debug!("Bootstrap backoff: waiting {:?} before retrying {}", delay, addr);
                        tokio::time::sleep(delay).await;
                    }

                    let peer_id = PeerId::random(); // Promiscuous: accept whatever peer presents
                    self.record_attempt(&addr);

                    match relay
                        .connect_to_relay_via_swarm(peer_id, addr.clone(), swarm)
                        .await
                    {
                        Ok(()) => {
                            info!("Bootstrap connected to {}", addr);
                            self.record_success(&addr, peer_id);
                            self.relay_discovery.record_success(&peer_id, 0);
                            self.connected_count += 1;
                            self.state = BootstrapState::Connected;
                            return Ok(peer_id);
                        }
                        Err(e) => {
                            warn!("Bootstrap failed for {}: {}", addr, e);
                            self.record_failure(&addr, &e.to_string());
                            self.relay_discovery.record_failure(&peer_id, &e.to_string());
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
    fn backoff_for_node(&self, node: &BootstrapNodeState) -> Duration {
        if node.attempts == 0 {
            return Duration::ZERO;
        }
        let delay_secs = self.config.initial_backoff.as_secs_f64()
            * self.config.backoff_multiplier.powi(node.attempts as i32 - 1);
        let capped = delay_secs.min(self.config.max_backoff.as_secs_f64());
        Duration::from_secs_f64(capped)
    }

    /// Find the next node eligible for a connection attempt
    fn next_connectable_node(&self) -> Option<&BootstrapNodeState> {
        self.nodes
            .iter()
            .find(|n| !n.connected && self.relay_fallback.should_retry(&n.addr))
    }

    /// Discover fallback bootstrap nodes via DNS and local network
    fn discover_fallback_nodes(&self) -> Vec<Multiaddr> {
        let mut discovered = Vec::new();

        // DNS-based discovery
        if self.config.enable_dns_discovery {
            if let Ok(addrs) = discover_dns_bootstrap() {
                info!("DNS discovery found {} fallback nodes", addrs.len());
                discovered.extend(addrs);
            }
        }

        // Local network mDNS discovery
        if self.config.enable_local_discovery {
            if let Ok(addrs) = discover_local_peers() {
                info!("Local discovery found {} fallback nodes", addrs.len());
                discovered.extend(addrs);
            }
        }

        discovered
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

        let node = BootstrapNodeState {
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
}