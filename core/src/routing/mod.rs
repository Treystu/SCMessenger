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
pub mod timeout_budget;
pub mod negative_cache;
pub mod resume_prefetch;
pub mod adaptive_ttl;
pub mod optimized_engine;

pub use local::{LocalCell, PeerInfo, PeerStatus, TransportType, PeerId, CellSummary, PeerEvent};
pub use neighborhood::{NeighborhoodTable, GatewayInfo, NeighborhoodSummary, NeighborhoodGossip};
pub use global::{GlobalRoutes, RouteAdvertisement, RouteRequest};
pub use engine::{
    RoutingEngine, RoutingDecision, NextHop, RoutingLayer, RoutingMaintenance, RoutingSummary,
};
pub use timeout_budget::{TimeoutBudget, DiscoveryPhase, BudgetSummary};
pub use negative_cache::{NegativeCache, NegativeCacheStats};
pub use resume_prefetch::{
    ResumePrefetchManager, PrefetchConfig, PrefetchedRoute, PrefetchStatus, PrefetchStats,
    FrequentPeer,
};
pub use adaptive_ttl::{AdaptiveTTLManager, ActivityHistory};
pub use optimized_engine::{
    OptimizedRoutingEngine, OptimizedRoutingMaintenance,
};
