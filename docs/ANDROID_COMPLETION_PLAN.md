# SCMessenger Android Completion Plan
## Strategic Roadmap for Play Store Deployment

**Created:** 2026-05-04
**Updated:** 2026-05-04 (LoC-only, sprint-optimized)
**Target:** Production-ready Android app for global deployment
**Scope:** Android-first completion, then leverage lessons for iOS/CLI/WASM

---

## Executive Summary

The Android app is ~75% functionally complete with solid architecture (Hilt DI, Compose UI, UniFFI bindings, foreground service, multi-transport BLE/WiFi). The remaining 25% requires **core wiring** (Rust↔Kotlin integration), **Polish & Stability** (ANR fixes, notification reliability), **Play Store Compliance**, and **Scale Preparation**.

**Total LoC Estimate:** ~2,900 - 3,500 across 6 sprints

---

## Sprint 1: Build & Bindings

### S1-T1: Fix Android Build 🔴 P0
**Status:** TODO
**Files:** `android/app/build.gradle.kts`, `android/gradle.properties`
**LoC:** ~50
**Depends:** None
**Actions:**
- Run `./gradlew assembleDebug -x lint --quiet` and capture all failures
- If Rust NDK build fails: verify `cargo-ndk` targets (`aarch64-linux-android`, `x86_64-linux-android`)
- If Kotlin compilation fails: fix import/type errors
- If UniFFI bindings missing: regenerate with `cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin`
- Verify `./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.RoleNavigationPolicyTest"` passes
**Verification:** `./gradlew assembleDebug -x lint` succeeds

### S1-T2: UniFFI Binding Verification
**Status:** TODO
**Files:** `core/target/generated-sources/uniffi/kotlin/uniffi/api/api.kt`, Android `MeshRepository.kt`
**LoC:** ~150
**Depends:** S1-T1
**Actions:**
- Run `cargo run -p scmessenger-core --features gen-bindings --bin gen_kotlin`
- Compare generated `api.kt` against all 34 files importing `uniffi.api.*`
- For each missing/renamed function: determine if Android stub exists or Rust needs implementation
- Create gap analysis: `docs/UNIFFI_GAP_ANALYSIS.md`
**Verification:** All `uniffi.api.*` calls in Android resolve to generated bindings

### S1-T3: Core Integration Audit
**Status:** TODO
**Files:** All Android files calling `uniffi.api.*` (34 files)
**LoC:** ~100
**Depends:** S1-T2
**Actions:**
- Enumerate all UniFFI functions called across Android codebase
- For each function: verify it exists in Rust `core/src/**/*.rs`
- Mark as: ✅ Implemented, ⚠️ Stub needed, 🔴 Missing in Rust
- Document gaps in `docs/UNIFFI_GAP_ANALYSIS.md`
**Verification:** Gap analysis document complete

### S1-T4: CI Pipeline Setup
**Status:** TODO
**Files:** `.github/workflows/android.yml` (create)
**LoC:** ~100
**Depends:** S1-T1
**Actions:**
- Create GitHub Actions workflow for Android
- Gate: `cargo check --workspace` → `./gradlew assembleDebug -x lint` → `./gradlew :app:testDebugUnitTest`
- Add Android NDK setup (`actions/cache`, `actions/setup-java`)
- Badge status in README
**Verification:** CI workflow runs and passes on PR

---

## Sprint 2: Core Wiring

### S2-T1: SwarmBridge Wiring
**Status:** TODO
**Files:** `MeshRepository.kt`, `TransportManager.kt`
**LoC:** ~200
**Depends:** S1-T3 (UniFFI gap analysis)
**Actions:**
- Locate `swarmBridge` initialization in `MeshRepository`
- Wire `swarmBridge.dial(peerId)` for INTERNET transport
- Verify `TransportManager.sendData()` falls back to INTERNET when BLE/WiFi unavailable
- Test peer connection: start service → verify mesh status shows peer count > 0
**Verification:** Two devices can discover each other over internet transport

### S2-T2: TopicManager Integration
**Status:** TODO
**Files:** `TopicManager.kt`, `MeshRepository.kt`
**LoC:** ~150
**Depends:** S2-T1
**Actions:**
- Verify `meshRepository.subscribeTopic(topic)` calls actual gossipsub
- Test topic subscription: subscribe to `/scmessenger/discovery/v1` → receive peer announcements
- Wire `TopicManager` initialization into `MeshRepository.onMeshServiceStarted()`
- Document default topics: global, discovery, relay
**Verification:** Discovery topic receives peer announcements within 30s

### S2-T3: Message Deduplication
**Status:** TODO
**Files:** `SmartTransportRouter.kt`, `TransportManager.kt`
**LoC:** ~150
**Depends:** S2-T1
**Actions:**
- Verify `SmartTransportRouter.checkAndRecordMessage()` called on send
- Verify cross-transport dedup: BLE delivery followed by WiFi delivery → single notification
- Add 300s TTL to dedup cache (per spec)
- Test: send message over BLE → verify same message not duplicated over WiFi
**Verification:** Cross-transport duplicate detection works

### S2-T4: Relay Bootstrap Infrastructure
**Status:** TODO
**Files:** `MeshRepository.kt`, `BootstrapSource` implementations
**LoC:** ~200
**Depends:** S2-T1
**Actions:**
- Implement `EnvironmentBootstrapSource` (reads `SC_BOOTSTRAP_NODES` env var)
- Add static fallback nodes (QUIC prioritized over TCP per spec)
- Add relay health monitoring: mark node unreachable after 3 failures
- Implement automatic failover to next healthy node
- Wire `MeshVpnService` as optional persistent connection option
**Verification:** Relay connection succeeds within 10s, survives single node failure

### S2-T5: MeshVpnService Wiring
**Status:** TODO
**Files:** `MeshVpnService.kt`, `SettingsViewModel.kt`
**LoC:** ~100
**Depends:** S2-T4
**Actions:**
- Wire VPN toggle in `PowerSettingsScreen`
- Implement VPN lifecycle: start → configure tunnel → connect → maintain
- Handle VPN permission request (`android.net.VpnService`)
- Test: enable VPN → verify persistent mesh connectivity
**Verification:** VPN mode maintains connection through network switch (WiFi↔Cellular)

---

## Sprint 3: BLE Completion

### S3-T1: BLE→Core Message Forwarding
**Status:** TODO
**Files:** `BleL2capManager.kt`, `BleGattClient.kt`, `BleGattServer.kt`
**LoC:** ~150
**Depends:** S1-T3
**Actions:**
- Wire received BLE data → Core message parser → `MeshEventBus.messageReceived`
- Verify `MessageRecord` construction from BLE payload
- Test: send message over BLE → verify it appears in conversation
- Handle BLE data fragmentation (messages > MTU)
**Verification:** BLE messages appear in chat history

### S3-T2: BLE Identity Handshake
**Status:** TODO
**Files:** `BleGattClient.kt`, `BleGattServer.kt`
**LoC:** ~200
**Depends:** S3-T1
**Actions:**
- Define BLE identity exchange protocol (exchange public keys over GATT)
- Implement handshake state machine: Initiating → KeyExchange → Established
- Store exchanged identity in contact manager
- Test: two devices meet over BLE → verify mutual identity exchange
**Verification:** After BLE handshake, both contacts show each other's public key

### S3-T3: BLE Quota→AutoAdjust Integration
**Status:** TODO
**Files:** `BleQuotaManager.kt`, `AndroidPlatformBridge.kt`, `MeshForegroundService.kt`
**LoC:** ~100
**Depends:** S3-T1
**Actions:**
- Wire `BleQuotaManager.currentCount` → `AndroidPlatformBridge.reportBleScanCount()`
- Report to `AutoAdjustEngine` for profile adjustment
- Test: exhaust BLE scan quota → verify reduced scan frequency
**Verification:** Scan frequency adapts when quota approaches limit

### S3-T4: BLE Graceful Degradation
**Status:** TODO
**Files:** `BleScanner.kt`, `BleAdvertiser.kt`, `TransportManager.kt`
**LoC:** ~100
**Depends:** S3-T1
**Actions:**
- Implement `TransportManager.handleBleFailure()` recovery path
- Prioritize WiFi Aware/Direct when BLE degrades
- Add `attemptBleRecovery()` after cooldown period
- Test: simulate BLE failure → verify WiFi fallback → verify BLE recovery
**Verification:** BLE failure triggers WiFi escalation; BLE recovery resumes scanning

---

## Sprint 4: Polish & Stability

### S4-T1: ANR Elimination 🔴 P0
**Status:** TODO
**Files:** `MeshRepository.kt`, `MeshForegroundService.kt`, `MeshServiceViewModel.kt`
**LoC:** ~300
**Depends:** S2-T1
**Actions:**
- Audit all FFI calls: ensure every `uniffi.api.*` call is on IO dispatcher
- Implement retry cap: max 10 attempts, exponential backoff (1s → 2s → 4s → max 30s)
- Fix message ID tracking: use UUID-based dedup instead of mutable state
- Handle `JobCancellationException` gracefully (don't re-throw to main thread)
- Test: rapid message sends → verify no ANR
**Verification:** No ANR events during 10-minute stress test

### S4-T2: Notification Reliability 🔴 P0
**Status:** TODO
**Files:** `NotificationHelper.kt`, `MeshForegroundService.kt`
**LoC:** ~100
**Depends:** S4-T1
**Actions:**
- Test all 5 notification channels on Android 12, 13, 14
- Verify reply-from-notification with `RemoteInput`
- Test DND suppression (respects `NotificationManager.shouldSuppressNotification()`)
- Test foreground app suppression (don't notify for current conversation)
- Test: known contact → DM channel; unknown → DM Request channel
**Verification:** All notification scenarios in test matrix pass

### S4-T3: Data Persistence & Recovery
**Status:** TODO
**Files:** `MeshRepository.kt`, `PreferencesRepository.kt`, `MeshApplication.kt`
**LoC:** ~150
**Depends:** S1-T3
**Actions:**
- Verify identity cache: cold start → check SharedPreferences first → load from cache
- Test: delete app data → restore from backup code → verify identity intact
- Implement outbox replay: crash recovery replays pending messages
- Verify offline message queue persists across app restarts
**Verification:** Messages in outbox survive app kill/restart

### S4-T4: Identity Cache Cold Start
**Status:** TODO
**Files:** `MeshRepository.kt`, `MainViewModel.kt`
**LoC:** ~50
**Depends:** S4-T3
**Actions:**
- Verify `getIdentityInfoNonBlocking()` returns cached data immediately
- Verify no 30-60s "Unavailable" gap on cold start
- Test: kill app → restart → verify identity shown within 1s
**Verification:** Identity UI loads within 1s of cold start

---

## Sprint 5: Play Store Compliance

### S5-T1: Privacy Compliance
**Status:** TODO
**Files:** `AndroidManifest.xml`, `strings.xml`, privacy policy HTML
**LoC:** ~50
**Depends:** S4-T1
**Actions:**
- Verify `privacy_policy_url` in manifest points to hosted policy
- Create privacy policy covering: data collection, encryption, third parties
- Verify `neverForLocation` flags on BLE/WiFi permissions
- Test: fresh install → verify permissions requested correctly
**Verification:** Play Store Data Safety form can be completed accurately

### S5-T2: Crash Reporting Integration
**Status:** TODO
**Files:** `MeshApplication.kt`, `build.gradle.kts`
**LoC:** ~100
**Depends:** S4-T1
**Actions:**
- Integrate Firebase Crashlytics (or equivalent)
- Verify ANR reporting enabled
- Test: trigger exception → verify appears in console within 5 minutes
- Add custom keys: `app_version`, `device_android_version`
**Verification:** Crashes appear in console within 5 minutes of occurrence

### S5-T3: Alpha Branding
**Status:** TODO
**Files:** `strings.xml`, `OnboardingScreen.kt`
**LoC:** ~20
**Depends:** None
**Actions:**
- Verify consent gate clearly states "Alpha Software"
- Add version suffix to app name (e.g., "SCMessenger α")
- Verify Play Store listing explains alpha status
**Verification:** Users understand app is alpha before installing

---

## Sprint 6: Feature Parity & Release

### S6-T1: Identity Backup/Restore
**Status:** TODO
**Files:** `IdentityScreen.kt`, `MainViewModel.kt`, `OnboardingScreen.kt`
**LoC:** ~200
**Depends:** S4-T3
**Actions:**
- Implement QR code generation from identity backup
- Implement QR code scanning for identity restore
- Handle "Import Identity" flow in onboarding
- Test: backup → wipe app → restore → verify same identity
**Verification:** Identity survives backup/restore cycle

### S6-T2: Contact QR Sharing
**Status:** TODO
**Files:** `AddContactScreen.kt`, `ContactDetailScreen.kt`
**LoC:** ~100
**Depends:** S6-T1
**Actions:**
- Generate QR code from contact's public key
- Scan QR code to add contact (prefill peerId, publicKey, nickname)
- Test: generate QR on Device A → scan on Device B → verify contact added
**Verification:** QR sharing adds contact with correct public key

### S6-T3: Deep Link Invite
**Status:** TODO
**Files:** `MainActivity.kt`, `AddContactScreen.kt`
**LoC:** ~50
**Depends:** S6-T1
**Actions:**
- Verify `scmessenger://add` deep link populates AddContact screen
- Implement `https://scmessenger.net/add` App Links verification
- Test: tap invite link → verify prefill of peerId/publicKey
**Verification:** Deep link correctly prefills contact fields

### S6-T4: Final Integration Test
**Status:** TODO
**Files:** All Android source
**LoC:** ~100
**Depends:** All previous sprints
**Actions:**
- Run full test suite: `./gradlew :app:testDebugUnitTest`
- Manual test: create identity → add contact → send message → receive reply
- Test: kill app → restart → verify no data loss
- Test: airplane mode → send message → turn on airplane → verify retry
**Verification:** All test suites pass; manual E2E flow works

---

## Dependency Graph

```
S1-T1 (Build)
    └── S1-T2 (Bindings) → S1-T3 (Audit) → S2-T1 (SwarmBridge) → S2-T2 (Topics) → S2-T3 (Dedup)
                                                                      ↓
                                                               S2-T4 (Relay) → S2-T5 (VPN)
                                                                      ↓
S3-T1 (BLE→Core) ← S1-T3
    ↓
S3-T2 (BLE Identity)
    ↓
S3-T3 (Quota)
    ↓
S3-T4 (BLE Degradation) ← S3-T1

S4-T1 (ANR) ← S2-T1
    ↓
S4-T2 (Notifications)
    ↓
S4-T3 (Persistence) ← S1-T3
    ↓
S4-T4 (Identity Cache) ← S4-T3

S5-T1 (Privacy) ← S4-T1
    ↓
S5-T2 (Crash Reporting)
    ↓
S5-T3 (Alpha Branding) ← None (independent)

S6-T1 (Identity Backup) ← S4-T3
    ↓
S6-T2 (QR Sharing) ← S6-T1
    ↓
S6-T3 (Deep Link) ← S6-T1
    ↓
S6-T4 (Final Test) ← All
```

---

## Pre-Merge Gate (Every PR)

```bash
cargo check --workspace
cargo test --workspace --no-run
./gradlew assembleDebug -x lint
./gradlew :app:testDebugUnitTest
```

## Pre-Release Gate (v0.3.0)

```bash
cargo test --workspace
./gradlew assembleRelease
# Play Store internal testing track
# 14-day soak + 100 device pre-launch report
```

---

## LoC Summary by Sprint

| Sprint | Tasks | Total LoC |
|--------|-------|----------|
| Sprint 1 | 4 | ~400 |
| Sprint 2 | 5 | ~800 |
| Sprint 3 | 4 | ~550 |
| Sprint 4 | 4 | ~600 |
| Sprint 5 | 3 | ~170 |
| Sprint 6 | 4 | ~450 |
| **Total** | **24** | **~2,970** |

*(Range: 2,900 - 3,500 depending on gaps found during audit)*

---

## Risk Register

| Risk | Prob. | Impact | Mitigation |
|------|-------|--------|------------|
| Rust core breaks bindings | Medium | High | Interface tests in S1-T3 |
| ANR persists | Medium | High | Main thread audit in S4-T1 |
| BLE quota exhaustion | High | Medium | Aggressive testing in S3 |
| Play Store rejects alpha | Low | Medium | Clear alpha branding |
| Relay servers offline | Medium | High | Health monitoring in S2-T4 |

---

## Next Steps After Android

1. **iOS:** Leverage Android learnings for Swift implementation
2. **CLI:** WASM daemon integration for browser client
3. **Scale:** Multi-region relay infrastructure
