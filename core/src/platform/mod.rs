//! Platform integration layer for mobile (iOS/Android) and desktop
//!
//! This module provides:
//! - Background service abstraction for platform-specific lifecycle management
//! - Automatic device state profiling and resource adjustment
//! - Settings persistence and validation for mesh operations
//! - Bridge interfaces for UniFFI platform code to control the mesh service

pub mod auto_adjust;
pub mod service;

pub use auto_adjust::{AdjustmentProfile, AdjustmentResult, DeviceState, SmartAutoAdjust};
pub use service::{
    MeshService, MeshServiceConfig, MeshServiceState, PlatformCapabilities, PlatformError,
    PlatformType, ServiceStats,
};
pub use settings::{DiscoveryMode, MeshSettings, PrivacyMode, SettingsError};
