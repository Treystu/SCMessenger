# iOS v0.4.0 Parity Implementation Plan

**Status:** Active  
**Last Updated:** 2026-07-27  
**Scope:** Full iOS feature parity with Android for v0.4.0 Josh alpha  
**Effort Metric:** LoC estimates only (no time estimates per project philosophy)  
**Target Implementer:** Gemini 3.6 Flash (tasks sized for single dispatch)  
**Sequencing:** Dependency DAG; parallel lanes where file-collision-safe  

---

## 0. Plan Philosophy

- **LoC, not hours:** Every task carries a LoC estimate. This is the only effort metric.
- **Single-dispatch tasks:** Each task must fit in one model dispatch. If it doesn't fit, split it.
- **Android as reference:** Every iOS task points to the exact Android source to copy/adapt.
- **No time estimates:** Per project rules. Progress is measured by tasks completed and gates passed.
- **Tier tags:** [HAIKU] = mechanical, verbatim diff; [SONNET] = implementation with patterns; [OPUS+] = design/spec (not assigned to Flash).
- **Hotspot rule:** `MeshRepository.swift` is a single-writer file. Tasks that touch it must queue.

---

## 1. Dependency DAG Overview

```
LANE A: MeshRepository Core Methods (single writer, serial)
  A1 -> A2 -> A3 -> A4 -> A5 -> A6 -> A7 -> A8 -> A9 -> A10

LANE B: ViewModels (parallel with Lane A, no file collision)
  B1 -> B2 -> B3 -> B4 -> B5

LANE C: Screens (parallel with Lane B, depends on B items)
  C1 (needs B1) -> C2 (needs B2) -> C3 (needs B5)

LANE D: Delivery State System (parallel with Lane A, touches MeshRepository + ViewModel + UI)
  D1 (needs A1) -> D2 (needs D1) -> D3 (needs D2)

LANE E: Diagnostics Enhancement (parallel, no collision)
  E1 -> E2

LANE F: Transport Health (parallel with Lane A)
  F1 -> F2

LANE G: XCTest Coverage (parallel with everything)
  G1 -> G2 -> G3

LANE H: Identity & Crypto Utils (parallel with Lane A)
  H1 -> H2
```

---

## 2. Lane A: MeshRepository Core Methods (Serial, Single Writer)

All tasks modify `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`. Execute strictly in order. Estimated total: ~1,200 LoC added.

### A1 [HAIKU] Add Block/Unblock/IsBlocked/ListBlocked/GetBlockedCount
**Tier:** [HAIKU]  
**Estimated LoC:** ~80 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:4121-4190`  
**Description:** Thin UniFFI wrappers for peer blocking.

**Exact methods to add:**
```swift
func blockPeer(peerId: String, deviceId: String? = nil, reason: String? = nil) throws {
    guard let core = ironCore else { throw MeshError.notInitialized("IronCore nil") }
    try core.blockPeer(peerId: peerId)
}

func unblockPeer(peerId: String, deviceId: String? = nil) throws {
    guard let core = ironCore else { throw MeshError.notInitialized("IronCore nil") }
    try core.unblockPeer(peerId: peerId)
}

func isBlocked(peerId: String, deviceId: String? = nil) -> Bool {
    guard let core = ironCore else { return false }
    return core.isPeerBlocked(peerId: peerId)
}

func listBlockedPeers() throws -> [BlockedIdentity] {
    guard let core = ironCore else { throw MeshError.notInitialized("IronCore nil") }
    return try core.listBlockedPeers()
}

func getBlockedCount() -> UInt {
    guard let core = ironCore else { return 0 }
    return core.blockedCount()
}

func mutePeer(peerId: String, deviceId: String? = nil, reason: String? = nil) throws {
    // Alias to blockPeer with "muted" reason
    try blockPeer(peerId: peerId, reason: reason ?? "muted")
}

func blockAndDeletePeer(peerId: String, deviceId: String? = nil, reason: String? = nil) throws {
    guard let core = ironCore else { throw MeshError.notInitialized("IronCore nil") }
    try core.blockAndDeletePeer(peerId: peerId)
}
```

**Acceptance Criteria:**
- [ ] All 7 methods compile
- [ ] `listBlockedPeers()` returns correct `BlockedIdentity` array type matching UniFFI generated API
- [ ] Methods are placed after existing contact methods in MeshRepository.swift (around line 3450)

---

### A2 [HAIKU] Add Message Request Accept/Reject
**Tier:** [HAIKU]  
**Estimated LoC:** ~40 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:4205-4249`  
**Description:** iOS `RequestsInboxView` already calls `repository.acceptMessageRequest(peerId:)` but the method does not exist. This is a latent runtime crash.

**Exact methods to add:**
```swift
func acceptMessageRequest(peerId: String) throws {
    guard let core = ironCore else { throw MeshError.notInitialized("IronCore nil") }
    // Add contact by peer ID; the Rust contactsManager has addContactByPeerId
    let cm = core.contactsManager()
    _ = try cm.add(contact: Contact(
        peerId: peerId,
        nickname: nil,
        localNickname: nil,
        publicKey: "", // Will be resolved by Rust
        addedAt: UInt64(Date().timeIntervalSince1970),
        lastSeen: nil,
        notes: nil,
        lastKnownDeviceId: nil,
        verifiedAt: nil,
        isTombstone: false
    ))
}

func rejectMessageRequest(peerId: String) throws {
    // Reject = block the sender
    try blockPeer(peerId: peerId, reason: "rejected message request")
}

func getPendingMessageRequests() -> [MessageRequestThread] {
    guard let core = ironCore else { return [] }
    return core.getMessageRequests(limit: 100)
}
```

**Acceptance Criteria:**
- [ ] `acceptMessageRequest` and `rejectMessageRequest` compile
- [ ] `getPendingMessageRequests()` returns `[MessageRequestThread]` (verify this type exists in generated `api.swift`)
- [ ] Callsites in `MainTabView.swift` no longer reference missing methods

---

### A3 [HAIKU] Add Pending Outbox / Delivery Tracking Methods
**Tier:** [HAIKU]  
**Estimated LoC:** ~120 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:751-798`  
**Description:** Message retry tracking utilities used by ChatViewModel.

**Exact methods to add:**
```swift
func loadPendingOutboxAsync() async -> [PendingOutboundEnvelope] {
    await Task.detached {
        self.loadPendingOutbox()
    }.value
}

func loadPendingOutbox() -> [PendingOutboundEnvelope] {
    guard FileManager.default.fileExists(atPath: pendingOutboxURL.path),
          let data = try? Data(contentsOf: pendingOutboxURL),
          let decoded = try? JSONDecoder().decode([PendingOutboundEnvelope].self, from: data) else {
        return []
    }
    return decoded
}

func getPendingOutboxCount() -> Int {
    loadPendingOutbox().count
}

func incrementAttemptCount(messageId: String) {
    var pending = loadPendingOutbox()
    guard let idx = pending.firstIndex(where: { $0.historyRecordId == messageId || $0.queueId == messageId }) else { return }
    pending[idx].attemptCount += 1
    savePendingOutbox(pending)
}

func shouldRetryMessage(messageId: String) -> Bool {
    guard let tracking = getMessageIdTracking(messageId: messageId) else { return true }
    return tracking.attemptCount < 12 && !tracking.isCorrupted
}

func getRetryDelay(attemptCount: Int) -> UInt64 {
    // Exponential backoff: 1s, 2s, 4s, 8s, 15s, 30s, 60s, 120s, 240s, 300s, 300s, 300s
    let delays: [UInt64] = [1, 2, 4, 8, 15, 30, 60, 120, 240, 300, 300, 300]
    let idx = min(max(attemptCount, 0), delays.count - 1)
    return delays[idx]
}

func logMessageDeliveryAttempt(messageId: String, attempt: Int, outcome: String) {
    logDiagnostic("delivery_attempt msg=\(messageId.prefix(8)) attempt=\(attempt) outcome=\(outcome)")
}

func markMessageCorrupted(messageId: String) {
    // Mark in pending outbox as terminal failure
    var pending = loadPendingOutbox()
    if let idx = pending.firstIndex(where: { $0.historyRecordId == messageId }) {
        pending[idx].terminalFailureCode = "corrupted"
        savePendingOutbox(pending)
    }
}

private struct MessageTracking {
    var attemptCount: Int = 0
    var lastAttemptAt: Date?
    var isCorrupted: Bool = false
}

private func getMessageIdTracking(messageId: String) -> MessageTracking? {
    // Simple in-memory tracking; expand if needed
    nil
}

private func savePendingOutbox(_ envelopes: [PendingOutboundEnvelope]) {
    do {
        let data = try JSONEncoder().encode(envelopes)
        try data.write(to: pendingOutboxURL)
    } catch {
        logger.error("Failed to save pending outbox: \(error.localizedDescription)")
    }
}
```

**Acceptance Criteria:**
- [ ] `loadPendingOutboxAsync()` compiles and returns deserialized pending envelopes
- [ ] `incrementAttemptCount` increments and persists
- [ ] `getRetryDelay` returns correct exponential backoff values

---

### A4 [SONNET] Add Delivery State Resolution to MeshRepository
**Tier:** [SONNET]  
**Estimated LoC:** ~80 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/chat/DeliveryStateSurface.kt` (full file, 64 lines)  
**Description:** Port Android's `DeliveryStateMapper` to Swift inside MeshRepository. This is the core logic that powers delivery state indicators.

**Exact type and method to add:**
```swift
// Add to MeshRepository.swift, after existing struct definitions

enum DeliveryStateSurface: String {
    case pending = "pending"
    case stored = "stored"
    case forwarding = "forwarding"
    case rejected = "rejected"
    case delivered = "delivered"

    var label: String { rawValue }
    var detail: String {
        switch self {
        case .pending: return "Queued locally. First route attempt is still in progress."
        case .stored: return "Stored for retry. The recipient is currently offline or unreachable."
        case .forwarding: return "Actively retrying through direct or relay paths."
        case .rejected: return "Rejected because the recipient identity is no longer valid for this device."
        case .delivered: return "Delivery receipt confirmed by the recipient node."
        }
    }
}

struct DeliveryStatePresentation {
    let state: DeliveryStateSurface
    let label: String
    let detail: String
}

func resolveDeliveryState(
    delivered: Bool,
    messageId: String,
    nowEpochSec: UInt64 = UInt64(Date().timeIntervalSince1970)
) -> DeliveryStatePresentation {
    let pending = getPendingDeliverySnapshot(messageId: messageId)

    let state: DeliveryStateSurface
    if delivered {
        state = .delivered
    } else if let code = pending?.terminalFailureCode {
        state = .rejected
    } else if let p = pending, p.nextAttemptAtEpochSec <= nowEpochSec {
        state = .forwarding
    } else if pending != nil {
        state = .stored
    } else {
        state = .pending
    }

    let detail: String
    switch pending?.terminalFailureCode {
    case "identity_device_mismatch":
        detail = "Rejected because this identity moved to another device. Refresh the contact before retrying."
    case "identity_abandoned":
        detail = "Rejected because the contact abandoned this identity. Re-verify the contact before retrying."
    default:
        detail = state.detail
    }

    return DeliveryStatePresentation(state: state, label: state.label, detail: detail)
}

func getPendingDeliverySnapshot(messageId: String) -> PendingDeliverySnapshot? {
    let pending = loadPendingOutbox()
    guard let envelope = pending.first(where: { $0.historyRecordId == messageId || $0.queueId == messageId }) else {
        return nil
    }
    return PendingDeliverySnapshot(
        attemptCount: Int(envelope.attemptCount),
        nextAttemptAtEpochSec: envelope.nextAttemptAtEpochSec,
        terminalFailureCode: envelope.terminalFailureCode
    )
}

func getPendingTerminalFailureCode(messageId: String) -> String? {
    loadPendingOutbox().first(where: { $0.historyRecordId == messageId })?.terminalFailureCode
}
```

**Note:** Update `PendingOutboundEnvelope` to make `attemptCount` and `nextAttemptAtEpochSec` mutable if they aren't already.

**Acceptance Criteria:**
- [ ] `resolveDeliveryState` returns correct state for all 5 cases (pending, stored, forwarding, rejected, delivered)
- [ ] Terminal failure codes produce custom detail messages
- [ ] Method matches Android `DeliveryStateMapper.resolve` logic exactly

---

### A5 [HAIKU] Add Identity/Crypto Utility Methods
**Tier:** [HAIKU]  
**Estimated LoC:** ~60 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:4250-4290`  
**Description:** Thin wrappers for identity inspection and crypto utilities.

**Exact methods to add:**
```swift
func signData(data: Data) throws -> SignatureResult {
    guard let core = ironCore else { throw MeshError.notInitialized("IronCore nil") }
    return try core.signData(data: data)
}

func verifySignature(data: Data, signature: Data, publicKeyHex: String) -> Bool {
    guard let core = ironCore else { return false }
    do {
        return try core.verifySignature(data: data, signature: signature, publicKeyHex: publicKeyHex)
    } catch {
        return false
    }
}

func getDeviceId() -> String? {
    ironCore?.getDeviceId()
}

func getSeniorityTimestamp() -> UInt64? {
    ironCore?.getSeniorityTimestamp()
}

func getRegistrationState(identityId: String) -> RegistrationStateInfo? {
    guard let core = ironCore else { return nil }
    return try? core.getRegistrationState(identityId: identityId)
}

func exportLogs() -> String? {
    ironCore?.exportLogs()
}

func updateContactDeviceId(peerId: String, deviceId: String?) throws {
    guard let cm = contactManager else { throw MeshError.notInitialized("ContactManager nil") }
    try cm.updateDeviceId(peerId: peerId, deviceId: deviceId)
}
```

**Acceptance Criteria:**
- [ ] All methods compile against UniFFI generated API
- [ ] `signData` returns `SignatureResult` type from generated API
- [ ] `getRegistrationState` returns `RegistrationStateInfo?` type from generated API

---

### A6 [HAIKU] Add Settings/Transport Utility Methods
**Tier:** [HAIKU]  
**Estimated LoC:** ~100 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:1278, 3707, 9014`  
**Description:** Transport health, bootstrap retry, and settings application methods.

**Exact methods to add:**
```swift
func applyTransportSettings(_ settings: MeshSettings) {
    // Restart BLE if setting changed
    if settings.bleEnabled {
        blePeripheralManager?.startAdvertising()
        bleCentralManager?.startScanning()
    } else {
        blePeripheralManager?.stopAdvertising()
        bleCentralManager?.stopScanning()
    }
    // Restart Multipeer if setting changed
    if settings.internetEnabled {
        multipeerTransport?.startAdvertising()
        multipeerTransport?.startBrowsing()
    } else {
        multipeerTransport?.disconnect()
    }
    logDiagnostic("transport_settings_applied ble=\(settings.bleEnabled) internet=\(settings.internetEnabled)")
}

func getDefaultSettings() -> MeshSettings {
    settingsManager?.defaultSettings() ?? MeshSettings(
        relayEnabled: true,
        maxRelayBudget: 200,
        batteryFloor: 20,
        bleEnabled: true,
        wifiAwareEnabled: false,
        wifiDirectEnabled: false,
        internetEnabled: true,
        discoveryMode: .normal,
        onionRouting: false,
        coverTrafficEnabled: false,
        messagePaddingEnabled: false,
        timingObfuscationEnabled: false,
        notificationsEnabled: true,
        notifyDmEnabled: true,
        notifyDmRequestEnabled: true,
        notifyDmInForeground: false,
        notifyDmRequestInForeground: true,
        soundEnabled: true,
        badgeEnabled: true,
        requirePq: false
    )
}

func resetAllData() async {
    stopMeshService()
    // Clear storage
    let fm = FileManager.default
    try? fm.removeItem(atPath: storagePath)
    try? fm.createDirectory(atPath: storagePath, withIntermediateDirectories: true)
    // Clear UserDefaults
    UserDefaults.standard.removeObject(forKey: IdentityCacheStore.key)
    UserDefaults.standard.removeObject(forKey: InstallMarker.key)
    UserDefaults.standard.removeObject(forKey: "ble_rotation_enabled")
    UserDefaults.standard.removeObject(forKey: "ble_rotation_interval")
    UserDefaults.standard.removeObject(forKey: "auto_adjust_enabled")
    // Reset published state
    clearPublishedIdentity()
    identityInfo = nil
    identityHydrationState = .absent
    logDiagnostic("reset_all_data complete")
}

func getBuildProvenance() -> String {
    let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "unknown"
    let build = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "unknown"
    return "SCMessenger iOS \(version) (build \(build))"
}
```

**Acceptance Criteria:**
- [ ] `applyTransportSettings` toggles BLE and Multipeer advertising based on settings
- [ ] `resetAllData` clears storage, UserDefaults, and resets identity state
- [ ] `getBuildProvenance` returns version string

---

### A7 [HAIKU] Add Diagnostics Utility Methods
**Tier:** [HAIKU]  
**Estimated LoC:** ~80 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:5707-5812`  
**Description:** Async diagnostics and log access methods.

**Exact methods to add:**
```swift
func getDiagnosticsSnapshot(limit: Int = 500) -> String {
    diagnosticsIOQueue.sync {
        let lines = diagnosticsBuffer.suffix(limit)
        return lines.joined(separator: "\n")
    }
}

func diagnosticsLogPath() -> String {
    diagnosticsLogURL.path
}

func clearDiagnostics() {
    diagnosticsIOQueue.sync {
        diagnosticsBuffer.removeAll()
    }
    try? FileManager.default.removeItem(at: diagnosticsLogURL)
}

func appendDiagnostic(_ line: String) {
    let timestamped = "[\(ISO8601DateFormatter().string(from: Date()))] \(line)"
    diagnosticsIOQueue.async { [weak self] in
        guard let self else { return }
        self.diagnosticsBuffer.append(timestamped)
        if self.diagnosticsBuffer.count > self.diagnosticsMaxLines {
            self.diagnosticsBuffer.removeFirst(self.diagnosticsBuffer.count - self.diagnosticsMaxLines)
        }
        // Periodically flush to disk
        let text = self.diagnosticsBuffer.joined(separator: "\n")
        try? text.write(to: self.diagnosticsLogURL, atomically: true, encoding: .utf8)
    }
}

func exportDiagnosticsAsync() async -> String {
    await Task.detached {
        self.exportDiagnostics()
    }.value
}

func getMissingRuntimePermissions() -> [String] {
    // iOS permissions are handled at app level; return common ones for diagnostics
    var missing: [String] = []
    // Bluetooth
    if #available(iOS 13.1, *) {
        // Check CBManager authorization if needed
    }
    return missing
}
```

**Acceptance Criteria:**
- [ ] `getDiagnosticsSnapshot` returns last N lines from in-memory buffer
- [ ] `appendDiagnostic` timestamps and buffers lines
- [ ] `clearDiagnostics` empties buffer and deletes log file

---

### A8 [HAIKU] Add Transport Health Methods
**Tier:** [HAIKU]  
**Estimated LoC:** ~60 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (transport health section)  
**Description:** Basic transport health queries for diagnostics screen.

**Exact methods to add:**
```swift
func getTransportHealthSummary() -> [String: String] {
    var summary: [String: String] = [:]
    summary["BLE"] = (bleCentralManager != nil && blePeripheralManager != nil) ? "active" : "inactive"
    summary["Multipeer"] = (multipeerTransport != nil) ? "active" : "inactive"
    summary["Swarm"] = (swarmBridge != nil) ? "active" : "inactive"
    summary["mDNS"] = (mdnsDiscovery != nil) ? "active" : "inactive"
    return summary
}

func getActiveTransports() -> [String] {
    var active: [String] = []
    if bleCentralManager != nil { active.append("BLE") }
    if multipeerTransport != nil { active.append("Multipeer") }
    if swarmBridge != nil { active.append("Swarm") }
    if mdnsDiscovery != nil { active.append("mDNS") }
    return active
}

func shouldUseTransport(_ transport: String) -> Bool {
    switch transport.lowercased() {
    case "ble": return bleCentralManager != nil
    case "multipeer": return multipeerTransport != nil
    case "swarm", "internet", "tcp": return swarmBridge != nil
    case "mdns": return mdnsDiscovery != nil
    default: return false
    }
}

func testLedgerRelayConnectivity() -> Bool {
    guard let ledger = ledgerManager else { return false }
    let relays = (try? ledger.getPreferredRelays(limit: 5)) ?? []
    return !relays.isEmpty
}
```

**Acceptance Criteria:**
- [ ] `getTransportHealthSummary` returns dictionary with transport statuses
- [ ] `getActiveTransports` returns array of active transport names
- [ ] `testLedgerRelayConnectivity` returns true if ledger has relay entries

---

### A9 [HAIKU] Add BLE Failure Handling Methods
**Tier:** [HAIKU]  
**Estimated LoC:** ~40 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (BLE failure methods)  
**Description:** Graceful degradation for BLE transport failures.

**Exact methods to add:**
```swift
func handleBleFailure() {
    logger.warning("BLE transport failure detected; degrading to other transports")
    bleCentralManager?.stopScanning()
    blePeripheralManager?.stopAdvertising()
    // Prioritize swarm and multipeer
    appendDiagnostic("ble_degraded")
}

func attemptBleRecovery() {
    logger.info("Attempting BLE recovery")
    bleCentralManager?.startScanning()
    blePeripheralManager?.startAdvertising()
    appendDiagnostic("ble_recovery_attempted")
}

func forceRestartScanning() {
    bleCentralManager?.stopScanning()
    blePeripheralManager?.stopAdvertising()
    DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
        self?.bleCentralManager?.startScanning()
        self?.blePeripheralManager?.startAdvertising()
    }
    appendDiagnostic("ble_force_restart")
}

func clearPeerCache() {
    // Clear any in-memory peer caches if they exist
    discoveredPeerMap.removeAll()
    appendDiagnostic("peer_cache_cleared")
}
```

**Acceptance Criteria:**
- [ ] `handleBleFailure` stops BLE scanning/advertising
- [ ] `attemptBleRecovery` restarts BLE
- [ ] `forceRestartScanning` stops then restarts after 1s delay

---

### A10 [HAIKU] Add Bootstrap Retry Methods
**Tier:** [HAIKU]  
**Estimated LoC:** ~50 added to MeshRepository.swift  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:9014`  
**Description:** Bootstrap retry with fallback strategy.

**Exact methods to add:**
```swift
func bootstrapWithFallbackStrategy() async {
    logger.info("Bootstrap retry with fallback strategy")
    appendDiagnostic("bootstrap_retry_start")

    // Try ledger relays first
    if let ledger = ledgerManager {
        let relays = (try? ledger.getPreferredRelays(limit: 10)) ?? []
        for relay in relays {
            do {
                try await swarmBridge?.dial(multiaddr: relay.multiaddr)
                logger.info("Bootstrap fallback succeeded: \(relay.multiaddr)")
                appendDiagnostic("bootstrap_fallback_success addr=\(relay.multiaddr)")
                return
            } catch {
                logger.warning("Bootstrap fallback failed for \(relay.multiaddr): \(error.localizedDescription)")
            }
        }
    }

    // Try static bootstrap nodes
    for addr in Self.defaultBootstrapNodes {
        do {
            try await swarmBridge?.dial(multiaddr: addr)
            logger.info("Bootstrap static succeeded: \(addr)")
            appendDiagnostic("bootstrap_static_success addr=\(addr)")
            return
        } catch {
            logger.warning("Bootstrap static failed for \(addr): \(error.localizedDescription)")
        }
    }

    appendDiagnostic("bootstrap_retry_exhausted")
    logger.error("Bootstrap retry exhausted all fallback options")
}
```

**Acceptance Criteria:**
- [ ] `bootstrapWithFallbackStrategy` tries ledger relays then static nodes
- [ ] Logs each attempt via `appendDiagnostic`
- [ ] Returns after first successful dial

---

## 3. Lane B: ViewModels (Parallel, No File Collision)

All tasks create new files in `iOS/SCMessenger/SCMessenger/ViewModels/`. Execute in any order relative to Lane A.

### B1 [SONNET] Create DashboardViewModel
**Tier:** [SONNET]  
**Estimated LoC:** ~300 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/DashboardViewModel.kt` (407 lines)  
**New File:** `iOS/SCMessenger/SCMessenger/ViewModels/DashboardViewModel.swift`  
**Description:** Port Android DashboardViewModel to Swift. Manages service stats, peer list, topology data.

**Exact struct and class to create:**
```swift
import Foundation
import Combine

struct DashboardPeer: Identifiable, Equatable {
    let id: String
    var peerId: String
    var publicKey: String?
    var nickname: String?
    var localNickname: String?
    var libp2pPeerId: String?
    var blePeerId: String?
    var transport: String
    var isOnline: Bool
    var isRelay: Bool
    var isFull: Bool
    var lastSeen: Date

    var displayName: String {
        let local = localNickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !local.isEmpty { return local }
        let fed = nickname?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        if !fed.isEmpty { return fed }
        return isFull ? "Node" : "Headless Node"
    }
}

struct NetworkTopology {
    var nodes: [TopologyNode] = []
    var edges: [TopologyEdge] = []
}

struct TopologyNode: Identifiable {
    let id: String
    var isSelf: Bool
    var isOnline: Bool
}

struct TopologyEdge: Identifiable {
    let id = UUID().uuidString
    var source: String
    var target: String
    var transport: String
}

@MainActor
@Observable
final class DashboardViewModel {
    private weak var repository: MeshRepository?
    private var cancellables = Set<AnyCancellable>()

    var stats: ServiceStats?
    var peers: [DashboardPeer] = []
    var topology: NetworkTopology = NetworkTopology()
    var isLoading = false
    var error: String?

    init(repository: MeshRepository) {
        self.repository = repository
        observeEvents()
    }

    func refreshData() {
        isLoading = true
        Task { @MainActor in
            do {
                try? await repository?.updateStats()
                self.stats = await repository?.serviceStats
                await loadPeers()
                buildTopology()
                self.error = nil
            }
            self.isLoading = false
        }
    }

    private func loadPeers() async {
        // Port Android DashboardViewModel.loadPeers() logic
        // ... (reference Android implementation)
    }

    private func buildTopology() {
        // Port Android DashboardViewModel.buildTopology() logic
        // ... (reference Android implementation)
    }

    private func observeEvents() {
        repository?.statusEvents
            .receive(on: DispatchQueue.main)
            .sink { [weak self] event in
                if case .statsUpdated(let s) = event {
                    self?.stats = s
                }
            }
            .store(in: &cancellables)
    }
}
```

**Acceptance Criteria:**
- [ ] File compiles without errors
- [ ] `refreshData()` loads stats and peers from repository
- [ ] Observes `StatusEvent.statsUpdated` for live updates
- [ ] Peer deduplication logic matches Android

---

### B2 [SONNET] Create IdentityViewModel
**Tier:** [SONNET]  
**Estimated LoC:** ~250 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt`  
**New File:** `iOS/SCMessenger/SCMessenger/ViewModels/IdentityViewModel.swift`  
**Description:** Port Android IdentityViewModel. Manages identity creation flow, progress stages, and export.

**Acceptance Criteria:**
- [ ] File compiles
- [ ] Handles identity creation with nickname validation
- [ ] Exposes identity info StateFlow-equivalent (published property)
- [ ] Handles identity export/import backup

---

### B3 [SONNET] Create RequestsInboxViewModel
**Tier:** [SONNET]  
**Estimated LoC:** ~150 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/RequestsInboxViewModel.kt` (141 lines)  
**New File:** `iOS/SCMessenger/SCMessenger/ViewModels/RequestsInboxViewModel.swift`  
**Description:** Port Android RequestsInboxViewModel to Swift.

**Exact class to create:**
```swift
import Foundation
import Combine

@MainActor
@Observable
final class RequestsInboxViewModel {
    private weak var repository: MeshRepository?

    var requests: [MessageRequestThread] = []
    var isLoading = false
    var error: String?

    init(repository: MeshRepository) {
        self.repository = repository
        loadRequests()
    }

    func loadRequests() {
        isLoading = true
        Task { @MainActor in
            self.requests = self.repository?.getPendingMessageRequests() ?? []
            self.isLoading = false
        }
    }

    func acceptRequest(peerId: String) {
        Task { @MainActor in
            do {
                try self.repository?.acceptMessageRequest(peerId: peerId)
                self.loadRequests()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }

    func rejectRequest(peerId: String) {
        Task { @MainActor in
            do {
                try self.repository?.rejectMessageRequest(peerId: peerId)
                self.loadRequests()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }

    func blockAndDelete(peerId: String) {
        Task { @MainActor in
            do {
                try self.repository?.blockAndDeletePeer(peerId: peerId)
                self.loadRequests()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }
}
```

**Acceptance Criteria:**
- [ ] File compiles
- [ ] `loadRequests()` calls `repository.getPendingMessageRequests()`
- [ ] `acceptRequest` calls `repository.acceptMessageRequest(peerId:)`
- [ ] `rejectRequest` calls `repository.rejectMessageRequest(peerId:)`
- [ ] `blockAndDelete` calls `repository.blockAndDeletePeer(peerId:)`

---

### B4 [SONNET] Create MainViewModel
**Tier:** [SONNET]  
**Estimated LoC:** ~200 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/MainViewModel.kt`  
**New File:** `iOS/SCMessenger/SCMessenger/ViewModels/MainViewModel.swift`  
**Description:** Port Android MainViewModel. Central coordinator for app-level state.

**Acceptance Criteria:**
- [ ] File compiles
- [ ] Observes repository serviceState and identityInfo
- [ ] Publishes navigation-relevant state (hasIdentity, serviceRunning)

---

### B5 [SONNET] Enhance ChatViewModel with Delivery State
**Tier:** [SONNET]  
**Estimated LoC:** ~120 modified/added to existing file  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` (455 lines)  
**File:** `iOS/SCMessenger/SCMessenger/ViewModels/ChatViewModel.swift`  
**Description:** Add delivery state tracking, pagination, contact loading, and retry display.

**Changes to make:**
1. Add `pendingOutboxCount` property
2. Add `isOnline` property (observes peer events)
3. Add `loadContact()` method
4. Add `getRetryDelayForAttempt(attemptCount:)` method
5. Add `shouldRetryMessage(messageId:)` method
6. Add `incrementAttemptCount(messageId:)` method
7. Add `logMessageDeliveryAttempt(messageId:attempt:outcome:)` method
8. Add `loadMoreMessages()` method with pagination
9. Enhance `sendMessage()` to use repository delivery tracking
10. Add `formatTimestamp(timestamp:)` utility

**Acceptance Criteria:**
- [ ] ChatViewModel compiles with new properties
- [ ] `loadMoreMessages()` increments conversation limit and reloads
- [ ] `sendMessage()` logs delivery attempts via repository
- [ ] Observes peer connect/disconnect for `isOnline`

---

## 4. Lane C: Screens (Parallel with B, depends on A+B)

### C1 [SONNET] Create BlockedPeersView
**Tier:** [SONNET]  
**Estimated LoC:** ~200 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/screens/BlockedPeersScreen.kt` (158 lines)  
**New File:** `iOS/SCMessenger/SCMessenger/Views/Settings/BlockedPeersView.swift`  
**Description:** Port Android BlockedPeersScreen to SwiftUI.

**Requirements:**
- List blocked peers with peerId, blocked date, reason
- Empty state with Block icon and message
- Swipe-to-unblock or button per row
- Confirmation dialog before unblock
- Navigation title "Blocked Peers"

**Acceptance Criteria:**
- [ ] View compiles
- [ ] Lists blocked peers from `repository.listBlockedPeers()`
- [ ] Shows unblock confirmation dialog
- [ ] Calls `repository.unblockPeer(peerId:)` on confirm
- [ ] Empty state shown when no blocked peers

---

### C2 [SONNET] Create ContactDetailView
**Tier:** [SONNET]  
**Estimated LoC:** ~350 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/contacts/ContactDetailScreen.kt` (375 lines)  
**New File:** `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactDetailView.swift`  
**Description:** Port Android ContactDetailScreen to SwiftUI.

**Requirements:**
- Display contact identity card with avatar/initials
- Show nickname, federated nickname, online status, verification status
- Send message button
- Copyable peer ID and public key
- Metadata: added date, last seen, notes
- Edit nickname dialog
- Delete confirmation dialog
- Verify safety number button

**Acceptance Criteria:**
- [ ] View compiles
- [ ] Displays contact info from repository
- [ ] Edit nickname persists via `repository.setLocalNickname`
- [ ] Delete removes contact and navigates back
- [ ] Send message navigates to chat

---

### C3 [SONNET] Enhance ChatView with Block Button and Delivery States
**Tier:** [SONNET]  
**Estimated LoC:** ~150 modified in existing file  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt` (489 lines)  
**File:** `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift` (`ChatView` struct, lines 424-508)  
**Description:** Add block/unblock toolbar button, blocked input banner, and delivery state indicators to message bubbles.

**Changes to make:**
1. Add toolbar with block/unblock button in `ChatView`
2. Add blocked banner when peer is blocked (replaces input bar)
3. Add delivery state indicator to `MessageBubble` (small text below timestamp)
4. Add "Add Contact" banner when peer is not in contacts but available

**Acceptance Criteria:**
- [ ] Block button shows in navigation bar
- [ ] Tapping block shows confirmation dialog
- [ ] Blocked peer shows red banner "Peer blocked. Unblock to send messages."
- [ ] Message bubbles show delivery state label (pending/stored/forwarding/delivered)
- [ ] Non-contact peer shows "Add to Contacts" banner

---

### C4 [SONNET] Add Blocked Peers Navigation Link to SettingsView
**Tier:** [SONNET]  
**Estimated LoC:** ~30 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`  
**Description:** Add navigation link to BlockedPeersView from SettingsView.

**Changes:**
- In SettingsView Advanced section, add:
```swift
NavigationLink("Blocked Peers") {
    BlockedPeersView()
}
```
- Show blocked count badge if count > 0

**Acceptance Criteria:**
- [ ] Navigation link appears in Advanced section
- [ ] Shows blocked peer count if any exist

---

### C5 [SONNET] Add ContactDetail Navigation to ContactsListView
**Tier:** [SONNET]  
**Estimated LoC:** ~40 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift`  
**Description:** Add navigation to ContactDetailView when tapping a contact row.

**Changes:**
- Change contact row from NavigationLink to conversation to a button that shows context menu + detail sheet
- Or add detail navigation alongside chat navigation
- Add tap gesture or button to open ContactDetailView as a sheet

**Acceptance Criteria:**
- [ ] Tapping a contact row (not the NavigationLink chevron) opens ContactDetailView
- [ ] ContactDetailView shows correct contact info

---

## 5. Lane D: Delivery State System (Depends on A4 + B5)

### D1 [HAIKU] Add DeliveryStatePresentation to MessageBubble
**Tier:** [HAIKU]  
**Estimated LoC:** ~40 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift` (`MessageBubble` struct, lines 513-551)  
**Description:** Show delivery state label below timestamp in message bubbles.

**Changes:**
```swift
// In MessageBubble body, after timestamp Text:
if let viewModel = viewModel, message.direction == .sent {
    let state = viewModel.resolveDeliveryState(for: message)
    Text(state.label)
        .font(Theme.labelSmall)
        .foregroundStyle(deliveryStateColor(state.state))
}

private func deliveryStateColor(_ state: DeliveryStateSurface) -> Color {
    switch state {
    case .delivered: return .green
    case .pending: return .gray
    case .stored: return .orange
    case .forwarding: return .blue
    case .rejected: return .red
    }
}
```

**Note:** `MessageBubble` will need access to viewModel or repository. Pass it as a parameter.

**Acceptance Criteria:**
- [ ] Sent messages show delivery state label
- [ ] Color coding matches state (green=delivered, orange=stored, blue=forwarding, red=rejected, gray=pending)

---

### D2 [HAIKU] Add Delivery State to ConversationListView Rows
**Tier:** [HAIKU]  
**Estimated LoC:** ~30 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift` (`ConversationRow` struct, lines 367-421)  
**Description:** Show delivery state indicator on conversation list rows.

**Changes:**
- Add small status indicator dot next to last message preview
- Use delivery state from most recent message

**Acceptance Criteria:**
- [ ] Conversation rows show delivery state of last sent message
- [ ] Visual indicator is subtle (small dot or icon)

---

## 6. Lane E: Diagnostics Enhancement (Parallel)

### E1 [SONNET] Enhance DiagnosticsView with Network Diagnostics
**Tier:** [SONNET]  
**Estimated LoC:** ~250 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Settings/DiagnosticsView.swift` (220 lines currently)  
**Description:** Port Android DiagnosticsScreen's network diagnostics, service health, and performance sections to SwiftUI.

**New sections to add:**
1. **Network Diagnostics Card**: Shows connection path state, NAT status, transport health summary
2. **Service Health Card**: Shows service healthy/unhealthy status
3. **Transport Status Card**: Shows active transports with status indicators
4. **Bootstrap Status**: Shows bootstrap relay connectivity test result
5. **Retry Bootstrap Button**: Calls `repository.bootstrapWithFallbackStrategy()`

**Acceptance Criteria:**
- [ ] Network diagnostics card shows connection path and NAT status
- [ ] Service health card shows running/stopped status
- [ ] Transport status shows BLE, Multipeer, Swarm, mDNS status
- [ ] Retry bootstrap button exists and calls repository method

---

### E2 [SONNET] Create NetworkStatusSheet
**Tier:** [SONNET]  
**Estimated LoC:** ~200 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/dialogs/NetworkStatusDialog.kt`  
**New File:** `iOS/SCMessenger/SCMessenger/Views/Settings/NetworkStatusSheet.swift`  
**Description:** Port Android NetworkStatusDialog to SwiftUI sheet.

**Requirements:**
- Shows network type (WiFi/Cellular/Offline)
- Shows transport priority list
- Shows port probe results
- Shows circuit breaker state summary
- Shows relay reachability status
- Retry bootstrap button

**Acceptance Criteria:**
- [ ] Sheet compiles
- [ ] Shows all network diagnostics
- [ ] Retry bootstrap button works

---

## 7. Lane F: Transport Health (Parallel)

### F1 [HAIKU] Add Transport Status to MeshDashboardView
**Tier:** [HAIKU]  
**Estimated LoC:** ~60 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift`  
**Description:** Enhance transport status section with real health data from repository.

**Changes:**
- Replace hardcoded `isActive: true` with calls to `repository.shouldUseTransport()`
- Add transport health summary card

**Acceptance Criteria:**
- [ ] Transport rows show actual active/inactive status
- [ ] Status updates when service starts/stops

---

### F2 [HAIKU] Add PeerList Subview to Dashboard
**Tier:** [HAIKU]  
**Estimated LoC:** ~100 (new file)  
**Android Reference:** `android/app/src/main/java/com/scmessenger/android/ui/dashboard/PeerListScreen.kt`  
**New File:** `iOS/SCMessenger/SCMessenger/Views/Dashboard/PeerListView.swift`  
**Description:** Create dedicated peer list view as a sheet or subview of MeshDashboardView.

**Acceptance Criteria:**
- [ ] Shows full peer list with details
- [ ] Can be accessed from MeshDashboardView

---

## 8. Lane G: XCTest Coverage (Parallel)

### G1 [HAIKU] Create MeshRepositoryBlockTests
**Tier:** [HAIKU]  
**Estimated LoC:** ~150 (new file)  
**New File:** `iOS/SCMessenger/SCMessengerTests/MeshRepositoryBlockTests.swift`  
**Description:** XCTest coverage for block/unblock/listBlocked methods.

**Tests:**
- `testBlockPeer` - blocks a peer, verifies `isBlocked` returns true
- `testUnblockPeer` - blocks then unblocks, verifies `isBlocked` returns false
- `testListBlockedPeers` - blocks multiple peers, verifies list count
- `testBlockAndDeletePeer` - blocks and deletes, verifies contact removed
- `testGetBlockedCount` - verifies count matches list

**Acceptance Criteria:**
- [ ] All 5 tests compile and pass in simulator

---

### G2 [HAIKU] Create MeshRepositoryDeliveryTests
**Tier:** [HAIKU]  
**Estimated LoC:** ~150 (new file)  
**New File:** `iOS/SCMessenger/SCMessengerTests/MeshRepositoryDeliveryTests.swift`  
**Description:** XCTest coverage for delivery state resolution and pending outbox.

**Tests:**
- `testResolveDeliveryStateDelivered` - delivered=true returns .delivered
- `testResolveDeliveryStatePending` - no pending snapshot returns .pending
- `testResolveDeliveryStateStored` - pending with future retry returns .stored
- `testResolveDeliveryStateForwarding` - pending with past retry returns .forwarding
- `testResolveDeliveryStateRejected` - terminal failure code returns .rejected
- `testGetRetryDelay` - verify exponential backoff values

**Acceptance Criteria:**
- [ ] All 6 tests compile and pass

---

### G3 [HAIKU] Create RequestsInboxViewModelTests
**Tier:** [HAIKU]  
**Estimated LoC:** ~120 (new file)  
**New File:** `iOS/SCMessenger/SCMessengerTests/RequestsInboxViewModelTests.swift`  
**Description:** XCTest coverage for RequestsInboxViewModel.

**Tests:**
- `testLoadRequests` - loads requests from repository
- `testAcceptRequest` - accepts request, reloads list
- `testRejectRequest` - rejects request, reloads list
- `testBlockAndDelete` - blocks and deletes, reloads list

**Acceptance Criteria:**
- [ ] All 4 tests compile and pass

---

## 9. Lane H: Identity & Crypto (Parallel with Lane A)

### H1 [HAIKU] Add Identity ViewModel Integration to Onboarding
**Tier:** [HAIKU]  
**Estimated LoC:** ~50 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Onboarding/OnboardingFlow.swift`  
**Description:** Wire IdentityViewModel into onboarding flow if B2 is done; otherwise use repository directly.

**Acceptance Criteria:**
- [ ] Onboarding uses ViewModel pattern if available
- [ ] Identity creation flow works end-to-end

---

### H2 [HAIKU] Add Safety Number Verification to ContactDetailView
**Tier:** [HAIKU]  
**Estimated LoC:** ~30 modified in existing file  
**File:** `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactDetailView.swift`  
**Description:** Wire existing `VerifySafetyNumberSheet` from ContactDetailView.

**Acceptance Criteria:**
- [ ] ContactDetailView has "Verify Safety Number" button
- [ ] Button opens VerifySafetyNumberSheet as sheet

---

## 10. Acceptance Criteria Summary (Per Milestone)

### Milestone 1: MeshRepository Core Methods (Lane A complete)
- [ ] All A1-A10 methods compile in MeshRepository.swift
- [ ] No compile errors in Xcode project
- [ ] `cargo check --workspace` passes (if Rust changes needed)

### Milestone 2: ViewModels (Lane B complete)
- [ ] All 5 ViewModels compile
- [ ] ViewModels have no retain cycles (weak repository references)
- [ ] Observable pattern works (SwiftUI updates on state changes)

### Milestone 3: Screens (Lane C complete)
- [ ] BlockedPeersView navigable from Settings
- [ ] ContactDetailView accessible from ContactsListView
- [ ] ChatView shows block button and blocked banner
- [ ] Message bubbles show delivery state labels

### Milestone 4: Diagnostics (Lane E complete)
- [ ] DiagnosticsView shows network diagnostics card
- [ ] DiagnosticsView shows transport health
- [ ] Retry bootstrap button works

### Milestone 5: Tests (Lane G complete)
- [ ] All 3 test files compile
- [ ] 15+ unit tests pass in simulator
- [ ] `xcodebuild test -scheme SCMessenger` passes

### Milestone 6: Integration Gate
- [ ] iOS simulator build succeeds with zero warnings
- [ ] App launches without crash
- [ ] Identity creation works
- [ ] Contact add/delete works
- [ ] Message send/receive works
- [ ] Block/unblock peer works
- [ ] Message request accept/reject works
- [ ] Delivery states display correctly
- [ ] Diagnostics screen shows all sections

---

## 11. File Collision & Lane Rules

- **Hotspot:** `MeshRepository.swift` -- single writer (Lane A serial). No other lane may edit this file while Lane A is in progress.
- **Parallel-safe lanes:** B, C, E, F, G, H can all run in parallel with each other.
- **C depends on B:** Screen tasks need ViewModels to compile. Dispatch C after corresponding B task completes.
- **D depends on A4 + B5:** Delivery state UI needs both MeshRepository methods and ChatViewModel enhancements.

---

## 12. Model Dispatch Sizing Guide

For **Gemini 3.6 Flash** and similar models:

| Task Size | LoC | Model Tier | Examples |
|:----------|:----|:-----------|:---------|
| Small | 30-80 | Flash | A1, A2, A5, A8, A9, A10, D1, D2, F1, H1, H2 |
| Medium | 80-150 | Flash | A3, A4, A6, A7, B3, C1, C4, C5, G1, G2, G3 |
| Large | 150-400 | Flash (with context) | B1, B2, B4, B5, C2, C3, E1, E2, F2 |

**Dispatch rule:** If a task exceeds 400 LoC, split it into two tasks before dispatching.

---

## 13. Reference Android Files (Canonical Implementations)

| Feature | Android File | Lines |
|:--------|:-------------|:------|
| Block methods | `data/MeshRepository.kt` | 4121-4190 |
| Message requests | `data/MeshRepository.kt` | 4205-4249 |
| Delivery state mapper | `ui/chat/DeliveryStateSurface.kt` | 1-64 |
| ChatViewModel | `ui/viewmodels/ChatViewModel.kt` | 1-455 |
| ContactsViewModel | `ui/viewmodels/ContactsViewModel.kt` | 1-866 |
| SettingsViewModel | `ui/viewmodels/SettingsViewModel.kt` | 1-1190 |
| DashboardViewModel | `ui/viewmodels/DashboardViewModel.kt` | 1-407 |
| RequestsInboxViewModel | `ui/viewmodels/RequestsInboxViewModel.kt` | 1-141 |
| BlockedPeersScreen | `ui/screens/BlockedPeersScreen.kt` | 1-158 |
| ContactDetailScreen | `ui/contacts/ContactDetailScreen.kt` | 1-375 |
| ChatScreen | `ui/screens/ChatScreen.kt` | 1-489 |
| DiagnosticsScreen | `ui/screens/DiagnosticsScreen.kt` | 1-451 |
| NetworkStatusDialog | `ui/dialogs/NetworkStatusDialog.kt` | -- |
| ReceiptUnificationTest | `test/ReceiptUnificationTest.kt` | -- |

---

*End of Implementation Plan. Execute lanes in dependency order. Update this document as tasks complete.*
