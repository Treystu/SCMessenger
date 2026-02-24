# Stubs and Unimplemented Functionality

> **Last updated:** 2026-02-24
>
> This file tracks previously-audited stubs/placeholders and their current resolution status.

---

## Resolution Summary

| Item | Previous Status | Current Status |
| --- | --- | --- |
| WASM swarm transport bail-out | S0 | Resolved |
| Android multi-share | S0 | Resolved |
| RelayClient simulated networking | S1 | Resolved |
| WebRtcPeer legacy simulated behavior | S1 | Resolved |
| Android placeholder tests (Contacts/Settings/Chat/Service/Repository) | S2 | Resolved |
| Chat typing indicator placeholder | S3 | Resolved |
| UniFFI-generated TODO comments | S3 | Intentionally unchanged (generated code) |
| BLE GATT static identity comment | S3 | Runtime-correct (no functional gap) |

---

## Implemented Changes

### 1) Android multi-share (`ACTION_SEND_MULTIPLE`)

- **File:** `android/app/src/main/java/com/scmessenger/android/utils/ShareReceiver.kt`
- Implemented `handleMultipleShare(...)`.
- Supports multi-text and multi-URI payload aggregation.
- Reuses existing contact picker and encrypted send flow.

### 2) Relay client real networking

- **File:** `core/src/relay/client.rs`
- `connect()` now performs real TCP connect + framed handshake exchange.
- `push_envelopes()` now sends `StoreRequest` and parses `StoreAck`.
- `pull_envelopes()` now sends `PullRequest` and parses `PullResponse`.
- `send_ping()` now sends `Ping` and validates `Pong`.
- Connection removal now removes socket state as well.

### 3) WebRtcPeer simulation removal

- **File:** `wasm/src/transport.rs`
- `WebRtcPeer` now delegates to `WebRtcTransport` for offer/answer/send behavior.
- Non-WASM behavior now returns explicit unsupported errors rather than simulated success.

### 4) Android placeholder tests replaced with real tests

- **Files:**
  - `android/app/src/test/java/com/scmessenger/android/test/ContactsViewModelTest.kt`
  - `android/app/src/test/java/com/scmessenger/android/test/SettingsViewModelTest.kt`
  - `android/app/src/test/java/com/scmessenger/android/test/ChatViewModelTest.kt`
  - `android/app/src/test/java/com/scmessenger/android/test/MeshForegroundServiceTest.kt`
  - `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt`
- Removed placeholder asserts and ignored test bodies.
- Added behavior-driven unit tests for loading, validation, routing decisions, message state, and settings persistence behavior.

### 5) Chat typing indicator implementation

- **File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt`
- `isTyping` now tracks compose input state.
- Resets on send success and input clear.

---

## Verification

### Rust

- `cargo check -p scmessenger-core` ✅
- `cargo check -p scmessenger-wasm` ✅

### Android unit tests (targeted)

- `./gradlew :app:testDebugUnitTest --tests ... -x buildRustAndroid -x generateUniFFIBindings` ✅
- Covered suites:
  - `ContactsViewModelTest`
  - `SettingsViewModelTest`
  - `ChatViewModelTest`
  - `MeshForegroundServiceTest`
  - `MeshRepositoryTest`

---

## Remaining Non-Action Items

1. UniFFI-generated TODO comments remain in generated artifacts by design and will regenerate.
2. BLE GATT static default identity comment describes transient startup state that is overwritten correctly at runtime.

No remaining functional stubs from this audit are open.
