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
    
    // Reassembly buffers per central
    private var reassemblyBuffers: [UUID: [Int: Data]] = [:]
    private var expectedFragments: [UUID: Int] = [:]
    
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
        logger.debug("Identity data updated for dynamic read: \(data.count) bytes")
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
    
    func broadcastDataToCentrals(_ data: Data) {
        for central in subscribedCentrals {
            sendDataToCentral(central, data: data)
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
        sendDataToCentral(central, data: data)
    }

    private func sendDataToCentral(_ central: CBCentral, data: Data) {
        let mtu = central.maximumUpdateValueLength
        let fragments = fragmentData(data, mtu: mtu)
        
        for fragment in fragments {
            let success = peripheralManager.updateValue(fragment, for: messageCharacteristic!, onSubscribedCentrals: [central])
            if !success {
                logger.warning("Failed to send fragment, buffering")
                meshRepository?.appendDiagnostic("ble_tx_buffer fragment to=\(central.identifier.uuidString.prefix(8))")
                pendingNotifications.append((central: central, data: fragment))
            }
        }
        if fragments.count > 1 {
            meshRepository?.appendDiagnostic("ble_tx_start fragments=\(fragments.count) to=\(central.identifier.uuidString.prefix(8))")
        }
    }

    private func fragmentData(_ data: Data, mtu: Int) -> [Data] {
        let maxChunk = min(512, mtu)
        let maxPayload = maxChunk - 4
        if maxPayload <= 0 { return [data] }

        let totalFragments = Int(ceil(Double(data.count) / Double(maxPayload)))
        var fragments: [Data] = []

        for i in 0..<totalFragments {
            let start = i * maxPayload
            let end = min(start + maxPayload, data.count)
            let chunk = data.subdata(in: start..<end)

            var header = Data(count: 4)
            header[0] = UInt8(totalFragments & 0xFF)
            header[1] = UInt8((totalFragments >> 8) & 0xFF)
            header[2] = UInt8(i & 0xFF)
            header[3] = UInt8((i >> 8) & 0xFF)

            fragments.append(header + chunk)
        }
        return fragments
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
            properties: [.write, .writeWithoutResponse, .notify],
            value: nil,
            permissions: .writeable
        )

        syncCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.syncCharUUID,
            properties: [.read, .write],
            value: nil,
            permissions: [.readable, .writeable]
        )

        identityCharacteristic = CBMutableCharacteristic(
            type: MeshBLEConstants.identityCharUUID,
            properties: .read,
            value: nil,
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
        logger.info("Rotating identity...")
        stopAdvertising()
        isAdvertising = false // Ensure we clear state so beginAdvertising succeeds
        
        // Short delay to let hardware settle
        DispatchQueue.global(qos: .utility).asyncAfter(deadline: .now() + 0.5) { [weak self] in
            self?.beginAdvertising()
        }
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
            let nsError = error as NSError
            if nsError.domain == CBErrorDomain && nsError.code == 12 /* CBError.alreadyAdvertising */ {
                logger.warning("Advertising already active, syncing state")
                meshRepository?.appendDiagnostic("ble_peripheral_adv_already_active")
                isAdvertising = true
                return
            }
            logger.error("Failed to start advertising: \(error.localizedDescription)")
            meshRepository?.appendDiagnostic("ble_peripheral_adv_fail err=\(error.localizedDescription)")
            isAdvertising = false
            return
        }
        logger.info("Advertising started successfully")
        isAdvertising = true
        meshRepository?.appendDiagnostic("ble_peripheral_adv_start")
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveWrite requests: [CBATTRequest]) {
        for request in requests {
            guard let data = request.value, !data.isEmpty else {
                peripheral.respond(to: request, withResult: .success)
                continue
            }
            
            if request.characteristic.uuid == MeshBLEConstants.syncCharUUID {
                // Fragmented sync data
                handleFragmentedWrite(data: data, centralId: request.central.identifier, isSync: true)
                peripheral.respond(to: request, withResult: .success)
            } else if request.characteristic.uuid == MeshBLEConstants.messageCharUUID {
                // Fragmented message data
                handleFragmentedWrite(data: data, centralId: request.central.identifier, isSync: false)
                peripheral.respond(to: request, withResult: .success)
            } else {
                peripheral.respond(to: request, withResult: .requestNotSupported)
            }
        }
    }

    private func handleFragmentedWrite(data: Data, centralId: UUID, isSync: Bool) {
        if data.count < 4 {
            logger.warning("Received tiny BLE packet (<4 bytes) from \(centralId)")
            return
        }

        let totalFrags = Int(data[0]) | (Int(data[1]) << 8)
        let fragIndex = Int(data[2]) | (Int(data[3]) << 8)
        let payload = data.subdata(in: 4..<data.count)

        if fragIndex == 0 {
            // New message starting - clear any stale fragments from previous failed attempts
            reassemblyBuffers[centralId] = [0: payload]
            if totalFrags > 1 {
                meshRepository?.appendDiagnostic("ble_rx_start total=\(totalFrags) from=\(centralId.uuidString.prefix(8))")
            }
        } else {
            var buffer = reassemblyBuffers[centralId] ?? [:]
            buffer[fragIndex] = payload
            reassemblyBuffers[centralId] = buffer
        }

        let currentCount = reassemblyBuffers[centralId]?.count ?? 0
        if currentCount == totalFrags {
            var completeData = Data()
            let buffer = reassemblyBuffers[centralId] ?? [:]
            for i in 0..<totalFrags {
                if let chunk = buffer[i] {
                    completeData.append(chunk)
                } else {
                    logger.error("Missing fragment \(i) in complete buffer for \(centralId)")
                    return
                }
            }
            reassemblyBuffers.removeValue(forKey: centralId)

            logger.info("Reassembled complete \(isSync ? "sync" : "message") (\(completeData.count) bytes) from \(centralId)")
            meshRepository?.appendDiagnostic("ble_rx_complete size=\(completeData.count) type=\(isSync ? "sync" : "msg")")
            DispatchQueue.main.async { [weak self] in
                self?.meshRepository?.onBleDataReceived(peerId: centralId.uuidString, data: completeData)
            }
        }
    }
    
    func peripheralManager(_ peripheral: CBPeripheralManager, didReceiveRead request: CBATTRequest) {
        if request.characteristic.uuid == MeshBLEConstants.identityCharUUID {
            guard let data = identityData else {
                peripheral.respond(to: request, withResult: .unlikelyError)
                return
            }
            
            let offset = request.offset
            
            if offset > data.count {
                peripheral.respond(to: request, withResult: .invalidOffset)
                return
            }
            
            request.value = data.subdata(in: offset..<data.count)
            peripheral.respond(to: request, withResult: .success)
            logger.debug("Responded to read for identity beacon, offset: \(offset), returning \(request.value?.count ?? 0) bytes")
        } else {
            peripheral.respond(to: request, withResult: .requestNotSupported)
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
        guard let messageChar = messageCharacteristic else { return }
        
        while !self.pendingNotifications.isEmpty {
            let next = self.pendingNotifications[0]
            let success = self.peripheralManager.updateValue(next.data, for: messageChar, onSubscribedCentrals: [next.central])
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
