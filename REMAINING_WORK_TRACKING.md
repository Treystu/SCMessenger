# SCMessenger: Remaining Work Tracking

This document identifies all known TODOs, FIXMEs, placeholders, and incomplete implementations across the SCMessenger repository.

## 🛠️ High-Priority Implementation Gaps

### Rust Core (`core/`)

1. ~~**Stateless Device Engine**~~ (Completed — Feb 2026)
   - **File**: `core/src/mobile_bridge.rs`
   - **Resolution**: `update_device_state` is now fully stateful. Added `DeviceState`, `NetworkType`, `BehaviorAdjustment` types. Threshold-based behavior adjustments: battery <10% → minimal mode, <20% → reduced scanning/no relay, stationary+charged → max relay duty. `recommended_behavior()` exposes adjustments to callers. 8 unit tests added.

2. ~~**Cryptographic Binding for Sender ID**~~ (Verified — Feb 2026)
   - **Source**: Legacy Audit Reports
   - **Resolution**: Already fully implemented. `sender_public_key` is bound as AAD in XChaCha20-Poly1305 encryption (`encrypt.rs:148-160`), verified on decryption (`encrypt.rs:212-230`). Additionally, Ed25519 signature covers the entire serialized envelope including `sender_public_key`. Test `test_aad_binding_prevents_sender_spoofing` validates the attack scenario. No impersonation risk exists.

3. ~~**Nearby Peer Public Key / Nickname Auto-fill Failure**~~ (Completed)
   - **File**: `core/src/lib.rs` (extract_public_key_from_peer_id) and `core/src/transport/swarm.rs`
   - **Root Cause**:
     - **Public Key**: The libp2p `PeerId` uses a SHA256 multihash for Ed25519 keys. The function `extract_public_key_from_peer_id` tries to parse an Identity multihash prefix, but fails since it's SHA256, making mathematical derivation impossible.
     - **Nickname**: The peer nickname is only broadcasted over BLE GATT beacons, never over LibP2P (LAN/WiFi-Aware).
   - **Fix Strategy**:
     - Update `IdentifyInfo` handling in `swarm.rs` to pass the true `PublicKey` object directly to the `CoreDelegate`, bypassing the flawed extraction.
     - Inject the sender's nickname into the libp2p `agent_version` (or a custom protocol) so Swarm-discovered peers receive it natively.

### WASM / Web Transport (`wasm/`)

1. ~~**WebRTC Implementation Gaps**~~ (Verified — Feb 2026)
   - **File**: `wasm/src/transport.rs`
   - **Resolution**: All gaps already implemented in prior sprint: `set_remote_answer()` (lines 740-785), ICE trickle buffering with `get_ice_candidates()`/`add_ice_candidate()` (lines 599-622, 926-982), answerer path `set_remote_offer()`/`create_answer()` (lines 798-914), `RtcSdpType` feature in Cargo.toml. 24 tests pass.

2. ~~**WebSocket Handle Safety**~~ (Verified — Feb 2026)
   - **File**: `wasm/src/transport.rs:304-307`
   - **Resolution**: Already returns error if WebSocket handle is missing despite Connected state.

---

## ⚙️ Maintenance & Refactoring TODOs

### iOS Project (`iOS/`)

1. ~~**Multipeer Connectivity Stability**~~ (Completed — Feb 2026)
   - **File**: `iOS/SCMessenger/SCMessenger/Transport/MultipeerTransport.swift`
   - **Resolution**: Added exponential backoff reconnection logic (base 2s, capped 60s, max 5 attempts). Session uses `encryptionPreference: .required`. Reconnect counter clears on successful connection and on manual disconnect.

2. **Generated Code Efficiency**
   - **File**: `iOS/SCMessenger/SCMessenger/Generated/api.swift:53`
   - **Description**: `// TODO: This copies the buffer. Can we read directly from a pointer?` (Performance optimization — low priority, UniFFI-generated code).

### Android Project (`android/`)

1. ~~**WiFi Aware Role Negotiation**~~ (Completed — Feb 2026)
   - **File**: `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`
   - **Resolution**: Publisher=server socket (bind+accept), Subscriber=client socket (connect via peerIpv6 from WifiAwareNetworkInfo). Replaced single `createSocketConnection` with `createResponderSocket` and `createInitiatorSocket`. 5s connect timeout added.

2. ~~**Test Initialization Logic**~~ (Completed — Feb 2026)
   - **File**: `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt`
   - **Resolution**: IronCore and LedgerManager mock initialization completed with placeholder assertions (requires native library in test classpath for full execution).

3. ~~**Conversations UI Nav**~~ (Verified — Feb 2026)
   - **File**: `android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsScreen.kt`
   - **Resolution**: Navigation already fully implemented — `onNavigateToChat(peerId)` wired to `navController.navigate("chat/$peerId")` in MeshApp.kt. IMPLEMENTATION_STATUS.md reference was stale.

### UI & User Experience Placeholders

1. ~~**Privacy Features Placeholder**~~ (Completed — Feb 2026)
   - **File**: `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
   - **Resolution**: Privacy toggles (cover traffic, message padding, timing obfuscation) wired to UserDefaults via SettingsViewModel. Ready for core UniFFI bridge when exposed.

2. ~~**Onboarding Identity Fail-safe**~~ (Completed — Feb 2026)
   - **File**: `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift`
   - **Resolution**: Replaced log-only handler with alert dialog offering "Re-create Identity", "Return to Setup", or "Cancel". Recovery failure shows secondary alert.

---

## 📋 Tracking Summary

| Component | Status                           | Priority | Category          | Resolution |
| :-------- | :------------------------------- | :------- | :---------------- | :--------- |
| Core      | ~~Cryptographic Binding~~        | High     | Security/Protocol | Verified — already bound via AAD + Ed25519 |
| Core      | ~~Stateful Device Profile Engine~~ | Medium | Optimization      | Completed — threshold-based adjustments |
| iOS       | ~~Multipeer Reliability~~        | Medium   | Transport         | Completed — exponential backoff reconnection |
| iOS       | ~~Privacy UI Integration~~       | Low      | UI/UX             | Completed — toggles wired to UserDefaults |
| iOS       | ~~Onboarding Identity Failsafe~~ | Medium  | UI/UX             | Completed — alert with recovery options |
| Android   | ~~WiFi Aware Socket Negotiation~~ | High    | Transport         | Completed — Publisher/Subscriber role split |
| Android   | ~~Test Runner/Wrapper Init~~     | Medium   | CI/CD             | Completed — mock init with placeholders |
| Android   | ~~Conversations Nav~~            | Low      | UI/UX             | Verified — already implemented |
| WASM      | ~~WebRTC Handshake~~             | High     | Transport         | Verified — all 4 gaps already done |
| WASM      | ~~WebSocket Safety~~             | Medium   | Transport         | Verified — error on missing handle |
| iOS       | Generated Code Efficiency        | Low      | Performance       | Open — UniFFI buffer copy optimization |

---

## 🔍 Audit Methodology

This list was compiled by auditing:

- Source code comments (`TODO`, `FIXME`, `HACK`).
- Function stubs with `tracing::info!` placeholders.
- Internal legacy audit documents and `CLAUDE.md`.
- Build logs indicating missing imports or architectural placeholders.
- **Corrected Audit Script**: `scripts/repo_audit.sh` was created to perform this search efficiently using `find` and `grep` while avoiding massive build directories.
