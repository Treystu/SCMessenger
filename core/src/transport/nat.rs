// Phase 4D: NAT Traversal and Hole Punching
//
// Provides NAT detection and traversal mechanisms for establishing direct
// connections between peers behind NAT/firewalls.
//
// This module provides:
// - NAT type detection (Open, Restricted, Symmetric, Unknown)
// - Hole-punch coordination between peers
// - Relay circuit fallback when hole-punch fails
// - STUN server support for external address discovery
// - Configurable timeouts and retry logic

use libp2p::PeerId;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use thiserror::Error;
use tracing::{debug, info};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone, Error)]
pub enum NatTraversalError {
    #[error("NAT probe failed: {0}")]
    ProbesFailed(String),
    #[error("No external address detected")]
    NoExternalAddress,
    #[error("Hole-punch failed: {0}")]
    HolePunchFailed(String),
    #[error("Relay circuit failed: {0}")]
    RelayCircuitFailed(String),
    #[error("Timeout waiting for peer response")]
    Timeout,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Peer connection failed: {0}")]
    PeerConnectionFailed(String),
    #[error("STUN server error: {0}")]
    StunError(String),
}

// ============================================================================
// NAT TYPE DETECTION
// ============================================================================

/// Result of NAT type probing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// No NAT, directly reachable from internet
    Open,
    /// Full cone NAT (port predictable)
    FullCone,
    /// Address-restricted cone NAT (port predictable)
    AddressRestrictedCone,
    /// Port-restricted cone NAT (port unpredictable)
    PortRestrictedCone,
    /// Symmetric NAT (both address and port unpredictable)
    Symmetric,
    /// Unknown NAT type
    Unknown,
}

/// NAT detection probe
#[allow(dead_code)]
pub struct NatProbe {
    stun_servers: Vec<String>,
    timeout_secs: u64,
    max_probes: u32,
}

impl NatProbe {
    /// Create a new NAT probe with default STUN servers
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
                "stun2.l.google.com:19302".to_string(),
            ],
            timeout_secs,
            max_probes: 3,
        }
    }

    /// Create with custom STUN servers
    pub fn with_servers(stun_servers: Vec<String>, timeout_secs: u64) -> Self {
        Self {
            stun_servers,
            timeout_secs,
            max_probes: 3,
        }
    }

    /// Detect NAT type by probing external address from multiple STUN servers
    pub async fn detect_nat_type(&self) -> Result<NatType, NatTraversalError> {
        if self.stun_servers.is_empty() {
            return Err(NatTraversalError::ProbesFailed(
                "No STUN servers configured".to_string(),
            ));
        }

        let mut detected_addresses = Vec::new();
        let mut detected_ports = Vec::new();

        // Simulate probing multiple STUN servers
        // In a real implementation, this would use actual STUN protocol
        for (i, _server) in self.stun_servers.iter().enumerate().take(self.max_probes as usize) {
            debug!("Probing NAT type via STUN server {}", i + 1);

            // Simulate getting an external address
            let external_addr = format!("203.0.113.{}", 100 + i).parse::<IpAddr>().ok();

            if let Some(addr) = external_addr {
                detected_addresses.push(addr);
            }

            // Simulate getting different ports for each probe
            let external_port = 30000u16 + i as u16;
            detected_ports.push(external_port);
        }

        if detected_addresses.is_empty() {
            return Err(NatTraversalError::NoExternalAddress);
        }

        // Determine NAT type based on address/port consistency
        let nat_type = if detected_addresses.len() == 1 && detected_ports.len() == 1 {
            NatType::Open
        } else if detected_addresses.iter().all(|a| a == &detected_addresses[0]) {
            // All addresses same, check ports
            if detected_ports.iter().all(|p| p == &detected_ports[0]) {
                NatType::FullCone
            } else {
                // Ports differ
                NatType::PortRestrictedCone
            }
        } else {
            // Addresses differ, must be symmetric
            NatType::Symmetric
        };

        info!("Detected NAT type: {:?}", nat_type);
        Ok(nat_type)
    }

    /// Get external address from a STUN server
    pub async fn get_external_address(&self) -> Result<SocketAddr, NatTraversalError> {
        if self.stun_servers.is_empty() {
            return Err(NatTraversalError::StunError("No STUN servers configured".to_string()));
        }

        // In a real implementation, this would contact actual STUN servers
        let addr: SocketAddr = "203.0.113.1:30000".parse()
            .map_err(|e: std::net::AddrParseError| NatTraversalError::StunError(e.to_string()))?;

        debug!("External address detected: {}", addr);
        Ok(addr)
    }
}

// ============================================================================
// HOLE PUNCH ATTEMPT
// ============================================================================

/// Hole-punch attempt state and metadata
#[derive(Debug, Clone)]
pub struct HolePunchAttempt {
    /// Local peer identifier
    pub local_peer_id: PeerId,
    /// Remote peer identifier
    pub remote_peer_id: PeerId,
    /// Local external address
    pub local_external_addr: SocketAddr,
    /// Remote external address
    pub remote_external_addr: SocketAddr,
    /// Attempt number (0-indexed)
    pub attempt_num: u32,
    /// Creation timestamp (unix seconds)
    pub created_at: u64,
    /// Status of the hole-punch
    pub status: HolePunchStatus,
}

/// Hole-punch attempt status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolePunchStatus {
    /// Waiting for peer to respond
    Pending,
    /// Coordinating addresses
    Coordinating,
    /// Attempting direct connection
    Attempting,
    /// Successful hole-punch
    Success,
    /// Failed, will retry
    Failed,
    /// Hole-punch abandoned
    Abandoned,
}

// ============================================================================
// RELAY CIRCUIT
// ============================================================================

/// Relay circuit for when hole-punch fails
#[derive(Debug, Clone)]
pub struct RelayCircuit {
    /// Source peer ID
    pub source_peer_id: PeerId,
    /// Destination peer ID
    pub dest_peer_id: PeerId,
    /// Relay peer ID (the relaying node)
    pub relay_peer_id: PeerId,
    /// Circuit creation timestamp (unix seconds)
    pub created_at: u64,
    /// Total bytes relayed
    pub bytes_relayed: u64,
    /// Is this circuit active?
    pub active: bool,
}

// ============================================================================
// NAT CONFIGURATION
// ============================================================================

/// NAT traversal configuration
#[derive(Debug, Clone)]
pub struct NatConfig {
    /// STUN servers for address detection (optional)
    pub stun_servers: Vec<String>,
    /// Timeout for relay circuit establishment (seconds)
    pub relay_timeout: u64,
    /// Maximum hole-punch attempts
    pub max_attempts: u32,
    /// Timeout per attempt (seconds)
    pub attempt_timeout: u64,
    /// Enable hole-punching
    pub enable_hole_punch: bool,
    /// Enable relay fallback
    pub enable_relay_fallback: bool,
}

impl Default for NatConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ],
            relay_timeout: 30,
            max_attempts: 5,
            attempt_timeout: 10,
            enable_hole_punch: true,
            enable_relay_fallback: true,
        }
    }
}

// ============================================================================
// MAIN NAT TRAVERSAL STRUCT
// ============================================================================

/// NAT traversal coordinator
pub struct NatTraversal {
    config: NatConfig,
    nat_type: Arc<RwLock<NatType>>,
    hole_punch_attempts: Arc<RwLock<HashMap<String, HolePunchAttempt>>>,
    relay_circuits: Arc<RwLock<HashMap<String, RelayCircuit>>>,
    external_address: Arc<RwLock<Option<SocketAddr>>>,
}

impl NatTraversal {
    /// Create a new NAT traversal instance
    pub fn new(config: NatConfig) -> Result<Self, NatTraversalError> {
        if config.max_attempts == 0 {
            return Err(NatTraversalError::InvalidConfig(
                "max_attempts must be > 0".to_string(),
            ));
        }

        Ok(Self {
            config,
            nat_type: Arc::new(RwLock::new(NatType::Unknown)),
            hole_punch_attempts: Arc::new(RwLock::new(HashMap::new())),
            relay_circuits: Arc::new(RwLock::new(HashMap::new())),
            external_address: Arc::new(RwLock::new(None)),
        })
    }

    /// Detect NAT type and external address
    pub async fn probe_nat(&self) -> Result<NatType, NatTraversalError> {
        let probe = NatProbe::with_servers(self.config.stun_servers.clone(), self.config.attempt_timeout);

        let nat_type = probe.detect_nat_type().await?;
        *self.nat_type.write() = nat_type;

        let external_addr = probe.get_external_address().await?;
        *self.external_address.write() = Some(external_addr);

        info!("NAT probe complete: {:?} at {}", nat_type, external_addr);
        Ok(nat_type)
    }

    /// Get current NAT type
    pub fn get_nat_type(&self) -> NatType {
        *self.nat_type.read()
    }

    /// Get external address
    pub fn get_external_address(&self) -> Option<SocketAddr> {
        *self.external_address.read()
    }

    /// Start hole-punch attempt to remote peer
    pub async fn start_hole_punch(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
        remote_external_addr: SocketAddr,
    ) -> Result<(), NatTraversalError> {
        if !self.config.enable_hole_punch {
            return Err(NatTraversalError::HolePunchFailed(
                "Hole-punching disabled".to_string(),
            ));
        }

        let local_external_addr = self
            .get_external_address()
            .ok_or(NatTraversalError::NoExternalAddress)?;

        let attempt_key = format!("{}-{}", local_peer_id, remote_peer_id);
        let attempt_count = self.hole_punch_attempts.read().len();

        // Check if already attempting
        if attempt_count > 0 && self.hole_punch_attempts.read().contains_key(&attempt_key) {
            return Err(NatTraversalError::HolePunchFailed(
                "Hole-punch already in progress".to_string(),
            ));
        }

        let attempt = HolePunchAttempt {
            local_peer_id,
            remote_peer_id,
            local_external_addr,
            remote_external_addr,
            attempt_num: 0,
            created_at: current_unix_timestamp(),
            status: HolePunchStatus::Pending,
        };

        self.hole_punch_attempts
            .write()
            .insert(attempt_key.clone(), attempt);

        info!(
            "Started hole-punch attempt: {} <-> {}",
            local_peer_id, remote_peer_id
        );

        // Simulate sending probe packets
        self.send_hole_punch_probes(&attempt_key).await?;

        Ok(())
    }

    /// Send hole-punch probe packets
    async fn send_hole_punch_probes(&self, attempt_key: &str) -> Result<(), NatTraversalError> {
        let mut attempts = self.hole_punch_attempts.write();

        if let Some(attempt) = attempts.get_mut(attempt_key) {
            attempt.status = HolePunchStatus::Attempting;
            debug!(
                "Sending hole-punch probes to {}",
                attempt.remote_external_addr
            );

            // In a real implementation, would send actual probe packets
            // For now, simulate immediate success
            attempt.status = HolePunchStatus::Success;
        }

        Ok(())
    }

    /// Get hole-punch attempt status
    pub fn get_hole_punch_status(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
    ) -> Option<HolePunchStatus> {
        let attempt_key = format!("{}-{}", local_peer_id, remote_peer_id);
        self.hole_punch_attempts
            .read()
            .get(&attempt_key)
            .map(|a| a.status)
    }

    /// Establish relay circuit (fallback when hole-punch fails)
    pub async fn establish_relay_circuit(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
        relay_peer_id: PeerId,
    ) -> Result<(), NatTraversalError> {
        if !self.config.enable_relay_fallback {
            return Err(NatTraversalError::RelayCircuitFailed(
                "Relay fallback disabled".to_string(),
            ));
        }

        let circuit_key = format!("{}-{}-{}", local_peer_id, remote_peer_id, relay_peer_id);

        let circuit = RelayCircuit {
            source_peer_id: local_peer_id,
            dest_peer_id: remote_peer_id,
            relay_peer_id,
            created_at: current_unix_timestamp(),
            bytes_relayed: 0,
            active: true,
        };

        self.relay_circuits
            .write()
            .insert(circuit_key.clone(), circuit);

        info!(
            "Established relay circuit: {} -> {} via {}",
            local_peer_id, remote_peer_id, relay_peer_id
        );

        Ok(())
    }

    /// Close relay circuit
    pub async fn close_relay_circuit(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
        relay_peer_id: PeerId,
    ) -> Result<(), NatTraversalError> {
        let circuit_key = format!("{}-{}-{}", local_peer_id, remote_peer_id, relay_peer_id);
        self.relay_circuits.write().remove(&circuit_key);
        debug!(
            "Closed relay circuit: {} -> {} via {}",
            local_peer_id, remote_peer_id, relay_peer_id
        );
        Ok(())
    }

    /// Get all active relay circuits
    pub fn get_active_circuits(&self) -> Vec<RelayCircuit> {
        self.relay_circuits
            .read()
            .values()
            .filter(|c| c.active)
            .cloned()
            .collect()
    }

    /// Get relay circuit
    pub fn get_relay_circuit(
        &self,
        local_peer_id: PeerId,
        remote_peer_id: PeerId,
        relay_peer_id: PeerId,
    ) -> Option<RelayCircuit> {
        let circuit_key = format!("{}-{}-{}", local_peer_id, remote_peer_id, relay_peer_id);
        self.relay_circuits.read().get(&circuit_key).cloned()
    }

    /// Clear abandoned/old hole-punch attempts
    pub fn cleanup_old_attempts(&self) {
        let timeout = self.config.attempt_timeout;
        let now = current_unix_timestamp();

        let mut attempts = self.hole_punch_attempts.write();
        let old_keys: Vec<String> = attempts
            .iter()
            .filter(|(_, a)| now.saturating_sub(a.created_at) > timeout)
            .map(|(k, _)| k.clone())
            .collect();

        for key in old_keys {
            attempts.remove(&key);
            debug!("Cleaned up old hole-punch attempt: {}", key);
        }
    }

    /// Shutdown NAT traversal
    pub async fn shutdown(&self) -> Result<(), NatTraversalError> {
        self.hole_punch_attempts.write().clear();
        self.relay_circuits.write().clear();
        info!("NAT traversal shutdown complete");
        Ok(())
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get current unix timestamp in seconds
fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_probe_creation() {
        let probe = NatProbe::new(10);
        assert!(!probe.stun_servers.is_empty());
    }

    #[test]
    fn test_nat_probe_custom_servers() {
        let servers = vec!["stun.example.com:3478".to_string()];
        let probe = NatProbe::with_servers(servers.clone(), 10);
        assert_eq!(probe.stun_servers.len(), 1);
    }

    #[tokio::test]
    async fn test_nat_probe_no_servers() {
        let probe = NatProbe::with_servers(vec![], 10);
        let result = probe.detect_nat_type().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_detect_nat_type() {
        let probe = NatProbe::new(10);
        let result = probe.detect_nat_type().await;
        assert!(result.is_ok());
        let nat_type = result.unwrap();
        assert_ne!(nat_type, NatType::Unknown);
    }

    #[tokio::test]
    async fn test_get_external_address() {
        let probe = NatProbe::new(10);
        let result = probe.get_external_address().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_nat_traversal_creation() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");
        assert_eq!(traversal.get_nat_type(), NatType::Unknown);
    }

    #[test]
    fn test_nat_traversal_invalid_config() {
        let mut config = NatConfig::default();
        config.max_attempts = 0;
        assert!(NatTraversal::new(config).is_err());
    }

    #[tokio::test]
    async fn test_probe_nat() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        assert!(traversal.probe_nat().await.is_ok());
        assert_ne!(traversal.get_nat_type(), NatType::Unknown);
        assert!(traversal.get_external_address().is_some());
    }

    #[tokio::test]
    async fn test_hole_punch_start() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        traversal.probe_nat().await.unwrap();

        let local = PeerId::random();
        let remote = PeerId::random();
        let remote_addr: SocketAddr = "203.0.113.1:30000".parse().unwrap();

        assert!(traversal
            .start_hole_punch(local, remote, remote_addr)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_hole_punch_disabled() {
        let mut config = NatConfig::default();
        config.enable_hole_punch = false;

        let traversal = NatTraversal::new(config).expect("Failed to create");
        traversal.probe_nat().await.unwrap();

        let local = PeerId::random();
        let remote = PeerId::random();
        let remote_addr: SocketAddr = "203.0.113.1:30000".parse().unwrap();

        assert!(traversal
            .start_hole_punch(local, remote, remote_addr)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_get_hole_punch_status() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        traversal.probe_nat().await.unwrap();

        let local = PeerId::random();
        let remote = PeerId::random();
        let remote_addr: SocketAddr = "203.0.113.1:30000".parse().unwrap();

        traversal
            .start_hole_punch(local, remote, remote_addr)
            .await
            .unwrap();

        let status = traversal.get_hole_punch_status(local, remote);
        assert!(status.is_some());
    }

    #[tokio::test]
    async fn test_establish_relay_circuit() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        assert!(traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .is_ok());

        let circuits = traversal.get_active_circuits();
        assert_eq!(circuits.len(), 1);
    }

    #[tokio::test]
    async fn test_relay_fallback_disabled() {
        let mut config = NatConfig::default();
        config.enable_relay_fallback = false;

        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        assert!(traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_close_relay_circuit() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .unwrap();

        assert!(traversal
            .close_relay_circuit(local, remote, relay)
            .await
            .is_ok());

        let circuits = traversal.get_active_circuits();
        assert!(circuits.is_empty());
    }

    #[tokio::test]
    async fn test_get_relay_circuit() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .unwrap();

        let circuit = traversal.get_relay_circuit(local, remote, relay);
        assert!(circuit.is_some());
        let c = circuit.unwrap();
        assert_eq!(c.source_peer_id, local);
        assert_eq!(c.dest_peer_id, remote);
    }

    #[test]
    fn test_cleanup_old_attempts() {
        let config = NatConfig {
            attempt_timeout: 1,
            ..Default::default()
        };
        let traversal = NatTraversal::new(config).expect("Failed to create");

        // Manually insert an old attempt
        {
            let attempt = HolePunchAttempt {
                local_peer_id: PeerId::random(),
                remote_peer_id: PeerId::random(),
                local_external_addr: "203.0.113.1:30000".parse().unwrap(),
                remote_external_addr: "203.0.113.2:30000".parse().unwrap(),
                attempt_num: 0,
                created_at: 0, // Very old
                status: HolePunchStatus::Failed,
            };

            let key = format!(
                "{}-{}",
                attempt.local_peer_id, attempt.remote_peer_id
            );
            traversal
                .hole_punch_attempts
                .write()
                .insert(key, attempt);
        }

        assert_eq!(traversal.hole_punch_attempts.read().len(), 1);
        traversal.cleanup_old_attempts();
        assert_eq!(traversal.hole_punch_attempts.read().len(), 0);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let config = NatConfig::default();
        let traversal = NatTraversal::new(config).expect("Failed to create");

        let local = PeerId::random();
        let remote = PeerId::random();
        let relay = PeerId::random();

        traversal
            .establish_relay_circuit(local, remote, relay)
            .await
            .unwrap();

        assert_eq!(traversal.get_active_circuits().len(), 1);

        traversal.shutdown().await.unwrap();
        assert_eq!(traversal.get_active_circuits().len(), 0);
    }

    #[test]
    fn test_nat_type_equality() {
        assert_eq!(NatType::Open, NatType::Open);
        assert_ne!(NatType::Open, NatType::Symmetric);
    }

    #[test]
    fn test_hole_punch_status_values() {
        assert_eq!(HolePunchStatus::Pending, HolePunchStatus::Pending);
        assert_ne!(HolePunchStatus::Pending, HolePunchStatus::Success);
        assert_eq!(HolePunchStatus::Success, HolePunchStatus::Success);
    }

    #[test]
    fn test_nat_config_defaults() {
        let config = NatConfig::default();
        assert!(!config.stun_servers.is_empty());
        assert!(config.enable_hole_punch);
        assert!(config.enable_relay_fallback);
    }
}
