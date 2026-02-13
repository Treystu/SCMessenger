//
//  MeshBLEConstants.swift
//  SCMessenger
//
//  BLE service UUIDs and characteristics for mesh networking
//  MUST match Android BLE constants exactly for interoperability
//

import CoreBluetooth

/// BLE constants for mesh networking
/// Shared between iOS and Android - MUST be identical
struct MeshBLEConstants {
    // MARK: - Service and Characteristics
    
    /// SCMesh GATT Service UUID
    /// Must match: android/.../transport/ble/BleScanner.kt SERVICE_UUID
    static let serviceUUID = CBUUID(string: "6E400001-B5A3-F393-E0A9-E50E24DCCA9E")
    
    /// TX Characteristic UUID (Write - phone → peer)
    /// Central writes to peripheral's TX characteristic
    static let txCharUUID = CBUUID(string: "6E400002-B5A3-F393-E0A9-E50E24DCCA9E")
    
    /// RX Characteristic UUID (Notify - peer → phone)
    /// Central subscribes to peripheral's RX characteristic for notifications
    static let rxCharUUID = CBUUID(string: "6E400003-B5A3-F393-E0A9-E50E24DCCA9E")
    
    /// Identity Characteristic UUID (Read - peer beacon)
    /// Contains truncated identity for quick peer recognition
    static let idCharUUID = CBUUID(string: "6E400004-B5A3-F393-E0A9-E50E24DCCA9E")
    
    // MARK: - L2CAP
    
    /// L2CAP PSM (Protocol/Service Multiplexer) for bulk data transfer
    /// Must match: android/.../transport/ble/BleL2capManager.kt L2CAP_PSM
    static let l2capPSM: CBL2CAPPSM = 0x1001
    
    // MARK: - Constraints
    
    /// Maximum identity data size for advertising (iOS background limit)
    /// iOS allows only 28 bytes total in background advertising
    /// After overhead, identity data must be ≤24 bytes
    static let maxIdentityDataSize = 24
    
    /// Maximum MTU for GATT writes
    /// iOS negotiates MTU automatically (up to 512 bytes on modern devices)
    /// Use conservative value for compatibility
    static let maxMTU = 512
    
    /// Maximum chunk size for fragmented writes
    /// Keep below MTU with safety margin
    static let maxChunkSize = 400
    
    // MARK: - Timing
    
    /// Default scan interval (seconds)
    static let defaultScanInterval: TimeInterval = 10.0
    
    /// Default scan window (seconds)
    static let defaultScanWindow: TimeInterval = 30.0
    
    /// Peer cache timeout (seconds) - for deduplication
    static let peerCacheTimeout: TimeInterval = 5.0
    
    /// Privacy rotation interval (seconds) - 15 minutes
    static let privacyRotationInterval: TimeInterval = 900.0
    
    /// Connection timeout (seconds)
    static let connectionTimeout: TimeInterval = 10.0
    
    /// Write timeout (seconds)
    static let writeTimeout: TimeInterval = 5.0
}

// MARK: - Service Names

extension MeshBLEConstants {
    /// Advertised local name (visible during scanning)
    static let advertisedName = "SCMesh"
    
    /// State restoration identifiers
    static let centralRestoreId = "com.scmessenger.central"
    static let peripheralRestoreId = "com.scmessenger.peripheral"
}

// MARK: - Helper Extensions

extension CBUUID {
    /// Short UUID string for logging
    var shortUUID: String {
        String(uuidString.prefix(8))
    }
}
