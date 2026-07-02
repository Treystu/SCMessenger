//! Self-Relay Network Protocol (Phase 6)
//!
//! Every node with internet connectivity is a relay server.
//! No third-party relays — sovereignty through distributed relaying.

#[cfg(not(target_arch = "wasm32"))]
pub mod bootstrap;
#[cfg(not(target_arch = "wasm32"))]
pub mod client;
#[cfg(not(target_arch = "wasm32"))]
pub mod delegate_prewarm;
#[cfg(not(target_arch = "wasm32"))]
pub mod findmy;
pub mod invite;
#[cfg(not(target_arch = "wasm32"))]
pub mod peer_exchange;
pub mod protocol;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;

#[cfg(not(target_arch = "wasm32"))]
pub use bootstrap::{BootstrapManager, BootstrapMethod, InvitePayload, SeedPeer};
#[cfg(not(target_arch = "wasm32"))]
pub use client::RelayClient;
#[cfg(not(target_arch = "wasm32"))]
pub use delegate_prewarm::{
    DelegateInfo, DelegatePrewarmConfig, DelegatePrewarmManager, DelegatePrewarmStats,
    WarmConnection,
};
#[cfg(not(target_arch = "wasm32"))]
pub use findmy::{FindMyBeaconManager, FindMyConfig, WakeUpPayload};
pub use invite::{InviteChain, InviteSystem, InviteToken};
#[cfg(not(target_arch = "wasm32"))]
pub use peer_exchange::{PeerExchangeManager, RelayPeerInfo};
pub use protocol::{RelayCapability, RelayMessage};
#[cfg(not(target_arch = "wasm32"))]
pub use server::{RelayServer, RelayServerConfig, RelayServerStats};
