// Transport module — libp2p swarm and networking

pub mod abstraction;
pub mod behaviour;
pub mod ble;
pub mod discovery;
pub mod escalation;
pub mod internet;
pub mod manager;
pub mod nat;
pub mod swarm;
pub mod wifi_aware;

pub use abstraction::{
    TransportCapabilities, TransportCommand, TransportError, TransportEvent, TransportType,
};
pub use behaviour::{MessageRequest, MessageResponse};
pub use discovery::{
    create_encrypted_beacon, decrypt_beacon, BeaconError, BeaconPayload, DiscoveryConfig,
    DiscoveryMode,
};
pub use escalation::{EscalationEngine, EscalationError, EscalationPolicy, EscalationState};
pub use manager::TransportManager;
pub use swarm::{start_swarm, SwarmCommand, SwarmEvent2 as SwarmEvent, SwarmHandle};
pub use wifi_aware::{
    WifiAwareConfig, WifiAwareError, WifiAwareState, DataPathInfo,
    DiscoveredPeer, WifiAwarePlatformBridge,
};
pub use internet::{
    InternetRelay, InternetTransportConfig, InternetTransportError, NatStatus, RelayMode,
};
pub use nat::{
    NatTraversal, NatConfig, NatType, NatProbe, NatTraversalError, HolePunchAttempt,
    HolePunchStatus,
};
