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
    
    // Notification Buffer (Performance Optimization)
    private var pendingNotifications: [(central: CBCentral, data: Data)] = []
    
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
        // This payload is served over GATT identity characteristic reads, not
        // advertisement service data. Full JSON identity is expected here.
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
            logger.warning("Failed to send notification, queue full — buffering")
            pendingNotifications.append((central: central, data: data))
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

    /// Broadcast data to all subscribed centrals.
    func broadcastDataToCentrals(_ data: Data) {
        for central in subscribedCentrals {
            sendNotification(to: central, data: data)
        }
    }
    
    // MARK: - Private Methods
    
    private func setupService() {
        // P3: Guard against re-adding service when already set up
        guard meshService == nil else {
            // Service already configured — just ensure advertising is running
            if !isAdvertising {
                beginAdvertising()
            }
            return
        }

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
        // P3: Guard against re-starting advertising when already active
        guard !isAdvertising else { return }

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
        // Timer must be added to main RunLoop — background queues have no active RunLoop
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            self.rotationTimer = Timer.scheduledTimer(withTimeInterval: self.rotationInterval, repeats: true) { [weak self] _ in
                self?.rotateIdentity()
            }
            RunLoop.main.add(self.rotationTimer!, forMode: .common)
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
            meshRepository?.appendDiagnostic("ble_peripheral_add_service_fail err=\(error.localizedDescription)")
            return
        }
        logger.info("Service added successfully")
        meshRepository?.appendDiagnostic("ble_peripheral_service_added")
        beginAdvertising()
    }
    
    func peripheralManagerDidStartAdvertising(_ peripheral: CBPeripheralManager, error: Error?) {
        if let error = error {
            logger.error("Failed to start advertising: \(error.localizedDescription)")
            meshRepository?.appendDiagnostic("ble_peripheral_adv_fail err=\(error.localizedDescription)")
            return
        }
        logger.info("Advertising started successfully")
        meshRepository?.appendDiagnostic("ble_peripheral_adv_start")
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveWrite requests: [CBATTRequest]) {
        for request in requests {
            if request.characteristic.uuid == MeshBLEConstants.messageCharUUID,
               let data = request.value {
                logger.debug("Received write: \(data.count) bytes from \(request.central.identifier)")
                DispatchQueue.main.async { [weak self] in
                    self?.meshRepository?.onBleDataReceived(peerId: request.central.identifier.uuidString, data: data)
                }
            }
            peripheral.respond(to: request, withResult: .success)
        }
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, central: CBCentral, didSubscribeTo characteristic: CBCharacteristic) {
        logger.info("Central \(central.identifier) subscribed to \(characteristic.uuid.shortUUID)")
        meshRepository?.appendDiagnostic("ble_peripheral_subscribed central=\(central.identifier)")
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
    
    func peripheralManagerIsReady(toUpdateSubscribers peripheral: CBPeripheralManager) {
        logger.debug("Peripheral manager ready to resume notifications")
        processPendingNotifications()
    }
    
    private func processPendingNotifications() {
        guard let syncChar = syncCharacteristic else { return }
        
        while !self.pendingNotifications.isEmpty {
            let next = self.pendingNotifications[0]
            let success = self.peripheralManager.updateValue(next.data, for: syncChar, onSubscribedCentrals: [next.central])
            if success {
                self.pendingNotifications.removeFirst()
                self.logger.debug("Processed buffered notification for \(next.central.identifier)")
            } else {
                // Queue still full, wait for next 'ready' callback
                self.logger.debug("Queue still full, remaining buffered: \(self.pendingNotifications.count)")
                break
            }
        }
    }
}
