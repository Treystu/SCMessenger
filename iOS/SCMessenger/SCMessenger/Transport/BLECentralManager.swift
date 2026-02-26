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

        // Write queue management (mirrors Android)
        if writeInProgress[peripheralId] == true {
            logger.debug("Write in progress, queueing data")
            pendingWrites[peripheralId, default: []].append(data)
            return true
        }

        writeInProgress[peripheralId] = true
        peripheral.writeValue(data, for: characteristic, type: .withResponse)
        logger.debug("Writing \(data.count) bytes to \(peripheralId)")
        return true
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
        guard let services = peripheral.services else { return }
        for service in services where service.uuid == MeshBLEConstants.serviceUUID {
            peripheral.discoverCharacteristics([
                MeshBLEConstants.messageCharUUID,
                MeshBLEConstants.syncCharUUID,
                MeshBLEConstants.identityCharUUID
            ], for: service)
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        guard let characteristics = service.characteristics else { return }
        for characteristic in characteristics {
            switch characteristic.uuid {
            case MeshBLEConstants.messageCharUUID:
                messageCharacteristics[peripheral.identifier] = characteristic
            case MeshBLEConstants.syncCharUUID:
                syncCharacteristics[peripheral.identifier] = characteristic
                peripheral.setNotifyValue(true, for: characteristic)
            case MeshBLEConstants.identityCharUUID:
                peripheral.readValue(for: characteristic)
            default:
                break
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
            // Message or sync data — route to mesh service
            logger.debug("Data from \(peripheral.identifier): \(data.count) bytes on \(characteristic.uuid.shortUUID)")
            DispatchQueue.main.async { [weak self] in
                self?.meshRepository?.onBleDataReceived(peerId: peripheral.identifier.uuidString, data: data)
            }
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, error: Error?) {
        // Dequeue next write (mirrors Android pattern)
        writeInProgress[peripheral.identifier] = false
        if let next = pendingWrites[peripheral.identifier]?.first {
            pendingWrites[peripheral.identifier]?.removeFirst()
            sendData(to: peripheral.identifier, data: next)
        }
    }
}
