# Stubs and Unimplemented Functionality

> **Audit date:** 2026-02-23
>
> Comprehensive inventory of every stub, placeholder, simulated, or
> unimplemented code path found in the SCMessenger codebase. Entries are
> grouped by severity tier and annotated with platform, file, and line
> references.

---

## Severity Tiers

| Tier                               | Meaning                                                                                      |
| ---------------------------------- | -------------------------------------------------------------------------------------------- |
| **S0 — Blocks core functionality** | Feature is missing and no workaround exists; affects runtime behavior.                       |
| **S1 — Simulated / partial**       | Code path exists but returns fake data or silently no-ops. Testing or non-critical use only. |
| **S2 — Placeholder test**          | Unit test body is a `assertTrue("Placeholder …", true)` No-Op; provides no coverage.         |
| **S3 — Cosmetic / future**         | Comment-level marker; no runtime impact.                                                     |

---

## S0 — Blocks Core Functionality

### 1. WASM swarm transport bail-out

- **File:** `core/src/transport/swarm.rs` line 1288
- **Code:** `anyhow::bail!("WASM transport not yet implemented");`
- **Impact:** `start_swarm()` unconditionally fails on `wasm32`. The Web
  client cannot establish a libp2p swarm—only the WebSocket relay path
  in `wasm/src/transport.rs` is functional.
- **Workaround:** Web client currently uses `WebSocketRelay` for message
  transport, bypassing the swarm entirely.
- **Resolution:** Implement a WASM-compatible swarm transport using
  `libp2p-wasm-ext` or equivalent, or explicitly document that Web is
  relay-only and update the parity tracking accordingly.

### 2. Android multi-share not implemented

- **File:** `android/app/src/main/java/com/scmessenger/android/utils/ShareReceiver.kt` lines 67–72
- **Code:**

  ```kotlin
  private fun handleMultipleShare(context: Context, intent: Intent) {
      Toast.makeText(context, "Multiple items sharing not yet supported", Toast.LENGTH_SHORT).show()
      Timber.w("Multiple share not implemented")
  }
  ```

- **Impact:** `ACTION_SEND_MULTIPLE` intents are silently dropped with a
  user-facing toast. Users who select multiple items to share into
  SCMessenger will see a failure.
- **Resolution:** Either implement multi-item share handling (iterate
  `EXTRA_STREAM` / `EXTRA_TEXT` lists) or remove `ACTION_SEND_MULTIPLE`
  from the manifest intent filter so the share sheet never offers the
  option.

---

## S1 — Simulated / Partial Implementation

### 3. Relay client networking is fully simulated

- **File:** `core/src/relay/client.rs`
- **Affected methods:**

  | Method             | Line    | Behavior                                                   |
  | ------------------ | ------- | ---------------------------------------------------------- |
  | `connect()`        | 151–156 | Sets state to `Handshaking` without opening a real socket. |
  | `push_envelopes()` | 214–218 | Always returns `Ok((0, 0))` — no network I/O.              |
  | `pull_envelopes()` | 243–254 | Always returns `Ok(Vec::new())` — no network I/O.          |
  | `send_ping()`      | 280–288 | No-op when connections exist.                              |

- **Impact:** The `RelayClient` struct compiles and passes tests but
  does not perform any actual network communication. Any code path that
  relies on `RelayClient` for store-and-forward delivery will silently
  succeed without actually delivering messages.
- **Context:** The primary message delivery path uses the libp2p swarm
  relay protocol (in `swarm.rs`), which _is_ fully implemented. This
  `RelayClient` is a secondary client intended for an independent relay
  protocol layer.
- **Resolution:** Either complete the TCP/TLS transport layer inside
  `RelayClient` or remove/deprecate the struct and document that the
  swarm relay protocol is the only supported relay mechanism.

### 4. WebRtcPeer legacy stub (non-functional on all platforms)

- **File:** `wasm/src/transport.rs` lines 1062–1208
- **Comment:** `// WebRtcPeer — legacy stub kept for WasmTransport compatibility`
- **Affected methods:**

  | Method            | Behavior                                                        |
  | ----------------- | --------------------------------------------------------------- |
  | `create_offer()`  | Returns a static simulated SDP string (not real WebRTC).        |
  | `create_answer()` | Returns a static simulated SDP string.                          |
  | `send_data()`     | Logs "Simulated send" on non-WASM; no actual data transmission. |

- **Impact:** `WebRtcPeer` cannot establish real peer-to-peer WebRTC
  connections. It exists purely for backward-compatible compilation of
  `WasmTransport`.
- **Context:** The newer `WebRtcTransport` struct (lines 380–1059 in the
  same file) is the real, browser-native WebRTC implementation using
  `web-sys` APIs.
- **Resolution:** Remove `WebRtcPeer` and update any remaining
  references to use `WebRtcTransport` directly, or clearly mark it as
  test-only infrastructure.

---

## S2 — Placeholder Tests (Android)

All Android ViewModel and service tests contain placeholder bodies that
always pass without exercising any logic. They are documented in
`android/app/src/test/README.md` as blocked by UniFFI MockK
limitations.

### 5. ContactsViewModelTest — 5 placeholder tests

- **File:** `android/app/src/test/java/com/scmessenger/android/test/ContactsViewModelTest.kt`
- **Placeholders:**
  - `testLoadContacts` (line 36) — "Placeholder - requires data loading"
  - `testAddContact` (line 50) — "Placeholder - requires add logic"
  - `testRemoveContact` (line 63) — "Placeholder - requires remove logic"
  - `testSearchContacts` (line 76) — "Placeholder - requires search logic"
  - `testOnlineStatus` (line 91) — "Placeholder - requires online status"

### 6. SettingsViewModelTest — 6 placeholder tests

- **File:** `android/app/src/test/java/com/scmessenger/android/test/SettingsViewModelTest.kt`
- **Placeholders:**
  - `testLoadSettings` (line 36) — "Placeholder - requires settings loading"
  - `testSaveSettings` (line 49) — "Placeholder - requires save logic"
  - `testToggleCoupling` (line 63) — "Placeholder - requires coupling enforcement"
  - `testSettingsValidation` (line 76) — "Placeholder - requires validation"
  - `testSettingsOverride` (line 89) — "Placeholder - requires override logic"
  - `testResetToDefaults` (line 102) — "Placeholder - requires reset logic"

### 7. ChatViewModelTest — 5 placeholder tests

- **File:** `android/app/src/test/java/com/scmessenger/android/test/ChatViewModelTest.kt`
- **Placeholders:**
  - `testViewModelCreation` (line 42) — "Placeholder - requires ViewModel instantiation"
  - `testSendMessage` (line 55) — "Placeholder - requires event emission"
  - `testMessageStatus` (line 70) — "Placeholder - requires status tracking"
  - `testPeerTracking` (line 84) — "Placeholder - requires peer tracking"
  - `testMessagePagination` (line 97) — "Placeholder - requires pagination logic"

### 8. MeshForegroundServiceTest — 8 placeholder tests

- **File:** `android/app/src/test/java/com/scmessenger/android/test/MeshForegroundServiceTest.kt`
- **Placeholders:**
  - `testServiceCreation` (line 37) — "Placeholder - requires service framework"
  - `testStartCommand` (line 52) — "Placeholder - requires intent handling"
  - `testStopCommand` (line 67) — "Placeholder - requires stop logic"
  - `testCallbackRegistration` (line 81) — "Placeholder - requires callback testing"
  - `testMessageHandling` (line 97) — "Placeholder - requires message handling"
  - `testBatteryMonitoring` (line 112) — "Placeholder - requires battery monitoring"
  - `testNotificationCreation` (line 125) — "Placeholder - requires notification testing"
  - `testWakeLockManagement` (line 139) — "Placeholder - requires WakeLock testing"

### 9. MeshRepositoryTest — 12 placeholder/ignored tests

- **File:** `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt`
- **Status:** 5 enforcement tests are real; remaining 12 are `@Ignore`d
  placeholders requiring mock infrastructure for `settingsManager`,
  `CoreDelegate`, `IronCore`, `LedgerManager`, `MeshSettingsManager`,
  and `ContactManager`.

**Total placeholder tests across Android:** **36 tests** (out of ~41
total unit tests).

---

## S3 — Cosmetic / Future Markers

### 10. Typing indicator placeholder

- **File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` line 50
- **Code:** `// Typing indicator (placeholder for future)`
- **Impact:** `_isTyping` state flow exists but is never set to `true`.
  No UI references it yet. Pure future-proofing with no runtime effect.

### 11. UniFFI-generated TODOs

- **Files:** `iOS/SCMessenger/SCMessenger/Generated/api.swift`,
  `iOS/SCMessenger/Generated/api.swift`,
  `core/target/generated-sources/uniffi/swift/api.swift`,
  `core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt`
- **Content:** UniFFI boilerplate comments such as:
  - `// TODO: This copies the buffer. Can we read directly from a…`
  - `// TODO: We'd like this to be 'private' but for Swifty reasons…`
  - `// TODO: maybe we should log a warning if called more than once?`
- **Impact:** None — these are upstream UniFFI scaffolding comments and
  will be regenerated on each `uniffi-bindgen` invocation.
- **Action:** No action required. Do not attempt to fix these in the
  generated files.

### 12. BleGattServer static identity placeholder

- **File:** `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt` line 42
- **Code:** `// Default is a static placeholder; call setIdentityData() once IronCore is ready.`
- **Impact:** The default is overwritten at runtime once `IronCore`
  initializes. Startup-only transient state; not a functional gap.

---

## Summary Statistics

| Tier      | Count  | Description                                                            |
| --------- | ------ | ---------------------------------------------------------------------- |
| S0        | 2      | WASM swarm bail-out; Android multi-share                               |
| S1        | 2      | Relay client simulated networking; WebRtcPeer legacy stub              |
| S2        | 36     | Android placeholder unit tests across 5 test files                     |
| S3        | 3      | Typing indicator comment; UniFFI-generated TODOs; BLE identity default |
| **Total** | **43** |                                                                        |

---

## Cross-Reference to REMAINING_WORK_TRACKING.md

| This doc entry   | Tracking item                                   |
| ---------------- | ----------------------------------------------- |
| #1 WASM swarm    | P0 item 8 (Web parity promotion)                |
| #2 Multi-share   | Not previously tracked — **new**                |
| #3 Relay client  | P0 item 9 (active-session reliability)          |
| #4 WebRtcPeer    | P0 item 8 (Web parity promotion)                |
| #5–#9 Tests      | P1 item 9 (Android test execution truthfulness) |
| #10 Typing       | Not tracked — informational only                |
| #11 UniFFI TODOs | P1 item 16 (TODO/FIXME accuracy sync)           |
| #12 BLE identity | Not a gap — runtime behavior is correct         |
