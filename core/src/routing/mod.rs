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

pub mod adaptive_ttl;
pub mod engine;
pub mod global;
pub mod local;
pub mod negative_cache;
pub mod neighborhood;
pub mod optimized_engine;
pub mod resume_prefetch;
pub mod timeout_budget;

pub use adaptive_ttl::{ActivityHistory, AdaptiveTTLManager};
pub use engine::{
    NextHop, RoutingDecision, RoutingEngine, RoutingLayer, RoutingMaintenance, RoutingSummary,
};
pub use global::{GlobalRoutes, RouteAdvertisement, RouteRequest};
pub use local::{CellSummary, LocalCell, PeerEvent, PeerId, PeerInfo, PeerStatus, TransportType};
pub use negative_cache::{NegativeCache, NegativeCacheStats};
pub use neighborhood::{GatewayInfo, NeighborhoodGossip, NeighborhoodSummary, NeighborhoodTable};
pub use optimized_engine::{OptimizedRoutingEngine, OptimizedRoutingMaintenance};
pub use resume_prefetch::{
    FrequentPeer, PrefetchConfig, PrefetchStats, PrefetchStatus, PrefetchedRoute,
    ResumePrefetchManager,
};
pub use timeout_budget::{BudgetSummary, DiscoveryPhase, TimeoutBudget};
