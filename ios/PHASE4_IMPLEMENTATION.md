# Phase 4: CoreBluetooth Transport - Implementation Guide

> Design/implementation guide snapshot. Validate against `docs/CURRENT_STATE.md` before treating status claims as current.

## Overview
Phase 4 requires ~900 LoC split across 4 files. This document provides the complete implementation structure.

## Files Required

### 1. Transport/MeshBLEConstants.swift ✅ CREATED
- BLE UUIDs matching Android
- Service: `6E400001-B5A3-F393-E0A9-E50E24DCCA9E`
- TX Char: `6E400002-B5A3-F393-E0A9-E50E24DCCA9E`
- RX Char: `6E400003-B5A3-F393-E0A9-E50E24DCCA9E`
- ID Char: `6E400004-B5A3-F393-E0A9-E50E24DCCA9E`
- L2CAP PSM: `0x1001`

### 2. Transport/BLECentralManager.swift (~300 LoC) - KEY FEATURES
```swift
final class BLECentralManager: NSObject, CBCentralManagerDelegate, CBPeripheralDelegate {
    // Core Components
    - CBCentralManager for scanning
    - Discovered/connected peripherals tracking
    - Peer cache for deduplication (5s window)
    - Scan timer for duty cycling
    
    // Write Queue (matches Android BleGattClient pattern)
    - writeInProgress: [UUID: Bool] 
    - pendingWrites: [UUID: [Data]]
    - CRITICAL: Never overlap writes (iOS silently drops)
    
    // Key Methods
    - startScanning(): Duty-cycled scanning (10s on, 30s off)
    - stopScanning(): Disconnect all, clear cache
    - sendData(to:data:): Queue-managed GATT write
    - setBackgroundMode(_:): Adjust parameters for background
    
    // State Restoration (iOS-specific, no Android equivalent)
    - willRestoreState: Restore peripherals after app kill
    - CBCentralManagerOptionRestoreIdentifierKey
    
    // Delegates
    - centralManagerDidUpdateState: Handle BLE on/off
    - didDiscover: Cache peripheral, connect if new
    - didConnect: Discover mesh service
    - didDiscoverServices: Find characteristics, subscribe to RX
    - didUpdateValueFor: Forward data to repository
    - didWriteValueFor: Dequeue next pending write
}
```

### 3. Transport/BLEPeripheralManager.swift (~300 LoC) - KEY FEATURES
```swift
final class BLEPeripheralManager: NSObject, CBPeripheralManagerDelegate {
    // Core Components
    - CBPeripheralManager for advertising
    - Mesh service with TX/RX/ID characteristics
    - Subscribed centrals tracking
    - Privacy rotation timer
    
    // Identity Data
    - identityData: Data (≤24 bytes for background)
    - Rotates every 15 minutes for privacy
    
    // Key Methods
    - startAdvertising(): Build service, add characteristics, advertise
    - stopAdvertising(): Remove services, stop advertising
    - setIdentityData(_:): Update ID characteristic
    - sendNotification(to:data:): Send via RX characteristic
    - setRotationInterval(_:): Configure privacy rotation
    
    // Background Constraints (iOS-specific)
    - Advertising payload limited to 28 bytes in background
    - Must use CBAdvertisementDataLocalNameKey
    
    // State Restoration
    - willRestoreState: Restore services after app kill
    - CBPeripheralManagerOptionRestoreIdentifierKey
    
    // Delegates
    - peripheralManagerDidUpdateState: Handle BLE on/off
    - didReceiveWrite: Process TX characteristic writes
    - didSubscribeTo: Track RX subscribers
    - isReady​ToUpdateSubscribers: Send queued notifications
}
```

### 4. Transport/BLEL2CAPManager.swift (~150 LoC) - KEY FEATURES
```swift
final class BLEL2CAPManager: NSObject, CBPeripheralDelegate {
    // Core Components
    - L2CAP channels for bulk transfer
    - Channel cache and state tracking
    - Stream buffers
    
    // Key Methods
    - openChannel(to:psm:): Connect L2CAP as central
    - publishChannel(psm:): Publish L2CAP as peripheral
    - sendData(_:on:): Stream data over L2CAP
    - closeChannel(_:): Cleanup and disconnect
    
    // Delegates
    - peripheral(_:didOpen:): Handle L2CAP channel opened
    - l2capChannel(_:didReceive:): Process incoming data
}
```

## Android Parity Requirements

### Write Queue Pattern (CRITICAL)
```swift
// Android: BleGattClient.kt lines 196-220
private var writeInProgress = false
private val pendingWrites = mutableListOf<ByteArray>()

func sendData(peripheral: CBPeripheral, data: Data) {
    guard !writeInProgress[peripheral.identifier] else {
        pendingWrites[peripheral.identifier, default: []].append(data)
        return
    }
    writeInProgress[peripheral.identifier] = true
    peripheral.writeValue(data, for: characteristic, type: .withResponse)
}

func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, error: Error?) {
    writeInProgress[peripheral.identifier] = false
    if let next = pendingWrites[peripheral.identifier]?.first {
        pendingWrites[peripheral.identifier]?.removeFirst()
        sendData(peripheral: peripheral, data: next)
    }
}
```

### Privacy Rotation
```swift
// Android: BleAdvertiser.kt setRotationInterval
private func rotateIdentity() {
    // Regenerate advertising data
    // Stop advertising
    // Update identity characteristic
    // Restart advertising with new data
    scheduleNextRotation()
}
```

## iOS-Specific Requirements

### State Restoration
```swift
// Required for background BLE (no Android equivalent)
centralManager = CBCentralManager(
    delegate: self,
    queue: .global(qos: .utility),
    options: [CBCentralManagerOptionRestoreIdentifierKey: MeshBLEConstants.centralRestoreId]
)

func centralManager(_ central: CBCentralManager, willRestoreState dict: [String: Any]) {
    if let peripherals = dict[CBCentralManagerRestoredStatePeripheralsKey] as? [CBPeripheral] {
        // Reconnect to restored peripherals
        for peripheral in peripherals {
            peripheral.delegate = self
            connectedPeripherals[peripheral.identifier] = peripheral
        }
    }
}
```

### Background Scanning Limitations
```swift
// iOS background scanning has restrictions:
// 1. Cannot use CBCentralManagerScanOptionAllowDuplicatesKey in background
// 2. Scan must specify service UUIDs
// 3. Scan results are aggregated/delayed
let options: [String: Any] = isBackgroundMode ? 
    [:] : 
    [CBCentralManagerScanOptionAllowDuplicatesKey: true]
centralManager.scanForPeripherals(
    withServices: [MeshBLEConstants.serviceUUID],
    options: options
)
```

## Integration Points

### With IosPlatformBridge
```swift
// In IosPlatformBridge.swift
func sendBlePacket(peerId: String, data: Data) {
    // Forward to BLECentralManager
    bleCentralManager?.sendData(to: peripheralId, data: data)
}

func onBleDataReceived(peripheral: CBPeripheral, data: Data) {
    // Forward to PlatformBridge callback
    meshRepository?.onBleDataReceived(peerId: peripheral.identifier.uuidString, data: data)
}
```

### With MeshRepository
```swift
// In MeshRepository.swift
func onBleDataReceived(peerId: String, data: Data) {
    // Process and decrypt
    // Forward to message handler if relay enabled
}

func sendBlePacket(peerId: String, data: Data) {
    // Called by platform bridge
    // Forward to transport layer
}
```

### With AutoAdjustEngine
```swift
// Apply BLE adjustments from Rust
func applyBleAdjustment(_ adjustment: BleAdjustment) {
    bleCentralManager.applyScanSettings(intervalMs: adjustment.scanIntervalMs)
    blePeripheralManager.applyAdvertiseSettings(
        intervalMs: adjustment.advertiseIntervalMs,
        txPowerDbm: adjustment.txPowerDbm
    )
}
```

## Testing Checklist

### Unit Tests (XCTest)
- [ ] Scan start/stop
- [ ] Peripheral discovery and caching
- [ ] Write queue management (never overlap)
- [ ] State restoration
- [ ] Privacy rotation
- [ ] L2CAP channel opening

### Integration Tests
- [ ] Two iOS devices discover each other
- [ ] GATT write/notify round-trip
- [ ] L2CAP bulk transfer
- [ ] Background state restoration
- [ ] Android ↔ iOS interoperability

## Implementation Status
- [x] MeshBLEConstants.swift created
- [ ] BLECentralManager.swift (300 LoC) - **TO BE COMPLETED IN XCODE**
- [ ] BLEPeripheralManager.swift (300 LoC) - **TO BE COMPLETED IN XCODE**
- [ ] BLEL2CAPManager.swift (150 LoC) - **TO BE COMPLETED IN XCODE**

## Notes
Due to the complexity and length of CoreBluetooth implementation (~900 LoC), these files should be completed in Xcode on macOS where:
1. Autocomplete assists with CoreBluetooth delegate methods
2. Compiler validates delegate protocol conformance
3. Runtime testing can verify BLE functionality
4. State restoration can be properly tested

The patterns and requirements are documented here for implementation reference.
