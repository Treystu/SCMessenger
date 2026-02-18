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
    // ⚠️ CRITICAL: These UUIDs MUST match Android exactly for cross-platform BLE interop
    // Android source: android/.../transport/ble/BleGattServer.kt + BleScanner.kt
    
    /// SCMesh GATT Service UUID
    /// Matches: BleScanner.SERVICE_UUID / BleGattServer.SERVICE_UUID
    static let serviceUUID = CBUUID(string: "0000DF01-0000-1000-8000-00805F9B34FB")
    
    /// Identity Characteristic UUID (Read - peer identity beacon)
    /// Matches: BleGattServer.IDENTITY_CHAR_UUID
    /// Contains truncated identity for quick peer recognition
    static let identityCharUUID = CBUUID(string: "0000DF02-0000-1000-8000-00805F9B34FB")
    
    /// Message Characteristic UUID (Write - central writes messages to peripheral)
    /// Matches: BleGattServer.MESSAGE_CHAR_UUID
    static let messageCharUUID = CBUUID(string: "0000DF03-0000-1000-8000-00805F9B34FB")
    
    /// Sync Characteristic UUID (Notify - peripheral notifies central of incoming data)
    /// Matches: BleGattServer.SYNC_CHAR_UUID
    static let syncCharUUID = CBUUID(string: "0000DF04-0000-1000-8000-00805F9B34FB")

    /// Client Configuration Descriptor UUID (standard BLE descriptor for notify/indicate)
    /// Matches: BleGattServer.CLIENT_CONFIG_DESCRIPTOR_UUID
    static let clientConfigDescriptorUUID = CBUUID(string: "00002902-0000-1000-8000-00805F9B34FB")

    // Legacy aliases for backward compatibility during migration
    static let txCharUUID = messageCharUUID
    static let rxCharUUID = syncCharUUID
    static let idCharUUID = identityCharUUID
    
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
