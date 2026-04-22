# SCMessenger Production Roadmap — Prioritized Completion Plan

## Platform Readiness Scores (as of 2026-04-21)

| Platform | Score | Status | Key Blocker |
|----------|-------|--------|-------------|
| **Rust Core / CLI** | 95% | Compiles clean | Integration test fixes (45 type errors) |
| **Android** | 55% | 30+ Kotlin compile errors | Suspend/non-suspend boundaries, Material 3 migration |
| **WASM** | 40% | 28 compile errors on wasm32 | Core crate cfg-gating (`websocket.rs`, `swarm.rs`) |
| **iOS** | 70% | Source-complete, needs macOS build | No `.a` binary, stale field build |

---

## P0 Blockers — Roadmap to Complete Build

### 1. Android Compilation Fixes (P0 — 30+ errors)
**Estimated effort:** 2–4 hours
**Files:** `android/app/src/main/java/com/scmessenger/android/`

| # | Error | File | Fix |
|---|-------|------|-----|
| 1 | Suspend function in non-suspend context | `MeshRepository.kt:260, 2368, 2912` | Mark caller as `suspend` or wrap in `lifecycleScope.launch` |
| 2 | Suspend function in non-suspend context | `TransportManager.kt:91, 116, 367, 391, 436, 460` | Same as above |
| 3 | Suspend function in non-suspend callback | `BleScanner.kt:164, 174, 188, 206, 207, 272, 319, 484, 505` | Use `suspend` lambdas or `CoroutineScope.launch` |
| 4 | `String.ifEmpty` lambda with `_` parameter | `MeshRepository.kt:6902` | `it[1].ifEmpty { it[2] }` (remove `_` param) |
| 5 | `async` without CoroutineScope receiver | `MeshRepository.kt:6918` | Add `coroutineScope { async { ... } }` |
| 6 | `await()` scope issues | `MeshRepository.kt:6935` | Same as above |
| 7 | Material 2 `SwipeToDismiss` in Material 3 project | `ContactsScreen.kt:278–312` | Add `androidx.compose.material:material` import OR migrate to `SwipeToDismissBox` |
| 8 | `getMeshStats` unresolved | `MeshRepository.kt:6968` | Check Rust UniFFI bindings — may need to expose `mesh_stats()` in Rust core |

**Verification:** `./gradlew :app:compileDebugKotlin` must pass with zero errors.

### 2. WASM Core Crate cfg-gating (P0 — 28 errors)
**Estimated effort:** 1–2 hours
**Files:** `core/src/transport/websocket.rs`, `core/src/transport/swarm.rs`, `core/src/transport/mod.rs`, `core/src/lib.rs`

| # | Error | File | Fix |
|---|-------|------|-----|
| 1 | `tokio_tungstenite` unavailable on wasm32 | `core/src/transport/websocket.rs` | Gate entire module with `#[cfg(not(target_arch = "wasm32"))]` in `mod.rs` |
| 2 | `RankedRoute` / `MultiPathDelivery` not in scope | `core/src/transport/swarm.rs:~25` | Remove `#[cfg(not(target_arch = "wasm32"))]` from the import OR gate the functions that use them |
| 3 | `is_peer_blocked` on `Weak<IronCore>` | `core/src/transport/swarm.rs:3732` | Upgrade weak ref: `core_handle.upgrade().map(|c| c.is_peer_blocked(...))` |
| 4 | `into_client_request` not found | `core/src/transport/websocket.rs:66` | Gate the import behind `#[cfg(not(target_arch = "wasm32"))]` |
| 5 | Type annotations / borrow checker | `core/src/transport/websocket.rs`, `core/src/lib.rs` | Resolve inference and borrow issues in WASM path |

**Verification:** `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` must pass.

### 3. Rust Core Integration Tests (P0 — 45 type errors)
**Estimated effort:** 2–3 hours
**Files:** `core/src/` test modules

- 45 type annotation errors prevent `cargo test --workspace` from passing
- rlib format errors and rustc crashes on `test_address_observation.rs`
- 8 ignored tests in `docs/TODO_TESTS.md` (drift sync, NAT transport)

**Verification:** `cargo test --workspace` must pass with zero failures.

### 4. Android Pixel 6a Runtime Stability (P0 Runtime)
**Estimated effort:** 3–5 hours
**Source:** `HANDOFF/backlog/ANDROID_PIXEL_6A_AUDIT_2026-04-17.md`

- Contacts missing (0 loaded despite persistence)
- All 4 bootstrap relay nodes failing
- ANR crashes (main-thread blocking >5s)
- BLE scan failures (`SCAN_FAILED_ALREADY_STARTED`)

**Verification:** Physical device testing on Google Pixel 6a with Android 16.

### 5. Forward Secrecy Completion (P0 Security)
**Estimated effort:** 8–12 hours
**Files:** `core/src/crypto/ratchet.rs`, `core/src/crypto/session_manager.rs`

- ECDH ephemeral encryption exists but no ratcheting
- Double Ratchet protocol partially implemented (core ~600 LoC done)
- Session management (~400 LoC) and message encryption layer (~300 LoC) still needed
- Cross-platform integration (~200 LoC) pending

**Verification:** `cargo test --lib crypto::ratchet` must pass.

### 6. Anti-Abuse Controls (P0 Security)
**Estimated effort:** 6–10 hours
**Files:** `core/src/abuse/` (new directory)

- Token-bucket rate limiting exists
- Missing: peer reputation scoring, spam detection, abuse pattern recognition, automatic blocking
- Needs `core/src/abuse/` module with sled-backed persistence

**Verification:** `cargo test --lib abuse` must pass with new test coverage.

### 7. Formal Verification Harness (P0 Build)
**Estimated effort:** 4–6 hours
**Files:** New harness scripts

- `cargo-kani` installation and baseline proof
- `proptest` property-based tests on `core/src/transport/address.rs`
- CI integration for automated verification

**Verification:** `cargo kani` and `cargo test --proptest` pass in CI.

---

## P1 Items — Polish & Feature Activation

### P1.1 Mycorrhizal Routing Activation
- 10 files in `core/src/routing/` fully unit-tested but **dormant** (not wired to production)
- Needs wiring into `TransportManager`, swarm dispatch, message delivery
- Estimated: 3–4 hours

### P1.2 iOS Notification Verification
- `NotificationManager.swift` exists, needs physical device testing
- Permission flow, delivery in all app states, tap routing, background processing
- Estimated: 2–3 hours (requires macOS + iPhone)

### P1.3 WASM Notification Verification
- Browser notification code exists, needs cross-browser testing
- Service worker integration (currently simulated), permission flow, click handling
- Estimated: 2–3 hours

### P1.4 Identity Backup Encryption
- Backup stores `secret_key_hex` in **plaintext JSON**
- Needs passphrase-based encryption (Argon2 + AES-256-GCM)
- Estimated: 4–6 hours

---

## P2 Items — Nice to Have

- STUN/TURN integration for NAT traversal
- Mesh health monitoring dashboard
- Bandwidth-adaptive compression
- Message search indexing
- Graceful shutdown (pending message drain, sled flush)
- Android permission request loop fix (`AND-PERMISSION-001`)
- Docker simulation verification (`TEST-ENV-001`)

---

## Orchestrator Queue Management

**Current state:** `HANDOFF/todo/` is empty, `HANDOFF/backlog/` has 11 items.

**Immediate action:** Populate `todo/` with top 2 P0 blockers:
1. **P0_ANDROID_008_Kotlin_Compile_Fixes** — Fix 30+ Kotlin errors
2. **P0_WASM_003_Core_Cfg_Gating** — Fix 28 wasm32 compilation errors

**Next cycle:** After these compile fixes pass:
3. P0_BUILD_003_Core_Test_Stabilization
4. ANDROID_PIXEL_6A_AUDIT_2026-04-17 runtime fixes
5. P0_SECURITY_002_Forward_Secrecy_Implementation
6. P0_SECURITY_003_Anti_Abuse_Controls
7. P1_CORE_003_Mycorrhizal_Routing_Activation
8. P1_IOS_002_NOTIFICATION_VERIFICATION
9. P1_WASM_002_NOTIFICATION_VERIFICATION
10. ORCHESTRATOR_001_Full_Ecosystem_Integration
11. AGENT_GUIDANCE_Philosophy_Enforcement

---

## Critical Finding: Done Directory Cleanup

**6 items were incorrectly in `done/` and have been moved to `backlog/`:**
- `P1_CORE_003_Mycorrhizal_Routing_Activation` — status was "Dormant (Not Wired)"
- `P1_IOS_002_NOTIFICATION_Verification` — status was "Open"
- `P1_WASM_002_NOTIFICATION_Verification` — status was "Open"
- `P0_BUILD_001_Core_Integration_Test_Fix` — status was "Open"
- `P0_BUILD_002_Integration_Test_Repair` — status was "Open"
- `P0_BUILD_003_Core_Test_Stabilization` — status was "P0 BLOCKER"
- `P0_AUDIT_001_Retroactive_Task_Verification` — status was "Open"

**7 duplicate/redundant files removed** from `done/` and `backlog/`.

---

## Verification Checklist for "Complete Build"

- [ ] `cargo check --workspace` passes with zero errors
- [ ] `cargo test --workspace` passes with zero failures
- [ ] `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes
- [ ] `./gradlew :app:compileDebugKotlin` passes with zero errors
- [ ] `./gradlew :app:assembleDebug` produces APK
- [ ] iOS builds on macOS with Xcode 15.2+
- [ ] All 8 transport paths active (BLE, WiFi Aware, WiFi Direct, mDNS, relay, WebSocket, TCP, internet)
- [ ] Forward secrecy ratchet active on all sessions
- [ ] Anti-abuse controls active (rate limiting + reputation + spam detection)
- [ ] Audit logging system captures security events
- [ ] Graceful shutdown drains pending messages and flushes sled