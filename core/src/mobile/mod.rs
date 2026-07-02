//! Mobile Platform Integration (Phase 5A+5B+5C)
//!
//! Provides abstractions for background service management, platform bridging,
//! automatic power/network adjustment, and iOS-specific background modes.

pub mod auto_adjust;
pub mod ios_strategy;
pub mod service;

pub use auto_adjust::{AdjustmentProfile, AutoAdjustEngine, DeviceProfile};
pub use ios_strategy::{BackgroundMode, CoreBluetoothState, IosBackgroundStrategy};
pub use service::{MeshService, MeshServiceConfig, ServiceState};
pub use settings::MeshSettings;
