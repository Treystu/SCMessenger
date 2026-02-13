//
//  IosPlatformBridge.swift
//  SCMessenger
//
//  Implements Rust PlatformBridge callback interface for iOS
//  Mirrors: android/.../service/AndroidPlatformBridge.kt
//

import UIKit
import CoreMotion
import Network
import os

/// Implements Rust PlatformBridge callback interface for iOS
/// Bridges iOS platform APIs to Rust core
///
/// Responsibilities:
/// - Monitor battery level and charging state
/// - Monitor network connectivity (WiFi/cellular)
/// - Monitor device motion state
/// - Forward BLE data between iOS and Rust
/// - Report app lifecycle events to Rust
final class IosPlatformBridge: PlatformBridge {
    private let logger = Logger(subsystem: "com.scmessenger", category: "Platform")
    private let motionManager = CMMotionActivityManager()
    private let pathMonitor = NWPathMonitor()
    private weak var meshRepository: MeshRepository?
    
    private var batteryObserver: NSObjectProtocol?
    private var chargingObserver: NSObjectProtocol?
    
    // MARK: - Configuration
    
    func configure(repository: MeshRepository) {
        self.meshRepository = repository
        startBatteryMonitoring()
        startNetworkMonitoring()
        startMotionMonitoring()
        logger.info("IosPlatformBridge configured")
    }
    
    // MARK: - PlatformBridge Protocol (called FROM Rust)
    
    func onBatteryChanged(batteryPct: UInt8, isCharging: Bool) {
        logger.debug("Battery changed: \(batteryPct)% charging=\(isCharging)")
        // Rust notifying us of battery change (we're the source, so just log)
    }
    
    func onNetworkChanged(hasWifi: Bool, hasCellular: Bool) {
        logger.debug("Network changed: wifi=\(hasWifi) cellular=\(hasCellular)")
        // Rust notifying us of network change (we're the source, so just log)
    }
    
    func onMotionChanged(motion: MotionState) {
        logger.debug("Motion changed: \(String(describing: motion))")
        // Rust notifying us of motion change (we're the source, so just log)
    }
    
    func onBleDataReceived(peerId: String, data: Data) {
        logger.debug("BLE data received from \(peerId): \(data.count) bytes")
        // Forward BLE data to repository for processing
        meshRepository?.onBleDataReceived(peerId: peerId, data: data)
    }
    
    func onEnteringBackground() {
        logger.info("Rust notified: App entering background")
        // Rust knows we're going to background, can adjust behavior
    }
    
    func onEnteringForeground() {
        logger.info("Rust notified: App entering foreground")
        // Rust knows we're in foreground, can resume full activity
    }
    
    func sendBlePacket(peerId: String, data: Data) {
        logger.debug("Rust requests BLE send to \(peerId): \(data.count) bytes")
        // Rust wants to send BLE data - forward to repository's BLE transport
        meshRepository?.sendBlePacket(peerId: peerId, data: data)
    }
    
    // MARK: - iOS System Monitoring
    
    private func startBatteryMonitoring() {
        UIDevice.current.isBatteryMonitoringEnabled = true
        
        // Observe battery level changes
        batteryObserver = NotificationCenter.default.addObserver(
            forName: UIDevice.batteryLevelDidChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.reportBatteryState()
        }
        
        // Observe charging state changes
        chargingObserver = NotificationCenter.default.addObserver(
            forName: UIDevice.batteryStateDidChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.reportBatteryState()
        }
        
        // Report initial state
        reportBatteryState()
    }
    
    private func reportBatteryState() {
        let level = UIDevice.current.batteryLevel
        let pct = UInt8(max(0, min(100, level * 100)))
        let charging = UIDevice.current.batteryState == .charging
                    || UIDevice.current.batteryState == .full
        
        logger.debug("Reporting battery: \(pct)% charging=\(charging)")
        meshRepository?.reportBattery(pct: pct, charging: charging)
    }
    
    private func startNetworkMonitoring() {
        pathMonitor.pathUpdateHandler = { [weak self] path in
            guard let self = self else { return }
            
            let hasWifi = path.usesInterfaceType(.wifi)
            let hasCellular = path.usesInterfaceType(.cellular)
            let isExpensive = path.isExpensive
            let isConstrained = path.isConstrained
            
            self.logger.debug("Network path updated: wifi=\(hasWifi) cellular=\(hasCellular) expensive=\(isExpensive) constrained=\(isConstrained)")
            
            self.meshRepository?.reportNetwork(wifi: hasWifi, cellular: hasCellular)
        }
        
        pathMonitor.start(queue: DispatchQueue.global(qos: .utility))
    }
    
    private func startMotionMonitoring() {
        guard CMMotionActivityManager.isActivityAvailable() else {
            logger.info("Motion activity not available on this device")
            return
        }
        
        motionManager.startActivityUpdates(to: .main) { [weak self] activity in
            guard let self = self, let activity = activity else { return }
            
            let state: MotionState
            if activity.automotive {
                state = .automotive
            } else if activity.running {
                state = .running
            } else if activity.walking {
                state = .walking
            } else if activity.stationary {
                state = .still
            } else {
                state = .unknown
            }
            
            self.logger.debug("Motion state: \(String(describing: state))")
            self.meshRepository?.reportMotion(state: state)
        }
    }
    
    // MARK: - Lifecycle
    
    deinit {
        logger.info("IosPlatformBridge deinit")
        
        // Stop monitoring
        pathMonitor.cancel()
        motionManager.stopActivityUpdates()
        UIDevice.current.isBatteryMonitoringEnabled = false
        
        // Remove observers
        if let batteryObserver = batteryObserver {
            NotificationCenter.default.removeObserver(batteryObserver)
        }
        if let chargingObserver = chargingObserver {
            NotificationCenter.default.removeObserver(chargingObserver)
        }
    }
}

// MARK: - Helper Extensions

extension MotionState: CustomStringConvertible {
    public var description: String {
        switch self {
        case .still: return "still"
        case .walking: return "walking"
        case .running: return "running"
        case .automotive: return "automotive"
        case .unknown: return "unknown"
        }
    }
}
