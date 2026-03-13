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

    // --- Adaptive throttling state ---
    private var lastBatteryReportTime: Date = .distantPast
    private var lastMotionReportTime: Date = .distantPast
    private var lastReportedMotionState: MotionState?
    private var pendingMotionState: MotionState?
    private var motionCoalesceTimer: Timer?
    private var lastReportedBatteryPct: UInt8 = 255  // sentinel
    private var lastReportedCharging: Bool?

    /// Adaptive minimum interval between power-related reports.
    /// High battery → long delay (stable, no rush).
    /// Low battery → shorter delay (user needs accurate info, drain matters).
    private var adaptiveIntervalSeconds: TimeInterval {
        let pct = lastReportedBatteryPct
        if pct == 255 { return 5 } // first report, use short interval

        // Adaptive report intervals based on battery health
        if pct >= 95 { return 300 } // Near full: report every 5 mins
        if pct >= 80 { return 120 } // Healthy: report every 2 mins
        if pct >= 50 { return 30 }  // Mid-range: report every 30s
        if pct >= 25 { return 10 }  // Getting low: report every 10s
        return 5                    // Low battery: report every 5s
    }

    private func withMeshRepositoryOnMain(_ body: @escaping @MainActor (MeshRepository) -> Void) {
        let meshRepository = self.meshRepository
        Task { @MainActor in
            guard let meshRepository else { return }
            // PlatformBridge callbacks are one-way notifications into MeshRepository.
            // Intentionally fire-and-forget so non-main system callbacks don't synchronously
            // block on MainActor hops or require callback-side error handling.
            body(meshRepository)
        }
    }

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
        withMeshRepositoryOnMain { $0.onBleDataReceived(peerId: peerId, data: data) }
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
        withMeshRepositoryOnMain { $0.sendBlePacket(peerId: peerId, data: data) }
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

        // Skip if nothing changed AND we're within the adaptive throttle window
        let elapsed = Date().timeIntervalSince(lastBatteryReportTime)
        if pct == lastReportedBatteryPct && charging == lastReportedCharging && elapsed < adaptiveIntervalSeconds {
            return
        }

        lastBatteryReportTime = Date()
        lastReportedBatteryPct = pct
        lastReportedCharging = charging
        logger.debug("Reporting battery: \(pct)% charging=\(charging)")
        withMeshRepositoryOnMain { $0.reportBattery(pct: pct, charging: charging) }
    }

    private func startNetworkMonitoring() {
        pathMonitor.pathUpdateHandler = { [weak self] path in
            guard let self = self else { return }

            let hasWifi = path.usesInterfaceType(.wifi)
            let hasCellular = path.usesInterfaceType(.cellular)
            let isExpensive = path.isExpensive
            let isConstrained = path.isConstrained

            self.logger.debug("Network path updated: wifi=\(hasWifi) cellular=\(hasCellular) expensive=\(isExpensive) constrained=\(isConstrained)")

            self.withMeshRepositoryOnMain { $0.reportNetwork(wifi: hasWifi, cellular: hasCellular) }
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

            self.coalesceMotionReport(state)
        }
    }

    /// Coalesce rapid-fire motion updates into a single report.
    /// If the state genuinely changes (e.g. still → walking), report immediately.
    /// If the state toggles rapidly (e.g. still ↔ unknown), debounce via timer.
    private func coalesceMotionReport(_ state: MotionState) {
        let elapsed = Date().timeIntervalSince(lastMotionReportTime)

        // Significant motion change (e.g. still → walking, or walking → automotive)
        // gets reported immediately regardless of throttle.
        let isSignificantChange: Bool = {
            guard let last = lastReportedMotionState else { return true }
            // still ↔ unknown is NOT significant — it's sensor noise
            let noiseStates: Set<MotionState> = [.still, .unknown]
            if noiseStates.contains(last) && noiseStates.contains(state) {
                return false
            }
            return last != state
        }()

        if isSignificantChange {
            motionCoalesceTimer?.invalidate()
            motionCoalesceTimer = nil
            deliverMotionReport(state)
            return
        }

        // Same state or noise toggle — throttle
        if elapsed < adaptiveIntervalSeconds {
            // Store the latest state; a timer will deliver it later
            pendingMotionState = state
            if motionCoalesceTimer == nil {
                let remaining = adaptiveIntervalSeconds - elapsed
                motionCoalesceTimer = Timer.scheduledTimer(withTimeInterval: remaining, repeats: false) { [weak self] _ in
                    guard let self, let pending = self.pendingMotionState else { return }
                    self.deliverMotionReport(pending)
                    self.motionCoalesceTimer = nil
                    self.pendingMotionState = nil
                }
            }
            return
        }

        // Throttle window elapsed — deliver now
        motionCoalesceTimer?.invalidate()
        motionCoalesceTimer = nil
        pendingMotionState = nil
        deliverMotionReport(state)
    }

    private func deliverMotionReport(_ state: MotionState) {
        lastMotionReportTime = Date()
        lastReportedMotionState = state
        logger.debug("Motion state: \(String(describing: state))")
        withMeshRepositoryOnMain { $0.reportMotion(state: state) }
    }

    // MARK: - Lifecycle

    deinit {
        logger.info("IosPlatformBridge deinit")

        // Stop monitoring
        motionCoalesceTimer?.invalidate()
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
