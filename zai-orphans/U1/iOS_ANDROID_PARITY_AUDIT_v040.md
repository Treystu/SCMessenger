# iOS/Android Parity Audit & v0.4.0 Plan

**Status:** Active  
**Last Updated:** 2026-07-27  
**Audited By:** Claude (Kimi Work session)  
**Scope:** Full codebase audit of iOS vs Android feature parity, with prioritized task list for v0.4.0 release.

---

## 1. Executive Summary

This document is the result of a line-by-line audit of the iOS and Android codebases for SCMessenger. The audit compared:

- **39 Swift files** (iOS) vs **122+ Kotlin files** (Android)
- **~7,057 lines** in `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift` vs **~10,002 lines** in `android/.../data/MeshRepository.kt`
- All screens, ViewModels, transport layers, and test coverage

**Bottom Line:** iOS has functional core parity for basic messaging, contacts, settings, and mesh dashboard. However, it lacks significant features present on Android including: blocked peers management, message request accept/reject, delivery state indicators, comprehensive diagnostics, dedicated identity screens, contact detail screen, and extensive test coverage. The iOS MeshRepository is also missing many utility methods exposed on Android.

**v0.4.0 Goal:** Bring iOS to functional parity with Android for the Josh alpha release. This means all core user-facing features must work identically across both platforms.

---

## 2. Screen / UI Parity Matrix

| Feature | Android Screen/File | iOS Screen/File | Status | Gap Notes |
|:--------|:--------------------|:----------------|:------:|:----------|
| **Main App Shell** | `MeshApp.kt`, `MainActivity.kt` | `SCMessengerApp.swift`, `MainTabView.swift` | [OK] | Both use tab-based navigation (Messages/Contacts/Mesh/Settings) |
| **Conversation List** | `ConversationsScreen.kt` | `MainTabView.swift` (`ConversationListView`) | [WARNING] | iOS lacks delivery state badges, unread counts, conversation stats (Total/Sent/Received/Delivered) |
| **Chat / Messaging** | `ChatScreen.kt` | `MainTabView.swift` (`ChatView`) | [WARNING] | iOS is "Zero-Status Architecture" -- no delivery state indicators. Android shows block/unblock, add-contact banner, delivery state surface |
| **Message Bubble** | `MessageBubble.kt` | `MainTabView.swift` (`MessageBubble`) | [FAIL] | iOS shows only content + timestamp. Android shows delivery status, read receipts, failed state |
| **Message Input** | `MessageInput.kt` | `MainTabView.swift` (`MessageInputBar`) | [OK] | Both have text input + send button |
| **Contacts List** | `ContactsScreen.kt` | `ContactsListView.swift` | [OK] | Both have search, nearby peers, add contact, swipe-to-delete, edit nickname, verify safety number |
| **Contact Detail** | `ContactDetailScreen.kt` | **MISSING** | [FAIL] | iOS has no dedicated contact detail screen; only context menu in list |
| **Add Contact** | `AddContactScreen.kt` | `ContactsListView.swift` (`AddContactView`) | [OK] | Both support manual entry, QR scan, paste identity export |
| **Blocked Peers** | `BlockedPeersScreen.kt` | **MISSING** | [FAIL] | No blocked peers management on iOS |
| **Message Requests** | `RequestsInboxScreen.kt` | `MainTabView.swift` (`RequestsInboxView`) | [WARNING] | iOS UI exists but `acceptMessageRequest` / `rejectMessageRequest` methods are **missing** from `MeshRepository` |
| **Mesh Dashboard** | `DashboardScreen.kt` | `MeshDashboardView.swift` | [WARNING] | iOS has basic dashboard. Android has separate `PeerListScreen` + `TopologyScreen` with graph visualization |
| **Settings** | `SettingsScreen.kt` | `SettingsView.swift` | [OK] | Both have service control, relay toggle, identity, app preferences, danger zone |
| **Mesh Settings** | `MeshSettingsScreen.kt` | `SettingsView.swift` (`MeshSettingsView`) | [OK] | Both have transport toggles, discovery mode, battery floor |
| **Privacy Settings** | (part of Settings) | `SettingsView.swift` (`PrivacySettingsView`) | [OK] | Both have onion routing, BLE rotation, cover traffic, padding, timing obfuscation |
| **Power Settings** | `PowerSettingsScreen.kt` | `SettingsView.swift` (`PowerSettingsView`) | [OK] | Both have AutoAdjust engine toggle |
| **Diagnostics** | `DiagnosticsScreen.kt` | `DiagnosticsView.swift` | [FAIL] | Android has network diagnostics, performance monitor, ANR stats, service health, notification stats. iOS has only log viewer + basic export |
| **Identity Screen** | `IdentityScreen.kt`, `IdentityCreationFlow.kt` | `OnboardingFlow.swift`, `SettingsView.swift` | [WARNING] | iOS handles identity in onboarding + settings. Android has dedicated flow with progress stages |
| **Join Mesh** | `JoinMeshScreen.kt` | `JoinMeshView.swift` | [OK] | Both exist |
| **Onboarding** | `OnboardingScreen.kt` | `OnboardingFlow.swift` | [WARNING] | iOS has 4-step flow. Android has more elaborate onboarding with install mode choice |
| **Network Status Dialog** | `NetworkStatusDialog.kt` | **MISSING** | [FAIL] | Android has network diagnostics dialog with retry bootstrap |
| **Reusable Components** | 8+ components (`ErrorBanner`, `Identicon`, `QrCode`, `StatusIndicator`, etc.) | **MISSING** | [FAIL] | iOS has no reusable component library; all UI is inline |

---

## 3. MeshRepository / Data Layer Parity Gaps

The following methods exist in **Android's `MeshRepository.kt`** but are **missing or incomplete** in **iOS's `MeshRepository.swift`**.

### 3.1 Peer Blocking & Moderation [CRITICAL]

| Method | Android | iOS | Priority |
|:-------|:-------:|:---:|:--------:|
| `blockPeer(peerId, deviceId, reason)` | [OK] | **MISSING** | P0 |
| `unblockPeer(peerId, deviceId)` | [OK] | **MISSING** | P0 |
| `mutePeer(peerId, deviceId, reason)` | [OK] | **MISSING** | P1 |
| `blockAndDeletePeer(peerId, deviceId, reason)` | [OK] | **MISSING** | P0 |
| `isBlocked(peerId, deviceId)` | [OK] | **MISSING** | P0 |
| `listBlockedPeers()` | [OK] | **MISSING** | P0 |
| `getBlockedCount()` | [OK] | **MISSING** | P0 |

**Evidence:** Android `MeshRepository.kt:4121-4190`. iOS grep for `blockPeer` returns no matches in `MeshRepository.swift`.

### 3.2 Message Requests [CRITICAL]

| Method | Android | iOS | Priority |
|:-------|:-------:|:---:|:--------:|
| `getPendingMessageRequests()` | [OK] | `getMessageRequests()` exists | P0 |
| `acceptMessageRequest(peerId)` / `addContactByPeerId(peerId)` | [OK] | **MISSING** | P0 |
| `rejectMessageRequest(peerId)` (blocks sender) | [OK] | **MISSING** | P0 |

**Evidence:** iOS `MainTabView.swift:341` calls `repository.acceptMessageRequest(peerId:)` but this method does **not exist** in iOS `MeshRepository.swift`. This is a compile-time gap if the call site is exercised.

### 3.3 Identity & Crypto Utilities [P1]

| Method | Android | iOS | Priority |
|:-------|:-------:|:---:|:--------:|
| `signData(data)` | [OK] | **MISSING** | P1 |
| `verifySignature(data, signature, publicKeyHex)` | [OK] | **MISSING** | P1 |
| `getDeviceId()` | [OK] | **MISSING** | P1 |
| `getSeniorityTimestamp()` | [OK] | **MISSING** | P1 |
| `getRegistrationState(identityId)` | [OK] | **MISSING** | P1 |
| `exportLogs()` | [OK] | **MISSING** | P1 |
| `updateContactDeviceId(peerId, deviceId)` | [OK] | **MISSING** | P1 |
| `getIdentityInfoNonBlocking()` | [OK] | **MISSING** | P1 |
| `getIdentityInfoSync()` | [OK] | `getIdentityInfo()` only (no sync variant) | P2 |
| `syncNicknameFromDatastore()` | [OK] | **MISSING** | P1 |

### 3.4 Message Delivery & Retry Tracking [P0-P1]

| Method | Android | iOS | Priority |
|:-------|:-------:|:---:|:--------:|
| `getPendingOutboxAsync()` / `loadPendingOutboxAsync()` | [OK] | **MISSING** | P0 |
| `incrementAttemptCount(messageId)` | [OK] | **MISSING** | P1 |
| `shouldRetryMessage(messageId)` | [OK] | **MISSING** | P1 |
| `getRetryDelay(attemptCount)` | [OK] | **MISSING** | P1 |
| `logMessageDeliveryAttempt(messageId, attempt, outcome)` | [OK] | **MISSING** | P1 |
| `markMessageCorrupted(messageId)` | [OK] | **MISSING** | P1 |
| `resolveDeliveryState(message, nowEpochSec)` (in ViewModel) | [OK] | **MISSING** | P0 |
| `getPendingDeliverySnapshot(messageId)` | [OK] | **MISSING** | P0 |
| `getPendingTerminalFailureCode(messageId)` | [OK] | **MISSING** | P0 |

**Note:** Android `ChatViewModel` and `ConversationsViewModel` have rich delivery state tracking. iOS `ChatViewModel` has `statusGlyph(for:)` but no actual delivery state resolution logic.

### 3.5 Transport Health & Diagnostics [P1]

| Method | Android | iOS | Priority |
|:-------|:-------:|:---:|:--------:|
| `getTransportHealthSummary()` | [OK] | **MISSING** | P1 |
| `getNetworkDiagnosticsSnapshot()` | [OK] | **MISSING** | P1 |
| `getNetworkFailureSummary()` | [OK] | **MISSING** | P1 |
| `getActiveTransports()` | [OK] | **MISSING** | P1 |
| `shouldUseTransport(transport)` | [OK] | **MISSING** | P1 |
| `handleBleFailure()` | [OK] | **MISSING** | P2 |
| `attemptBleRecovery()` | [OK] | **MISSING** | P2 |
| `forceRestartScanning()` | [OK] | **MISSING** | P2 |
| `clearPeerCache()` | [OK] | **MISSING** | P2 |
| `testLedgerRelayConnectivity()` | [OK] | **MISSING** | P1 |
| `retryBootstrap()` / `bootstrapWithFallbackStrategy()` | [OK] | **MISSING** | P1 |
| `racingBootstrapWithFallback()` | [OK] | **MISSING** | P2 |

### 3.6 Settings & Preferences [P1]

| Method | Android | iOS | Priority |
|:-------|:-------:|:---:|:--------:|
| `applyTransportSettings(settings)` | [OK] | **MISSING** | P1 |
| `getDefaultSettings()` | [OK] | `settingsManager?.defaultSettings()` exists | OK |
| `resetAllData()` | [OK] | `resetAllData()` exists | OK |
| `getBuildProvenance()` | [OK] | **MISSING** | P2 |
| `getMissingRuntimePermissions()` | [OK] | **MISSING** | P2 |

### 3.7 Outbox & Message Sending [P0]

| Method | Android | iOS | Priority |
|:-------|:-------:|:---:|:--------:|
| `sendMessage(peerId, content)` (suspend) | [OK] | `sendMessage(peerId, content) async throws` | OK |
| `dial(multiaddr)` (suspend) | [OK] | `connectToPeer(peerId, addresses)` exists | OK |
| `flushPendingOutbox(reason)` | [OK] | `flushPendingOutbox(reason)` exists | OK |
| `startPendingOutboxRetryLoop()` | [OK] | `startPendingOutboxRetryLoop()` exists | OK |
| `loadPendingOutbox()` / `loadPendingOutboxAsync()` | [OK] | **MISSING** | P0 |
| `getPendingOutboxCount()` | [OK] | **MISSING** | P0 |

### 3.8 Service Lifecycle [OK]

Both platforms have: `startMeshService`, `stopMeshService`, `pauseMeshService`, `resumeMeshService`, `ensureServiceInitialized`, lazy initialization, identity federation, BLE beacon broadcasting, mDNS discovery, swarm startup.

---

## 4. ViewModel Parity Gaps

### 4.1 ViewModels Present on Android but Missing on iOS

| ViewModel | Android | iOS | Priority |
|:----------|:-------:|:---:|:--------:|
| `MainViewModel` | [OK] | **MISSING** | P1 |
| `IdentityViewModel` | [OK] | **MISSING** | P1 |
| `MeshServiceViewModel` | [OK] | **MISSING** | P2 |
| `DashboardViewModel` | [OK] | **MISSING** | P1 |
| `RequestsInboxViewModel` | [OK] | **MISSING** | P0 |
| `ConversationsViewModel` | [OK] | Embedded in `MainTabView` | WARNING |

### 4.2 Feature-Rich ViewModels vs Simplified iOS Equivalents

**Android `ChatViewModel`** (~455 lines) vs **iOS `ChatViewModel`** (~134 lines):
- Android: pagination (`loadMoreMessages`), contact loading, pending outbox count, retry delay display, typing indicator, online status, message delivery attempt logging, increment attempt count
- iOS: basic message loading, send, error handling, status glyph (no actual state resolution)

**Android `ContactsViewModel`** (~866 lines) vs **iOS `ContactsViewModel`** (~447 lines):
- Android: contact import from JSON, promote nearby peer to contact, refresh discovery with rescan, transport inference, dismiss nearby peer, debounced nickname updates, comprehensive identity matching
- iOS: basic CRUD, nearby peers, safety number computation

**Android `SettingsViewModel`** (~1190 lines) vs **iOS `SettingsViewModel`** (~435 lines):
- Android: build provenance, ledger summary, connection path state, NAT status, diagnostics bundle builder, transport health, network diagnostics, network failure summary, active transports, BLE failure handling/recovery, bootstrap retry, message count, blocked count, inbox count, bootstrap nodes
- iOS: basic settings load/save, nickname, identity backup export/import, service control, counts

---

## 5. Transport Layer Parity

| Transport | Android | iOS | Notes |
|:----------|:-------:|:---:|:------|
| BLE (L2CAP) | `BleL2capManager.kt` | `BLEL2CAPManager.swift` | [OK] Both have L2CAP |
| BLE (GATT) | `BleGattServer.kt`, `BleScanner.kt` | `BLEPeripheralManager.swift`, `BLECentralManager.swift` | [OK] Both have GATT peripheral + central |
| BLE MAC Rotation | [OK] | [OK] | Both support |
| mDNS Discovery | `MdnsServiceDiscovery.kt` | `mDNSServiceDiscovery.swift` | [OK] Both browse + advertise |
| WiFi Aware | `WifiAwareTransport.kt` | **N/A** | Intentional -- iOS uses Multipeer |
| WiFi Direct | `WifiDirectTransport.kt` | **N/A** | Intentional -- iOS uses Multipeer |
| Multipeer Connectivity | **N/A** | `MultipeerTransport.swift` | iOS only -- intentional equivalent |
| TCP/mDNS (libp2p) | [OK] via SwarmBridge | [OK] via SwarmBridge | [OK] Both use Rust SwarmBridge |
| Smart Transport Router | `SmartTransportRouter.kt` | `SmartTransportRouter.swift` | [OK] Both have 500ms fallback |
| Local Transport Fallback | `TransportManager.kt` | `LocalTransportFallback.swift` | [OK] Both have fallback logic |

**Verdict:** Transport layer is at parity for v0.4.0 scope. Platform-specific differences (Multipeer vs WiFi Direct/Aware) are intentional and documented.

---

## 6. Test Coverage Gaps

| Test Category | Android | iOS | Gap |
|:--------------|:-------:|:---:|:----|
| **Unit Tests** | 24 files | **0 files** | iOS has no XCTest unit tests in `SCMessengerTests/` |
| **Integration Tests** | `ReceiptUnificationTest.kt`, `UniffiIntegrationTest.kt`, `MeshRepositoryTest.kt` | **MISSING** | |
| **ViewModel Tests** | `ChatViewModelTest.kt`, `ContactsViewModelTest.kt`, `ConversationsViewModelTest.kt`, `SettingsViewModelTest.kt`, `MeshServiceViewModelTest.kt`, `IdentityViewModelTest.kt` | **MISSING** | |
| **Transport Tests** | `BleScannerTest.kt`, `MdnsServiceDiscoveryTest.kt` | **MISSING** | |
| **UI Tests** | `IdentityCreationFlowTest.kt` (androidTest) | **MISSING** | |
| **Other Tests** | `BootReceiverTest.kt`, `DeliveryStateMapperTest.kt`, `DiagnosticsBundleFormatterTest.kt`, `RoleNavigationPolicyTest.kt`, `BackupPassphraseValidatorTest.kt`, `ReceiptWindowTest.kt`, `IdentityFlowRegressionTest.kt`, `MeshApplicationScheduleTest.kt`, `MeshForegroundServiceTest.kt`, `AndroidPlatformBridgeTest.kt` | **MISSING** | |
| **iOS-Specific Tests** | N/A | `local_transport_fallback_tests.swift` (1 file) | Only 1 test file exists outside Xcode target |
| **iOS Outbox Tests** | N/A | `OutboxRetryPolicyTests.swift` (mentioned in docs) | Exists in `SCMessengerTests` target per `FEATURE_PARITY.md` |

**iOS Test Target Status:** `SCMessengerTests` target exists and is executable (per `FEATURE_PARITY.md`), containing 3 outbox retry-policy parity tests. However, there is **no comprehensive test suite** comparable to Android's 24+ test files.

---

## 7. Prioritized Task List for iOS v0.4.0 Parity

### P0 -- Critical (Blocks Josh Alpha)

These are user-facing features that Android has and iOS lacks. They block parity for the v0.4.0 release.

| # | Task | Files to Modify / Create | Effort | Dependencies |
|:--|:-----|:------------------------|:------:|:-------------|
| P0-1 | **Add Blocked Peers to iOS MeshRepository** | `MeshRepository.swift` | Small | None |
| P0-2 | **Create BlockedPeersScreen / BlockedPeersView** | `Views/Settings/BlockedPeersView.swift` | Medium | P0-1 |
| P0-3 | **Add Message Request Accept/Reject to MeshRepository** | `MeshRepository.swift` | Small | None |
| P0-4 | **Wire Message Request Accept/Reject in RequestsInboxView** | `MainTabView.swift` | Small | P0-3 |
| P0-5 | **Add Block/Unblock to ChatView** | `MainTabView.swift` (`ChatView`) | Medium | P0-1 |
| P0-6 | **Add Delivery State Resolution to iOS** | `MeshRepository.swift`, `ChatViewModel.swift`, `MainTabView.swift` (`MessageBubble`) | Medium | None |
| P0-7 | **Add `loadPendingOutboxAsync` / `getPendingOutboxCount`** | `MeshRepository.swift` | Small | None |
| P0-8 | **Create ContactDetailScreen** | `Views/Contacts/ContactDetailView.swift` | Medium | None |
| P0-9 | **Add `addContactByPeerId` for message request acceptance** | `MeshRepository.swift` | Small | P0-3 |

### P1 -- Important (Significant UX Gaps)

| # | Task | Files to Modify / Create | Effort | Dependencies |
|:--|:-----|:------------------------|:------:|:-------------|
| P1-1 | **Create DashboardViewModel** | `ViewModels/DashboardViewModel.swift` | Medium | None |
| P1-2 | **Add PeerListScreen + TopologyScreen subviews** | `Views/Dashboard/PeerListView.swift`, `TopologyView.swift` | Medium | P1-1 |
| P1-3 | **Add signData / verifySignature** | `MeshRepository.swift` | Small | None |
| P1-4 | **Add getDeviceId / getSeniorityTimestamp / getRegistrationState** | `MeshRepository.swift` | Small | None |
| P1-5 | **Add exportLogs** | `MeshRepository.swift` | Small | None |
| P1-6 | **Add updateContactDeviceId** | `MeshRepository.swift` | Small | None |
| P1-7 | **Add transport health methods** | `MeshRepository.swift` | Medium | None |
| P1-8 | **Add testLedgerRelayConnectivity / retryBootstrap** | `MeshRepository.swift` | Small | None |
| P1-9 | **Add incrementAttemptCount / shouldRetryMessage / getRetryDelay / logMessageDeliveryAttempt** | `MeshRepository.swift` | Small | None |
| P1-10 | **Create IdentityViewModel** | `ViewModels/IdentityViewModel.swift` | Medium | None |
| P1-11 | **Create MainViewModel** | `ViewModels/MainViewModel.swift` | Medium | None |
| P1-12 | **Add getIdentityInfoNonBlocking / syncNicknameFromDatastore** | `MeshRepository.swift` | Small | None |
| P1-13 | **Enhance DiagnosticsView** | `Views/Settings/DiagnosticsView.swift` | Medium | P1-7, P1-8 |
| P1-14 | **Add delivery state mapper / pending delivery snapshot** | `MeshRepository.swift`, `ChatViewModel.swift` | Medium | P0-6 |
| P1-15 | **Add applyTransportSettings** | `MeshRepository.swift` | Small | None |
| P1-16 | **Add getBuildProvenance / getMissingRuntimePermissions** | `MeshRepository.swift` | Small | None |

### P2 -- Polish & Depth

| # | Task | Files to Modify / Create | Effort | Dependencies |
|:--|:-----|:------------------------|:------:|:-------------|
| P2-1 | **Create MeshServiceViewModel** | `ViewModels/MeshServiceViewModel.swift` | Medium | None |
| P2-2 | **Add BLE failure handling / recovery methods** | `MeshRepository.swift` | Small | None |
| P2-3 | **Add NetworkStatusDialog equivalent** | `Views/Settings/NetworkStatusSheet.swift` | Medium | P1-7 |
| P2-4 | **Create reusable component library** | `Views/Components/` | Large | None |
| P2-5 | **Add conversation stats (Total/Sent/Received/Delivered)** | `MainTabView.swift` | Small | P0-6 |
| P2-6 | **Add unread count badges to ConversationListView** | `MainTabView.swift` | Small | None |
| P2-7 | **Expand XCTest coverage** | `SCMessengerTests/` | Large | None |
| P2-8 | **Add `mutePeer` method** | `MeshRepository.swift` | Small | P0-1 |

---

## 8. Implementation Recommendations

### 8.1 Recommended Order of Work

1. **Week 1: P0 Critical Features**
   - P0-1, P0-3, P0-7: Add missing MeshRepository methods (block, message requests, outbox)
   - P0-4, P0-5: Wire block/unblock and message requests in UI
   - P0-6: Add delivery state resolution (highest UX impact)

2. **Week 2: P1 ViewModels & Screens**
   - P1-1, P1-10, P1-11: Create missing ViewModels
   - P1-2, P0-2, P0-8: Create missing screens (BlockedPeers, ContactDetail, PeerList, Topology)
   - P1-3 through P1-6: Add missing identity/crypto/util methods

3. **Week 3: P1 Diagnostics & Transport**
   - P1-7, P1-8, P1-13: Transport health + enhanced diagnostics
   - P1-9, P1-14: Message retry tracking + delivery state mapper
   - P1-15, P1-16: Settings utilities

4. **Week 4: P2 Polish + Testing**
   - P2-3, P2-4, P2-5, P2-6: UI polish
   - P2-7: XCTest expansion (target: at least 10 test files matching Android coverage)

### 8.2 Code Reuse Strategy

Many of the missing MeshRepository methods are thin wrappers around UniFFI APIs. The Android implementations can serve as direct reference:

- **Blocking methods:** `ironCore.blockPeer(peerId)`, `ironCore.unblockPeer(peerId)`, etc.
- **Message requests:** `ironCore.contactsManager().addContactByPeerId(peerId)` for acceptance
- **Delivery state:** Port `DeliveryStateMapper.kt` logic to Swift
- **Diagnostics:** Port `DiagnosticsBundleFormatter.kt` to Swift

### 8.3 Risk Areas

1. **Delivery State Architecture:** iOS currently uses "Zero-Status Architecture" (no delivery indicators). Adding delivery states requires:
   - MeshRepository method to resolve pending delivery snapshots
   - ChatViewModel to observe delivery updates
   - MessageBubble UI to show status glyphs
   This touches multiple layers and has high regression risk.

2. **Message Request Acceptance:** The iOS `RequestsInboxView` already calls `repository.acceptMessageRequest(peerId:)` but the method is missing. This may be a **silent runtime crash** if the code path is exercised.

3. **Block Peer UI:** Adding block/unblock to ChatView requires careful UX design to avoid accidental blocks. Follow Android's pattern (confirmation dialog).

4. **Test Coverage:** iOS has minimal tests. Adding comprehensive XCTests for the new P0/P1 features should be done in parallel with implementation, not deferred.

---

## 9. Verification Checklist

Before declaring iOS v0.4.0 parity complete, verify:

- [ ] Block peer from chat -> peer appears in blocked list -> unblock restores messaging
- [ ] Receive message from unknown sender -> appears in Requests Inbox -> accept adds contact -> reject blocks sender
- [ ] Send message -> see delivery state transition: queued -> sent -> delivered
- [ ] Diagnostics screen shows transport health, network diagnostics, and service health
- [ ] Contact detail screen shows full contact info, safety number verification, block option
- [ ] Mesh dashboard shows peer list with transport types and topology graph
- [ ] Settings screen shows all counts (contacts, messages, blocked, inbox) correctly
- [ ] All P0 MeshRepository methods compile and pass basic smoke tests
- [ ] XCTest suite runs green with >10 test files covering ViewModels and Repository

---

## 10. Appendix: File Inventory

### iOS Files (39 Swift files)
```
iOS/SCMessenger/SCMessenger/
  SCMessengerApp.swift
  ContentView.swift
  ContactManagerFix.swift
  Data/MeshRepository.swift
  Data/TopicManager.swift
  Models/Models.swift
  Background/NotificationBackgroundProcessor.swift
  Generated/api.swift
  Services/CoreDelegateImpl.swift
  Services/IosPlatformBridge.swift
  Services/MeshBackgroundService.swift
  Services/MeshEventBus.swift
  Services/NotificationManager.swift
  Transport/BLECentralManager.swift
  Transport/BLEL2CAPManager.swift
  Transport/BLEPeripheralManager.swift
  Transport/BLEPeripheralManager.swift
  Transport/LocalTransportFallback.swift
  Transport/MeshBLEConstants.swift
  Transport/mDNSServiceDiscovery.swift
  Transport/MultipeerTransport.swift
  Transport/SmartTransportRouter.swift
  Utils/BackupPassphraseValidator.swift
  Utils/NotificationLogger.swift
  Utils/QRCodeGenerator.swift
  Utils/Theme.swift
  ViewModels/ChatViewModel.swift
  ViewModels/ContactsViewModel.swift
  ViewModels/OnboardingViewModel.swift
  ViewModels/SettingsViewModel.swift
  Views/Contacts/ContactsListView.swift
  Views/Contacts/VerifySafetyNumberSheet.swift
  Views/Dashboard/MeshDashboardView.swift
  Views/Navigation/MainTabView.swift
  Views/NotificationGuidanceView.swift
  Views/Onboarding/OnboardingFlow.swift
  Views/Settings/DiagnosticsView.swift
  Views/Settings/IdentityBackupSheets.swift
  Views/Settings/SettingsView.swift
  Views/Topics/JoinMeshView.swift
```

### Android Key Files (122+ Kotlin files)
```
android/app/src/main/java/com/scmessenger/android/
  data/MeshRepository.kt
  di/AppModule.kt
  service/MeshForegroundService.kt
  service/AndroidPlatformBridge.kt
  transport/TransportManager.kt
  transport/ble/*.kt (7 files)
  ui/MainActivity.kt
  ui/MeshApp.kt
  ui/chat/*.kt (3 files)
  ui/components/*.kt (8 files)
  ui/contacts/*.kt (3 files)
  ui/dashboard/*.kt (2 files)
  ui/diagnostics/*.kt (1 file)
  ui/dialogs/*.kt (1 file)
  ui/identity/*.kt (3 files)
  ui/join/*.kt (1 file)
  ui/screens/*.kt (10 files)
  ui/settings/*.kt (2 files)
  ui/theme/*.kt (3 files)
  ui/viewmodels/*.kt (9 files)
```

---

*End of Audit. This document should be treated as the canonical reference for iOS v0.4.0 parity work.*
