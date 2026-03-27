Status: Active
Last updated: 2026-03-26

# Feature Parity & Cross-Platform Function Audit

This document tracks the implementation status of all core API functions across
SCMessenger platforms (Core/Rust, CLI, iOS, Android, Web/WASM). It serves as the
canonical parity audit and gap tracker.

Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md,
docs/REPO_CONTEXT.md, and DOCUMENTATION.md.

---

## [Current] Complete Function Parity Matrix (2026-03-26)

Legend: ✅ = Wired & callable | ⚠️ = Partially wired | ❌ = Not wired | N/A = Not applicable

### IronCore Methods

| Function                     | Core | CLI | Android | iOS | WASM | Notes |
|:-----------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `initializeIdentity`        | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getIdentityInfo`           | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `setNickname`               | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getDeviceId`               | ✅   | ❌  | ❌      | ❌  | ✅   | WS13 device ID; WASM wired 2026-03-26 |
| `getSeniorityTimestamp`      | ✅   | ❌  | ❌      | ❌  | ✅   | WS13 seniority; WASM wired 2026-03-26 |
| `getRegistrationState`       | ✅   | ❌  | ❌      | ❌  | ✅   | WS13 registration; WASM wired 2026-03-26 |
| `exportIdentityBackup`      | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `importIdentityBackup`      | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `signData`                   | ✅   | ❌  | ❌      | ❌  | ✅   | Crypto utility; no mobile UI |
| `verifySignature`            | ✅   | ❌  | ❌      | ❌  | ✅   | Crypto utility; no mobile UI |
| `extractPublicKeyFromPeerId` | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `resolveIdentity`           | ✅   | ✅  | ✅      | ✅  | ✅   | iOS wired 2026-03-26; WASM wired 2026-03-26 |
| `resolveToIdentityId`       | ✅   | ✅  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `prepareMessage`            | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `prepareMessageWithId`      | ✅   | ❌  | ✅      | ✅  | ✅   | CLI uses `prepareMessage` |
| `prepareReceipt`            | ✅   | ❌  | ✅      | ✅  | ✅   | CLI has no receipt support |
| `prepareCoverTraffic`       | ✅   | ❌  | ✅      | ❌  | ✅   | Android sends cover traffic periodically |
| `receiveMessage`            | ✅   | ✅  | ❌      | ❌  | ✅   | Mobile decrypts via delegate callback path |
| `markMessageSent`           | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `outboxCount`               | ✅   | ✅  | ✅      | ❌  | ✅   | |
| `inboxCount`                | ✅   | ✅  | ❌      | ❌  | ✅   | |
| `classifyNotification`      | ✅   | N/A | ✅      | ✅  | ✅   | |
| `blockPeer`                 | ✅   | ❌  | ✅      | ✅  | ✅   | iOS wired 2026-03-26; WASM wired 2026-03-26 |
| `unblockPeer`               | ✅   | ❌  | ✅      | ✅  | ✅   | iOS wired 2026-03-26; WASM wired 2026-03-26 |
| `isPeerBlocked`             | ✅   | ❌  | ✅      | ✅  | ✅   | iOS wired 2026-03-26; WASM wired 2026-03-26 |
| `listBlockedPeers`          | ✅   | ❌  | ✅      | ✅  | ✅   | iOS wired 2026-03-26; WASM wired 2026-03-26 |
| `blockedCount`              | ✅   | ❌  | ✅      | ✅  | ✅   | iOS wired 2026-03-26; WASM wired 2026-03-26 |
| `setDelegate`               | ✅   | N/A | ✅      | ✅  | ❌   | WASM uses polling via `drainReceivedMessages` |
| `contactsManager`           | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `historyManager`            | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `updateDiskStats`           | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `performMaintenance`        | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `recordLog`                 | ✅   | N/A | ❌      | ✅  | ✅   | WASM wired 2026-03-26 |
| `exportLogs`                | ✅   | N/A | ❌      | ❌  | ✅   | WASM wired 2026-03-26 |
| `notifyPeerDiscovered`      | ✅   | ✅  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `notifyPeerDisconnected`    | ✅   | ✅  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |

### ContactManager Methods

| Function            | Core | CLI | Android | iOS | WASM | Notes |
|:--------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `add`               | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `get`               | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `remove`            | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `list`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `search`            | ✅   | ✅  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `setNickname`       | ✅   | ❌  | ✅      | ✅  | ✅   | Federated nickname; WASM wired 2026-03-26 |
| `setLocalNickname`  | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `updateLastSeen`    | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `updateDeviceId`    | ✅   | N/A | ❌      | ❌  | ✅   | WS13 device ID; WASM wired 2026-03-26 |
| `count`             | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `flush`             | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |

### HistoryManager Methods

| Function             | Core | CLI | Android | iOS | WASM | Notes |
|:---------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `add`                | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `get`                | ✅   | ❌  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `recent`             | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `conversation`       | ✅   | ❌  | ✅      | ✅  | ✅   | |
| `removeConversation` | ✅   | ❌  | ✅      | ❌  | ✅   | WASM wired 2026-03-26 |
| `search`             | ✅   | ❌  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `markDelivered`      | ✅   | ❌  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `clear`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `clearConversation`  | ✅   | ❌  | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |
| `delete`             | ✅   | ❌  | ✅      | ✅  | ✅   | iOS wired 2026-03-26; WASM wired 2026-03-26 |
| `stats`              | ✅   | ❌  | ✅      | ✅  | ✅   | |
| `count`              | ✅   | ❌  | ✅      | ✅  | ✅   | |
| `enforceRetention`   | ✅   | ✅  | ✅      | ✅  | ✅   | iOS+Android wired 2026-03-26 |
| `pruneBefore`        | ✅   | ✅  | ✅      | ✅  | ✅   | iOS+Android wired 2026-03-26 |
| `flush`              | ✅   | N/A | ✅      | ✅  | ✅   | WASM wired 2026-03-26 |

### MeshService / SwarmBridge Methods

| Function                | Core | CLI | Android | iOS | WASM | Notes |
|:------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `start` / `startSwarm`  | ✅   | ✅  | ✅      | ✅  | ✅   | WASM uses `startSwarm` |
| `stop`                  | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `pause` / `resume`      | ✅   | N/A | ✅      | ✅  | N/A  | Mobile lifecycle |
| `sendMessage`           | ✅   | ✅  | ✅      | ✅  | ✅   | WASM via `sendPreparedEnvelope` |
| `getPeers`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getListeners`          | ✅   | ✅  | ✅      | ✅  | ❌   | WASM N/A (browser) |
| `getExternalAddresses`  | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `subscribeTopic`        | ✅   | N/A | ✅      | ✅  | ❌   | |
| `unsubscribeTopic`      | ✅   | N/A | ✅      | ✅  | ❌   | |
| `publishTopic`          | ✅   | N/A | ✅      | ✅  | ❌   | |
| `sendToAllPeers`        | ✅   | ✅  | ✅      | ❌  | ❌   | |
| `dial`                  | ✅   | ✅  | ✅      | ✅  | ❌   | |
| `shutdown`              | ✅   | ✅  | ✅      | ✅  | ✅   | |
| `getConnectionPathState` | ✅  | ✅  | ✅      | ✅  | ✅   | |
| `getNatStatus`          | ✅   | ✅  | ✅      | ✅  | ❌   | |
| `exportDiagnostics`     | ✅   | ✅  | ✅      | ✅  | ✅   | |

### AutoAdjustEngine / Settings / Ledger / Bootstrap

| Function                  | Core | CLI | Android | iOS | WASM | Notes |
|:--------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| `computeProfile`          | ✅   | N/A | ✅      | ✅  | ❌   | Mobile power management |
| `computeBleAdjustment`    | ✅   | N/A | ✅      | ✅  | N/A  | |
| `computeRelayAdjustment`  | ✅   | N/A | ✅      | ✅  | N/A  | |
| `overrideBleScanInterval` | ✅   | N/A | ✅      | ✅  | N/A  | |
| `overrideRelayMaxPerHour` | ✅   | N/A | ✅      | ✅  | N/A  | |
| `clearOverrides`          | ✅   | N/A | ✅      | ✅  | N/A  | |
| `loadSettings`            | ✅   | N/A | ✅      | ✅  | ⚠️   | WASM has internal settings only |
| `saveSettings`            | ✅   | N/A | ✅      | ✅  | ⚠️   | |
| `validateSettings`        | ✅   | N/A | ✅      | ✅  | ❌   | |
| `defaultSettings`         | ✅   | N/A | ✅      | ✅  | ✅   | |
| `BootstrapResolver`       | ✅   | ✅  | ✅      | ✅  | ❌   | WASM takes bootstrap addrs directly |
| `LedgerManager`           | ✅   | ✅  | ✅      | ✅  | ❌   | |

### Transport Layer (Platform-Specific)

| Feature                  | Core | CLI | Android | iOS | WASM | Notes |
|:-------------------------|:----:|:---:|:-------:|:---:|:----:|:------|
| BLE (L2CAP/GATT)        | N/A  | N/A | ✅      | ✅  | N/A  | |
| WiFi Direct              | N/A  | N/A | ✅      | ❌  | N/A  | Android only |
| WiFi Aware               | N/A  | N/A | ✅      | ❌  | N/A  | Android only |
| Multipeer Connectivity   | N/A  | N/A | N/A     | ✅  | N/A  | iOS only |
| mDNS Discovery           | ✅   | ✅  | ✅      | ✅  | N/A  | |
| TCP/QUIC (libp2p)        | ✅   | ✅  | ❌      | ❌  | ❌   | CLI only; mobile uses native |
| WebSocket/WebRTC         | ✅   | ❌  | N/A     | N/A | ⚠️   | WASM partial |

---

## [Current] Remaining Parity Gaps

### Critical (blocking for parity)
- **CLI missing blocking commands**: `block`, `unblock`, `list-blocked` CLI subcommands
- **iOS missing `prepareCoverTraffic`**: Android sends periodic cover traffic; iOS does not
- **iOS missing `sendToAllPeers`**: Android can broadcast to all; iOS only sends to specific peers

### Medium (functional but incomplete)
- **Android/iOS missing `signData`/`verifySignature`**: Crypto utilities not exposed to mobile UI
- **Android/iOS missing `getDeviceId`/`getSeniorityTimestamp`/`getRegistrationState`**: WS13 device management not exposed to mobile
- **Android missing `recordLog`**: Not using core log manager
- **iOS/Android missing `exportLogs`**: Core log export not wired
- **WASM missing `LedgerManager`**: No reputation tracking in browser
- **WASM missing `setDelegate`**: Uses polling model instead

### Low (design differences, not bugs)
- **WASM missing topic management**: Topics not yet relevant for browser
- **WASM missing `getListeners`/`getNatStatus`**: Browser networking is different
- **Mobile missing `receiveMessage`**: Mobile decrypts via delegate callback path, not direct call
- **CLI missing `prepareMessageWithId`/`prepareReceipt`**: CLI uses simpler message flow

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

