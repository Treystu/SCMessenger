//
//  BLEPeripheralManager.swift
//  SCMessenger
//
//  Advertises self and serves GATT characteristics
//  Mirrors: android/.../transport/ble/BleAdvertiser.kt + BleGattServer.kt
//

import CoreBluetooth
import os

/// Advertises self and serves GATT characteristics (iOS Peripheral role)
///
/// Responsibilities:
/// - BLE advertising with mesh service
/// - GATT server for Message/Sync/Identity characteristics (matches Android GATT)
/// - Handle central subscriptions
/// - Privacy rotation (15 min intervals)
final class BLEPeripheralManager: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "BLE-Peripheral")
    private var peripheralManager: CBPeripheralManager!
    private weak var meshRepository: MeshRepository?
    
    // GATT Service and Characteristics (names match Android BleGattServer)
    private var meshService: CBMutableService?
    private var messageCharacteristic: CBMutableCharacteristic?  // Write: central → peripheral
    private var syncCharacteristic: CBMutableCharacteristic?     // Notify: peripheral → central
    private var identityCharacteristic: CBMutableCharacteristic? // Read: identity beacon
    
    // Subscribed centrals
    private var subscribedCentrals: [CBCentral] = []
    
    // Privacy rotation
    private var rotationInterval: TimeInterval = MeshBLEConstants.privacyRotationInterval
    private var rotationTimer: Timer?
    private var identityData: Data?
    
    // Advertising state
    private var isAdvertising = false
    private var isRotationEnabled = true
    
    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        peripheralManager = CBPeripheralManager(
            delegate: self,
            queue: .global(qos: .utility),
            options: [CBPeripheralManagerOptionRestoreIdentifierKey: MeshBLEConstants.peripheralRestoreId]
        )
    }
    
    // MARK: - Public API
    
    func startAdvertising() {
        logger.info("Starting BLE advertising")
        guard peripheralManager.state == .poweredOn else {
            logger.warning("Cannot start advertising: BLE not powered on")
            return
        }
        
        setupService()
        startPrivacyRotation()
    }
    
    func stopAdvertising() {
        logger.info("Stopping BLE advertising")
        peripheralManager.stopAdvertising()
        peripheralManager.removeAllServices()
        rotationTimer?.invalidate()
        rotationTimer = nil
        isAdvertising = false
    }
    
    func setIdentityData(_ data: Data) {
        guard data.count <= MeshBLEConstants.maxIdentityDataSize else {
            logger.error("Identity data too large: \(data.count) bytes (max \(MeshBLEConstants.maxIdentityDataSize))")
            return
        }
        identityData = data
        identityCharacteristic?.value = data
        logger.debug("Identity data set: \(data.count) bytes")
    }
    
    func setRotationInterval(_ interval: TimeInterval) {
        rotationInterval = interval
        logger.debug("Rotation interval set: \(interval)s")
        if isAdvertising && isRotationEnabled {
            rotationTimer?.invalidate()
            startPrivacyRotation()
        }
    }
    
    func setRotationEnabled(_ enabled: Bool) {
        isRotationEnabled = enabled
        logger.debug("Rotation enabled: \(enabled)")
        if isAdvertising {
            if enabled {
                startPrivacyRotation()
            } else {
                rotationTimer?.invalidate()
                rotationTimer = nil
            }
        }
    }
    
    func applyAdvertiseSettings(intervalMs: UInt32, txPowerDbm: Int8) {
        logger.debug("Advertise settings: interval=\(intervalMs)ms, txPower=\(txPowerDbm)dBm")
        // iOS doesn't allow direct control of advertising interval/power
        // Settings are advisory only
    }
    
    func sendNotification(to central: CBCentral, data: Data) {
        guard let syncCharacteristic = syncCharacteristic else {
            logger.error("Sync characteristic not available")
            return
        }

        let success = peripheralManager.updateValue(data, for: syncCharacteristic, onSubscribedCentrals: [central])
        if success {
            logger.debug("Sent notification to \(central.identifier): \(data.count) bytes")
        } else {
            logger.warning("Failed to send notification, queue full")
        }
    }

    /// Send data to a subscribed central identified by its UUID string.
    /// Used when we are acting as Peripheral and need to push data to a Central peer.
    func sendDataToConnectedCentral(peerId: String, data: Data) {
        guard let uuid = UUID(uuidString: peerId) else {
            logger.warning("sendDataToConnectedCentral: invalid UUID string \(peerId)")
            return
        }
        guard let central = subscribedCentrals.first(where: { $0.identifier == uuid }) else {
            logger.warning("sendDataToConnectedCentral: no subscribed central for \(peerId)")
            return
        }
        sendNotification(to: central, data: data)
    }
    
    // MARK: - Private Methods
    
    private func setupService() {
        // Create characteristics (names match Android BleGattServer)
        messageCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.messageCharUUID,
            properties: [.write, .writeWithoutResponse],
            value: nil,
            permissions: .writeable
        )

        syncCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.syncCharUUID,
            properties: [.notify],
            value: nil,
            permissions: .readable
        )

        identityCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.identityCharUUID,
            properties: .read,
            value: identityData,
            permissions: .readable
        )

        // Create service
        meshService = CBMutableService(type: MeshBLEConstants.serviceUUID, primary: true)
        meshService?.characteristics = [messageCharacteristic!, syncCharacteristic!, identityCharacteristic!]

        // Add service and start advertising
        peripheralManager.add(meshService!)
    }
    
    private func beginAdvertising() {
        let advertisementData: [String: Any] = [
            CBAdvertisementDataServiceUUIDsKey: [MeshBLEConstants.serviceUUID],
            CBAdvertisementDataLocalNameKey: MeshBLEConstants.advertisedName
        ]
        
        peripheralManager.startAdvertising(advertisementData)
        isAdvertising = true
        logger.info("BLE advertising started")
    }
    
    private func startPrivacyRotation() {
        guard isRotationEnabled else { return }
        rotationTimer?.invalidate()
        rotationTimer = Timer.scheduledTimer(withTimeInterval: rotationInterval, repeats: true) { [weak self] _ in
            self?.rotateIdentity()
        }
    }
    
    private func rotateIdentity() {
        logger.info("Rotating identity for privacy")
        // Stop advertising
        peripheralManager.stopAdvertising()
        
        // Update identity data (would be regenerated by repository)
        // For now, just restart advertising
        
        // Restart advertising
        beginAdvertising()
    }
}

// MARK: - CBPeripheralManagerDelegate

extension BLEPeripheralManager: CBPeripheralManagerDelegate {
    func peripheralManagerDidUpdateState(_ peripheral: CBPeripheralManager) {
        logger.info("Peripheral manager state: \(peripheral.state.rawValue)")
        
        switch peripheral.state {
        case .poweredOn:
            if !isAdvertising {
                setupService()
            }
        case .poweredOff, .unauthorized, .unsupported:
            stopAdvertising()
        default:
            break
        }
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didAdd service: CBService, error: Error?) {
        if let error = error {
            logger.error("Failed to add service: \(error.localizedDescription)")
            return
        }
        logger.info("Service added successfully")
        beginAdvertising()
    }
    
    func peripheralManagerDidStartAdvertising(_ peripheral: CBPeripheralManager, error: Error?) {
        if let error = error {
            logger.error("Failed to start advertising: \(error.localizedDescription)")
            return
        }
        logger.info("Advertising started successfully")
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveWrite requests: [CBATTRequest]) {
        for request in requests {
            if request.characteristic.uuid == MeshBLEConstants.messageCharUUID,
               let data = request.value {
                logger.debug("Received write: \(data.count) bytes from \(request.central.identifier)")
                meshRepository?.onBleDataReceived(peerId: request.central.identifier.uuidString, data: data)
            }
            peripheral.respond(to: request, withResult: .success)
        }
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, central: CBCentral, didSubscribeTo characteristic: CBCharacteristic) {
        logger.info("Central \(central.identifier) subscribed to \(characteristic.uuid.shortUUID)")
        if !subscribedCentrals.contains(where: { $0.identifier == central.identifier }) {
            subscribedCentrals.append(central)
        }
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, central: CBCentral, didUnsubscribeFrom characteristic: CBCharacteristic) {
        logger.info("Central \(central.identifier) unsubscribed from \(characteristic.uuid.shortUUID)")
        subscribedCentrals.removeAll(where: { $0.identifier == central.identifier })
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, willRestoreState dict: [String: Any]) {
        // State restoration for background BLE
        if let services = dict[CBPeripheralManagerRestoredStateServicesKey] as? [CBMutableService] {
            logger.info("Restoring \(services.count) services")
            meshService = services.first
        }
    }
}
