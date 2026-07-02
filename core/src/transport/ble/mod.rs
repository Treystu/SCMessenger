/// BLE (Bluetooth Low Energy) Transport Module
///
/// This module provides the protocol-level abstractions for BLE-based messaging
/// in the SCMessenger sovereign mesh network. It includes:
///
/// - **beacon**: Encrypted beacon construction, parsing, and rotation for DarkBLE discovery
/// - **l2cap**: L2CAP channel abstraction with fragmentation and reassembly
/// - **gatt**: GATT service definitions with characteristic-based messaging
/// - **scanner**: BLE scanner with adaptive duty cycle management
///
/// The module is designed to work with platform-specific implementations (Swift/Kotlin)
/// that handle the actual BLE hardware operations. The core logic here is testable
/// without actual BLE hardware.
pub mod beacon;
pub mod gatt;
pub mod l2cap;
pub mod scanner;

// Re-export commonly used types
pub use beacon::{
    BeaconBuilder, BeaconParser, BleBeacon, BleBeaconError, BLE_BEACON_SERVICE_UUID,
    DEFAULT_BEACON_ROTATION_SECS,
};

pub use gatt::{
    GattCharacteristic, GattClient, GattError, GattFragmentHeader, GattFragmenter, GattReassembler,
    GattServer, GattWriteQueue, GattWriteRequest, GATT_SERVICE_UUID, MAX_CHARACTERISTIC_SIZE,
};

pub use l2cap::{
    append_crc32, ChannelState, DropReason, FragmentHeader, L2capChannel, L2capConfig, L2capError,
    L2capFragmenter, L2capReassembler, L2capReassemblyManager, ProtocolServiceMultiplexer,
};

pub use scanner::{
    BatteryState, BleScanConfig, BleScanner, DutyCycleManager, ScanResult, ScannerError,
    ScannerState,
};
