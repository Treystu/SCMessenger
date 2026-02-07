// Transport module — libp2p swarm and networking

pub mod behaviour;
pub mod discovery;
pub mod swarm;

pub use behaviour::{MessageRequest, MessageResponse};
pub use discovery::{
    create_encrypted_beacon, decrypt_beacon, BeaconError, BeaconPayload, DiscoveryConfig,
    DiscoveryMode,
};
pub use swarm::{start_swarm, SwarmCommand, SwarmEvent2 as SwarmEvent, SwarmHandle};
