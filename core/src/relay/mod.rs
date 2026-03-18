//! Self-Relay Network Protocol (Phase 6)
//!
//! Every node with internet connectivity is a relay server.
//! No third-party relays — sovereignty through distributed relaying.

pub mod bootstrap;
pub mod client;
pub mod delegate_prewarm;
pub mod findmy;
pub mod invite;
pub mod peer_exchange;
pub mod protocol;
pub mod server;

pub use bootstrap::{BootstrapManager, BootstrapMethod, InvitePayload, SeedPeer};
pub use client::RelayClient;
pub use delegate_prewarm::{
    DelegateInfo, DelegatePrewarmConfig, DelegatePrewarmManager, DelegatePrewarmStats,
    WarmConnection,
};
pub use findmy::{FindMyBeaconManager, FindMyConfig, WakeUpPayload};
pub use invite::{InviteChain, InviteSystem, InviteToken};
pub use peer_exchange::{PeerExchangeManager, RelayPeerInfo};
pub use protocol::{RelayCapability, RelayMessage};
pub use server::{RelayServer, RelayServerConfig, RelayServerStats};
