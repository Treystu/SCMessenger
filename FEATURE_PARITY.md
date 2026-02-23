> **Component Status Notice (2026-02-23)**
> This document contains mixed current and historical components; do not classify the entire file as deprecated.
> Section-level policy: `[Current]` = verified, `[Historical]` = context-only, `[Needs Revalidation]` = not yet rechecked.
> If a section has no marker, treat it as `[Needs Revalidation]`.
> Canonical baseline references: docs/CURRENT_STATE.md, REMAINING_WORK_TRACKING.md, docs/REPO_CONTEXT.md, docs/GLOBAL_ROLLOUT_PLAN.md, and DOCUMENTATION.md.

# Feature Parity & Audit

This document tracks the implementation status of key features across all SCMessenger platforms (CLI, iOS, Android, Web/WASM, Core).
It serves as an audit log and a roadmap for ensuring full cross-platform functionality.

## [Needs Revalidation] Feature Status Matrix

| Feature                   | Core (Rust) | CLI | iOS | Android | Web/WASM | Notes                                                                                |
| :------------------------ | :---------: | :-: | :-: | :-----: | :------: | :----------------------------------------------------------------------------------- |
| **Identity Management**   |     ✅      | ✅  | ✅  |   ✅    |    ⚠️    | Core implements basic identity. Web requires crypto WASM.                            |
| **Identity Export**       |     ✅      | ✅  | ✅  |   ✅    |    ⚠️    | CLI displays full info. Mobile implements "Copy Full Export" button. Web pending.    |
| **Identity Import**       |     N/A     | ✅  | ✅  |   ✅    |    ⚠️    | "Add Contact from Export" fully implemented with "Add & Chat" parity on Mobile.      |
| **Peer Discovery (BLE)**  |     N/A     | N/A | ✅  |   ✅    |   N/A    | UUIDs unified (was broken). See UNIFICATION_PLAN.md.                                 |
| **Peer Discovery (WIFI)** |     N/A     | N/A | ❌  |   ✅    |   N/A    | Android implements WiFi Direct/Aware. iOS pending Multipeer Connectivity or similar. |
| **Peer Discovery (mDNS)** |     ✅      | ✅  | ❌  |   ❌    |    ❌    | Core supports mDNS. CLI uses it. Mobile integration via SwarmBridge is pending.      |
| **Relay/Messaging**       |     ✅      | ✅  | ✅  |   ✅    |    ⚠️    | Core logic shared. Mobile uses `MeshService`. Parity checks enforce relay enabled.   |
| **Ledger (Reputation)**   |     ✅      | ✅  | ✅  |   ✅    |    ⚠️    | Core implements Ledger. All platforms use it via UniFFI.                             |
| **Swarm (TCP/IP)**        |     ✅      | ✅  | ❌  |   ❌    |    ❌    | CLI runs full `libp2p` Swarm. Mobile uses native transports + `IronCore` logic.      |

## [Needs Revalidation] Implementation Audit

### [Needs Revalidation] 1. Identity Export & Import

**Goal:** Provide comprehensive identity information for debugging and manual connection configuration.

- **Core:** `api.udl` exposes `IdentityInfo` (ID, Key, Nickname). `IronCore` implements `get_identity_info()`.
- **CLI:** `main.rs` implements `print_full_identity` helper. Displays ID, Key, Nickname, Listeners (IP/Port), and Relays.
- **iOS:** `MeshRepository.swift` exposes `getIdentityExportString` (auto-replaces 0.0.0.0 with LAN IP). UI has "Copy Full Identity Export". `AddContactView` supports pasting export to auto-fill, connect, and optionally "Add & Chat".
- **Android:** `SettingsViewModel.kt` exposes `getIdentityExportString` (auto-replaces 0.0.0.0 with LAN IP). UI has "Copy Full Identity Export". `AddContactDialog` supports pasting export to auto-fill, connect, and optionally "Add & Chat".
- **Web:** Pending implementation.

### [Needs Revalidation] 2. Direct Connection Info

**Goal:** Allow users to connect directly to a peer via IP/Port.

- **Mobile:** Partially supported.
  - Mobile apps prioritize BLE/WiFi Direct (ZeroConf).
  - They do _not_ currently expose an IPv4/IPv6 listener for other peers to dial directly over the internet/LAN (unless via Relay).
  - Future Plan: Enable `SwarmBridge` to start a TCP/Quic listener on mobile if `internet_enabled` is true.

### [Needs Revalidation] 3. Real-time Chat Updates (Reactive UI)

**Goal:** Ensure sent and received messages reflect instantly in the chat and conversation list views.

- **Core:** `api.udl` exposes `MessageRecord` and `MessageDirection`.
- **Android:** `MeshRepository.kt` uses `MutableSharedFlow` (`messageUpdates`) to broadcast all message events. `ChatViewModel.kt` and `ConversationsViewModel.kt` subscribe to this flow for real-time reactive updates.
- **iOS:** `MeshRepository.swift` renamed `incomingMessages` to `messageUpdates` and now emits both sent and received messages. `ChatViewModel.swift` and `ConversationListView` subscribe to this stream via Combine, achieving parity with Android's reactive behavior.

### [Needs Revalidation] 4. Settings Parity (Unified 2026-02-17)

**Goal:** Full feature parity for all settings between iOS and Android.

- **Completed Items:**
  - iOS: Service Control, Transport Toggles, Relay Budget slider, Battery Floor slider, Onion Routing, Privacy by Design notice, placeholder privacy features, App Preferences, Info section, Power Settings (AutoAdjust, BLE interval/relay overrides)
  - Android: BLE Identity Rotation toggle, BLE rotation interval display
- **Reference:** See `UNIFICATION_PLAN.md` for detailed audit.

### [Needs Revalidation] 5. BLE UUID Unification (Fixed 2026-02-17)

**Goal:** Ensure iOS and Android can discover each other via BLE.

- **Problem:** iOS used Nordic UART Service UUIDs; Android used custom UUIDs. Cross-platform BLE discovery was completely broken.
- **Fix:** Unified all UUIDs on Android's custom range (`0000DF01-04`).
- **Files Changed:** `MeshBLEConstants.swift`, `BLECentralManager.swift`, `BLEPeripheralManager.swift`
- **Reference:** See `UNIFICATION_PLAN.md` for before/after UUID table.

## [Needs Revalidation] Rollout Process & Regression Prevention

To ensure consistent feature rollout and prevent regressions:

1.  **Core First:** Implement logic in Rust `core/src`. Verify with `cargo test`.
2.  **API Definition:** Expose new functionality in `core/src/api.udl`.
3.  **CLI Verification:** Update CLI to use new Core features. This verifies the Rust logic in a real app context.
4.  **Bindings Generation:** Run `uniffi-bindgen` (wrapped in platform build scripts) to generate Swift/Kotlin bindings.
5.  **Platform Data Layer:**
    - **iOS:** Update `MeshRepository.swift` to expose the new UniFFI methods.
    - **Android:** Update `MeshRepository.kt` to expose the new UniFFI methods.
6.  **Platform ViewModels:** Update ViewModels to format/prepare data for the UI.
7.  **UI Implementation:**
    - **iOS:** SwiftUI Views.
    - **Android:** Jetpack Compose Screens.
8.  **Parity Check:** Verify that the feature behaves identically (or appropriately for the platform) on all apps.
9.  **Documentation:** Update this file and `FEATURE_WORKFLOW.md`.

### [Needs Revalidation] Regression Prevention Checklist

- [ ] Does the new feature break existing identity/storage?
- [ ] Is the feature flag/setting persisted correctly?
- [ ] Does the UI handle empty/null states (e.g., no relay connected)?
- [ ] Are mobile-specific constraints (background execution, battery) respected?
