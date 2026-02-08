//! Transport Manager — multiplexes multiple transports
//!
//! This module coordinates transport abstraction and intelligently selects
//! the best transport for each peer based on capabilities and connection state.

use crate::transport::abstraction::{
    TransportCapabilities, TransportEvent, TransportType, TransportError,
};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// State of a registered transport
#[derive(Debug, Clone)]
pub struct TransportState {
    /// Whether this transport is currently running
    pub running: bool,
    /// Set of connected peer IDs via this transport
    pub connected_peers: HashSet<[u8; 32]>,
    /// Capabilities of this transport
    pub capabilities: TransportCapabilities,
}

impl TransportState {
    /// Create a new transport state
    pub fn new(capabilities: TransportCapabilities) -> Self {
        Self {
            running: false,
            connected_peers: HashSet::new(),
            capabilities,
        }
    }
}

/// Pending outgoing data
#[derive(Debug, Clone)]
pub struct PendingSend {
    /// Target peer ID
    pub peer_id: [u8; 32],
    /// Data to send
    pub data: Vec<u8>,
    /// Priority (0-255, higher is more important)
    pub priority: u8,
    /// Preferred transport for this send (if None, any is acceptable)
    pub preferred_transport: Option<TransportType>,
    /// When this was queued
    pub created_at: SystemTime,
}

/// Priority queue of outgoing data
#[derive(Debug, Clone)]
pub struct OutgoingQueue {
    items: Vec<PendingSend>,
}

impl OutgoingQueue {
    /// Create a new outgoing queue
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add an item to the queue (maintains priority order)
    pub fn enqueue(&mut self, item: PendingSend) {
        self.items.push(item);
        // Sort so highest priority items are first
        self.items.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Dequeue the highest priority item
    pub fn dequeue(&mut self) -> Option<PendingSend> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    /// Get the count of pending items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Clear all pending sends
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Default for OutgoingQueue {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RECONNECTION WITH EXPONENTIAL BACKOFF
// ============================================================================

/// Minimum backoff interval for reconnection attempts
const RECONNECT_BASE_INTERVAL: Duration = Duration::from_secs(1);

/// Maximum backoff interval (capped to prevent absurd waits)
const RECONNECT_MAX_INTERVAL: Duration = Duration::from_secs(60);

/// Backoff multiplier per failed attempt
const RECONNECT_BACKOFF_MULTIPLIER: u32 = 2;

/// Maximum consecutive failures before giving up on a peer
const RECONNECT_MAX_FAILURES: u32 = 10;

/// Maximum peers to reconnect per tick cycle (prevents Resume Storm —
/// all peers reconnecting simultaneously after app wake, overwhelming the OS)
const RECONNECT_MAX_CONCURRENT: usize = 3;

/// Minimum interval between successive reconnection dials (stagger)
const RECONNECT_STAGGER_INTERVAL: Duration = Duration::from_millis(200);

/// Per-peer reconnection state with exponential backoff
#[derive(Debug, Clone)]
pub struct ReconnectionState {
    /// The peer we want to reconnect to
    pub peer_id: [u8; 32],
    /// Last known transport(s) for this peer
    pub last_transports: HashSet<TransportType>,
    /// Last known address bytes (opaque to manager, passed through to transport)
    pub last_addr: Vec<u8>,
    /// Number of consecutive failed reconnection attempts
    pub failures: u32,
    /// When the next reconnection attempt is allowed
    pub next_attempt_at: SystemTime,
    /// When this peer was first lost
    pub disconnected_at: SystemTime,
}

impl ReconnectionState {
    fn new(peer_id: [u8; 32], transports: HashSet<TransportType>, addr: Vec<u8>) -> Self {
        Self {
            peer_id,
            last_transports: transports,
            last_addr: addr,
            failures: 0,
            next_attempt_at: SystemTime::now() + RECONNECT_BASE_INTERVAL,
            disconnected_at: SystemTime::now(),
        }
    }

    /// Calculate the next backoff interval after a failed attempt
    fn backoff_interval(&self) -> Duration {
        let base = RECONNECT_BASE_INTERVAL.as_millis() as u64;
        let multiplier = RECONNECT_BACKOFF_MULTIPLIER
            .checked_pow(self.failures)
            .unwrap_or(u32::MAX) as u64;
        let interval_ms = base.saturating_mul(multiplier);
        let capped = Duration::from_millis(interval_ms).min(RECONNECT_MAX_INTERVAL);
        capped
    }

    /// Record a failed reconnection attempt, advancing the backoff
    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.next_attempt_at = SystemTime::now() + self.backoff_interval();
    }

    /// Whether this peer has exceeded the maximum failure count
    pub fn is_exhausted(&self) -> bool {
        self.failures >= RECONNECT_MAX_FAILURES
    }

    /// Whether enough time has passed to attempt reconnection
    pub fn is_ready(&self) -> bool {
        SystemTime::now() >= self.next_attempt_at
    }
}

/// Result of queuing a send — explicitly NOT a delivery confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendResult {
    /// Message queued for delivery via the specified transport.
    /// This does NOT mean the peer received it.
    Queued(TransportType),
}

/// Manages multiple transports and provides intelligent transport selection
pub struct TransportManager {
    /// Transport state per transport type
    transports: Arc<RwLock<HashMap<TransportType, TransportState>>>,

    /// Maps peer IDs to available transports
    peer_transports: Arc<RwLock<HashMap<[u8; 32], HashSet<TransportType>>>>,

    /// Pending outgoing data
    outgoing: Arc<RwLock<OutgoingQueue>>,

    /// Last time each peer was seen
    peer_last_seen: Arc<RwLock<HashMap<[u8; 32], SystemTime>>>,

    /// Peers we want to stay connected to (survive disconnects)
    target_peers: Arc<RwLock<HashMap<[u8; 32], Vec<u8>>>>,

    /// Peers awaiting reconnection with backoff state
    reconnection_queue: Arc<RwLock<HashMap<[u8; 32], ReconnectionState>>>,
}

impl TransportManager {
    /// Create a new transport manager
    pub fn new() -> Self {
        Self {
            transports: Arc::new(RwLock::new(HashMap::new())),
            peer_transports: Arc::new(RwLock::new(HashMap::new())),
            outgoing: Arc::new(RwLock::new(OutgoingQueue::new())),
            peer_last_seen: Arc::new(RwLock::new(HashMap::new())),
            target_peers: Arc::new(RwLock::new(HashMap::new())),
            reconnection_queue: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a transport with capabilities
    pub fn register_transport(&self, transport_type: TransportType, capabilities: TransportCapabilities) {
        let mut transports = self.transports.write();
        transports.insert(transport_type, TransportState::new(capabilities));
        info!("Transport registered: {}", transport_type);
    }

    /// Handle a transport event
    pub fn handle_event(&self, event: TransportEvent) {
        match event {
            TransportEvent::PeerDiscovered {
                peer_id,
                transport,
                ..
            } => {
                let mut peer_transports = self.peer_transports.write();
                peer_transports
                    .entry(peer_id)
                    .or_insert_with(HashSet::new)
                    .insert(transport);

                let mut last_seen = self.peer_last_seen.write();
                last_seen.insert(peer_id, SystemTime::now());

                debug!("Peer {:x?} discovered on {}", &peer_id[..8], transport);
            }
            TransportEvent::PeerDisconnected {
                peer_id,
                transport,
            } => {
                let mut peer_transports = self.peer_transports.write();
                if let Some(transports) = peer_transports.get_mut(&peer_id) {
                    transports.remove(&transport);
                    if transports.is_empty() {
                        peer_transports.remove(&peer_id);

                        // If this was a target peer, queue for reconnection
                        let target_peers = self.target_peers.read();
                        if let Some(addr) = target_peers.get(&peer_id) {
                            let mut reconnect_queue = self.reconnection_queue.write();
                            if !reconnect_queue.contains_key(&peer_id) {
                                let mut known_transports = HashSet::new();
                                known_transports.insert(transport);
                                reconnect_queue.insert(
                                    peer_id,
                                    ReconnectionState::new(peer_id, known_transports, addr.clone()),
                                );
                                info!(
                                    "Peer {:x?} lost on {} — queued for reconnection",
                                    &peer_id[..8], transport
                                );
                            }
                        }
                    }
                }
                debug!("Peer {:x?} disconnected from {}", &peer_id[..8], transport);
            }
            TransportEvent::DataReceived { peer_id, .. } => {
                let mut last_seen = self.peer_last_seen.write();
                last_seen.insert(peer_id, SystemTime::now());
            }
            TransportEvent::ConnectionEstablished { peer_id, transport } => {
                let mut transports = self.transports.write();
                if let Some(state) = transports.get_mut(&transport) {
                    state.connected_peers.insert(peer_id);
                }
                debug!("Connection established to {:x?} via {}", &peer_id[..8], transport);
            }
            TransportEvent::TransportError { .. } => {
                // Log and continue
            }
        }
    }

    /// Queue data for delivery to a peer via the best available transport.
    ///
    /// **Important:** Returns `SendResult::Queued`, which means the message is
    /// in the outgoing queue — NOT that the peer has received it. Actual delivery
    /// confirmation requires an application-level receipt (see `CoreDelegate::on_receipt_received`).
    pub fn send_to_peer(&self, peer_id: [u8; 32], data: Vec<u8>, priority: u8) -> Result<SendResult, TransportError> {
        let best = self.best_transport_for_peer(peer_id)?;

        let mut outgoing = self.outgoing.write();
        outgoing.enqueue(PendingSend {
            peer_id,
            data,
            priority,
            preferred_transport: Some(best),
            created_at: SystemTime::now(),
        });

        Ok(SendResult::Queued(best))
    }

    /// Determine the best transport for a peer
    pub fn best_transport_for_peer(&self, peer_id: [u8; 32]) -> Result<TransportType, TransportError> {
        let peer_transports = self.peer_transports.read();
        let available = peer_transports
            .get(&peer_id)
            .ok_or(TransportError::PeerNotFound(format!("{:x?}", &peer_id[..8])))?;

        if available.is_empty() {
            return Err(TransportError::PeerNotFound(format!("{:x?}", &peer_id[..8])));
        }

        let transports = self.transports.read();

        // Score each transport and pick the best
        let best = available.iter().max_by_key(|&&transport_type| {
            let state = transports.get(&transport_type);
            let caps = state.map(|s| &s.capabilities);

            let mut score = 0u64;

            // Prefer connected transports
            if let Some(state) = state {
                if state.connected_peers.contains(&peer_id) {
                    score += 1000;
                }
            }

            // Prefer streaming capability
            if let Some(caps) = caps {
                if caps.supports_streaming {
                    score += 500;
                }

                // Bandwidth
                let bandwidth_score = std::cmp::min(
                    100,
                    (caps.estimated_bandwidth_bps / 1_000_000) as u64,
                );
                score += bandwidth_score * 5;

                // Prefer lower latency
                let latency_score = std::cmp::max(0, 100 - caps.estimated_latency_ms as u64);
                score += latency_score;
            }

            score
        });

        best.copied().ok_or(TransportError::PeerNotFound(format!("{:x?}", &peer_id[..8])))
    }

    /// Get all discovered peers
    pub fn connected_peers(&self) -> Vec<[u8; 32]> {
        let peer_transports = self.peer_transports.read();
        peer_transports.keys().copied().collect()
    }

    /// Get peers on a specific transport
    pub fn peers_on_transport(&self, transport: TransportType) -> Vec<[u8; 32]> {
        let transports = self.transports.read();
        if let Some(state) = transports.get(&transport) {
            state.connected_peers.iter().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a peer is connected on any transport
    pub fn is_peer_connected(&self, peer_id: [u8; 32]) -> bool {
        let peer_transports = self.peer_transports.read();
        peer_transports
            .get(&peer_id)
            .map(|transports| !transports.is_empty())
            .unwrap_or(false)
    }

    /// Get all transports where a peer is connected
    pub fn transports_for_peer(&self, peer_id: [u8; 32]) -> Vec<TransportType> {
        let peer_transports = self.peer_transports.read();
        peer_transports
            .get(&peer_id)
            .map(|transports| transports.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get the pending sends queue
    pub fn pending_sends(&self) -> Vec<PendingSend> {
        let outgoing = self.outgoing.read();
        outgoing.items.clone()
    }

    // --------------------------------------------------------------------
    // RECONNECTION MANAGEMENT
    // --------------------------------------------------------------------

    /// Register a peer as a "target peer" — one we want to stay connected to.
    /// When a target peer disconnects, it's automatically queued for reconnection
    /// with exponential backoff.
    pub fn add_target_peer(&self, peer_id: [u8; 32], addr: Vec<u8>) {
        self.target_peers.write().insert(peer_id, addr);
        debug!("Added target peer {:x?}", &peer_id[..8]);
    }

    /// Remove a peer from the target set. Stops reconnection attempts.
    pub fn remove_target_peer(&self, peer_id: &[u8; 32]) {
        self.target_peers.write().remove(peer_id);
        self.reconnection_queue.write().remove(peer_id);
        debug!("Removed target peer {:x?}", &peer_id[..8]);
    }

    /// Returns peers that are due for a reconnection attempt.
    /// The caller is responsible for actually dialing these peers and then
    /// calling `record_reconnect_success` or `record_reconnect_failure`.
    /// Returns peers ready for reconnection, rate-limited to prevent Resume Storm.
    ///
    /// After app resume, all disconnected peers become ready simultaneously.
    /// This method caps the batch to `RECONNECT_MAX_CONCURRENT` peers per tick,
    /// with staggered `next_attempt_at` times applied to remaining peers so they
    /// spread across subsequent ticks instead of all firing at once.
    pub fn peers_needing_reconnect(&self) -> Vec<ReconnectionState> {
        let mut queue = self.reconnection_queue.write();

        let mut ready: Vec<[u8; 32]> = queue
            .iter()
            .filter(|(_, state)| state.is_ready() && !state.is_exhausted())
            .map(|(id, _)| *id)
            .collect();

        // If more peers are ready than our concurrency limit, stagger the excess
        if ready.len() > RECONNECT_MAX_CONCURRENT {
            // Sort by disconnected_at so longest-waiting peers go first
            ready.sort_by(|a, b| {
                let a_disc = queue.get(a).map(|s| s.disconnected_at).unwrap_or(SystemTime::UNIX_EPOCH);
                let b_disc = queue.get(b).map(|s| s.disconnected_at).unwrap_or(SystemTime::UNIX_EPOCH);
                a_disc.cmp(&b_disc)
            });

            // Stagger the ones we're NOT returning this tick
            for (i, peer_id) in ready[RECONNECT_MAX_CONCURRENT..].iter().enumerate() {
                if let Some(state) = queue.get_mut(peer_id) {
                    state.next_attempt_at = SystemTime::now()
                        + RECONNECT_STAGGER_INTERVAL * (i as u32 + 1);
                }
            }

            ready.truncate(RECONNECT_MAX_CONCURRENT);
        }

        ready
            .iter()
            .filter_map(|id| queue.get(id).cloned())
            .collect()
    }

    /// Record a successful reconnection — removes from reconnect queue.
    pub fn record_reconnect_success(&self, peer_id: &[u8; 32]) {
        self.reconnection_queue.write().remove(peer_id);
        info!("Reconnected to peer {:x?}", &peer_id[..8]);
    }

    /// Record a failed reconnection attempt — advances the backoff timer.
    pub fn record_reconnect_failure(&self, peer_id: &[u8; 32]) {
        let mut queue = self.reconnection_queue.write();
        if let Some(state) = queue.get_mut(peer_id) {
            state.record_failure();
            if state.is_exhausted() {
                warn!(
                    "Peer {:x?} exhausted {} reconnection attempts — giving up",
                    &peer_id[..8],
                    RECONNECT_MAX_FAILURES
                );
            } else {
                debug!(
                    "Reconnection to {:x?} failed (attempt {}), next try in {:?}",
                    &peer_id[..8],
                    state.failures,
                    state.backoff_interval()
                );
            }
        }
    }

    /// How many peers are currently in the reconnection queue
    pub fn reconnection_queue_len(&self) -> usize {
        self.reconnection_queue.read().len()
    }

    /// Maintenance: clean up stale peer entries and prune exhausted reconnections
    pub fn tick(&self) {
        let now = SystemTime::now();
        let mut last_seen = self.peer_last_seen.write();
        let mut peer_transports = self.peer_transports.write();

        // Remove peers not seen for 5 minutes
        last_seen.retain(|peer_id, seen_at| {
            match now.duration_since(*seen_at) {
                Ok(elapsed) if elapsed.as_secs() > 300 => {
                    peer_transports.remove(peer_id);
                    false
                }
                Err(_) => false,
                Ok(_) => true,
            }
        });

        // Prune exhausted reconnection entries
        self.reconnection_queue.write().retain(|peer_id, state| {
            if state.is_exhausted() {
                info!("Pruning exhausted reconnection entry for {:x?}", &peer_id[..8]);
                false
            } else {
                true
            }
        });
    }
}

impl Default for TransportManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_peer_id(val: u8) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = val;
        id
    }

    #[test]
    fn test_transport_state_creation() {
        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        let state = TransportState::new(caps);
        assert!(!state.running);
        assert!(state.connected_peers.is_empty());
    }

    #[test]
    fn test_outgoing_queue_fifo_with_priority() {
        let mut queue = OutgoingQueue::new();

        let send1 = PendingSend {
            peer_id: create_peer_id(1),
            data: vec![1],
            priority: 5,
            preferred_transport: None,
            created_at: SystemTime::now(),
        };

        let send2 = PendingSend {
            peer_id: create_peer_id(2),
            data: vec![2],
            priority: 10,
            preferred_transport: None,
            created_at: SystemTime::now(),
        };

        queue.enqueue(send1);
        queue.enqueue(send2);

        // Higher priority should dequeue first
        let first = queue.dequeue().unwrap();
        assert_eq!(first.priority, 10);

        let second = queue.dequeue().unwrap();
        assert_eq!(second.priority, 5);

        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_outgoing_queue_len() {
        let mut queue = OutgoingQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);

        queue.enqueue(PendingSend {
            peer_id: create_peer_id(1),
            data: vec![1],
            priority: 1,
            preferred_transport: None,
            created_at: SystemTime::now(),
        });

        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[test]
    fn test_outgoing_queue_clear() {
        let mut queue = OutgoingQueue::new();
        queue.enqueue(PendingSend {
            peer_id: create_peer_id(1),
            data: vec![1],
            priority: 1,
            preferred_transport: None,
            created_at: SystemTime::now(),
        });

        assert_eq!(queue.len(), 1);
        queue.clear();
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_transport_manager_creation() {
        let manager = TransportManager::new();
        assert_eq!(manager.connected_peers().len(), 0);
    }

    #[test]
    fn test_register_transport() {
        let manager = TransportManager::new();
        let caps = TransportCapabilities::for_transport(TransportType::BLE);

        manager.register_transport(TransportType::BLE, caps);

        let peers = manager.peers_on_transport(TransportType::BLE);
        assert_eq!(peers.len(), 0);
    }

    #[test]
    fn test_peer_discovered_event() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };

        manager.handle_event(event);

        let peers = manager.connected_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0], peer_id);
    }

    #[test]
    fn test_peer_disconnected_event() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(discovered);

        assert_eq!(manager.connected_peers().len(), 1);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        assert_eq!(manager.connected_peers().len(), 0);
    }

    #[test]
    fn test_multiple_transports_per_peer() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiDirect,
            addr: vec![4, 5, 6],
        };
        manager.handle_event(event2);

        let transports = manager.transports_for_peer(peer_id);
        assert_eq!(transports.len(), 2);
    }

    #[test]
    fn test_best_transport_prefers_connected() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps_ble = TransportCapabilities::for_transport(TransportType::BLE);
        let caps_wifi = TransportCapabilities::for_transport(TransportType::WiFiDirect);

        manager.register_transport(TransportType::BLE, caps_ble);
        manager.register_transport(TransportType::WiFiDirect, caps_wifi);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(discovered);

        let discovered2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiDirect,
            addr: vec![4, 5, 6],
        };
        manager.handle_event(discovered2);

        let established = TransportEvent::ConnectionEstablished {
            peer_id,
            transport: TransportType::WiFiDirect,
        };
        manager.handle_event(established);

        let best = manager.best_transport_for_peer(peer_id).expect("should have transport");
        assert_eq!(best, TransportType::WiFiDirect);
    }

    #[test]
    fn test_best_transport_prefers_streaming() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(2);

        let caps_ble = TransportCapabilities::for_transport(TransportType::BLE);
        let caps_aware = TransportCapabilities::for_transport(TransportType::WiFiAware);

        manager.register_transport(TransportType::BLE, caps_ble);
        manager.register_transport(TransportType::WiFiAware, caps_aware);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::WiFiAware,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let best = manager.best_transport_for_peer(peer_id).expect("should have transport");
        assert_eq!(best, TransportType::WiFiAware);
    }

    #[test]
    fn test_best_transport_prefers_low_latency() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(3);

        let caps_internet = TransportCapabilities::for_transport(TransportType::Internet);
        let caps_local = TransportCapabilities::for_transport(TransportType::Local);

        manager.register_transport(TransportType::Internet, caps_internet);
        manager.register_transport(TransportType::Local, caps_local);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::Internet,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::Local,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let best = manager.best_transport_for_peer(peer_id).expect("should have transport");
        assert_eq!(best, TransportType::Local);
    }

    #[test]
    fn test_best_transport_fails_for_unknown_peer() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(99);

        let result = manager.best_transport_for_peer(peer_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_peer_connected() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        assert!(!manager.is_peer_connected(peer_id));

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        assert!(manager.is_peer_connected(peer_id));
    }

    #[test]
    fn test_peers_on_transport() {
        let manager = TransportManager::new();
        let peer1 = create_peer_id(1);
        let peer2 = create_peer_id(2);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id: peer1,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id: peer2,
            transport: TransportType::BLE,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let established1 = TransportEvent::ConnectionEstablished {
            peer_id: peer1,
            transport: TransportType::BLE,
        };
        manager.handle_event(established1);

        let established2 = TransportEvent::ConnectionEstablished {
            peer_id: peer2,
            transport: TransportType::BLE,
        };
        manager.handle_event(established2);

        let peers = manager.peers_on_transport(TransportType::BLE);
        assert_eq!(peers.len(), 2);
    }

    #[test]
    fn test_send_to_peer_queues_data() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        let result = manager.send_to_peer(peer_id, vec![1, 2, 3], 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SendResult::Queued(TransportType::BLE));

        let pending = manager.pending_sends();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].priority, 5);
    }

    #[test]
    fn test_pending_sends_priority_ordering() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        manager.send_to_peer(peer_id, vec![1], 3).unwrap();
        manager.send_to_peer(peer_id, vec![2], 10).unwrap();
        manager.send_to_peer(peer_id, vec![3], 5).unwrap();

        let pending = manager.pending_sends();
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].priority, 10);
        assert_eq!(pending[1].priority, 5);
        assert_eq!(pending[2].priority, 3);
    }

    #[test]
    fn test_tick_cleanup() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        assert_eq!(manager.connected_peers().len(), 1);

        // Manually set last seen to far in the past
        {
            let mut last_seen = manager.peer_last_seen.write();
            last_seen.insert(
                peer_id,
                SystemTime::now() - std::time::Duration::from_secs(301),
            );
        }

        manager.tick();

        assert_eq!(manager.connected_peers().len(), 0);
    }

    #[test]
    fn test_transports_for_peer() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event1 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event1);

        let event2 = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::Internet,
            addr: vec![2],
        };
        manager.handle_event(event2);

        let transports = manager.transports_for_peer(peer_id);
        assert_eq!(transports.len(), 2);
        assert!(transports.contains(&TransportType::BLE));
        assert!(transports.contains(&TransportType::Internet));
    }

    #[test]
    fn test_connected_peers_deduplication() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        let peers = manager.connected_peers();
        assert_eq!(peers.len(), 1);
    }

    // ====================================================================
    // RECONNECTION TESTS
    // ====================================================================

    #[test]
    fn test_target_peer_queued_for_reconnect_on_disconnect() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        // Register as target peer
        manager.add_target_peer(peer_id, vec![1, 2, 3]);

        // Discover and then disconnect
        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1, 2, 3],
        };
        manager.handle_event(discovered);
        assert_eq!(manager.connected_peers().len(), 1);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        // Should be in reconnection queue
        assert_eq!(manager.reconnection_queue_len(), 1);
    }

    #[test]
    fn test_non_target_peer_not_queued_for_reconnect() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        // Discover WITHOUT registering as target
        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(discovered);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        // Should NOT be in reconnection queue
        assert_eq!(manager.reconnection_queue_len(), 0);
    }

    #[test]
    fn test_reconnection_backoff_increases() {
        let mut state = ReconnectionState::new(
            create_peer_id(1),
            HashSet::new(),
            vec![],
        );

        let first = state.backoff_interval();
        state.record_failure();
        let second = state.backoff_interval();
        state.record_failure();
        let third = state.backoff_interval();

        // Each interval should be larger (exponential backoff)
        assert!(second > first);
        assert!(third > second);
    }

    #[test]
    fn test_reconnection_backoff_capped_at_max() {
        let mut state = ReconnectionState::new(
            create_peer_id(1),
            HashSet::new(),
            vec![],
        );

        // Hit it many times to saturate
        for _ in 0..20 {
            state.record_failure();
        }

        assert!(state.backoff_interval() <= RECONNECT_MAX_INTERVAL);
    }

    #[test]
    fn test_reconnection_exhaustion() {
        let mut state = ReconnectionState::new(
            create_peer_id(1),
            HashSet::new(),
            vec![],
        );

        assert!(!state.is_exhausted());

        for _ in 0..RECONNECT_MAX_FAILURES {
            state.record_failure();
        }

        assert!(state.is_exhausted());
    }

    #[test]
    fn test_reconnect_success_removes_from_queue() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        manager.add_target_peer(peer_id, vec![1]);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(discovered);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        assert_eq!(manager.reconnection_queue_len(), 1);

        manager.record_reconnect_success(&peer_id);
        assert_eq!(manager.reconnection_queue_len(), 0);
    }

    #[test]
    fn test_remove_target_peer_stops_reconnection() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        manager.add_target_peer(peer_id, vec![1]);

        let discovered = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(discovered);

        let disconnected = TransportEvent::PeerDisconnected {
            peer_id,
            transport: TransportType::BLE,
        };
        manager.handle_event(disconnected);

        assert_eq!(manager.reconnection_queue_len(), 1);

        manager.remove_target_peer(&peer_id);
        assert_eq!(manager.reconnection_queue_len(), 0);
    }

    #[test]
    fn test_send_result_is_queued_not_delivered() {
        let manager = TransportManager::new();
        let peer_id = create_peer_id(1);

        let caps = TransportCapabilities::for_transport(TransportType::BLE);
        manager.register_transport(TransportType::BLE, caps);

        let event = TransportEvent::PeerDiscovered {
            peer_id,
            transport: TransportType::BLE,
            addr: vec![1],
        };
        manager.handle_event(event);

        let result = manager.send_to_peer(peer_id, vec![1, 2, 3], 5).unwrap();

        // Explicitly verify it's Queued, not some "Delivered" status
        match result {
            SendResult::Queued(transport) => assert_eq!(transport, TransportType::BLE),
        }
    }
}
