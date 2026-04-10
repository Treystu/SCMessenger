Status: Active
Last updated: 2026-04-10

# Feature Parity & Cross-Platform Function Audit

This document tracks the implementation status of all core API functions across
SCMessenger platforms (Core/Rust, CLI, iOS, Android, Web/WASM). It serves as the
canonical parity audit and gap tracker.

Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md,
docs/REPO_CONTEXT.md, and DOCUMENTATION.md.

---

## [Current] Complete Function Parity Matrix (2026-03-27)

Legend: ✅ = Wired & callable | ⚠️ = Partially wired | ❌ = Not wired | N/A = Not applicable

### IronCore Methods

| Function                     | Core | CLI | Android | iOS | WASM | Notes |
|:-----------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `initializeIdentity`        | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getIdentityInfo`           | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `setNickname`               | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getDeviceId`               | ✅   | ✅  | ✅      | ✅  | ✅   | CLI/Android/iOS wired 2026-03-27 |
| `getSeniorityTimestamp`      | ✅   | ✅  | ✅      | ✅  | ✅   | CLI/Android/iOS wired 2026-03-27 |
| `getRegistrationState`       | ✅   | ✅  | ✅      | ✅  | ✅   | CLI/Android/iOS wired 2026-03-27 |
| `exportIdentityBackup`      | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `importIdentityBackup`      | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `signData`                   | ✅   | ✅  | ✅      | ✅  | ✅   | CLI/Android/iOS wired 2026-03-27 |
| `verifySignature`            | ✅   | ✅  | ✅      | ✅  | ✅   | CLI/Android/iOS wired 2026-03-27 |
| `extractPublicKeyFromPeerId` | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `resolveIdentity`           | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `resolveToIdentityId`       | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `prepareMessage`            | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `prepareMessageWithId`      | ✅   | N/A | ✅      | ✅  | ✅   | CLI uses `prepareMessage` (design choice) |
| `prepareReceipt`            | ✅   | N/A | ✅      | ✅  | ✅   | CLI has automatic receipt handling in event loop |
| `prepareCoverTraffic`       | ✅   | N/A | ✅      | ✅  | ✅   | iOS wired via sendCoverTraffic() |
| `receiveMessage`            | ✅   | ✅  | N/A     | N/A | ✅   | Mobile decrypts via delegate callback path |
| `markMessageSent`           | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `outboxCount`               | ✅   | ✅  | ✅      | ✅  | ✅   | iOS wired 2026-03-27 |
| `inboxCount`                | ✅   | ✅  | ✅      | ✅  | ✅   | Android/iOS wired 2026-03-27 |
| `classifyNotification`      | ✅   | N/A | ✅      | ✅  | ✅   | |
| `blockPeer`                 | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `unblockPeer`               | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `blockAndDeletePeer`        | ✅   | ✅  | ✅      | ✅  | ✅   | Wired across all platforms 2026-03-31 |
| `isPeerBlocked`             | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `listBlockedPeers`          | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27; includes `is_deleted` field |
| `blockedCount`              | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `setDelegate`               | ✅   | N/A | ✅      | ✅  | N/A  | WASM uses polling via `drainReceivedMessages` |
| `contactsManager`           | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `historyManager`            | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `updateDiskStats`           | ✅   | N/A | ✅      | ✅  | ✅   | |
| `performMaintenance`        | ✅   | N/A | ✅      | ✅  | ✅   | |
| `recordLog`                 | ✅   | N/A | ✅      | ✅  | ✅   | Android via FileLoggingTree |
| `exportLogs`                | ✅   | N/A | ✅      | ✅  | ✅   | Android/iOS wired 2026-03-27 |
| `notifyPeerDiscovered`      | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `notifyPeerDisconnected`    | ✅   | ✅  | ✅      | ✅  | ✅   | |

### ContactManager Methods

| Function            | Core | CLI | Android | iOS | WASM | Notes |
|:--------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `add`               | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `get`               | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `remove`            | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `list`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `search`            | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `setNickname`       | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `setLocalNickname`  | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `updateLastSeen`    | ✅   | N/A | ✅      | ✅  | ✅   | |
| `updateDeviceId`    | ✅   | N/A | ✅      | ✅  | ✅   | Android/iOS wired 2026-03-27 |
| `count`             | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `flush`             | ✅   | N/A | ✅      | ✅  | ✅   | |

### HistoryManager Methods

| Function             | Core | CLI | Android | iOS | WASM | Notes |
|:---------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `add`                | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `get`                | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `recent`             | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `conversation`       | ✅   | ✅  | ✅      | ✅  | ✅   | CLI via `history --peer` |
| `removeConversation` | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27; iOS wired |
| `search`             | ✅   | ✅  | ✅      | ✅  | ✅   | CLI via `history --search` |
| `markDelivered`      | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `clear`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `clearConversation`  | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `delete`             | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `stats`              | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `count`              | ✅   | ✅  | ✅      | ✅  | ✅   | CLI wired 2026-03-27 |
| `enforceRetention`   | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `pruneBefore`        | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `flush`              | ✅   | N/A | ✅      | ✅  | ✅   | |

### MeshService / SwarmBridge Methods

| Function                | Core | CLI | Android | iOS | WASM | Notes |
|:------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `start` / `startSwarm`  | ✅   | ✅  | ✅      | ✅  | ✅   | WASM uses `startSwarm` |
| `stop`                  | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `pause` / `resume`      | ✅   | N/A | ✅      | ✅  | N/A  | Mobile lifecycle |
| `sendMessage`           | ✅   | ✅  | ✅      | ✅  | ✅   | WASM via `sendPreparedEnvelope` |
| `getPeers`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getListeners`          | ✅   | ✅  | ✅      | ✅  | ✅   | WASM wired 2026-03-27 |
| `getExternalAddresses`  | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `subscribeTopic`        | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-27 |
| `unsubscribeTopic`      | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-27 |
| `publishTopic`          | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-27 |
| `sendToAllPeers`        | ✅   | ✅  | ✅      | ✅  | ✅   | WASM wired 2026-03-27; iOS wired |
| `dial`                  | ✅   | ✅  | ✅      | ✅  | ✅   | WASM wired 2026-03-27 |
| `shutdown`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getConnectionPathState` | ✅  | ✅  | ✅      | ✅  | ✅   | |
| `getNatStatus`          | ✅   | ✅  | ✅      | ✅  | ✅   | WASM returns "unknown" (browser) |
| `exportDiagnostics`     | ✅   | ✅  | ✅      | ✅  | ✅   | |

### AutoAdjustEngine / Settings / Ledger / Bootstrap

| Function                  | Core | CLI | Android | iOS | WASM | Notes |
|:--------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `computeProfile`          | ✅   | N/A | ✅      | ✅  | N/A  | Mobile power management; browser always plugged in |
| `computeBleAdjustment`    | ✅   | N/A | ✅      | ✅  | N/A  | |
| `computeRelayAdjustment`  | ✅   | N/A | ✅      | ✅  | N/A  | |
| `overrideBleScanInterval` | ✅   | N/A | ✅      | ✅  | N/A  | |
| `overrideRelayMaxPerHour` | ✅   | N/A | ✅      | ✅  | N/A  | |
| `clearOverrides`          | ✅   | N/A | ✅      | ✅  | N/A  | |
| `loadSettings`            | ✅   | N/A | ✅      | ✅  | ✅   | WASM via getSettings/updateSettings |
| `saveSettings`            | ✅   | N/A | ✅      | ✅  | ✅   | WASM via getSettings/updateSettings |
| `validateSettings`        | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-27 |
| `defaultSettings`         | ✅   | N/A | ✅      | ✅  | ✅   | |
| `BootstrapResolver`       | ✅   | ✅  | ✅      | ✅  | N/A  | WASM takes bootstrap addrs directly |
| `LedgerManager`           | ✅   | ✅  | ✅      | ✅  | N/A  | Browser has no persistent ledger storage |

### Transport Layer (Platform-Specific)

| Feature                  | Core | CLI | Android | iOS | WASM | Notes |
|:-------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| BLE (L2CAP/GATT)        | N/A  | N/A | ✅      | ✅  | N/A  | |
| WiFi Direct              | N/A  | N/A | ✅      | N/A | N/A  | Android only |
| WiFi Aware               | N/A  | N/A | ✅      | N/A | N/A  | Android only |
| Multipeer Connectivity   | N/A  | N/A | N/A     | ✅  | N/A  | iOS only |
| mDNS Discovery           | ✅   | ✅  | ✅      | ✅  | N/A  | |
| TCP/QUIC (libp2p)        | ✅   | ✅  | ✅      | ✅  | N/A  | Mobile via SwarmBridge TCP_MDNS transport (2026-04-09) |
| TCP/mDNS LAN delivery    | ✅   | ✅  | ✅      | ✅  | N/A  | SmartTransportRouter scores TCP_MDNS for LAN peers |
| WebSocket/WebRTC         | ✅   | N/A | N/A     | N/A | ⚠️   | WASM partial |

---

## [Current] Remaining Parity Gaps

### None — 100% feature parity achieved (2026-03-27, transport parity 2026-04-09)

All core API functions are now wired across all applicable platforms. TCP/mDNS transport
parity was achieved on 2026-04-09: SmartTransportRouter on Android and iOS now includes
a `TCP_MDNS` transport type that scores and routes LAN-discovered peers via libp2p TCP
through SwarmBridge, separate from internet relay. The remaining
items marked N/A are intentional design choices:

- **`setDelegate` on WASM**: Browser uses polling via `drainReceivedMessages` instead
- **`receiveMessage` on mobile**: Mobile platforms decrypt via delegate callback path
- **`prepareMessageWithId`/`prepareReceipt` on CLI**: CLI uses simpler message flow with automatic receipt handling
- **`BootstrapResolver`/`LedgerManager` on WASM**: Browser takes bootstrap addrs directly and has no persistent ledger storage
- **`computeProfile`/BLE adjustments on WASM**: Browser is always plugged in with no BLE
- **Transport differences**: Platform-specific transport layers (BLE, WiFi Direct, Multipeer, etc.) are not cross-platform by design

### Web UI Parity (2026-04-10)

The Web/WASM UI (`ui/index.html` + `ui/app.js` + `ui/styles.css`) now provides full
functional parity with the Android/iOS native apps for all non-transport features:

| Android Screen        | Web UI Equivalent       | Status |
|:----------------------|:------------------------|:------:|
| ConversationsScreen   | Chats tab               | ✅     |
| ChatScreen            | Chat overlay view       | ✅     |
| ContactsScreen        | Contacts tab + FAB      | ✅     |
| DashboardScreen       | Mesh tab                | ✅     |
| SettingsScreen         | Settings tab            | ✅     |
| OnboardingScreen      | First-run modal         | ✅     |
| DiagnosticsScreen     | Connection path in Mesh | ✅     |
| IdentityScreen        | Identity card (Settings)| ✅     |

**Web UI features matching Android:**
- Bottom navigation bar (Chats / Contacts / Mesh / Settings)
- Conversation list with delete, stats (Total / Sent / Received / Delivered)
- Chat view with message bubbles (sent/received), block/unblock, timestamp
- Contact list with search, add (FAB), edit nickname, delete
- Mesh dashboard: status card, peer count, transports, discovered nodes, uptime
- Settings: service start/stop, identity section, mesh toggles, bootstrap config, blocked peers, data reset
- All Material 3 dark theme tokens faithfully replicated
---

## Rollout Process & Regression Prevention

To ensure consistent feature rollout and prevent regressions:

1. **Core First:** Implement logic in Rust `core/src`. Verify with `cargo test`.
2. **API Definition:** Expose new functionality in `core/src/api.udl`.
3. **CLI Verification:** Update CLI to use new Core features.
4. **Bindings Generation:** Run `uniffi-bindgen` to generate Swift/Kotlin bindings.
5. **Platform Data Layer:** Update `MeshRepository.swift` (iOS) and `MeshRepository.kt` (Android).
6. **Platform ViewModels:** Format/prepare data for the UI.
7. **UI Implementation:** SwiftUI (iOS), Jetpack Compose (Android).
8. **Parity Check:** Verify identical behavior on all platforms.
9. **Documentation:** Update this file.

### Regression Prevention Checklist

- Does the new feature break existing identity/storage?
- Is the feature flag/setting persisted correctly?
- Does the UI handle empty/null states?
- Are mobile-specific constraints (background execution, battery) respected?

---

## [Current] 2026-03-09 Contacts Feature Parity Gap

### Issue Identified
Android contacts screen lacks feature parity with iOS for:
1. Swipe-to-delete gesture
2. Nickname editing after contact creation
3. Long-press context menu

### Priority
P1 - Core UX parity required for alpha

