//
//  BLECentralManager.swift
//  SCMessenger
//
//  Scans for and connects to BLE mesh peers
//  Mirrors: android/.../transport/ble/BleScanner.kt + BleGattClient.kt
//

import CoreBluetooth
import os

/// Scans for and connects to BLE mesh peers (iOS Central role)
///
/// Responsibilities:
/// - Duty-cycled BLE scanning for mesh service
/// - Connect to discovered peripherals
/// - GATT client operations (read/write characteristics)
/// - Write queue management (mirrors Android pattern)
/// - State restoration for background operation
final class BLECentralManager: NSObject {
    private let logger = Logger(subsystem: "com.scmessenger", category: "BLE-Central")
    private var centralManager: CBCentralManager!
    private weak var meshRepository: MeshRepository?

    // Peripheral tracking
    private var discoveredPeripherals: [UUID: CBPeripheral] = [:]
    private var connectedPeripherals: [UUID: CBPeripheral] = [:]
    private var peerCache: [UUID: Date] = [:] // Dedup cache

    // Scanning parameters
    private var scanInterval: TimeInterval = MeshBLEConstants.defaultScanInterval
    private var scanWindow: TimeInterval = MeshBLEConstants.defaultScanWindow
    private var isBackgroundMode = false
    private var scanTimer: Timer?
    private var isScanning = false
    private var pendingScanOnReady = false  // P3: Defer scan until BLE is poweredOn

    // Write queue (mirrors Android BleGattClient pattern - CRITICAL)
    private var writeInProgress: [UUID: Bool] = [:]
    private var pendingWrites: [UUID: [Data]] = [:]

    // Reassembly buffers per peripheral
    private var reassemblyBuffers: [UUID: [Int: Data]] = [:]

    // Characteristics cache (names match Android BleGattServer)
    private var messageCharacteristics: [UUID: CBCharacteristic] = [:] // Write: central → peripheral
    private var syncCharacteristics: [UUID: CBCharacteristic] = [:]    // Notify: peripheral → central

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
        super.init()
        centralManager = CBCentralManager(
            delegate: self,
            queue: .global(qos: .utility),
            options: [CBCentralManagerOptionRestoreIdentifierKey: MeshBLEConstants.centralRestoreId]
        )
    }

    // MARK: - Public API

    func startScanning() {
        logger.info("Starting BLE scanning")
        guard centralManager.state == .poweredOn else {
            logger.warning("Cannot start scanning: BLE not powered on (state=\(self.centralManager.state.rawValue)), will auto-start when ready")
            // P3: Don't log as failure — just defer until BLE is ready
            pendingScanOnReady = true
            if self.centralManager.state == .unknown {
                // State .unknown means CBCentralManager hasn't reported yet — this is normal at launch.
                // Scanning will begin automatically when centralManagerDidUpdateState fires with .poweredOn.
                return
            }
            meshRepository?.appendDiagnostic("ble_central_start_deferred state=\(self.centralManager.state.rawValue)")
            return
        }
        pendingScanOnReady = false
        meshRepository?.appendDiagnostic("ble_central_scan_start")
        scheduleDutyCycle()
    }

    func stopScanning() {
        logger.info("Stopping BLE scanning")
        scanTimer?.invalidate()
        scanTimer = nil
        centralManager.stopScan()
        isScanning = false
        disconnectAll()
    }

    func setBackgroundMode(_ background: Bool) {
        isBackgroundMode = background
        logger.info("Background mode: \(background)")
    }

    func applyScanSettings(intervalMs: UInt32) {
        scanInterval = TimeInterval(intervalMs) / 1000.0
        logger.debug("Scan interval updated: \(self.scanInterval)s")
    }

    @discardableResult
    func sendData(to peripheralId: UUID, data: Data) -> Bool {
        guard let peripheral = connectedPeripherals[peripheralId],
              let characteristic = messageCharacteristics[peripheralId] else {
            logger.error("Cannot send: peripheral \(peripheralId) not connected or Message char not found")
            return false
        }

        let mtu = peripheral.maximumWriteValueLength(for: .withResponse)
        let fragments = fragmentData(data, mtu: mtu)
        
        meshRepository?.appendDiagnostic("ble_central_tx_start fragments=\(fragments.count) to=\(peripheralId.uuidString.prefix(8))")
        for fragment in fragments {
            enqueueFragment(fragment, for: peripheralId)
        }
        return true
    }

    private func enqueueFragment(_ fragment: Data, for peripheralId: UUID) {
        guard let peripheral = connectedPeripherals[peripheralId],
              let characteristic = messageCharacteristics[peripheralId] else { return }

        if writeInProgress[peripheralId] == true {
            pendingWrites[peripheralId, default: []].append(fragment)
        } else {
            writeInProgress[peripheralId] = true
            peripheral.writeValue(fragment, for: characteristic, type: .withResponse)
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

    /// Broadcast data to all connected peripherals.
    func broadcastData(_ data: Data) {
        for peripheralId in connectedPeripherals.keys {
            sendData(to: peripheralId, data: data)
        }
    }

    // MARK: - Private Methods

    private func scheduleDutyCycle() {
        // Timer MUST run on the main RunLoop — background dispatch queues don't
        // have a running RunLoop, so Timer.scheduledTimer would silently never fire.
        DispatchQueue.main.async { [weak self] in
            guard let self = self else { return }
            self.scanTimer?.invalidate()
            self.scanTimer = Timer.scheduledTimer(withTimeInterval: self.scanInterval, repeats: true) { [weak self] _ in
                self?.performScanCycle()
            }
            RunLoop.main.add(self.scanTimer!, forMode: .common)
            self.performScanCycle() // Start immediately
        }
    }

    private func performScanCycle() {
        if isBackgroundMode {
            // Background: duty-cycle to preserve battery
            if !isScanning {
                startScan()
                DispatchQueue.global(qos: .utility).asyncAfter(deadline: .now() + scanWindow) { [weak self] in
                    self?.stopScan()
                }
            }
        } else {
            // Foreground: scan continuously — never stop between cycles so we
            // don't miss advertisement windows during active use/testing.
            if !isScanning {
                startScan()
            }
        }
    }

    private func startScan() {
        let options: [String: Any] = isBackgroundMode ? [:] : [CBCentralManagerScanOptionAllowDuplicatesKey: true]
        centralManager.scanForPeripherals(
            withServices: [MeshBLEConstants.serviceUUID],
            options: options
        )
        isScanning = true
        logger.debug("Scan started")
    }

    private func stopScan() {
        centralManager.stopScan()
        isScanning = false
        logger.debug("Scan stopped")
    }

    private func disconnectAll() {
        for peripheral in connectedPeripherals.values {
            centralManager.cancelPeripheralConnection(peripheral)
        }
        connectedPeripherals.removeAll()
        messageCharacteristics.removeAll()
        syncCharacteristics.removeAll()
    }

    private func cleanupPeerCache() {
        let now = Date()
        peerCache = peerCache.filter { now.timeIntervalSince($0.value) < MeshBLEConstants.peerCacheTimeout }
    }
}

// MARK: - CBCentralManagerDelegate

extension BLECentralManager: CBCentralManagerDelegate {
    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        logger.info("Central manager state: \(central.state.rawValue)")
        if central.state == .poweredOn {
            // P3: If startScanning() was called before BLE was ready, start now
            if pendingScanOnReady {
                logger.info("BLE now powered on — starting deferred scan")
                pendingScanOnReady = false
                meshRepository?.appendDiagnostic("ble_central_scan_start_deferred")
                scheduleDutyCycle()
            }
        }
    }

    func centralManager(_ central: CBCentralManager, didDiscover peripheral: CBPeripheral, advertisementData: [String: Any], rssi RSSI: NSNumber) {
        logger.debug("Discovered peripheral: \(peripheral.identifier)")

        // Check cache to avoid duplicate processing
        cleanupPeerCache()
        if peerCache[peripheral.identifier] != nil {
            return // Recently processed
        }
        peerCache[peripheral.identifier] = Date()

        // Store and connect
        discoveredPeripherals[peripheral.identifier] = peripheral
        peripheral.delegate = self
        centralManager.connect(peripheral, options: nil)
    }

    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        logger.info("Connected to \(peripheral.identifier)")
        meshRepository?.appendDiagnostic("ble_central_connected id=\(peripheral.identifier)")
        connectedPeripherals[peripheral.identifier] = peripheral
        peripheral.discoverServices([MeshBLEConstants.serviceUUID])
    }

    func centralManager(_ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral, error: Error?) {
        logger.error("Failed to connect to \(peripheral.identifier): \(error?.localizedDescription ?? "unknown")")
        meshRepository?.appendDiagnostic("ble_central_connect_fail id=\(peripheral.identifier) err=\(error?.localizedDescription ?? "none")")
    }

    func centralManager(_ central: CBCentralManager, didDisconnectPeripheral peripheral: CBPeripheral, error: Error?) {
        logger.info("Disconnected from \(peripheral.identifier)")
        meshRepository?.appendDiagnostic("ble_central_disconnected id=\(peripheral.identifier) err=\(error?.localizedDescription ?? "none")")
        connectedPeripherals.removeValue(forKey: peripheral.identifier)
        messageCharacteristics.removeValue(forKey: peripheral.identifier)
        syncCharacteristics.removeValue(forKey: peripheral.identifier)
    }

    func centralManager(_ central: CBCentralManager, willRestoreState dict: [String: Any]) {
        // State restoration (iOS-specific for background BLE)
        if let peripherals = dict[CBCentralManagerRestoredStatePeripheralsKey] as? [CBPeripheral] {
            logger.info("Restoring \(peripherals.count) peripherals")
            for peripheral in peripherals {
                peripheral.delegate = self
                connectedPeripherals[peripheral.identifier] = peripheral
            }
        }
    }
}

// MARK: - CBPeripheralDelegate

extension BLECentralManager: CBPeripheralDelegate {
    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        if let error = error {
            logger.error("Failed to discover services for \(peripheral.identifier): \(error.localizedDescription)")
            meshRepository?.appendDiagnostic("ble_central_discover_services_fail id=\(peripheral.identifier) err=\(error.localizedDescription)")
            return
        }

        guard let services = peripheral.services, !services.isEmpty else {
            logger.warning("No services found for \(peripheral.identifier)")
            meshRepository?.appendDiagnostic("ble_central_no_services id=\(peripheral.identifier)")
            return
        }

        meshRepository?.appendDiagnostic("ble_central_services_discovered id=\(peripheral.identifier) count=\(services.count)")

        for service in services where service.uuid == MeshBLEConstants.serviceUUID {
            peripheral.discoverCharacteristics([
                MeshBLEConstants.messageCharUUID,
                MeshBLEConstants.syncCharUUID,
                MeshBLEConstants.identityCharUUID
            ], for: service)
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        if let error = error {
            logger.error("Failed to discover characteristics for \(peripheral.identifier): \(error.localizedDescription)")
            meshRepository?.appendDiagnostic("ble_central_discover_chars_fail id=\(peripheral.identifier) err=\(error.localizedDescription)")
            return
        }

        guard let characteristics = service.characteristics else {
            meshRepository?.appendDiagnostic("ble_central_no_chars id=\(peripheral.identifier)")
            return
        }

        meshRepository?.appendDiagnostic("ble_central_chars_discovered id=\(peripheral.identifier) count=\(characteristics.count)")

        for characteristic in characteristics {
            switch characteristic.uuid {
            case MeshBLEConstants.messageCharUUID:
                messageCharacteristics[peripheral.identifier] = characteristic
                peripheral.setNotifyValue(true, for: characteristic)
                meshRepository?.appendDiagnostic("ble_central_subscribed_message id=\(peripheral.identifier)")
            case MeshBLEConstants.syncCharUUID:
                syncCharacteristics[peripheral.identifier] = characteristic
                meshRepository?.appendDiagnostic("ble_central_found_sync id=\(peripheral.identifier)")
            case MeshBLEConstants.identityCharUUID:
                meshRepository?.appendDiagnostic("ble_central_reading_identity id=\(peripheral.identifier)")
                peripheral.readValue(for: characteristic)
                // Schedule retry reads at T+900ms and T+2200ms (mirrors Android
                // IDENTITY_REFRESH_DELAYS_MS) for peripherals whose GATT server
                // may not be fully populated at characteristic discovery time.
                scheduleIdentityRefreshReads(peripheral: peripheral, characteristic: characteristic)
            default:
                break
            }
        }
    }

    private func scheduleIdentityRefreshReads(peripheral: CBPeripheral, characteristic: CBCharacteristic) {
        let peripheralId = peripheral.identifier
        for delayNs: UInt64 in [900_000_000, 2_200_000_000] {
            Task { [weak self] in
                try? await Task.sleep(nanoseconds: delayNs)
                guard let self, self.connectedPeripherals[peripheralId] != nil else { return }
                peripheral.readValue(for: characteristic)
            }
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        if let error = error {
            logger.error("Characteristic update error for \(characteristic.uuid.shortUUID): \(error.localizedDescription)")
            return
        }
        guard let data = characteristic.value, !data.isEmpty else { return }

        if characteristic.uuid == MeshBLEConstants.identityCharUUID {
            // Parse identity beacon — extract Ed25519 public key, do NOT treat as message data
            logger.debug("Identity beacon from \(peripheral.identifier): \(data.count) bytes")
            if let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
               let publicKeyHex = json["public_key"] as? String,
               publicKeyHex.count == 64 {
                DispatchQueue.main.async { [weak self] in
                    self?.meshRepository?.onPeerIdentityRead(
                        blePeerId: peripheral.identifier.uuidString,
                        info: json
                    )
                }
            } else {
                logger.warning("Could not parse identity beacon from \(peripheral.identifier)")
            }
        } else {
            // Message or sync data — handle reassembly
            if data.count < 4 {
                logger.warning("Received tiny BLE packet (<4 bytes) from \(peripheral.identifier)")
                return
            }

            let totalFrags = Int(data[0]) | (Int(data[1]) << 8)
            let fragIndex = Int(data[2]) | (Int(data[3]) << 8)
            let payload = data.subdata(in: 4..<data.count)

            let peripheralID = peripheral.identifier
            if fragIndex == 0 {
                reassemblyBuffers[peripheralID] = [0: payload]
                if totalFrags > 1 {
                    meshRepository?.appendDiagnostic("ble_central_rx_start total=\(totalFrags) from=\(peripheralID.uuidString.prefix(8))")
                }
            } else {
                var buffer = reassemblyBuffers[peripheralID] ?? [:]
                buffer[fragIndex] = payload
                reassemblyBuffers[peripheralID] = buffer
            }

            let currentCount = reassemblyBuffers[peripheralID]?.count ?? 0
            if currentCount == totalFrags {
                var completeData = Data()
                let buffer = reassemblyBuffers[peripheralID] ?? [:]
                for i in 0..<totalFrags {
                    if let chunk = buffer[i] {
                        completeData.append(chunk)
                    } else {
                        logger.error("Missing fragment \(i) in complete buffer for \(peripheralID)")
                        return
                    }
                }
                reassemblyBuffers.removeValue(forKey: peripheralID)

                logger.info("Reassembled complete message (\(completeData.count) bytes) from \(peripheralID)")
                meshRepository?.appendDiagnostic("ble_central_rx_complete size=\(completeData.count)")
                DispatchQueue.main.async { [weak self] in
                    self?.meshRepository?.onBleDataReceived(peerId: peripheralID.uuidString, data: completeData)
                }
            }
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, error: Error?) {
        if let error = error {
            logger.error("Write error for \(peripheral.identifier): \(error.localizedDescription)")
            meshRepository?.appendDiagnostic("ble_central_write_fail id=\(peripheral.identifier) err=\(error.localizedDescription)")
            // Clear current write state to allow retry/next
            writeInProgress[peripheral.identifier] = false
            return
        }

        // Dequeue next write
        let peripheralId = peripheral.identifier
        writeInProgress[peripheralId] = false
        if let next = pendingWrites[peripheralId]?.first {
            pendingWrites[peripheralId]?.removeFirst()
            enqueueFragment(next, for: peripheralId)
        }
    }
}
