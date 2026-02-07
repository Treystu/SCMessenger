//! Mycorrhizal Routing — bio-inspired mesh routing
//!
//! Three-layer routing modeled on fungal mycorrhizal networks:
//! - Layer 1 (Mycelium): Full local cell topology — knows every peer within direct range
//! - Layer 2 (Rhizomorphs): Neighborhood gossip summaries — knows gateways 2-3 hops away
//! - Layer 3 (CMN): Global route advertisements via internet-connected nodes (separate task)
//! - Engine: Decision engine combining all layers for optimal routing decisions
//!
//! The local cell maintains real-time awareness of all peers, their capabilities, and
//! message availability. Gateway peers act as connectors to neighboring cells, whose
//! summaries are aggregated and gossipped through the network.

pub mod local;
pub mod neighborhood;
pub mod global;
pub mod engine;

pub use local::{LocalCell, PeerInfo, PeerStatus, TransportType, PeerId, CellSummary, PeerEvent};
pub use neighborhood::{NeighborhoodTable, GatewayInfo, NeighborhoodSummary, NeighborhoodGossip};
pub use global::{GlobalRoutes, RouteAdvertisement, RouteRequest};
pub use engine::{
    RoutingEngine, RoutingDecision, NextHop, RoutingLayer, RoutingMaintenance, RoutingSummary,
};
