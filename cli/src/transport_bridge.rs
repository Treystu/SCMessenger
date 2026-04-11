// CLI Transport Bridge - Universal transport capability extender for WASM
//
// This module transforms the CLI from a simple relay into a sophisticated
// transport bridge that can leverage multiple transport types and provide
// transport awareness to browser-based WASM clients.
//
// NOTE: Some code is currently unused as the transport API endpoints are temporarily
// disabled due to warp filter chaining complexity. This will be activated in future.
#[allow(dead_code)]

use crate::api::ApiContext;
use libp2p::PeerId;
use scmessenger_core::transport::abstraction::TransportType;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents a transport path from WASM through CLI to destination
#[derive(Debug, Clone, PartialEq)]
pub struct TransportPath {
    pub source: TransportType,      // WASM → CLI transport
    pub bridge: TransportType,      // CLI internal transport
    pub destination: TransportType, // CLI → Peer transport
    pub peer_id: PeerId,            // Final destination peer
    pub reliability_score: f32,     // 0.0 (unreliable) to 1.0 (reliable)
    pub latency_estimate: u32,     // Estimated latency in ms
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
    /// WASM peer ID (browser client)
    wasm_peer_id: Option<PeerId>,
    /// Known peers and their transport capabilities
    peer_capabilities: HashMap<PeerId, Vec<TransportType>>,
    /// CLI's own transport capabilities
    cli_capabilities: Vec<TransportType>,
    /// Current active paths
    active_paths: HashMap<PeerId, TransportPath>,
    /// Path performance statistics
    path_stats: HashMap<String, PathStatistics>,
    /// API context for communication
    api_context: Option<Arc<Mutex<ApiContext>>>,
}

/// Performance statistics for a transport path
#[derive(Debug, Clone, Default)]
struct PathStatistics {
    success_count: u32,
    failure_count: u32,
    total_latency: u64,
    message_count: u32,
}

impl PathStatistics {
    fn reliability_score(&self) -> f32 {
        if self.message_count == 0 {
            0.5 // Neutral score for untried paths
        } else {
            self.success_count as f32 / self.message_count as f32
        }
    }
    
    fn average_latency(&self) -> u32 {
        if self.success_count == 0 {
            0
        } else {
            (self.total_latency / self.success_count as u64) as u32
        }
    }
}

impl TransportBridge {
    /// Create a new transport bridge
    pub fn new() -> Self {
        Self {
            wasm_peer_id: None,
            peer_capabilities: HashMap::new(),
            cli_capabilities: Self::detect_cli_capabilities(),
            active_paths: HashMap::new(),
            path_stats: HashMap::new(),
            api_context: None,
        }
    }
    
    /// Set API context for communication
    pub fn with_api_context(mut self, ctx: Arc<Mutex<ApiContext>>) -> Self {
        self.api_context = Some(ctx);
        self
    }
    
    /// Set WASM peer ID
    pub fn set_wasm_peer(&mut self, peer_id: PeerId) {
        self.wasm_peer_id = Some(peer_id);
    }
    
    /// Detect CLI transport capabilities
    fn detect_cli_capabilities() -> Vec<TransportType> {
        let mut caps = vec![
            TransportType::Internet, // WebSocket relay + daemon UI bridge
            TransportType::Local,    // TCP/QUIC/mDNS
        ];
        // WiFi Direct deferred (unstable cross-platform). BLE is exposed for the native daemon path.
        #[cfg(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ))]
        caps.push(TransportType::BLE);
        caps
    }
    
    /// Register a peer with its transport capabilities
    pub fn register_peer(&mut self, peer_id: PeerId, capabilities: Vec<TransportType>) {
        let capabilities_clone = capabilities.clone();
        self.peer_capabilities.insert(peer_id, capabilities);
        tracing::info!("Registered peer {} with capabilities: {:?}", peer_id, capabilities_clone);
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
            b.reliability_score.total_cmp(&a.reliability_score)
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
    fn find_cli_bridge_transport(&self, _wasm_to_cli: TransportType, _cli_to_peer: TransportType) -> TransportType {
        // For now, use Internet as the bridge (will be enhanced)
        TransportType::Internet
    }
    
    /// Get reliability score for a path combination
    fn get_path_reliability(&self, wasm_to_cli: TransportType, cli_to_peer: TransportType) -> f32 {
        let path_key = format!("{:?}-{:?}", wasm_to_cli, cli_to_peer);
        
        if let Some(stats) = self.path_stats.get(&path_key) {
            return stats.reliability_score();
        }
        
        // Default scores based on transport types
        match (wasm_to_cli, cli_to_peer) {
            (TransportType::Local, TransportType::WiFiDirect) => 0.95,  // Best: local + high bandwidth
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
        let path_key = format!("{:?}-{:?}", wasm_to_cli, cli_to_peer);
        
        if let Some(stats) = self.path_stats.get(&path_key) {
            return stats.average_latency();
        }
        
        // Default latency estimates in ms
        match (wasm_to_cli, cli_to_peer) {
            (TransportType::Local, TransportType::WiFiDirect) => 5,    // Very fast
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
    
    /// Update path statistics based on message delivery outcome
    pub fn update_path_stats(&mut self, path: &TransportPath, success: bool, latency: u32) {
        let path_key = format!("{:?}-{:?}", path.source, path.destination);
        
        let stats = self.path_stats.entry(path_key).or_default();
        
        if success {
            stats.success_count += 1;
            stats.total_latency += latency as u64;
        } else {
            stats.failure_count += 1;
        }
        
        stats.message_count += 1;
        
        tracing::debug!(
            "Updated stats for path {:?}-{:?}: reliability={:.2}, avg_latency={}ms",
            path.source, path.destination, stats.reliability_score(), stats.average_latency()
        );
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
    pub fn get_available_peer_capabilities(&self) -> std::collections::HashMap<String, Vec<String>> {
        self.peer_capabilities.iter()
            .map(|(peer_id, caps)| {
                (peer_id.to_string(), 
                 caps.iter().map(|c| format!("{:?}", c)).collect())
            })
            .collect()
    }

    /// Register peer capabilities
    pub fn register_peer_capabilities(&mut self, peer_id: PeerId, capabilities: Vec<TransportType>) {
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
            caps.iter().any(|dest_cap| {
                self.cli_capabilities.iter().any(|cli_cap| {
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
            a.reliability_score.total_cmp(&b.reliability_score)
                .then(b.latency_estimate.cmp(&a.latency_estimate))
        })
    }
}

/// Transport path with additional routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportRoute {
    pub peer_id: String,
    pub path_type: String,
    pub source_transport: String,
    pub bridge_transport: String,
    pub destination_transport: String,
    pub reliability: f32,
    pub estimated_latency: u32,
    pub is_active: bool,
}

impl From<&TransportPath> for TransportRoute {
    fn from(path: &TransportPath) -> Self {
        Self {
            peer_id: path.peer_id.to_string(),
            path_type: format!("{:?}-{:?}", path.source, path.destination),
            source_transport: path.source.to_string(),
            bridge_transport: path.bridge.to_string(),
            destination_transport: path.destination.to_string(),
            reliability: path.reliability_score,
            estimated_latency: path.latency_estimate,
            is_active: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;
    
    fn create_test_peer_id(name: &str) -> PeerId {
        // Create deterministic peer ID for testing
        let mut bytes = [0u8; 32];
        for (i, b) in name.bytes().enumerate() {
            if i < 32 {
                bytes[i] = b;
            }
        }
        PeerId::from_bytes(&bytes).unwrap()
    }
    
    #[test]
    fn test_transport_bridge_creation() {
        let bridge = TransportBridge::new();
        assert_eq!(bridge.cli_capabilities.len(), 2); // Internet + Local
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
        
        let latency1 = bridge.estimate_path_latency(TransportType::Local, TransportType::WiFiDirect);
        let latency2 = bridge.estimate_path_latency(TransportType::Internet, TransportType::Internet);
        
        assert!(latency1 < latency2); // Local+WiFiDirect should be faster
    }
    
    #[test]
    fn test_stats_update() {
        let mut bridge = TransportBridge::new();
        let peer_id = create_test_peer_id("stats-test");
        
        bridge.register_peer(peer_id, vec![TransportType::Internet]);
        let path = bridge.find_best_path(&peer_id).unwrap();
        
        // Initial stats should be default
        assert!(path.reliability_score > 0.6);
        
        // Update with success
        bridge.update_path_stats(&path, true, 45);
        
        // Get updated path
        let updated_path = bridge.find_best_path(&peer_id).unwrap();
        assert_eq!(updated_path.reliability_score, 1.0); // Should be 100% after first success
    }
}