// Transport module — libp2p swarm and networking

pub mod behaviour;
pub mod swarm;
pub mod reflection;
pub mod nat;
pub mod internet;
pub mod observation;
pub mod multiport;
pub mod mesh_routing;

pub use swarm::{start_swarm, start_swarm_with_config, SwarmHandle, SwarmEvent2 as SwarmEvent, SwarmCommand};
pub use behaviour::{MessageRequest, MessageResponse};
pub use reflection::{AddressReflectionRequest, AddressReflectionResponse, AddressReflectionService};
pub use observation::{AddressObserver, ConnectionTracker, AddressObservation};
pub use multiport::{MultiPortConfig, BindResult, BindAnalysis, ConnectivityStatus};
pub use mesh_routing::{
    RelayStats, RelayReputation, ReputationTracker, RetryStrategy,
    DeliveryAttempt, MultiPathDelivery, BootstrapCapability,
};
