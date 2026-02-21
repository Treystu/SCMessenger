# SCMessenger: Remaining Work Tracking

This document identifies all known TODOs, FIXMEs, placeholders, and incomplete implementations across the SCMessenger repository.

## 🛠️ High-Priority Implementation Gaps

### Rust Core (`core/`)

1. **Stateless Device Engine**
   - **File**: `core/src/mobile_bridge.rs:427`
   - **Sentiment**: Placeholder
   - **Description**: `update_device_state` (battery, motion, network) currently only logs the update. The auto-adjustment engine needs to be made stateful or integrated to react to these changes (e.g., slowing down scans when battery is low).

2. **Cryptographic Binding for Sender ID**
   - **Source**: Legacy Audit Reports
   - **Sentiment**: Security Gap
   - **Description**: "The sender_public_key is NOT cryptographically bound". This implies a potential impersonation risk if not addressed.

3. ~~**Nearby Peer Public Key / Nickname Auto-fill Failure**~~ (Completed)
   - **File**: `core/src/lib.rs` (extract_public_key_from_peer_id) and `core/src/transport/swarm.rs`
   - **Root Cause**:
     - **Public Key**: The libp2p `PeerId` uses a SHA256 multihash for Ed25519 keys. The function `extract_public_key_from_peer_id` tries to parse an Identity multihash prefix, but fails since it's SHA256, making mathematical derivation impossible.
     - **Nickname**: The peer nickname is only broadcasted over BLE GATT beacons, never over LibP2P (LAN/WiFi-Aware).
   - **Fix Strategy**:
     - Update `IdentifyInfo` handling in `swarm.rs` to pass the true `PublicKey` object directly to the `CoreDelegate`, bypassing the flawed extraction.
     - Inject the sender's nickname into the libp2p `agent_version` (or a custom protocol) so Swarm-discovered peers receive it natively.

### WASM / Web Transport (`wasm/`)

1. **WebRTC Implementation Gaps**
   - **File**: `wasm/src/transport.rs:193, 368-378`
   - **Sentiment**: Significant TODOs
   - **Description**: `set_remote_answer()` and ICE candidate gathering logic are currently body-less "TODO" prescriptions. WASM transport is not yet functional for WebRTC.

2. **WebSocket Handle Safety**
   - **File**: `wasm/src/transport.rs:305`
   - **Sentiment**: Missing logic
   - **Description**: Return error if WebSocket handle is missing despite `Connected` state.

---

## ⚙️ Maintenance & Refactoring TODOs

### iOS Project (`iOS/`)

1. **Multipeer Connectivity Stability**
   - **Status**: Skeleton implemented in `MultipeerTransport.swift`.
   - **Remaining**: Verify session reliable/unreliable settings and ensure robust reconnection logic for WiFi-Direct equivalents.

2. **Generated Code Efficiency**
   - **File**: `iOS/SCMessenger/SCMessenger/Generated/api.swift:53`
   - **Description**: `// TODO: This copies the buffer. Can we read directly from a pointer?` (Performance optimization).

### Android Project (`android/`)

1. **WiFi Aware Role Negotiation**
   - **File**: `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt:326`
   - **Sentiment**: Unfinished Negotiation Flow
   - **Description**: `FIXME - Socket role negotiation needed` to correctly assign Publisher/Subscriber socket roles when a connection is formed.

2. **Test Initialization Logic**
   - **File**: `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt`
   - **Sentiment**: Incomplete Test Mocks
   - **Description**: `IronCore` initialization and `LedgerManager` setup are marked as `TODO: Implement once X is ready` inside test mocks.

3. **Conversations UI Nav**
   - **File**: `android/IMPLEMENTATION_STATUS.md`
   - **Description**: `TODO: Navigate to conversation detail` inside `ConversationsScreen`.

### UI & User Experience Placeholders

1. **Privacy Features Placeholder**
   - **File**: `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift:355`
   - **Description**: "Future Privacy Features (mirrors Android placeholders)". Includes toggles or settings that are not yet wired to core privacy modules.

2. **Onboarding Identity Fail-safe**
   - **File**: `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift:49`
   - **Description**: Resetting onboarding state if identity is missing after start. Needs better UX flow than just a log print.

---

## 📋 Tracking Summary

| Component | Status                           | Priority | Category          |
| :-------- | :------------------------------- | :------- | :---------------- |
| Core      | `Cryptographic Binding`          | High     | Security/Protocol |
| Core      | `Stateful Device Profile Engine` | Medium   | Optimization      |
| iOS       | `Multipeer Reliability`          | Medium   | Transport         |
| iOS       | `Privacy UI Integration`         | Low      | UI/UX             |
| Android   | `WiFi Aware Socket Negotiation`  | High     | Transport         |
| Android   | `Test Runner/Wrapper Init`       | Medium   | CI/CD             |
| WASM      | `WebRTC Handshake`               | High     | Transport         |

---

## 🔍 Audit Methodology

This list was compiled by auditing:

- Source code comments (`TODO`, `FIXME`, `HACK`).
- Function stubs with `tracing::info!` placeholders.
- Internal legacy audit documents and `CLAUDE.md`.
- Build logs indicating missing imports or architectural placeholders.
- **Corrected Audit Script**: `scripts/repo_audit.sh` was created to perform this search efficiently using `find` and `grep` while avoiding massive build directories.
