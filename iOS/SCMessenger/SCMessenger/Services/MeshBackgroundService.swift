import BackgroundTasks
import os

/// Manages all iOS background execution strategies
/// iOS equivalent of Android's MeshForegroundService
///
/// iOS has no persistent foreground service like Android. Uses:
/// - BGTaskScheduler for periodic wakeups (background fetch/processing)
/// - CoreBluetooth background modes for mesh keepalive
/// - Location services for optional background triggers
@Observable
@MainActor
final class MeshBackgroundService {
    private let logger: Logger = Logger(subsystem: "com.scmessenger", category: "Background")
    private let meshRepository: MeshRepository

    // BGTask identifiers - must match Info.plist BGTaskSchedulerPermittedIdentifiers
    static let refreshTaskId: String = "com.scmessenger.mesh.refresh"
    static let processingTaskId: String = "com.scmessenger.mesh.processing"

    // Background task state
    private var refreshTaskScheduled: Bool = false
    private var processingTaskScheduled: Bool = false

    init(meshRepository: MeshRepository) {
        self.meshRepository = meshRepository
    }

    // MARK: - Public API

    /// Register background tasks — call from app init
    func registerBackgroundTasks() {
        logger.info("Registering background tasks")

        // Register refresh task (quick sync, 30 seconds max)
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: Self.refreshTaskId,
            using: nil
        ) { [weak self] task in
            guard let self = self else {
                task.setTaskCompleted(success: false)
                return
            }
            guard let refreshTask = task as? BGAppRefreshTask else {
                logger.error("Background refresh: unexpected task type \(type(of: task))")
                task.setTaskCompleted(success: false)
                return
            }
            self.handleBackgroundRefresh(refreshTask)
        }

        // Register processing task (longer operations, several minutes)
        BGTaskScheduler.shared.register(
            forTaskWithIdentifier: Self.processingTaskId,
            using: nil
        ) { [weak self] task in
            guard let self = self else {
                task.setTaskCompleted(success: false)
                return
            }
            guard let processingTask = task as? BGProcessingTask else {
                logger.error("Background processing: unexpected task type \(type(of: task))")
                task.setTaskCompleted(success: false)
                return
            }
            self.handleBackgroundProcessing(processingTask)
        }

        logger.info("Background tasks registered successfully")
    }

    /// Called when app enters background
    func onEnteringBackground() {
        logger.info("App entering background")
        meshRepository.onEnteringBackground()
        scheduleBackgroundRefresh()
        scheduleBackgroundProcessing()
    }

    /// Called when app enters foreground
    func onEnteringForeground() {
        logger.info("App entering foreground")
        meshRepository.onEnteringForeground()
    }

    // MARK: - Background Task Scheduling

    /// Schedule next background fetch
    private func scheduleBackgroundRefresh() {
        guard !refreshTaskScheduled else { return }

        let request: BGAppRefreshTaskRequest = BGAppRefreshTaskRequest(identifier: Self.refreshTaskId)
        request.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60) // 15 min

        do {
            try BGTaskScheduler.shared.submit(request)
            refreshTaskScheduled = true
            logger.info("Background refresh scheduled")
        } catch {
            logger.error("Failed to schedule background refresh: \(error.localizedDescription)")
        }
    }

    /// Schedule background processing (longer tasks)
    private func scheduleBackgroundProcessing() {
        guard !processingTaskScheduled else { return }

        let request: BGProcessingTaskRequest = BGProcessingTaskRequest(identifier: Self.processingTaskId)
        request.requiresNetworkConnectivity = false // mesh works offline
        request.requiresExternalPower = false
        request.earliestBeginDate = Date(timeIntervalSinceNow: 60 * 60) // 1 hour

        do {
            try BGTaskScheduler.shared.submit(request)
            processingTaskScheduled = true
            logger.info("Background processing scheduled")
        } catch {
            logger.error("Failed to schedule background processing: \(error.localizedDescription)")
        }
    }

    // MARK: - Background Task Handlers

    /// Handle background refresh wakeup (quick sync, 30 seconds max)
    private func handleBackgroundRefresh(_ task: BGAppRefreshTask) {
        logger.info("Background refresh triggered")
        refreshTaskScheduled = false

        // Schedule next one
        scheduleBackgroundRefresh()

        // Set expiration handler
        task.expirationHandler = { [weak self] in
            Task { @MainActor in
                self?.logger.warning("Background refresh expired, pausing service")
                self?.meshRepository.pauseService()
            }
        }

        // Perform quick sync operations
        Task { @MainActor [weak self] in
            guard let self = self else { return }
            do {
                // Sync pending messages
                try await self.meshRepository.syncPendingMessages()

                // Update stats
                self.meshRepository.updateStats()

                // Discover nearby peers (quick scan)
                try await self.meshRepository.quickPeerDiscovery()

                task.setTaskCompleted(success: true)
                self.logger.info("Background refresh completed successfully")
            } catch {
                self.logger.error("Background refresh failed: \(error.localizedDescription)")
                task.setTaskCompleted(success: false)
            }
        }
    }

    /// Handle background processing (bulk operations, several minutes)
    private func handleBackgroundProcessing(_ task: BGProcessingTask) {
        logger.info("Background processing triggered")
        processingTaskScheduled = false

        // Schedule next one
        scheduleBackgroundProcessing()

        // Set expiration handler
        task.expirationHandler = { [weak self] in
            Task { @MainActor in
                self?.logger.warning("Background processing expired, pausing service")
                self?.meshRepository.pauseService()
            }
        }

        // Perform bulk operations
        Task { @MainActor [weak self] in
            guard let self = self else { return }
            do {
                // Full sync with all known peers
                try await self.meshRepository.performBulkSync()

                // Cleanup old messages
                try await self.meshRepository.cleanupOldMessages()

                // Update peer connection ledger
                try await self.meshRepository.updatePeerLedger()

                // Run Rust core maintenance cycle (25s budget)
                let report: String = self.meshRepository.ironCore?.runMaintenanceCycle(budgetMs: 25000) ?? "no core"
                self.logger.info("Maintenance cycle: \(report)")

                task.setTaskCompleted(success: true)
                self.logger.info("Background processing completed successfully")
            } catch {
                self.logger.error("Background processing failed: \(error.localizedDescription)")
                task.setTaskCompleted(success: false)
            }
        }
    }
}

// MARK: - Simulated background tasks for testing
#if DEBUG
extension MeshBackgroundService {
    @discardableResult
    func simulateBackgroundRefresh() -> Task<Void, Never> {
        logger.debug("[INFO] Simulating background refresh")
        return Task {
            do {
                try await meshRepository.syncPendingMessages()
                meshRepository.updateStats()
                try await meshRepository.quickPeerDiscovery()
                logger.debug("[OK] Simulated background refresh completed")
            } catch {
                logger.error("[ERROR] Simulated background refresh failed: \(error.localizedDescription)")
            }
        }
    }

    @discardableResult
    func simulateBackgroundProcessing() -> Task<Void, Never> {
        logger.debug("[INFO] Simulating background processing")
        return Task {
            do {
                try await meshRepository.performBulkSync()
                try await meshRepository.cleanupOldMessages()
                try await meshRepository.updatePeerLedger()
                // Note: runMaintenanceCycle(budgetMs:) intentionally excluded from simulation
                // to avoid side effects and timing behavior unsuitable for a fast test loop.
                logger.debug("[OK] Simulated background processing completed")
            } catch {
                logger.error("[ERROR] Simulated background processing failed: \(error.localizedDescription)")
            }
        }
    }
}
#endif