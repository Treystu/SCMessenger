# SCMessenger v0.2.1 Alpha Release Burndown

Status: Active
Last updated: 2026-03-31
Author: Copilot audit — derived from live codebase scan

---

## How to Read This Document

Each item is classified by **category** and **severity**:

| Severity | Meaning |
|----------|---------|
| **BLOCKER** | Must be resolved before alpha ships |
| **P1** | Should be resolved before alpha ships; workaround must be documented if deferred |
| **P2** | Can ship in alpha with known-issue note; must be resolved before beta |
| **P3** | Tracked tech debt — fine to defer past beta |
| **INFO** | Informational; no action required |

---

## Section 1 — CI / Build / Test Health

### 1.1 CI Status (as of commit `ae54174`)

| Job | Status | Notes |
|-----|--------|-------|
| Rust Core (ubuntu-latest) | ✅ PASS | 670+ tests, 0 failures |
| Rust Core (macos-latest) | ✅ PASS | clippy -D warnings clean, cargo fmt clean |
| WASM / Web — Core wasm32 check | ✅ PASS | `--features wasm` resolves uniffi_core Send bound |
| WASM / Web — WASM crate build | ✅ PASS | `list_blocked_peers_raw()` resolves E0599 |
| WASM / Web — wasm-pack test | ✅ PASS (CI) | Requires Firefox headless in CI |
| Android | ✅ PASS | All 8 `MessageRecord()` constructors have `hidden = false` |
| iOS | ✅ PASS | `verify-test.sh` auto-generates bindings; `@MainActor` isolation fixed |
| Repo Hygiene (docs sync) | ✅ PASS | `docs_sync_check.sh` passes |
| Repo Hygiene (path governance) | ✅ PASS | No lowercase `ios/` paths, no nested git repos |

### 1.2 Ignored Tests — Full Inventory

All 18 ignored tests have documented reasons. None represent hidden failures.

| Test | Location | Reason | Severity |
|------|----------|--------|----------|
| `test_sync_large_symmetric_difference` | `core/src/drift/sync.rs:560` | 50 symmetric differences exceeds IBLT capacity for small tables — known algorithmic limit; fails when `--include-ignored` | **P2** — IBLT sizing needs tuning for large-diff scenarios |
| `test_detect_nat_type_with_peers` | `core/src/transport/nat.rs:639` | Requires live SwarmHandle (real libp2p connections) | P3 — needs integration harness |
| `test_get_external_address_from_peer` | `core/src/transport/nat.rs:646` | Requires live SwarmHandle | P3 |
| `test_get_hole_punch_status` | `core/src/transport/nat.rs:653` | Requires live SwarmHandle | P3 |
| `test_hole_punch_disabled` | `core/src/transport/nat.rs:676` | Requires live SwarmHandle | P3 |
| `test_hole_punch_start` | `core/src/transport/nat.rs:683` | Requires live SwarmHandle | P3 |
| `test_peer_discovery_no_peers` | `core/src/transport/nat.rs:690` | Requires live SwarmHandle | P3 |
| `test_probe_nat` | `core/src/transport/nat.rs:697` | Requires live SwarmHandle | P3 |
| `test_two_node_address_reflection` | `core/tests/integration_nat_reflection.rs:19` | Requires real networking | P3 |
| `test_peer_address_discovery_with_live_swarm` | `core/tests/integration_nat_reflection.rs:87` | Requires real networking | P3 |
| `test_nat_traversal_with_live_swarms` | `core/tests/integration_nat_reflection.rs:157` | Requires real networking | P3 |
| `test_multiple_address_reflections` | `core/tests/integration_nat_reflection.rs:265` | Requires real networking | P3 |
| `test_address_reflection_timeout` | `core/tests/integration_nat_reflection.rs:321` | Requires real networking | P3 |
| `test_multiport_swarm_integration` | `core/tests/test_multiport.rs:256` | Requires real networking (TCP bind) | P3 |
| `offline_recipient_receives_after_reconnect_without_sender_resend` | `core/tests/integration_relay_custody.rs:75` | Requires libp2p socket permissions | **P1** — This IS run in CI via `--include-ignored`; passing ✅ |
| `test_all_six_phases_integrated` | `core/tests/integration_all_phases.rs:19` | Requires real networking + mDNS multicast | P3 |
| `test_message_retry_on_failure` | `core/tests/integration_all_phases.rs:251` | Requires real networking + mDNS multicast | P3 |
| *(3rd all_phases test)* | `core/tests/integration_all_phases.rs:323` | Requires real networking + mDNS multicast | P3 |

**Action for P2:** The IBLT test (`test_sync_large_symmetric_difference`) fails when `--include-ignored` is used. The IBLT is sized for typical peer sync windows, not 50-item diffs. This is a known, documented limitation — the fix is to tune IBLT table size or chunk large diffs. Should be addressed before beta where larger peer sets are expected.

### 1.3 Integration Tests Not in CI

The following integration test files exist in `core/tests/` but are **not invoked in CI**:

| Test file | Tests | Should be in CI? |
|-----------|-------|-----------------|
| `integration_contact_block.rs` | 3 (all pass) | **YES — missing from CI** |
| `integration_e2e.rs` | 5 (all pass) | YES (validates full send→receive roundtrip) |
| `integration_ironcore_roundtrip.rs` | 7 (all pass) | YES (crypto correctness) |
| `integration_registration_protocol.rs` | 3 (all pass) | P2 — should be added |
| `test_address_observation.rs` | 4 (all pass) | P2 |
| `test_mesh_routing.rs` | 17 (all pass) | P2 |
| `test_persistence_restart.rs` | 1 (all pass) | P2 |

**Recommendation:** Add the always-passing integration tests to the CI `check-core` step. Only the real-networking-required tests belong in the `--include-ignored` category.

---

## Section 2 — Incomplete Implementations

### 2.1 WASM — `startReceiveLoop` (Deprecated, Not Removed)

- **Location:** `wasm/src/lib.rs:656-660`
- **Status:** Exported as `startReceiveLoop(relayUrl)` but immediately returns a deprecation error string.
- **Impact:** Dead export — JS callers get a confusing non-fatal error rather than a compile-time signal. Any JavaScript consumer still calling `startReceiveLoop` silently fails.
- **Fix:** Gate behind `#[deprecated]` wasm-bindgen annotation or remove entirely. If any external web app uses this, it needs migration to `startSwarm(bootstrapAddrs)`.
- **Severity:** **P2**

### 2.2 WebRTC ICE Trickle + Answerer Path (Partial)

- **Location:** `wasm/src/transport.rs`
- **Status:** `set_remote_answer()`, `set_remote_offer()`, `create_answer()`, `get_ice_candidates()`, `add_ice_candidate()` are implemented. The doc-comments describe a complete trickle-ICE signalling flow.
- **Gap:** No integration-level test for the full offer→answer→ICE trickle path. The methods exist but have never been exercised in a real WebRTC session (non-WASM stubs return `Err` on native).
- **Severity:** P2 — Functionality exists but is untested end-to-end. The WASM browser test suite covers `wasm-pack test` but not WebRTC peer establishment.

### 2.3 BLE Transport — Protocol Layer Only (No Platform Bridge)

- **Location:** `core/src/transport/ble/`
- **Status:** Complete protocol layer (beacon, GATT, L2CAP, scanner). **No platform bridge to CoreBluetooth (iOS) or BluetoothGatt (Android) is wired.**
- **Gap:** `BleTransport` protocol types exist in core but are never called from `iOS/SCMessenger/` or `android/`. The actual BLE scanning/advertising is done entirely by native platform code (Swift/Kotlin) outside of core. The Rust BLE layer is a design artifact — it defines the protocol but isn't in the execution path.
- **Severity:** **P1** — Needs explicit documentation that BLE is platform-native-only and the `core/src/transport/ble/` module is a protocol specification / future FFI target, not an active transport.

### 2.4 FindMy Wake-Up — Protocol Complete, Not Wired to Runtime

- **Location:** `core/src/relay/findmy.rs`
- **Status:** `FindMyBeaconManager`, `encode_wakeup`, `decode_wakeup`, `is_our_wakeup` are implemented and tested.
- **Gap:** `FindMyBeaconManager` is **not instantiated anywhere in `core/src/lib.rs`, `core/src/transport/swarm.rs`, or any mobile bridge**. The feature is protocol-complete but dead code in the runtime.
- **Severity:** **P2** — Should be documented as "protocol-ready, not yet activated". Must be wired before findmy-style offline wake-up is used.

### 2.5 Onion Routing — Module Complete, Not in Message Path

- **Location:** `core/src/privacy/onion.rs`, `core/src/privacy/circuit.rs`
- **Status:** `construct_onion`, `peel_layer`, `CircuitBuilder::build_circuit` are implemented and unit-tested.
- **Gap:** `construct_onion` and `build_circuit` are **never called from `prepare_message()` or the relay pipeline**. Onion routing is a standalone module — it is not active for any real message in v0.2.1.
- **Severity:** **P2** — Must be documented clearly. The privacy model for v0.2.1 alpha is transport encryption only (XChaCha20-Poly1305 with envelope signatures), **not** onion routing. Misleading if not called out.

### 2.6 Device ID Blocking — Infrastructure Blocked

- **Location:** `core/src/store/blocked.rs:4,17,59`
- **Status:** `BlockedIdentity.device_id` field exists in schema. Three TODOs note that device-level blocking (granular per-device blocking within a multi-device identity) requires a device ID pairing infrastructure not yet built.
- **Gap:** All current blocking operates on `identity_id` only; `device_id` is always `None` in practice.
- **Severity:** P3 — Correct default behavior. Must be tracked before multi-device rollout.

---

## Section 3 — Deprecated / Non-Wired Code

### 3.1 `DeliveryStatus::Read` — Deprecated Variant

- **Location:** `core/src/message/types.rs:23`
- **Status:** `#[deprecated]` variant kept for backward-compatible deserialization from older peers. Processing path maps `Read` → `Delivered` behavior via `#[allow(deprecated)]` match.
- **Impact:** Zero-status architecture is correct; this is intentional. The variant must stay for the rolling upgrade window.
- **Severity:** INFO — Correct design decision. Can be removed post-beta when all peers are on v0.2.x.

### 3.2 `normalizeContactId` Deprecated in Android

- **Location:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2635`
- **Status:** `@deprecated Use canonicalContactId() for new code.` — delegates to `canonicalContactId()`.
- **Impact:** Internal Android-only. Zero callers in the codebase use the deprecated path (delegation is a shim).
- **Severity:** P3 — Can be removed in next cleanup sprint.

### 3.3 Duplicate Manager Accessors (`contacts_manager` / `contacts_store_manager`)

- **Location:** `core/src/lib.rs:1654-1679`
- **Issue:** Two separate accessor pairs exist for the same underlying store:
  - `contacts_manager()` — cfg-gated: non-wasm32 returns `Arc<ContactManager>` (bridge type); wasm32 returns `store::ContactManager`
  - `contacts_store_manager()` — always available, returns `store::ContactManager`
  - Same pattern for `history_manager` / `history_store_manager`
- **Impact:** Callers must know which version to use. `contacts_store_manager` was added to give wasm32 and test code a stable path. The `contacts_manager` bridge version is the UniFFI-facing API. **These serve different purposes but have confusing names.**
- **Severity:** P3 — Rename or consolidate in next API cleanup. Low functional risk.

### 3.4 `startReceiveLoop` WASM Export (Duplicate of Section 2.1)

See 2.1 above.

### 3.5 WASM Swarm — `start_swarm()` Returns Stub Error on wasm32

- **Location:** `core/src/transport/swarm.rs:1286-1290`
- **Status:** `start_swarm()` returns `Err("Swarm networking is not supported in WASM")` on wasm32. This is correct design — WASM uses `WasmTransport` (WebRTC + WebSocket relay) not libp2p TCP/QUIC.
- **Impact:** Zero: the correct transport path is `WasmTransport`. This is intentional and documented.
- **Severity:** INFO — Already documented. The stub error should be surfaced clearly to WASM callers that try to call `startSwarm` through the wrong path.

---

## Section 4 — Alpha Readiness Checklist

### 4.1 What IS alpha-ready (ships as-is)

- [x] **End-to-end encrypted messaging** — XChaCha20-Poly1305 with Ed25519 sender auth, AAD binding. 670+ passing tests.
- [x] **Contact block / blocked-only retention / block+delete cascade** — 3-state machine, 3 integration tests.
- [x] **Unblock restores hidden messages** — `unhide_messages_for_peer` validated in integration test.
- [x] **Ingress drop for blocked+deleted** — `receive_message()` returns `Err(Blocked)`.
- [x] **Cross-platform block wiring** — Android, iOS, WASM, CLI all call `blockAndDeletePeer()`.
- [x] **Single active device (WS13)** — `relay_custody.rs` relay-based dedup.
- [x] **DM notifications (WS14)** — `classify_notification()` wired on iOS; WASM binding present.
- [x] **Offline store-and-forward** — Outbox flushed on `PeerDiscovered`; `integration_relay_custody` passing.
- [x] **Delivery receipts** — `MessageType::Receipt` + `DeliveryStatus` end-to-end; `prepare_receipt()` present.
- [x] **Internet relay client** — `connect_to_relay_via_swarm()` with real `swarm.dial()`.
- [x] **Resume storm protection** — `RECONNECT_MAX_CONCURRENT=3` rate limiter.
- [x] **Zombie loop protection** — Inbox eviction high-water mark.
- [x] **Slow Loris protection** — `FRAME_READ_TIMEOUT=5s`, `FRAME_MAX_PAYLOAD=64KB`.
- [x] **Cover traffic API** — `prepare_cover_traffic()` wired on Android, iOS, WASM.
- [x] **WASM WebRTC transport** — Offer/answer/ICE candidate methods all present.
- [x] **WASM WebSocket relay** — Full connect/send/disconnect with buffered sends.

### 4.2 What needs a **known-issues note** in alpha release notes

| Issue | Note |
|-------|------|
| BLE transport (§2.3) | BLE is platform-native (Swift/Kotlin). Rust `core/src/transport/ble/` is protocol spec only — not in the active transport stack. iOS CoreBluetooth and Android BluetoothGatt handle scanning/advertising independently. |
| Onion routing (§2.5) | Privacy layer for v0.2.1 is per-message XChaCha20-Poly1305 + envelope signatures. Onion/multi-hop routing module exists but is not active. |
| FindMy wake-up (§2.4) | Protocol implementation is complete and tested but not wired to the runtime scheduler. Offline wake-up via FindMy is not active in v0.2.1. |
| Device-level blocking (§2.6) | Blocking is by identity (Blake3 hash of public key). Per-device granular blocking is not yet implemented. |
| WebRTC ICE trickle (§2.2) | ICE candidate exchange methods exist; no end-to-end WebRTC peer establishment test. WASM browser P2P requires manual testing. |
| IBLT large-diff (§1.2) | Sync sketch IBLT decodes correctly for typical peer windows (<20 differences). Diffs >~30 items in a single sync window will exceed IBLT capacity and fall back to full exchange. |

### 4.3 What MUST be done before alpha ships

| # | Item | Severity | Owner |
|---|------|----------|-------|
| A | Add `integration_contact_block`, `integration_e2e`, `integration_ironcore_roundtrip` to CI `check-core` step | **BLOCKER** | Copilot / CI |
| B | Document BLE transport as "protocol spec, not active" in `docs/TRANSPORT_ARCHITECTURE.md` | **P1** | Docs |
| C | Remove or properly deprecate `startReceiveLoop` WASM export | **P2** | Copilot |
| D | Add known-issues section to v0.2.1 release notes covering all items in §4.2 | **P1** | Release |
| E | Verify Firefox headless wasm-pack test passes on CI (Step 7 of WASM job) | **P1** | CI |

---

## Section 5 — CI Step Additions (Item A from §4.3)

The following lines should be added to `.github/workflows/ci.yml` in the `check-core` job's `Test` step (or as a new dedicated step):

```yaml
- name: Core Integration Tests (always-pass suite)
  run: |
    cargo test -p scmessenger-core --test integration_contact_block
    cargo test -p scmessenger-core --test integration_e2e
    cargo test -p scmessenger-core --test integration_ironcore_roundtrip
    cargo test -p scmessenger-core --test integration_registration_protocol
    cargo test -p scmessenger-core --test test_address_observation
    cargo test -p scmessenger-core --test test_mesh_routing
    cargo test -p scmessenger-core --test test_persistence_restart
```

These 7 test files have **0 ignored tests** and **0 failures** — they are safe to add to CI unconditionally.

---

## Section 6 — Duplicate / Overlapping Functionality

| Area | Overlap | Recommended Resolution |
|------|---------|----------------------|
| `contacts_manager()` vs `contacts_store_manager()` | Same underlying store, two accessors with cfg-split return types | Rename `contacts_store_manager` → `contacts_raw()` to signal it's the non-bridge path; document the distinction |
| `list_blocked_peers()` vs `list_blocked_peers_raw()` | Both return blocked list; bridge type vs store type | Keep both — they serve different call sites (UniFFI mobile bridge vs WASM). Add doc-comment cross-reference |
| `history_manager()` vs `history_store_manager()` | Same pattern as contacts | Same resolution — rename for clarity |
| `canonicalContactId()` vs `PeerIdValidator.normalize()` (Android) | Both normalize peer IDs but `canonicalContactId` uses contact lookup; `PeerIdValidator.normalize` lowercases hex | These are intentionally different; add doc-comment explaining when to use each |
| `resolve_identity()` vs `resolve_to_identity_id()` | Both resolve any ID format; former is broader | `resolve_to_identity_id` is the stricter form (always returns identity_id). Document clearly — these are NOT duplicates |

---

## Section 7 — Dead Code / Unused Exports

| Item | Location | Action |
|------|----------|--------|
| `relay_reservation_multiaddr` | `core/src/transport/swarm.rs:102` | Private fn, unused — 16 warnings on WASM build include this. Delete or use. **P3** |
| `storage_path` field in `MeshSettingsManager` | `wasm/src/lib.rs:103` | Dead field — `#[warn(dead_code)]` on WASM build. Delete. **P3** |
| `FindMyBeaconManager` | `core/src/relay/findmy.rs` | Exported but never instantiated in runtime. Mark `#[allow(dead_code)]` or document as "future activation". **P2** |
| `force_state_for_test` | `core/src/store/relay_custody.rs:1407` | Test-only helper not in `#[cfg(test)]` block. Move inside `#[cfg(test)]`. **P3** |

---

## Section 8 — Security Notes for Alpha

| Item | Status |
|------|--------|
| XChaCha20-Poly1305 envelope encryption | ✅ Production-grade |
| Ed25519 sender authentication + AAD binding | ✅ Production-grade |
| Replay prevention (inbox dedup + eviction high-water) | ✅ In place |
| Slow Loris / frame flooding | ✅ `FRAME_READ_TIMEOUT=5s` + `FRAME_MAX_PAYLOAD=64KB` |
| Resume storm | ✅ `RECONNECT_MAX_CONCURRENT=3` |
| Intermediate key material zeroization | ✅ Zeroize-on-Drop for `shared_secret_bytes`, `ephemeral_bytes`, `nonce_bytes` |
| FindMy XOR-keystream | ⚠️ Doc-commented as "experimental, not production-grade" — **not active** in alpha, no risk |
| Onion routing | ⚠️ Not active — see §2.5. Transport encryption covers all real messages |
| TLS/noise on relay connections | ✅ libp2p Noise protocol on all TCP/QUIC connections |

---

*Document generated: 2026-03-31 from live codebase analysis.*
*Next review: prior to v0.2.1 beta tag.*
