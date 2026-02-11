// Transport module — libp2p swarm and networking

pub mod behaviour;
pub mod internet;
pub mod mesh_routing;
pub mod multiport;
pub mod nat;
pub mod observation;
pub mod reflection;
pub mod swarm;

pub use behaviour::{MessageRequest, MessageResponse, RelayRequest, RelayResponse};
pub use mesh_routing::{
    BootstrapCapability, DeliveryAttempt, MultiPathDelivery, RelayReputation, RelayStats,
    ReputationTracker, RetryStrategy,
};
pub use multiport::{BindAnalysis, BindResult, ConnectivityStatus, MultiPortConfig};
pub use observation::{AddressObservation, AddressObserver, ConnectionTracker};
pub use reflection::{
    AddressReflectionRequest, AddressReflectionResponse, AddressReflectionService,
};
pub use swarm::{
    start_swarm, start_swarm_with_config, SwarmCommand, SwarmEvent2 as SwarmEvent, SwarmHandle,
};
