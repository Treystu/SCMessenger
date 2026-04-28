// Transport module — libp2p swarm and networking

pub mod abstraction;
pub mod behaviour;
pub mod bootstrap;
pub mod circuit_breaker;
pub mod health;
pub mod internet;
pub mod mesh_routing;
pub mod multiport;
pub mod nat;
pub mod observation;
pub mod peer_broadcast;
pub mod reflection;
pub mod relay_health;
pub mod reputation;
pub mod routing;
pub mod swarm;
#[cfg(not(target_arch = "wasm32"))]
pub mod websocket;

pub use behaviour::{
    DeregistrationPayload, DeregistrationRequest, IronCoreBehaviour, LedgerExchangeRequest,
    LedgerExchangeResponse, MessageRequest, MessageResponse, RegistrationMessage,
    RegistrationPayload, RegistrationRequest, RegistrationResponse, RelayRequest, RelayResponse,
    SharedPeerEntry,
};
pub use mesh_routing::{
    BootstrapCapability, DeliveryAttempt, MultiPathDelivery, RelayReputation, RelayStats,
    ReputationTracker, RetryStrategy, ROUTE_REASON_DIRECT_FIRST, ROUTE_REASON_RELAY_RECENCY_SUCCESS,
    ROUTE_REASON_RELAY_SUCCESS_SCORE, ROUTE_REASON_RELAY_TIEBREAK_LAST_SUCCESS,
    ROUTE_REASON_RELAY_TIEBREAK_PEER_ID,
};
pub use routing::{
    engine::{NextHop, RoutingDecision, RoutingEngine, RoutingLayer, RoutingMaintenance, RoutingSummary},
    local::{CellSummary, LocalCell, PeerEvent, PeerId, PeerInfo, PeerStatus, TransportType},
    neighborhood::{GatewayInfo, NeighborhoodGossip, NeighborhoodSummary, NeighborhoodTable},
    global::{GlobalRoutes, RouteAdvertisement, RouteRequest},
    adaptive_ttl::{ActivityHistory, AdaptiveTTLManager},
    negative_cache::{NegativeCache, NegativeCacheStats},
    optimized_engine::{OptimizedRoutingEngine, OptimizedRoutingMaintenance},
    resume_prefetch::{FrequentPeer, PrefetchConfig, PrefetchStats, PrefetchStatus, PrefetchedRoute},
    smart_retry::{calculate_next_attempt, BackoffStrategy, DeliveryTrigger},
    timeout_budget::{BudgetSummary, DiscoveryPhase, TimeoutBudget},
};
pub use multiport::{BindAnalysis, BindResult, ConnectivityStatus, MultiPortConfig};
pub use observation::{AddressObservation, AddressObserver, ConnectionTracker};
pub use peer_broadcast::PeerBroadcaster;
pub use reflection::{
    AddressReflectionRequest, AddressReflectionResponse, AddressReflectionService,
};
pub use reputation::{
    AbuseReputationManager, AbuseSignal, PeerAbuseStats, ReputationScore,
};
pub use health::{
    ConnectionStats, ConnectionState, GlobalTransportMetrics, TransportHealthMonitor,
};
pub use relay_health::{RelayDiscovery, RelayFallback, RelayMetrics};
pub use bootstrap::{BootstrapConfig, BootstrapManager, BootstrapState};
pub use circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager, CircuitBreakerStats, CircuitState};
pub use swarm::{
    start_swarm, start_swarm_with_config, SwarmCommand, SwarmEvent2 as SwarmEvent, SwarmHandle,
};
