// Address Observation and Consensus
//
// Tracks observations from multiple peers to determine our actual external addresses.
// Implements consensus-based address discovery without relying on external STUN servers.

use libp2p::{Multiaddr, PeerId};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

/// Observation of our address from a peer
#[derive(Debug, Clone)]
pub struct AddressObservation {
    /// The peer that observed this address
    pub observer: PeerId,
    /// The observed address
    pub address: SocketAddr,
    /// When this observation was made (unix timestamp)
    pub timestamp: u64,
    /// How many times this peer has confirmed this address
    pub confirmation_count: u32,
}

/// Tracks and aggregates address observations from multiple peers
#[derive(Debug, Clone)]
pub struct AddressObserver {
    /// Observations indexed by observer peer ID
    observations: HashMap<PeerId, AddressObservation>,
    /// Cached consensus result (recalculated when observations change)
    cached_external_addresses: Vec<SocketAddr>,
}

impl AddressObserver {
    /// Create a new address observer
    pub fn new() -> Self {
        Self {
            observations: HashMap::new(),
            cached_external_addresses: Vec::new(),
        }
    }

    /// Record an observation from a peer
    pub fn record_observation(&mut self, observer: PeerId, address: SocketAddr) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.observations
            .entry(observer)
            .and_modify(|obs| {
                if obs.address == address {
                    // Same address confirmed
                    obs.confirmation_count += 1;
                    obs.timestamp = now;
                } else {
                    // Address changed
                    obs.address = address;
                    obs.confirmation_count = 1;
                    obs.timestamp = now;
                }
            })
            .or_insert(AddressObservation {
                observer,
                address,
                timestamp: now,
                confirmation_count: 1,
            });

        // Recalculate consensus
        self.recalculate_consensus();
    }

    /// Get the most likely external addresses based on consensus
    pub fn external_addresses(&self) -> &[SocketAddr] {
        &self.cached_external_addresses
    }

    /// Get the primary external address (most commonly observed)
    pub fn primary_external_address(&self) -> Option<SocketAddr> {
        self.cached_external_addresses.first().copied()
    }

    /// Get all observations for debugging
    pub fn all_observations(&self) -> Vec<AddressObservation> {
        self.observations.values().cloned().collect()
    }

    /// Remove observations older than max_age_secs
    pub fn expire_old_observations(&mut self, max_age_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.observations
            .retain(|_, obs| now - obs.timestamp < max_age_secs);

        self.recalculate_consensus();
    }

    /// Recalculate consensus addresses from observations
    fn recalculate_consensus(&mut self) {
        // Count observations per address
        let mut address_counts: HashMap<SocketAddr, u32> = HashMap::new();

        for obs in self.observations.values() {
            *address_counts.entry(obs.address).or_insert(0) +=
                obs.confirmation_count;
        }

        // Sort by count (most observed first)
        let mut addresses: Vec<(SocketAddr, u32)> =
            address_counts.into_iter().collect();
        addresses.sort_by(|a, b| b.1.cmp(&a.1));

        // Cache the sorted addresses
        self.cached_external_addresses = addresses.into_iter().map(|(addr, _)| addr).collect();
    }
}

/// Connection endpoint information
#[derive(Debug, Clone)]
pub struct ConnectionEndpoint {
    /// Remote peer ID
    pub peer_id: PeerId,
    /// Remote address (what we see for them)
    pub remote_addr: Multiaddr,
    /// Local address (what we're using to connect)
    pub local_addr: Multiaddr,
    /// Connection ID
    pub connection_id: String,
    /// Timestamp when connection was established
    pub established_at: u64,
}

/// Tracks active connections and their endpoints
#[derive(Debug, Clone)]
pub struct ConnectionTracker {
    /// Active connections indexed by peer ID
    connections: HashMap<PeerId, ConnectionEndpoint>,
}

impl ConnectionTracker {
    /// Create a new connection tracker
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Record a new connection
    pub fn add_connection(
        &mut self,
        peer_id: PeerId,
        remote_addr: Multiaddr,
        local_addr: Multiaddr,
        connection_id: String,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.connections.insert(
            peer_id,
            ConnectionEndpoint {
                peer_id,
                remote_addr,
                local_addr,
                connection_id,
                established_at: now,
            },
        );
    }

    /// Remove a connection
    pub fn remove_connection(&mut self, peer_id: &PeerId) {
        self.connections.remove(peer_id);
    }

    /// Get connection info for a peer
    pub fn get_connection(&self, peer_id: &PeerId) -> Option<&ConnectionEndpoint> {
        self.connections.get(peer_id)
    }

    /// Get all active connections
    pub fn all_connections(&self) -> Vec<ConnectionEndpoint> {
        self.connections.values().cloned().collect()
    }

    /// Extract SocketAddr from a Multiaddr (best effort)
    pub fn extract_socket_addr(addr: &Multiaddr) -> Option<SocketAddr> {
        use libp2p::multiaddr::Protocol;

        let mut ip = None;
        let mut port = None;

        for protocol in addr.iter() {
            match protocol {
                Protocol::Ip4(addr) => ip = Some(std::net::IpAddr::V4(addr)),
                Protocol::Ip6(addr) => ip = Some(std::net::IpAddr::V6(addr)),
                Protocol::Tcp(p) => port = Some(p),
                Protocol::Udp(p) => port = Some(p),
                _ => {}
            }
        }

        match (ip, port) {
            (Some(ip), Some(port)) => Some(SocketAddr::new(ip, port)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_observer_consensus() {
        let mut observer = AddressObserver::new();

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let peer3 = PeerId::random();

        let addr1: SocketAddr = "1.2.3.4:1234".parse().unwrap();
        let addr2: SocketAddr = "5.6.7.8:5678".parse().unwrap();

        // Three peers observe addr1
        observer.record_observation(peer1, addr1);
        observer.record_observation(peer2, addr1);
        observer.record_observation(peer3, addr1);

        // One peer observes addr2
        observer.record_observation(PeerId::random(), addr2);

        // Consensus should be addr1 (3 votes vs 1)
        assert_eq!(observer.primary_external_address(), Some(addr1));
        assert_eq!(observer.external_addresses().len(), 2);
        assert_eq!(observer.external_addresses()[0], addr1);
    }

    #[test]
    fn test_address_confirmation_count() {
        let mut observer = AddressObserver::new();
        let peer = PeerId::random();
        let addr: SocketAddr = "1.2.3.4:1234".parse().unwrap();

        // Record same observation multiple times
        observer.record_observation(peer, addr);
        observer.record_observation(peer, addr);
        observer.record_observation(peer, addr);

        let obs = observer.all_observations();
        assert_eq!(obs.len(), 1);
        assert_eq!(obs[0].confirmation_count, 3);
    }

    #[test]
    fn test_extract_socket_addr() {
        let addr: Multiaddr = "/ip4/1.2.3.4/tcp/1234".parse().unwrap();
        let socket_addr = ConnectionTracker::extract_socket_addr(&addr);
        assert_eq!(
            socket_addr,
            Some("1.2.3.4:1234".parse().unwrap())
        );
    }
}
