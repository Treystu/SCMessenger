// CLI Transport Bridge - Transport capability extender
//
// This module provides transport path management for the CLI.

use libp2p::PeerId;
use scmessenger_core::transport::abstraction::TransportType;
use std::collections::HashMap;

/// Represents a transport path from WASM through CLI to destination
#[derive(Debug, Clone, PartialEq)]
pub struct TransportPath {
    pub source: TransportType,      // WASM → CLI transport
    pub bridge: TransportType,      // CLI internal transport
    pub destination: TransportType, // CLI → Peer transport
    pub peer_id: PeerId,            // Final destination peer
    pub reliability_score: f32,     // 0.0 (unreliable) to 1.0 (reliable)
    pub latency_estimate: u32,      // Estimated latency in ms
}

// Custom serialization for TransportPath to handle PeerId
impl serde::Serialize for TransportPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("TransportPath", 7)?;
        state.serialize_field("source", &format!("{:?}", self.source))?;
        state.serialize_field("bridge", &format!("{:?}", self.bridge))?;
        state.serialize_field("destination", &format!("{:?}", self.destination))?;
        state.serialize_field("peer_id", &self.peer_id.to_string())?;
        state.serialize_field("reliability_score", &self.reliability_score)?;
        state.serialize_field("latency_estimate", &self.latency_estimate)?;
        state.serialize_field("is_active", &true)?;
        state.end()
    }
}

/// Transport bridge that manages all available paths
pub struct TransportBridge {
    /// Known peers and their transport capabilities
    peer_capabilities: HashMap<PeerId, Vec<TransportType>>,
    /// CLI's own transport capabilities
    cli_capabilities: Vec<TransportType>,
}

impl TransportBridge {
    /// Create a new transport bridge
    pub fn new() -> Self {
        Self {
            peer_capabilities: HashMap::new(),
            cli_capabilities: Self::detect_cli_capabilities(),
        }
    }

    /// Detect CLI transport capabilities
    fn detect_cli_capabilities() -> Vec<TransportType> {
        let mut caps = vec![
            TransportType::Internet, // WebSocket relay + daemon UI bridge
            TransportType::Local,    // TCP/QUIC/mDNS
        ];
        // WiFi Direct deferred (unstable cross-platform). BLE is exposed for the native daemon path.
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        caps.push(TransportType::BLE);
        caps
    }

    /// Register a peer with its transport capabilities
    pub fn register_peer(&mut self, peer_id: PeerId, capabilities: Vec<TransportType>) {
        let capabilities_clone = capabilities.clone();
        self.peer_capabilities.insert(peer_id, capabilities);
        tracing::info!(
            "Registered peer {} with capabilities: {:?}",
            peer_id,
            capabilities_clone
        );
    }

    /// Find all possible transport paths to a peer
    pub fn find_all_paths(&self, peer_id: &PeerId) -> Vec<TransportPath> {
        let mut paths = Vec::new();

        // Get peer capabilities or return empty if unknown
        let peer_caps = match self.peer_capabilities.get(peer_id) {
            Some(caps) => caps,
            None => return paths, // No capabilities known for this peer
        };

        // Generate all possible path combinations
        for wasm_to_cli in self.get_wasm_transports() {
            for cli_to_peer in peer_caps {
                // Skip incompatible combinations
                if !self.is_compatible_path(wasm_to_cli, *cli_to_peer) {
                    continue;
                }

                let path = TransportPath {
                    source: wasm_to_cli,
                    bridge: self.find_cli_bridge_transport(wasm_to_cli, *cli_to_peer),
                    destination: *cli_to_peer,
                    peer_id: *peer_id,
                    reliability_score: self.get_path_reliability(wasm_to_cli, *cli_to_peer),
                    latency_estimate: self.estimate_path_latency(wasm_to_cli, *cli_to_peer),
                };

                paths.push(path);
            }
        }

        // Sort by reliability and latency
        paths.sort_by(|a, b| {
            // First by reliability (higher is better)
            b.reliability_score
                .total_cmp(&a.reliability_score)
                // Then by latency (lower is better)
                .then(a.latency_estimate.cmp(&b.latency_estimate))
        });

        paths
    }

    /// Find the best transport path to a peer
    pub fn find_best_path(&self, peer_id: &PeerId) -> Option<TransportPath> {
        let paths = self.find_all_paths(peer_id);
        paths.into_iter().next() // Return highest ranked path
    }

    /// Get available transports from WASM to CLI
    fn get_wasm_transports(&self) -> Vec<TransportType> {
        vec![
            TransportType::Internet, // Daemon WebSocket JSON-RPC to this CLI
            TransportType::Local,    // Same-machine loopback path
        ]
    }

    /// Check if transport combination is compatible
    fn is_compatible_path(&self, _wasm_to_cli: TransportType, _cli_to_peer: TransportType) -> bool {
        // All combinations are theoretically possible through the CLI bridge
        true
    }

    /// Find the CLI bridge transport for a path
    fn find_cli_bridge_transport(
        &self,
        _wasm_to_cli: TransportType,
        _cli_to_peer: TransportType,
    ) -> TransportType {
        // For now, use Internet as the bridge (will be enhanced)
        TransportType::Internet
    }

    /// Get reliability score for a path combination
    fn get_path_reliability(&self, wasm_to_cli: TransportType, cli_to_peer: TransportType) -> f32 {
        // Default scores based on transport types
        match (wasm_to_cli, cli_to_peer) {
            (TransportType::Local, TransportType::WiFiDirect) => 0.95, // Best: local + high bandwidth
            (TransportType::Local, TransportType::WiFiAware) => 0.90,
            (TransportType::Local, TransportType::BLE) => 0.85,
            (TransportType::Local, TransportType::Local) => 0.90,
            (TransportType::Local, TransportType::Internet) => 0.80,
            (TransportType::Internet, TransportType::WiFiDirect) => 0.85,
            (TransportType::Internet, TransportType::WiFiAware) => 0.80,
            (TransportType::Internet, TransportType::BLE) => 0.75,
            (TransportType::Internet, TransportType::Local) => 0.80,
            (TransportType::Internet, TransportType::Internet) => 0.70,
            _ => 0.60, // Conservative default
        }
    }

    /// Estimate latency for a path combination
    fn estimate_path_latency(&self, wasm_to_cli: TransportType, cli_to_peer: TransportType) -> u32 {
        // Default latency estimates in ms
        match (wasm_to_cli, cli_to_peer) {
            (TransportType::Local, TransportType::WiFiDirect) => 5, // Very fast
            (TransportType::Local, TransportType::WiFiAware) => 10,
            (TransportType::Local, TransportType::BLE) => 20,
            (TransportType::Local, TransportType::Local) => 2,
            (TransportType::Local, TransportType::Internet) => 50,
            (TransportType::Internet, TransportType::WiFiDirect) => 30,
            (TransportType::Internet, TransportType::WiFiAware) => 40,
            (TransportType::Internet, TransportType::BLE) => 60,
            (TransportType::Internet, TransportType::Local) => 20,
            (TransportType::Internet, TransportType::Internet) => 100,
            _ => 150, // Conservative default
        }
    }

    /// Get all available transport paths (for UI display)
    pub fn get_available_paths(&self) -> HashMap<PeerId, Vec<TransportPath>> {
        let mut result = HashMap::new();

        for peer_id in self.peer_capabilities.keys() {
            let paths = self.find_all_paths(peer_id);
            if !paths.is_empty() {
                result.insert(*peer_id, paths);
            }
        }

        result
    }

    /// Get CLI capabilities (for WASM awareness)
    pub fn get_cli_capabilities(&self) -> &[TransportType] {
        &self.cli_capabilities
    }

    /// Get transport capabilities for a specific peer
    pub fn get_peer_capabilities(&self, peer_id: &PeerId) -> Option<&[TransportType]> {
        self.peer_capabilities.get(peer_id).map(|v| v.as_slice())
    }

    /// Get capabilities for all known peers
    pub fn get_available_peer_capabilities(
        &self,
    ) -> std::collections::HashMap<String, Vec<String>> {
        self.peer_capabilities
            .iter()
            .map(|(peer_id, caps)| {
                (
                    peer_id.to_string(),
                    caps.iter().map(|c| format!("{:?}", c)).collect(),
                )
            })
            .collect()
    }

    /// Register peer capabilities
    pub fn register_peer_capabilities(
        &mut self,
        peer_id: PeerId,
        capabilities: Vec<TransportType>,
    ) {
        self.peer_capabilities.insert(peer_id, capabilities);
    }

    /// Check if CLI can forward requests on behalf of WASM
    pub fn can_forward_for_wasm(&self) -> bool {
        // CLI can forward if it has at least one transport capability
        !self.cli_capabilities.is_empty()
    }

    /// Get forwarding capability for specific request type
    pub fn get_forwarding_capability(&self, request_type: &str) -> Option<TransportType> {
        // Map request types to appropriate transport types
        match request_type.to_lowercase().as_str() {
            "message" | "chat" | "direct" => Some(TransportType::Local),
            "relay" | "indirect" => Some(TransportType::Internet),
            "ble" => Some(TransportType::BLE),
            "wifi" | "local_network" => Some(TransportType::WiFiDirect),
            _ => self.cli_capabilities.first().cloned(),
        }
    }

    /// Check if CLI can reach destination via any transport
    pub fn can_reach_destination(&self, peer_id: &PeerId) -> bool {
        if let Some(caps) = self.get_peer_capabilities(peer_id) {
            // Check if we have any compatible transport
            caps.iter().any(|_dest_cap| {
                self.cli_capabilities.iter().any(|_cli_cap| {
                    // Both have at least one transport in common
                    true // Simplified - in reality would check compatibility
                })
            })
        } else {
            false
        }
    }

    /// Get best forwarding path for a request
    pub fn get_best_forwarding_path(&self, peer_id: &PeerId) -> Option<TransportPath> {
        let all_paths = self.find_all_paths(peer_id);
        all_paths.into_iter().max_by(|a, b| {
            // Prioritize by reliability, then latency
            a.reliability_score
                .total_cmp(&b.reliability_score)
                .then(b.latency_estimate.cmp(&a.latency_estimate))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    fn create_test_peer_id(name: &str) -> PeerId {
        // Create a valid Ed25519-based PeerId for testing
        use libp2p::identity::Keypair;
        let mut seed = [0u8; 32];
        for (i, b) in name.bytes().enumerate() {
            if i < 32 {
                seed[i] = b;
            }
        }
        let keypair = Keypair::ed25519_from_bytes(seed).expect("valid seed");
        keypair.public().to_peer_id()
    }

    #[test]
    fn test_transport_bridge_creation() {
        let bridge = TransportBridge::new();
        // Internet + Local always; +BLE on desktop platforms (linux/windows/macos)
        #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
        assert_eq!(bridge.cli_capabilities.len(), 3);
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        assert_eq!(bridge.cli_capabilities.len(), 2);
        assert!(bridge.cli_capabilities.contains(&TransportType::Internet));
        assert!(bridge.cli_capabilities.contains(&TransportType::Local));
    }

    #[test]
    fn test_peer_registration() {
        let mut bridge = TransportBridge::new();
        let peer_id = create_test_peer_id("test-peer");

        let capabilities = vec![
            TransportType::WiFiDirect,
            TransportType::BLE,
            TransportType::Internet,
        ];

        bridge.register_peer(peer_id, capabilities.clone());

        assert_eq!(bridge.peer_capabilities.len(), 1);
        assert_eq!(bridge.peer_capabilities[&peer_id], capabilities);
    }

    #[test]
    fn test_path_finding() {
        let mut bridge = TransportBridge::new();
        let peer_id = create_test_peer_id("android-peer");

        let capabilities = vec![
            TransportType::WiFiDirect,
            TransportType::BLE,
            TransportType::Internet,
        ];

        bridge.register_peer(peer_id, capabilities);

        let paths = bridge.find_all_paths(&peer_id);
        assert!(!paths.is_empty());

        // Should find multiple paths (WASM transports × peer transports)
        assert_eq!(paths.len(), 6); // 2 WASM × 3 peer transports

        // Best path should be Local → WiFiDirect
        let best_path = bridge.find_best_path(&peer_id);
        assert!(best_path.is_some());
        let best_path = best_path.unwrap();
        assert_eq!(best_path.destination, TransportType::WiFiDirect);
    }

    #[test]
    fn test_path_scoring() {
        let bridge = TransportBridge::new();

        // Test default scoring
        let score1 = bridge.get_path_reliability(TransportType::Local, TransportType::WiFiDirect);
        let score2 = bridge.get_path_reliability(TransportType::Internet, TransportType::Internet);

        assert!(score1 > score2); // Local+WiFiDirect should be more reliable than Internet+Internet

        let latency1 =
            bridge.estimate_path_latency(TransportType::Local, TransportType::WiFiDirect);
        let latency2 =
            bridge.estimate_path_latency(TransportType::Internet, TransportType::Internet);

        assert!(latency1 < latency2); // Local+WiFiDirect should be faster
    }
}
