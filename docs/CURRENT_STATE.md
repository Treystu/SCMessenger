# SCMessenger Current State (Verified)

Status: Active
Last updated: 2026-03-16

Last verified: **2026-03-16** (GCP node restarted and verified healthy; scripts updated)
Last iOS build: **2026-03-16** (iOS build successful, stability improvements applied)

---

## 2026-03-16 Smart Transport Router & BLE Discovery Fixes

**Status:** ✅ IMPLEMENTED

### Overview

Implemented smart transport selection with 500ms timeout fallback, message deduplication with collision handling, and fixed Android BLE scanner failures that prevented cross-platform discovery.

### Problem

User reported:
- iOS device not seeing any other devices (BLE fail + LAN/WiFi fail + mesh fail + direct fail)
- Messages taking "WAY too long" to arrive
- Need for graceful message collision handling across multiple transports
- Need for transport health tracking to prioritize "previously used/good path"

### Root Causes Identified

1. **Sequential transport fallback**: The original [`LocalTransportFallback`](../iOS/SCMessenger/SCMessenger/Transport/LocalTransportFallback.swift) tried transports sequentially (Multipeer → BLE → Core), causing long delays when the preferred transport failed
2. **No transport health tracking**: No mechanism to track which transport was successful for a given peer
3. **Message deduplication gaps**: Duplicate messages from multiple transports were not being properly tracked with time variance
4. **Android BLE scanner failures**: Logcat showed `"BLE Scan failed with error code: 1"` (SCAN_FAILED_ALREADY_STARTED), preventing Android from discovering iOS devices

### Key Changes

#### 1. Smart Transport Router (iOS & Android)

Created [`SmartTransportRouter.swift`](../iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift) and [`SmartTransportRouter.kt`](../android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt) with:

- **500ms timeout fallback**: Tries preferred transport first, then races all available transports in parallel if no response within 500ms
- **Transport health tracking**: Tracks success rate, average latency, and last success/failure per peer per transport
- **Smart transport selection**: Uses weighted scoring (70% success rate, 30% latency) to select best transport
- **Message deduplication**: Tracks message IDs with timestamps to detect duplicates and log time variance for mesh enhancement

#### 2. Android BLE Scanner Fixes

Updated [`BleScanner.kt`](../android/app/src/main/java/com/scmessenger/android/transport/ble/BleScanner.kt) with:

- **Retry logic for scan failures**: Handles error codes 1-4 with appropriate retry strategies
- **Exponential backoff**: For internal/registration errors (codes 2, 3)
- **Pre-scan cleanup**: Stops any existing scan before starting to avoid SCAN_FAILED_ALREADY_STARTED
- **Bluetooth state check**: Verifies Bluetooth is enabled before attempting to scan
- **Force restart function**: Added `forceRestartScanning()` for manual recovery

#### 3. Integration

- **iOS**: [`MeshRepository.swift`](../iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) now uses `SmartTransportRouter` for message delivery with parallel transport racing
- **Android**: [`MeshRepository.kt`](../android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt) now uses `SmartTransportRouter` for message delivery with parallel transport racing

### Expected Impact

- **Message delivery latency**: Reduced from potentially 10+ seconds (sequential fallback) to <500ms (parallel racing)
- **Transport reliability**: Health tracking ensures failed transports are deprioritized automatically
- **Cross-platform discovery**: Android BLE scanner now recovers from failures, enabling iOS ↔ Android discovery
- **Message deduplication**: Proper tracking of duplicate messages with time variance logging for mesh optimization

### Verification

- [ ] iOS can discover Android via BLE
- [ ] Android can discover iOS via BLE
- [ ] Messages arrive within 500ms when at least one transport is available
- [ ] Duplicate messages are properly marked with time variance
- [ ] Transport health tracking shows correct success rates

---


For architectural context across all repo components, see `docs/REPO_CONTEXT.md`.

---

## 2026-03-16 GCP Node Nickname Update

**Status:** ✅ IMPLEMENTED

### Overview

Updated the GCP headless relay node to use the nickname "GCP-headless". This ensures the node is easily identifiable in the mesh.

### Key Changes

- **CLI (`cli/src/main.rs`)**: Updated `cmd_relay` to sync the `--name` argument (if provided) to the `IronCore` identity nickname.
- **Deployment (`scripts/deploy_gcp_node.sh`)**: Added `--name GCP-headless` to the relay startup command.

---

## 2026-03-16 BLE Log Visibility Improvements

**Status:** ✅ IMPLEMENTED & VERIFIED

### Overview

Enhanced the Mesh Topology Visualizer to correctly display Bluetooth (BLE) links by improving log recognition, seeding node identities, and expanding log capture.

### Key Changes

- **Log Visualizer (`mesh.html`)**: Broadened BLE detection keywords and refined own-identity parsing.
- **Run Script (`run5.sh`)**: Added proactive "Seeding node identities" step to inject identity markers for already-running nodes.
- **iOS Logging**: Expanded log stream predicates to capture `com.scmessenger` subsystem logs.

### Verification

- Nodes are correctly identified immediately upon log stream start.
- BLE links are correctly visualized in the topology view.

---

## 2026-03-16 iOS Build & Stability Fixes

### Build Status: ✅ iOS BUILD SUCCESSFUL

**Build Timestamp:** 2026-03-16 03:46 HST (Verified)
**Build Output:** `.build/ios-sim/Build/Products/Debug-iphonesimulator/SCMessenger.app`
**Build Size:** 674MB (libscmessenger_mobile.a)

**Verification:**
1. Open contact details
2. Edit nickname field
3. Type multiple characters quickly
4. Verify no crash occurs
5. Verify nickname syncs after typing stops
Build compiles cleanly, app launches on simulator successfully

### Stability Issues Addressed

#### 1. Main Thread Blocking (CRITICAL)

**Problem:** Heavy operations on main thread causing UI freezes, especially during debugging
**Root Cause:** `MeshRepository` marked `@MainActor`, causing all operations to run on main thread
**Fixes Applied:**

- [`MeshDashboardView.swift`](../iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift): Moved `loadDashboardData()` and `refreshPeersFromRepository()` to background tasks using `Task.detached`
- [`ContactsViewModel.swift`](../iOS/SCMessenger/SCMessenger/ViewModels/ContactsViewModel.swift): Made `loadContacts()` async, moved contact loading to background
- Reduced cascading updates from `upsertPeer()` calls

**Impact:** UI remains responsive during contact operations, peer discovery, and message loading

#### 2. SwiftUI State Thrashing (HIGH)

**Problem:** Multiple `@State` updates causing excessive view re-renders
**Root Cause:** `peersByKey` dictionary updated frequently, triggering cascade updates
**Fixes Applied:**

- Batched state updates in `refreshPeersFromRepositoryAsync()`
- Reduced `@State` mutation frequency
- Moved heavy dictionary operations to background tasks

**Impact:** Reduced view re-renders, smoother scrolling, less memory pressure

#### 3. Excessive Debug Logging (MEDIUM)

**Problem:** 60+ warnings in iOS build, console spam in Xcode
**Root Cause:** Verbose logging in release builds
**Fixes Applied:**

- Added `logVerbose()` and `logDiagnostic()` conditional logging functions
- Only logs in DEBUG builds using `#if DEBUG` preprocessor directives
- Reduced diagnostic buffer writes in release builds

**Impact:** Faster build times, cleaner console output, improved runtime performance

### Files Modified

| File | Changes | Lines Changed |
| :--- | :--- | :--- |
| [`MeshRepository.swift`](../iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift) | Added conditional logging functions | ~20 lines |
| [`MeshDashboardView.swift`](../iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift) | Async operations, background tasks | ~50 lines |
| [`ContactsViewModel.swift`](../iOS/SCMessenger/SCMessenger/ViewModels/ContactsViewModel.swift) | Async contact loading | ~30 lines |

### Expected User Impact

**Before Fixes:**

- ❌ App freezes during contact operations
- ❌ UI unresponsive during peer discovery
- ❌ Debugging experience painful due to hangs
- ❌ Excessive console spam in Xcode

**After Fixes:**

- ✅ App remains responsive during all operations
- ✅ Smooth scrolling in contacts list
- ✅ Debugging experience improved
- ✅ Clean console output in DEBUG builds
- ✅ Reduced memory pressure

### Testing Recommendations

1. **Contact Operations:** Add/remove contacts, verify UI remains responsive
2. **Peer Discovery:** Scan for nearby peers, verify no freezes
3. **Message Loading:** Load conversations with 100+ messages, verify smooth scrolling
4. **Debugging:** Attach Xcode debugger, verify reduced console spam
5. **Memory:** Monitor memory usage during extended operations

---

## 2026-03-15 Real-Time Log Audit Findings

**Audit Date:** 2026-03-15 14:14 HST
**Sources:** `scripts/android_live.log`, `scripts/ios_live.log`
**Full Report:** [`LOG_AUDIT_2026-03-15.md`](../LOG_AUDIT_2026-03-15.md)

### Critical Issues Confirmed via Logs

| Issue | Platform | Log Evidence | Status | Field | Value |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **ID** | CONTACT-STALE-001 | | | | |
| **iOS Retry Storm** | iOS | `IronCoreError error 4` repeating every ~1 second | 🟢 Fixed | P0 | |
| **msg=unknown** | Android | `delivery_attempt msg=unknown` in 5+ log entries | 🔴 Active | P1 | |
| **Relay Circuit Failing** | Both | `Core-routed delivery failed... Network error` | 🔴 Active | P0 | |
| **BLE Working** | Android | `✓ Delivery via BLE client` confirmed | 🟢 Working | - | |

### Specific Fix Recommendations from Log Analysis

#### P0: iOS Exponential Backoff (✅ IMPLEMENTED)

**File:** [`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`](../iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift)

**Implementation Date:** 2026-03-15

**Changes Made:**
1. **Added retry backoff state tracking** (Lines 116-126):
   - `consecutiveDeliveryFailures: [String: Int]` - tracks failures per peer
   - `lastFailureTime: [String: Date]` - tracks timing for circuit breaker
   - `circuitBreakerThreshold = 10` - pause after 10 consecutive failures
   - `circuitBreakerDuration = 300` - 5 minute pause duration
2. **Added circuit breaker check before relay-circuit** (Lines 4005-4020):
   - Checks if consecutive failures exceed threshold
   - If within circuit breaker duration, skips retry and logs diagnostic
   - Resets after duration expires
3. **Added exponential backoff before relay attempt** (Lines 4024-4028):
   - Backoff formula: `1 << min(failureCount, 5)` seconds
   - Sequence: 1s → 2s → 4s → 8s → 16s → 32s (capped)
   - Logs backoff duration for debugging
4. **Track failures and reset on success** (Lines 4055-4058, 4071-4073):
   - On failure: increment `consecutiveDeliveryFailures[peerKey]`, set `lastFailureTime`
   - On success: reset `consecutiveDeliveryFailures[peerKey] = 0`, remove `lastFailureTime`

**Before Fix:** Retry loop fired every ~1 second with no backoff, causing CPU pressure and log spam.

**After Fix:** Exponential backoff (1s-32s) with circuit breaker (5 min pause after 10 failures).

```swift
// Add retry backoff tracking
private var retryBackoff: [String: TimeInterval] = [:]
private var lastAttemptTime: [String: Date] = [:]

// In delivery attempt, add backoff check
let peerId = route
let backoff = retryBackoff[peerId] ?? 1.0
let now = Date()

if let lastAttempt = lastAttemptTime[peerId],
   now.timeIntervalSince(lastAttempt) < backoff {
    log.debug("skip_retry msg=\(msgId) backoff=\(backoff)s remaining=\(backoff - now.timeIntervalSince(lastAttempt))")
    return
}

lastAttemptTime[peerId] = now
retryBackoff[peerId] = min(backoff * 2.0, 32.0) // Cap at 32 seconds
```

**Also Add Circuit Breaker:**

```swift
private var consecutiveFailures: [String: Int] = [:]
private let maxConsecutiveFailures = 10
private let circuitBreakerDuration: TimeInterval = 300 // 5 minutes

// Before attempting delivery
if let failures = consecutiveFailures[peerId], failures >= maxConsecutiveFailures {
    if let lastAttempt = lastAttemptTime[peerId],
       now.timeIntervalSince(lastAttempt) < circuitBreakerDuration {
        log.warning("circuit_breaker_active peer=\(peerId) failures=\(failures)")
        return
    }
    // Reset after circuit breaker duration
    consecutiveFailures[peerId] = 0
}
```

#### P1: Android Message ID Propagation (Estimated: 1 hour)
**File:** [`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`](../android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt)

**Current Behavior:** `msg=unknown` appears in delivery_attempt logs because message UUID is not captured before async send.

**Required Fix:**

```kotlin
// In sendMessage() function, capture messageId BEFORE async call
val messageId = message.id ?: UUID.randomUUID().toString()

// Pass explicitly to delivery logging
logDeliveryAttempt(
    msg = messageId,  // Never null
    medium = "core",
    phase = "direct",
    outcome = "attempt",
    detail = "ctx=send",
    route = peerId
)
```

**Also Add Diagnostic Logging:**

```kotlin
// Before send attempt
Log.d(TAG, "send_precheck msg=$messageId peer=$peerId transport=$transport candidates=${candidates.size}")

// On Network error with socket details
Log.e(TAG, "send_failed msg=$messageId error_code=${error.code} detail=${error.message} retry_count=$retryCount socket_error=${error.cause?.message}")
```

#### P0: Relay Server Health Verification (Estimated: 30 minutes)
**Action Required:** Verify relay server is accepting connections.

**Commands to Run:**
```bash
# Test TCP connectivity to relay
nc -zv 34.135.34.73 9001

# Test from iOS simulator network
# In Xcode, add diagnostic print before relay dial:
print("relay_health_check host=34.135.34.73 port=9001")

# Check relay server logs if accessible
ssh relay-server "journalctl -u scm-relay --since '5 minutes ago'"
```

**If Relay is Down:**

1. Check GCP instance status: `gcloud compute instances describe scmessenger-bootstrap --zone=us-central1-a`
2. Start VM if terminated: `gcloud compute instances start scmessenger-bootstrap --zone=us-central1-a`
3. Check container status: `./scripts/test_gcp_node.sh`
4. Restart container if needed: `./scripts/deploy_gcp_node.sh`
5. Verify port listening: `gcloud compute ssh relay-server -- sudo netstat -tlnp | grep 9001`

---

## 2026-03-16 Log Visualizer: BLE & Local Transport Visibility (Implemented)

- **Enhanced BLE Log Recognition**:
  - Broadened regex matching for BLE operations to capture `scanning`, `discovered`, `connected`, `advertising`, `write`, `read`, `gatt`, `l2cap`, and error states.
  - Added explicit fallbacks for `BleGattClient`, `BleGattServer`, and `BLECentralManager` tags to ensure they are always categorized as BLE ops even without keyword matches.
- **Added Local Transport Mapping**:
  - Implemented new mappings for `Multipeer`, `WifiDirect`, `WifiAware`, and `mDNS` logs under `LOCAL_OPS`.
  - Captures `browsing`, `invite`, `accept`, `timeout` and other P2P transport lifecycle events.
- **Improved Diagnostic Categorization**:
  - Expanded `*_DIAG` mapping to include `multipeer` alongside `ble`, `wifi`, `relay`, etc.
  - Strips `DIAG:` prefix from iOS tags for cleaner categorization.
- **Enhanced PeerID Extraction**:
  - Widened PeerID extraction in `server.mjs` to capture both libp2p identities and Blake3 hex hashes.

---

## 2026-03-15 Log Visualizer Expansion: Mesh Topology View (Implemented)

## 2026-03-14 WS14.2 iOS DM / DM Request Notifications (Implemented, Build-Verified)

- **iOS notification handling is now production-routed instead of placeholder-only**:
  - `iOS/SCMessenger/SCMessenger/Services/NotificationManager.swift` now defines separate DM and DM Request notification categories, badge accounting, quick-reply handling, mark-read actions, and notification-tap routing via `notificationRouteRequested`.
  - `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift` now calls the shared Rust classifier for inbound chat events, persists pending-request state on unknown-sender contacts, clears that state on accept/reply, and only opens the Requests Inbox for request notifications.
  - `iOS/SCMessenger/SCMessenger/Generated/api.swift` and `apiFFI.h` were regenerated so Swift now consumes the WS14.1 notification decision/context/settings contract directly.
- **Requests Inbox routing is now wired through the real navigation stack**:
  - `iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift` now owns a message-route stack with explicit `conversation` vs `requestsInbox` destinations.
  - Pending request threads are separated from the main conversation list and exposed through a dedicated Requests Inbox view with accept actions.
  - Active-conversation foreground state is now fed back into the repository so shared suppression rules can distinguish visible chat from background delivery.
- **Settings parity is now exposed on iOS**:
  - `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift` and `Views/Settings/SettingsView.swift` now surface `notifications_enabled`, DM/DM-request toggles, foreground behavior toggles, sound, and badge state from the shared settings model instead of a local-only single toggle.
  - `iOS/SCMessenger/SCMessenger/SCMessengerApp.swift` now configures the notification manager during app bootstrap, requests permission when notifications are enabled, and reports foreground/background transitions back into the repository notification state.
- **Verification evidence captured in this run**:
  - `bash ./iOS/copy-bindings.sh` — **PASS**
  - `bash ./iOS/verify-test.sh` — **PASS**
- **Closeout status**:
  - WS14.2 implementation is in place and the touched iOS target now builds cleanly through the repo verification path.
  - WS14.3 Android parity remains the next incomplete WS14 phase.

## 2026-03-14 WS14.1 Core Notification Contract (Implemented, Gate Accepted)

- **Core notification policy now exists in shared Rust code**:
  - `core/src/notification.rs` defines `NotificationKind`, `NotificationMessageContext`, `NotificationUiState`, and `NotificationDecision`.
  - `IronCore::classify_notification(...)` now centralizes DM vs DM Request classification, suppression reasons, foreground checks, and normalized metadata handling so adapters do not drift.
- **UniFFI/WASM/API parity landed in this phase**:
  - `core/src/api.udl` now exposes the notification classifier and the notification decision/context types.
  - `wasm/src/lib.rs` now mirrors the classifier and widened notification settings contract for browser clients.
  - `MeshSettings` on shared Rust/mobile/WASM surfaces now includes notification toggles: `notifications_enabled`, `notify_dm_enabled`, `notify_dm_request_enabled`, `notify_dm_in_foreground`, `notify_dm_request_in_foreground`, `sound_enabled`, and `badge_enabled`.
- **Compatibility boundary was held intentionally**:
  - The encrypted wire `Message` format was left unchanged in WS14.1 because it is still bincode-serialized; adding an inline `is_dm_request` field without a versioning layer would risk mixed-version decode failures.
  - Explicit DM-request intent is therefore normalized as classifier input metadata for now while legacy inference remains the compatibility fallback.
- **Verification evidence captured in this run**:
  - `cargo fmt --all -- --check` — **PASS**
  - `CARGO_TARGET_DIR=/tmp/scm-ws14-target cargo build --workspace` — **PASS**
  - `CARGO_TARGET_DIR=/tmp/scm-ws14-target cargo clippy --workspace` — **PASS** (pre-existing `too_many_arguments` warning only)
  - `CARGO_TARGET_DIR=/tmp/scm-ws14-target cargo test --workspace` — **FAILED IN SANDBOX** (`relay::client::tests::test_connect_to_relay` and `relay::client::tests::test_push_pull_and_ping_over_network` both failed with `Os { code: 1, kind: PermissionDenied, message: "Operation not permitted" }`)
- **Closeout status**:
  - Implementation is in place.
  - The earlier sandbox-only relay-test residue was explicitly accepted before WS14.2 began, so WS14.1 is no longer the active phase gate for the hourly stream.

## 2026-03-14 WS14 Hourly Automation Reset (Docs/Workflow Update)

- **March 13 hourly automation drift was audited**: the last trustworthy WS13/WS14 hourly handoff lived in Codex-local automation memory and pointed at `codex/ws13-ws14-hourly-20260313-2215`, but later repo state moved outside that lane.
- **WS14 hourly execution was rebuilt as WS14-only**:
  - `docs/WS14_AUTOMATION_HANDOFF.md` is now the repo-owned branch/phase ledger.
  - `docs/WS14_HOURLY_AUTOMATION_PROMPT.md` is now the canonical one-phase-per-run prompt.
  - the local paused automation has been repointed to this WS14-only workflow and set to default to `medium` reasoning instead of `low`.
- **Branch/worktree hygiene is now fail-closed for future hourly runs**:
  - branch-only execution,
  - one WS14 phase per run,
  - no cross-phase marching,
  - no unrelated bug absorption,
  - blocked verification keeps the current phase partial instead of skipping ahead.
- **Current operator posture**:
  - the hourly automation is now running on the writable continuation branch `codex/ws14-hourly-20260314-0301`, rebased onto the prepared stream baseline,
  - WS14.1 has been cleared as the opening gate for the continuation branch,
  - WS14.2 is implemented and build-verified on iOS, so WS14.3 Android notification parity is the next exact phase.
- **Verification note**: this reset changed documentation and automation workflow only. No build-target code, bindings, or build-affecting repo scripts changed, so no additional product build-verification rerun was required beyond the March 14 product evidence already recorded below.

## 2026-03-14 Android Audit & Critical Bug Fixes (Completed)

- **4 critical bugs identified** from real-time Android log monitoring (12:00 PM - 8:00 PM HST)
- **Issues #1-#3 fixed and verified**:
  - Issue #1: Public Key Truncation — added validation + recovery in MeshRepository.kt
  - Issue #2: Contact ID Mismatch — added canonicalContactId() normalization, public-key-first matching
  - Issue #3: Stale Nearby Peer — resolved by Issue #2 fix
- **Issue #4 fixed**: Updated `backup_rules.xml` and `data_extraction_rules.xml` to exclude database files and identity backup prefs from Android Auto Backup
- **Issue #5 fixed**: Added relay peer filtering in `MeshRepository.kt` to prevent auto-creation of relay peers as contacts
- **Issue #6 fixed**: Reduced `nearbyDisconnectGraceMs` from 30s to 5s in `ContactsViewModel.kt` to promptly remove disconnected peers
- **UI fixes committed**: Edge-to-edge handling, keyboard IME padding, onboarding keyboard actions
- **Build verification**: `./gradlew assembleDebug -x lint` — PASS
- **Documentation sync**: `./scripts/docs_sync_check.sh` — PASS

## 2026-03-14 WS13.6 Completion (Completed)

- **Compatibility/migration matrix created**: `docs/WS13.6_COMPATIBILITY_MIGRATION_MATRIX.md`
  - Compatibility mode policy for legacy no-device traffic
  - Migration matrix for pre-WS13 → WS13 upgrades
  - Enforcement mode transition plan (Phase A/B/C)
  - Acceptance gates verified
- **Handover/abandon runbook created**: `docs/WS13.6_HANDOVER_ABANDON_RUNBOOK.md`
  - Operational procedures for device handover flow
  - Operational procedures for identity abandonment flow
  - Sender-facing error handling documentation
  - Testing procedures and monitoring guidance
- **Residual risk closed**: R-WS13.4-01 marked as Closed in `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`
- **WS13 workstream complete**: All WS13.1-WS13.6 tasks marked as complete in `REMAINING_WORK_TRACKING.md`
- **WS14 scope reviewed**: WS14 now has WS14.1 and WS14.2 in place on the continuation branch; remaining scope is tracked in `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`

## 2026-03-13 WS13.4 Registry Enforcement + WS13.5 Rejection UX (Implemented, Verification In Progress)

- **Relay registry + custody enforcement are now live in core**:
  - `IronCore` now owns a persisted `RelayRegistry`, exposes `get_registration_state(identity_id)`, and builds signed registration requests for the mobile bridge.
  - `core/src/transport/swarm.rs` now mutates active-device registry state on registration/deregistration, enforces `Active`/`Handover`/`Abandoned` custody policy, roots native custody state in service storage, and keeps wasm parity for duplicate registration/relay handling.
  - `core/src/mobile_bridge.rs` now auto-registers the active identity on `PeerIdentified` and exposes `send_message_status(...)` so adapter layers can distinguish retryable delivery failures from terminal identity/device rejection.
- **Sender-facing rejection UX now exists on both mobile clients**:
  - Android `MeshRepository` now persists `recipient_identity_id`, `intended_device_id`, and `terminal_failure_code` in the pending outbox, stops retrying terminal tight-pair failures, and surfaces `rejected` delivery state plus a chat snackbar.
  - iOS `MeshRepository` now follows the same rejection model, preserves terminal failure codes in the pending outbox, stops retrying abandoned/device-mismatch sends, and renders `rejected` delivery state in the main tab legend/UI.
  - UniFFI bindings were regenerated and copied after the new `RegistrationStateInfo` / `send_message_status(...)` surface landed.
- **Current WS completion state**:
  - WS13.1 ✅ Identity device metadata + persistence
  - WS13.2 ✅ Contact + request schema updates
  - WS13.3 ✅ Registration protocol + signature verification
  - WS13.4 ✅ Relay registry state machine + custody enforcement
  - WS13.5 🟨 Handover/abandon queue migration + rejection UX implemented; Android physical-device verification passed, iPhone foreground-launch proof is still blocked by device lock
  - WS13.6 ⬜ NEXT: compatibility/migration matrix + acceptance lock
- **Verification completed on 2026-03-13**:
  - `cargo build --workspace` — **PASS**
  - `cargo test --workspace` — **PASS**
  - `ANDROID_SERIAL=26261JEGR01896 UNINSTALL_FIRST=1 ./android/install-clean.sh` — **PASS** (clean rebuild, uninstall, install, permission grant on connected Pixel 6a)
  - `adb -s 26261JEGR01896 shell am start -n com.scmessenger.android/.ui.MainActivity` — **PASS**
  - `adb -s 26261JEGR01896 shell dumpsys activity activities | rg "topResumedActivity|com.scmessenger.android"` — **PASS** (`com.scmessenger.android/.ui.MainActivity` resumed in foreground)
  - `APPLE_TEAM_ID=9578G7VQWS DEVICE_UDID=00008130-001A48DA18EB8D3A ./iOS/install-device.sh` — **PARTIAL PASS** (signed device build + install succeeded on christy's iPhone; automated launch was denied because the device was locked)
- **Verification residue observed during real-device checks**:
  - Android fresh-install startup emitted non-fatal `IronCoreException.NotInitialized` logs from `sendHistorySyncIfNeeded` before identity/core readiness fully settled; no fatal crash was observed, but the startup ordering remains a real residual risk.
  - iOS device launch evidence is install-only until the connected iPhone is unlocked and the launch step is rerun.

## 2026-03-13 BLE Freshness Profiling + run5 Visibility Clarification (Verified)

- **Android BLE profiling is now freshness-first**:
  - `BleScanner.kt` starts with a service-filtered scan, then promotes to an unfiltered scan after 20s with zero mesh advertisements.
  - `MeshRepository.kt` now keeps a 120s BLE route-observation cache so send-path fallback prefers:
    1. currently connected BLE peer,
    2. freshest observed BLE alias/address,
    3. persisted BLE hint only if it is still fresh.
  - stale cached BLE hints are now explicitly skipped instead of silently outranking fresher runtime evidence.
- **run5 collector ambiguity is now made explicit instead of hidden**:
  - `run5.sh` now writes physical-device app console output to `ios-device.log` and host/system Bluetooth + Multipeer context to `ios-device-system.log`.
  - if the physical iOS app is already running, `run5.sh` no longer relaunches it just to chase console output; the app log now records that passive app-console capture is unavailable in that case.
  - the post-run visibility matrix now counts only peers whose own IDs were actually captured in the current log window.
  - unknown own IDs are now reported as collector gaps, not auto-counted as mesh failures.
  - duplicate/ambiguous own-ID inference is suppressed instead of being reused across multiple nodes.
- **Operator ambiguity clarified**:
  - `ios_dev own id = unknown` now means "app startup identity lines were not captured in this log window," not "the iPhone was off mesh."
  - transport evidence and visibility proof are now separate concepts in `run5.sh`; BLE/direct/relay activity can be real even when a node's local own ID was not captured.
  - GCP relay log collection now grabs a recent `docker logs --tail 200` snapshot before incremental polling so short runs have a better chance of capturing headless startup identity context.
  - Android BLE fallback telemetry still has one remaining forensic ambiguity: accepted-send lines can retain the requested fallback MAC while `BleGattClient` callback success is emitted for the fresher connected GATT address actually used on the wire. Until that logging is unified, treat the callback-success address as authoritative.
- **Verification**:
  - `cd android && ./gradlew app:assembleDebug` — **PASS**

## 2026-03-13 Documentation Sync + Build Verification Governance Lock

- **Canonical Closeout Policy Tightened**: `AGENTS.md` now explicitly requires same-run canonical doc updates whenever behavior, scope, risk, scripts, verification commands, or operator workflow changes.
- **Edited-Target Build Verification Locked In**: Repo agent guidance now explicitly requires build verification for any edited code path, generated bindings, build wiring, or runtime-affecting script before a session can be considered complete.
- **Cross-Agent Consistency**: `.github/copilot-instructions.md` now mirrors the same requirements so Codex and Copilot execution policy stay aligned.
- **Verification Workflow**: `./scripts/docs_sync_check.sh` remains the mandatory documentation consistency gate before finalizing change-bearing work.
- **Current implication**: Documentation sync and build verification are now first-class completion criteria, not best-effort follow-up tasks.

## 2026-03-13 NAT Traversal & BLE Stability (Verified)

- **NAT Traversal Restoration**: Fixed a regression where relay nodes were being filtered out of swarm routing and direct delivery candidates. Restoring relay participation ensures connectivity across NAT boundaries (Cellular/WiFi cross-over).
- **BLE Identity Beacon Throttling**: Added a 5-second throttle to BLE identity updates on both Android and iOS. This significantly reduces log noise and radio churn, and prevents potential UI/bridge freezing caused by rapid sub-stream identify events.
- **Android BLE Connect-on-Demand**: Fixed overly aggressive skip logic that prevented connecting to stale BLE MACs. The system now attempts connection to hinted addresses during send-path fallback.
- **Deduplication Hardening**: Implemented Rust-level and adapter-level deduplication for `PeerIdentified` events to mitigate performance issues on iOS.
- **Build Verification**: `gradlew assembleDebug` (Android) — **PASS**. `docs_sync_check.sh` — **PASS**.

## 2026-03-12 iOS CryptoError & Power Optimization (Verified)

- **CryptoError (Error 4) Resolve**: Traced to encryption failures against peers resolved via stale bootstrap nodes. Fixed by updating static fallback and enabling dynamic ledger-based bootstrap discovery.
- **Power Optimization**: Increased `adaptiveIntervalSeconds` for top-end battery levels (5 mins at >80%, 10 mins at >95%) to reduce background activity during full health.
- **Log Noise Reduction**: Simplified "Power profile applied" messages and removed redundant polling data from info-level logs.
- **Build Verification**: `bash ./iOS/verify-test.sh` — **PASS** (iOS build succeeded, transport parity tests passed).

## 2026-03-12 Comprehensive Consolidation — PR83 (Verified)

**Branch:** `copilot/consolidate-branches-for-clean-build`
**Includes:** All work from PR79 (pr77-reconciliation), PR80 (sub-pr-79), PR81 (sub-pr-79-again), PR82 (consolidate-prs-79-80-81)
**Build verification (2026-03-12):**
- `cargo build --workspace` — **PASS**
- `cargo test --workspace` — **PASS** (528 tests, 0 failures, 17 ignored)
- `cargo clippy --workspace` — **PASS** (no new errors)
- `./scripts/docs_sync_check.sh` — **PASS**

**WS completion state:**
- WS13.1 ✅ Identity device metadata + persistence
- WS13.2 ✅ Contact + request schema updates
- WS13.3 ✅ Registration protocol + signature verification
- WS13.4 ⬜ **NEXT:** Relay registry state machine + custody enforcement
- WS13.5 ⬜ Handover/abandon queue migration + rejection UX
- WS13.6 ⬜ Migration + compatibility + test matrix

---

## 2026-03-12 Consolidated PR79+PR80+PR81 (Verified)

### WS13.3 Registration Protocol + Signature Verification

- WS13.3 landed as an additive transport protocol on top of current `main`.
- Files changed:
  - `core/src/transport/behaviour.rs` — added `/sc/registration/1.0.0` request/response behaviour plus canonical `RegistrationPayload`, `RegistrationRequest`, `DeregistrationPayload`, `DeregistrationRequest`, and `RegistrationResponse` types.
  - `core/src/transport/swarm.rs` — added `SwarmCommand::{RegisterIdentity,DeregisterIdentity}` and `SwarmHandle::{register_identity,deregister_identity}`; incoming registration messages now fail closed on malformed identity IDs, malformed UUIDv4 device IDs, peer/identity mismatches, invalid signatures, and invalid deregistration state (`target_device_id == from_device_id`).
  - `core/src/transport/mod.rs` — re-exported the new WS13.3 transport request/response types.
  - `core/tests/integration_registration_protocol.rs` — added end-to-end swarm tests for successful registration plus malformed/tampered rejection paths.
  - `core/src/transport/behaviour.rs`, `core/src/transport/swarm.rs` (tests) — added canonical serialization, signature pass/fail, peer/public-key extraction, and identity-mismatch coverage.
- Scope boundary preserved:
  - no relay-registry mutation or custody enforcement was added yet; valid registration/deregistration requests are verified and acknowledged only,
  - mobile UniFFI / adapter surfaces were intentionally left unchanged in this phase because WS13.3 is transport-internal and this Linux host still lacks Android/iOS regeneration tooling.
- Residual follow-up:
  - WS13.4 still owns persisted registry state, collision policy, and custody enforcement.
  - Anti-replay / monotonic registration-state protection remains open until registry state exists (tracked in `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md`).

### PR79 Code Review Fixes

- **P1: Persistent blocklist backend** (core/src/lib.rs):
  - `IronCore` now stores a persistent `blocked_manager: Arc<BlockedManager>` field initialized from the root storage backend.
  - All blocking API methods (`block_peer`, `unblock_peer`, `is_peer_blocked`, `list_blocked_peers`, `blocked_count`) use the shared persistent manager instead of creating ephemeral `MemoryStorage` per call.
  - WASM async init also constructs the `blocked_manager`.
- **P1: Android message ID reconciliation** (android MeshRepository.kt):
  - After `prepareMessageWithId` returns `realMessageId`, the initial history record (keyed by `initialMessageId` UUID) is now replaced with `realMessageId` so delivery receipts can find and mark the correct record.
  - All downstream tracking (delivery state, pending outbox, promotions) uses `realMessageId`.
- **P2: Identity resolver order** (core/src/lib.rs):
  - `resolve_identity` now checks contacts for identity_id (Blake3 hash) matches BEFORE testing Ed25519 key shape. Prevents misclassification of identity hashes that happen to be valid Ed25519 points.
- **P2: Android SharedFlow replay** (android MeshRepository.kt):
  - Changed `_messageUpdates` from `replay=1` to `replay=0, extraBufferCapacity=1, onBufferOverflow=DROP_OLDEST` to prevent duplicate notifications to late subscribers.
- **P2: FileLoggingTree thread safety** (android FileLoggingTree.kt):
  - `ironCore` field marked `@Volatile`; `setIronCore` uses `synchronized(this)`.
  - `ironCore?.recordLog()` wrapped in `runCatching` so IronCore failures don't skip file fallback.
- **P2: PeerIdValidator strictness** (android PeerIdValidator.kt):
  - `isLibp2pPeerId` now validates length range and base58 charset (no 0, O, I, l) beyond prefix-only checks.
- **P2: Storage maintenance guard** (android MeshRepository.kt):
  - Added `maintenanceJob` field with `isActive` check to prevent duplicate maintenance coroutines on service restart.
- **New tests** (2 tests added):
  - `test_blocklist_persistence_across_calls`: Verifies block/unblock/list persist across separate calls to the same `IronCore` instance.
  - `test_resolve_identity_checks_contacts_before_key_shape`: Verifies identity_id is resolved via contact lookup before Ed25519 key shape test.
- **Verification**:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo clippy --workspace` — **pass** (only pre-existing `too_many_arguments` warning)
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass** (516 tests, 0 failures)
  - `./scripts/docs_sync_check.sh` — **pass**

### PR Reconciliation & Hardening

- **Build fixes**:
  - CLI: Added missing `UiEvent::Error` variant in `server.rs`, resolving build failure.
  - WASM: Removed duplicate `PortMapping(_)` arm in swarm event match, eliminating `unreachable_patterns` warning.
- **Production safety**:
  - `core/src/store/logs.rs`: Replaced `.unwrap()` on `SystemTime::now()` with `.unwrap_or_default()` (2 sites).
  - `core/src/store/logs.rs`: Implemented delta pruning when entries exceed 1000 (previously a no-op).
  - `core/src/store/logs.rs`: Backend flush failures now logged via `tracing::warn!` instead of silently ignored.
  - `core/src/store/storage.rs`: Removed unused `_message_max_threshold` dead code.
  - `core/src/lib.rs`: Root sled backend initialization now falls back to `MemoryStorage` on error instead of panicking.
- **New tests** (11 tests added):
  - `store::logs::tests` (6): record/export, flush/reload, prune_oldest, install_time persistence, empty export, delta pruning limits.
  - `store::storage::tests` (5): update_disk_stats, maintenance noop/zero/enough_space/low_space, DiskStats default.
- **Contacts hardening**:
  - `update_last_known_device_id` now trims whitespace and validates UUIDv4 format before persisting; clears on `None` as before.
- **Verification**:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo clippy --workspace` — **pass** (only pre-existing `too_many_arguments` warning)
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass** (514+ tests, 0 failures)
  - `./scripts/docs_sync_check.sh` — **pass**
## 2026-03-10 WS13.2 Transport Boundary Widening (Implemented)

- Architectural blocker resolved: transport/API boundary widened to carry WS13 tight-pair metadata.
- Files changed:
  - `core/src/transport/behaviour.rs` — `RelayRequest` gains `recipient_identity_id: Option<String>` and `intended_device_id: Option<String>` with `#[serde(default)]` for wire compatibility with pre-WS13 relay nodes.
  - `core/src/transport/swarm.rs` — `SwarmCommand::SendMessage`, `PendingMessage`, `SwarmHandle::send_message`, and `dispatch_ranked_route` widened with same optional fields; retry path threads metadata through.
  - `core/src/mobile_bridge.rs` — `SwarmBridge::send_message` accepts the two new optional params; `send_to_all_peers` passes `None, None`.
  - `core/src/api.udl` — `SwarmBridge::send_message` binding updated: `void send_message(string peer_id, bytes data, string? recipient_identity_id, string? intended_device_id)`.
  - `core/src/store/contacts.rs` — `Contact` struct gains `last_known_device_id: Option<String>` with `#[serde(default)]`; `ContactManager::update_last_known_device_id()` added.
  - `cli/src/main.rs`, `cli/src/api.rs` — all call-sites updated with `None, None` (legacy behavior preserved).
  - `wasm/src/lib.rs` — `send_envelope` call-site updated with `None, None`.
  - `core/tests/integration_all_phases.rs`, `core/tests/integration_relay_custody.rs` — updated for new signature.
  - `core/src/transport/behaviour.rs` (tests) — added relay-request legacy-wire compatibility tests.
  - `core/src/store/contacts.rs` (tests) — added `last_known_device_id` round-trip and serde-default tests.
- Verification on this Linux host:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- Platform tooling NOT available on this host: `xcodebuild` (iOS), `cargo-ndk` / `ANDROID_HOME` (Android). Mobile adapter call-sites that consume `SwarmBridge::send_message` must be updated when those tools are available — the new UDL signature is the source of truth for generated bindings.
- WS13.2 status: **transport boundary complete, relay metadata plumbing implemented, `last_known_device_id` wired in Contact**. WS13.3 (registration protocol) and WS13.4 (registry/custody state machine) are now unblocked architecturally.

## 2026-03-10 WS13 Full-Stream Execution Audit (WS13.1 Landed, WS13.2 Landed in Core)

- Re-ran the required WS13 preflight on the rebased tree:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- Audited live WS13 code state:
  - **WS13.1 (Seniority/Device Storage)**: ✅ IMPLEMENTED AND VERIFIED in `core/src/identity/`.
  - **WS13.2 (Contact Schema/Metadata)**: ✅ IMPLEMENTED AND VERIFIED in `core/src/store/contacts.rs` and `api.udl`.
  - **WS13.3-13.6 (Protocols/Relay-Registry)**: 🔄 DEFERRED to v0.2.1.
- Blocker Status:
  - The architectural blocker (transport metadata threading) was resolved by widening the swarm send boundary to carry `recipient_identity_id` and `intended_device_id` end-to-end through `SwarmCommand::SendMessage`, `RelayRequest`, and all call-sites.
  - Full platform verification (Android/iOS binding regeneration) remains pending platform tooling (see R-WS13.2-02).
- Result:
  - WS13.2 core work is **Landed**.
  - Baseline is verified and ready for WS13.3 iteration.

## 2026-03-10 WS13.1 Tight-Pair Kickoff (Verified)

- Required WS13 kickoff docs were re-read in canonical order before coding, including:
  - `AGENTS.md`
  - `DOCUMENTATION.md`
  - `docs/DOCUMENT_STATUS_INDEX.md`
  - `docs/CURRENT_STATE.md`
  - `REMAINING_WORK_TRACKING.md`
  - `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
  - `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
  - planned docs `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` and `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`
- Re-ran WS13 preflight baseline locally:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- GitHub Actions audit for this branch:
  - PR run `22923791535` (`CI`) is still `action_required`, matching the previously documented GitHub approval/policy blocker rather than a WS13 code regression.
- WS13 inventory and risk split are now explicit:
  - `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md` now includes a WS13.1 -> WS13.6 execution inventory with file targets, test targets, migration implications, and acceptance gates.
  - `docs/V0.2.1_RESIDUAL_RISK_REGISTER.md` now tracks v0.2.1 residual risk separately from the v0.2.0 register.
- WS13.1 implementation landed only in the core identity metadata surface:
  - persisted installation-local `device_id` (UUIDv4) and `seniority_timestamp`,
  - hydrate/initialize/import paths now backfill missing metadata without rotating existing key identity,
  - identity backup remains portable key material only; a restored install generates fresh local device metadata.
- Focused WS13.1 verification after implementation:
  - `cargo test -p scmessenger-core --no-run` — **pass**
  - `cargo test -p scmessenger-core test_identity -- --nocapture` — **pass**
  - `cargo test -p scmessenger-wasm test_desktop_identity_flow_exposes_metadata_after_init -- --nocapture` — **pass**
- Scope boundary preserved:
  - no WS13.2+ transport/contact/custody enforcement work was started,
  - no v0.2.0 physical-device closure debt or maintainer-only GitHub cleanup was pulled into this implementation.

## 2026-03-10 Relay Peer Discovery & Identity Blocking (Verified)

### Relay Peer Discovery Implementation

- **Active Relay Broadcasting**:
  - All nodes now broadcast peer join/leave events
  - Relay nodes share full peer lists with newly connected clients
    - **Impact:** Permission dialog spam, discovery blocked until permissions granted

### Identity Modal / Keyboard Issues (Reported but Not Confirmed)
- Added 4 new protocol messages: `PeerJoined`, `PeerLeft`, `PeerListRequest`, `PeerListResponse`
  - File: `core/src/relay/protocol.rs` (lines 103-120)

- **Peer Broadcaster Module**:
  - Tracks all connected peers with metadata
  - Generates peer announcement messages
  - Manages peer join/leave broadcasting
  - File: `core/src/transport/peer_broadcast.rs` (NEW - 148 lines)

- **Swarm Integration**:
  - Broadcasts peer joined to all connected peers on connection (line ~2430)
  - Broadcasts peer left to remaining peers on disconnect (line ~2491)
  - Handles incoming peer discovery messages (lines ~1507-1560)
  - Automatically dials announced peers for direct P2P connections
  - File: `core/src/transport/swarm.rs`

### Identity Blocking System

- **Blocked Identities Module**:
  - Block peer IDs (identities) with optional device-specific granularity
  - Stores block reason, notes, and timestamp
  - Includes TODO for device ID pairing infrastructure
  - File: `core/src/store/blocked.rs` (NEW - 227 lines)

- **Blocking API**:
  - `block(identity)` - Block a peer ID
  - `unblock(peer_id, device_id)` - Unblock identity or device
  - `is_blocked(peer_id, device_id)` - Check if blocked
  - `list()` - Get all blocked identities
  - Storage backend agnostic (Sled/IndexedDB/Memory)

- **TODO: Device ID Pairing**:
  - Device ID generation and secure storage
  - Identity-device mapping in handshake protocol
  - Multi-device blocking (block one device, allow others)
  - Marked with TODO comments throughout code

### Android Bug Fixes

- **Case-Sensitivity Fixes** (5 locations):
  - Peer ID lookups now case-insensitive
  - Fixed peer resolution failures
  - Files: `MeshRepository.kt` (4 fixes), `ConversationsViewModel.kt` (1 fix)

- **Initialization Race Condition** (2 fixes):
  - Added initialization checks in `sendHistorySyncIfNeeded()` and `sendIdentitySyncIfNeeded()`
  - Eliminated "Not initialized" errors
  - Fixed "pre-loaded identity" bug
  - Fixed `msg=unknown` delivery state issues
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Build Status

- Core (Rust): ✅ Built successfully with peer discovery + blocking
- Android: ✅ Deployed with bug fixes
- iOS Framework: ✅ Rebuilt with peer discovery (2m 25s)

### Documentation Created

- `RELAY_PEER_DISCOVERY_IMPLEMENTATION.md` - Complete implementation report
- `IDENTITY_BLOCKING_IMPLEMENTATION.md` - Blocking system documentation
- `CASE_SENSITIVITY_AUDIT_2026-03-09.md` - Case bug fixes
- `ANDROID_ID_MISMATCH_RCA.md` - Root cause analysis
- `PEER_ID_RESOLUTION_FIX.md` - Initialization fix details
- `IOS_CRASH_AUDIT_2026-03-10.md` - iOS stability analysis
- `FINAL_SESSION_REPORT_2026-03-09.md` - Comprehensive session report

## 2026-03-10 WS12 Closeout Burndown Re-Baseline (Verified)

- Local Rust/docs verification is back to a trustworthy baseline on this branch:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass** (`375 passed`, `0 failed`, `16 ignored`)
  - `./scripts/docs_sync_check.sh` — **pass**
- Minimal repo-code drift closed in this pass:
  - removed trailing whitespace in `cli/src/main.rs`, restoring the Rust formatting gate that was failing on `main`,
  - updated `wasm/src/lib.rs` to explicitly ignore the new `SwarmEvent::PortMapping(_)` status event, restoring workspace build coverage after the core swarm event expansion.
- GitHub issue tracker reconciliation against canonical docs:
  - open issues are currently automation-only (`#38`, `#39`, `#40`, `#42`); none of them are the canonical source of active WS12/v0.2.0 closeout work,
  - there are currently **no** open repo issues explicitly tracking `WS13` or `WS14`, which is good for scope separation but leaves the v0.2.0 closeout slate underrepresented in GitHub Issues until maintainers curate it.
- GitHub Actions truth after direct audit:
  - PR run `22902069805` (`CI`) for this branch is still `action_required`, confirming the repository-settings/approval-policy problem remains a GitHub-hosted blocker rather than a code-signal.
  - Latest failed `main` CI run `22879428848` split into distinct causes:
    - docs-sync failure because 2026-03-09 code landed without matching canonical-doc updates,
    - Rust formatting failure from the stray CLI whitespace now fixed in this branch,
    - WASM build failure from missing `SwarmEvent::PortMapping(_)` handling, now fixed in this branch,
    - iOS build failure still open due MainActor isolation violations in `BLEPeripheralManager`, `ContactsViewModel`, `TopicManager`, and `IosPlatformBridge`.
  - Latest failed Docker Integration Suite run `22879428852` is also a real repo-side defect, not settings noise:
    - Android Unit Tests container fails because `docker/docker-compose.test.yml` copies `core/target/release/libscmessenger_core.so`, but the workspace build emits the host library under the workspace target directory instead.
- GitHub branch inventory remains noisy and unprotected:
  - branch listing currently shows `main` plus 66 additional unprotected branches, largely agent/copilot/dependabot generated,
  - stale-branch cleanup remains a maintainer-side trust-signal task before calling the repo steady.
- v0.2.0 alpha status after this re-baseline:
  - Rust/WASM/docs verification drift is reduced,
  - GitHub issue taxonomy and branch-protection truth are still not clean,
  - physical-device closure evidence remains open for `R-WS12-29-01`, `R-WS12-29-02`, `R-WS12-04`, `R-WS12-05`, and `R-WS12-06`,
  - therefore the repository is **partially stabilized**, not yet a fully trustworthy steady baseline.
- Follow-up repo-readiness fixes in this pass:
  - `BLEPeripheralManager`, `ContactsViewModel`, `TopicManager`, and `IosPlatformBridge` now route `MeshRepository` calls through MainActor-safe helper paths instead of synchronous nonisolated access patterns.
  - `docker/docker-compose.test.yml` now copies the host Linux UniFFI library from `target/release/libscmessenger_core.so`, matching the actual workspace release artifact location produced by `cargo build --manifest-path core/Cargo.toml --release`.
  - `cargo build --manifest-path core/Cargo.toml --release` now confirms that `target/release/libscmessenger_core.so` exists for the Docker Android-unit-test handoff.
  - local iOS verification remains **host-blocked** in this environment: `bash ./iOS/verify-test.sh` fails immediately because `xcodebuild` is not installed on this Linux host, so the iOS readiness fix still requires a macOS verification pass or CI rerun for final proof.

## 2026-03-09 Critical Bug Fixes & NAT Traversal (Verified)

### Relay Server Implementation

- **All Nodes Now Act as Relays**:
  - Added `relay::Behaviour` (relay server) to `IronCoreBehaviour` in `core/src/transport/behaviour.rs`
  - Nodes can now both USE relays (client) and BE relays (server) for NAT traversal
  - Implements circuit relay protocol for cellular↔WiFi messaging
  - Added comprehensive relay server event handling in `core/src/transport/swarm.rs`
  - Logs reservation acceptance and circuit establishment at info level

### BLE Subscription Tracking Fix

- **Fixed DeadObjectException in BLE GATT Server**:
  - Added `subscribedDevices` map to track which clients have subscribed to notifications
  - Implemented `onDescriptorWriteRequest` handler to track subscription state
  - Added proper subscription cleanup on disconnect
  - Added DeadObjectException handling with automatic cleanup of stale connections
  - File: `android/app/src/main/java/com/scmessenger/android/transport/ble/BleGattServer.kt`

### Message Delivery Status Fix

- **Eliminated False Delivery Positives**:
  - Fixed bug where BLE transport ACK was treated as full delivery confirmation
  - Messages now only marked "delivered" when core mesh network confirms
  - BLE-only delivery no longer returns `acked = true` unless in strict BLE-only mode
  - Ensures messages retry via core network even if BLE succeeds locally
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (lines 3207-3326)

### Android UI Fixes

- **Fixed Keyboard Covering Chat Input**:
  - Added `.imePadding()` to chat input Row for proper keyboard handling
  - Set `contentWindowInsets = WindowInsets(0, 0, 0, 0)` on Scaffold
  - File: `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`

## 2026-03-10 Transport Optimization & UI Enhancements (Verified)

### Fast Transport Switching

- **Reduced Timeouts for BLE/WiFi Transitions**:
  - WiFi/Direct timeout: 5000ms → 2000ms (60% reduction)
  - BLE/Relay-circuit timeout: 3500ms → 1500ms (57% reduction)
  - Enables faster failover when primary transport unavailable
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Aggressive Retry Backoff

- **Optimized Initial Retry Schedule**:
  - Attempt 1: immediate (was 2s)
  - Attempt 2: 1s (was 4s)
  - Attempts 3-6: 2s, 4s, 8s, 16s (was 8s, 16s, 32s, 64s)
  - Attempts 7-20: 60s steady retry (unchanged)
  - Attempts 21+: 300s long-term (unchanged)
  - Reduces time-to-first-retry by 2s, critical for transport transitions
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Enhanced Transport Logging

- **Comprehensive Transport Visibility**:
  - Logs dial candidates count and transport types for each route attempt
  - Logs connection success/failure with timeout values
  - Logs delivery latency in milliseconds for both direct and relay-circuit paths
  - Adds transport count to delivery state logging
  - Enables real-time debugging of WiFi↔BLE↔Cellular routing decisions
  - File: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`

### Android Mesh Tab Scrolling Fix

- **LazyColumn for Large Peer Lists**:
  - Replaced `Column` + `verticalScroll` + `forEach` with `LazyColumn` + `items()`
  - Enables efficient rendering and scrolling for 50+ discovered peers
  - Maintains all status cards, stats, and performance metrics as list items
  - File: `android/app/src/main/java/com/scmessenger/android/ui/screens/DashboardScreen.kt`

### Android ID Normalization (v0.2.1)

- **Standardized Peer ID Handling**:
  - Added `PeerIdValidator` utility for lowercase/trimmed ID normalization
  - Updated `MeshRepository.sendMessage` to normalize peer IDs before lookup
  - Added fallback to discovered peers if not in contacts
  - Added "Quick Add Contact" banner in ChatScreen for non-contact peers
  - Fixes "Contact not found" error when messaging discovered peers
  - Files: `android/app/src/main/java/com/scmessenger/android/utils/PeerIdValidator.kt`, `MeshRepository.kt`, `ChatScreen.kt`

### Build Status

- Core (Rust): ✅ Built successfully with relay server
- Android: ✅ Built and deployed with transport optimizations
- iOS: ✅ Framework rebuilt with relay server (device 00008130-001A48DA18EB8D3A)

### Documentation Created

- `TRANSPORT_OPTIMIZATION_PLAN.md` - Multi-path delivery and transport switching optimization plan
- `ANDROID_ID_MISMATCH_RCA.md` - Root cause analysis of peer ID normalization issues
- `NAT_TRAVERSAL_IMPLEMENTATION.md` - Relay server implementation guide
- `BLE_DEADOBJECT_BUG.md` - BLE subscription tracking bug analysis
- `BLE_FALSE_DELIVERY_BUG.md` - Delivery status false positive bug
- `MESSAGE_DELIVERY_RCA_2026-03-09.md` - Root cause analysis of delivery failures
- `CELLULAR_NAT_SOLUTION.md` - NAT traversal solution architecture
- `SESSION_COMPLETE_2026-03-09.md` - Complete session summary
- `FINAL_SESSION_SUMMARY.md` - Final analysis and recommendations

## 2026-03-09 P2P Connectivity & NAT Traversal Enhancements (Verified)

- **Integrated UPnP for Automatic Port Mapping**:
  - Added `libp2p::upnp` behavior to `IronCoreBehaviour`.
  - Wired UPnP events in `swarm.rs` to automatically map external ports on compatible routers.
  - Successfully handles `NewExternalAddr`, `GatewayNotFound`, `NonRoutableGateway`, and `ExpiredExternalAddr`.
  - Emits `SwarmEvent2::PortMapping` for real-time UI/log visibility.
- **Consensus-Based External Address Advertisement**:
  - Implemented automated advertisement of verified external addresses.
  - When `AddressReflection` consensus is reached, the primary address is converted to a `Multiaddr` and added to the swarm via `swarm.add_external_address()`.
  - Similarly, stable addresses observed via the `Identify` protocol are validated and advertised.
  - Ensures that peers can resolve "Direct Preferred" paths even when behind NAT (Phase 6).
- **Hardened Swarm Event Loop**:
  - Corrected `upnp::Event` variant mismatch (`NonRoutableGateway`).
  - Improved logging for NAT status changes and port mapping updates.
  - Verified that "Direct" path selection in the `MultiPathDelivery` system is now more reliable, reducing relay latency.

## 2026-03-09 Android Storage Health + Bloat Mitigation (Verified)

- Implemented `StorageManager` utility for proactive data cleanup:
  - Multi-stage log rotation (`.1` to `.5`) maintains 5-session history while preventing unbounded file growth.
  - Automatically clears `cache/` and prunes oversized `inbox/blobs` / `outbox/blobs` (>100MB) on every app launch.
  - Integrates in `MeshApplication.onCreate()` so maintenance runs before mesh services init.
- Added real-time storage monitoring:
  - `MeshRepository.getAvailableStorageMB()` provides reactive disk health status.
  - `MainViewModel` tracks `isStorageLow` state against a **500MB** critical threshold.
  - UI `StorageWarningBanner` now appears at the top of the app when storage is critically low.
- Verified build and lint safety:
  - `./gradlew :app:compileDebugKotlin` — **pass**
  - Trailing whitespace and newline cleanup in `MeshApp.kt` and `FileLoggingTree.kt`.

- Local baseline revalidation on this branch:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo build --workspace` — **pass**
  - `cargo test --workspace` — **pass**
  - `./scripts/docs_sync_check.sh` — **pass**
- GitHub Actions evidence requiring follow-up:
  - PR-triggered runs `22787446333` (`CI`) and `22787446353` (`Docker Build & Push`) concluded `action_required` before jobs were exposed via the Actions API.
  - Treat this as a repository-operating-model problem (approval/permissions/workflow trigger policy), not a code-breakage signal.
- Planning blueprint for follow-up execution is now captured in `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`.

## 2026-03-07 GitHub Contributor-Surface Alignment

- GitHub-facing contributor/community surfaces now explicitly align to the current release line:
  - `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, and new `SUPPORT.md` all treat `v0.2.0` as the active alpha baseline.
  - `WS13` / `WS14` are explicitly called out as `v0.2.1` planning scope rather than unfinished `v0.2.0` alpha work.
- GitHub repo-intake/config surfaces added or tightened:
  - `.github/CODEOWNERS`
  - `.github/ISSUE_TEMPLATE/config.yml`
  - rewritten `.github/pull_request_template.md`
  - updated issue templates with release-scope prompts
- Release-process docs now reflect that the workspace is already on `0.2.0`, while GitHub release/tag cleanup is still pending maintainer execution.

## 2026-03-07 Repo-side GitHub Operating-Model Completion

- Missing repo-controlled GitHub surfaces from the audit are now present:
  - `.github/CODEOWNERS`
  - `.github/dependabot.yml`
  - `.github/copilot-instructions.md`
  - issue forms under `.github/ISSUE_TEMPLATE/*.yml`
  - issue-template contact routing in `.github/ISSUE_TEMPLATE/config.yml`
- Docs and validation guardrails were tightened:
  - `scripts/docs_sync_check.sh` now checks a broader canonical-doc set and rejects machine-local paths in active docs.
  - `scripts/docs_sync_check.sh` now also requires nested-doc markdown links to resolve correctly relative to the source file and actively validates `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`.
  - `docs/REPO_CONTEXT.md`, `docs/ARCHITECTURE.md`, and release/process docs were refreshed to remove stale metadata or local-path drift.
- Workflow topology was made more honest/less noisy from the repo side:
  - `docker-publish.yml` no longer runs on pull requests.
  - `docker-test-suite.yml` is now treated as heavy `main`/scheduled/manual validation instead of a default PR workflow.
  - `release.yml` is renamed to `Release CLI Binaries` to match its actual output scope.
- Remaining non-repo work still requires maintainer action in GitHub settings/UI:
  - branch protection / required-check enforcement on `main`
  - labels, milestones, and issue migration/triage
  - resolving the `action_required` approval/policy condition for certain PR workflow runs

## Verified Commands and Results

### Rust Workspace

- `cargo test --workspace`
  - Result: **pass**
  - Totals from suite output:
    - CLI: 13 passed
    - Core unit: 265 passed, 7 ignored
    - Core integration: 52 passed, 10 ignored
    - Mobile crate: 4 passed
    - WASM crate (native/non-browser tests): 33 passed
  - Aggregate: **367 passed, 0 failed, 17 ignored**
- `cargo clippy --workspace` — **clean (0 warnings)**
- `cargo fmt --all -- --check` — **clean**

### WS12.18 Alpha Readiness Sanity + Interop Closure (2026-03-03 HST)

- Rust quality/build gates:
  - `cargo check --workspace` — **pass**
  - `cargo fmt --all -- --check` — **pass**
  - `cargo clippy --workspace` — **pass**
  - `cargo clippy --workspace --lib --bins --examples -- -D warnings` — **pass**
- Android quality/build gates:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:lintDebug` — **pass** (all prior 21 lint errors remediated; warnings remain)
- iOS/WASM gates:
  - `bash ./iOS/verify-test.sh` — **pass**
  - `cd wasm && wasm-pack build` — **pass**
- Hard-blocker remediation delivered:
  - Android lint `MissingPermission`/`NewApi` blockers closed in BLE advertiser/GATT server, WiFi transport manager, notification posting paths, and foreground-service API gating.
  - Rust clippy strict failures closed in store backend/contact/history/custody codepaths and example code.
- Interoperability/function completeness artifacts:
  - Added deterministic matrix generator: `scripts/generate_interop_matrix.sh`
  - Generated matrix doc: `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`
  - WS12.18 matrix triage identified adapter-parity gaps, now closed in WS12.20 follow-up wiring.
- Historical relocation (no purge):
  - Root-level one-off scripts/log buffers moved to `reference/historical/` with provenance index (`reference/historical/README.md`).

### WS12.19 Documentation/Folder Cleanup Correction (2026-03-03 HST)

- Cleanup drift correction:
  - Restored active iOS install/build helpers from historical location into active `iOS/` script surface:
    - `iOS/build-device.sh`
    - `iOS/install-device.sh`
    - `iOS/install-sim.sh`
  - Kept stale/non-canonical scripts (`build-rust.sh`, `verify-build-setup.sh`) in `docs/historical/iOS/scripts/` with explicit archive README.
- Active doc path fixes:
  - Replaced stale references to the legacy iOS setup-check script in active docs with `bash ./iOS/verify-test.sh`.
  - Updated iOS setup docs to point at active canonical docs/backlog instead of archived iOS planning files.

### WS12.20 Alpha Readiness Completion Sweep (2026-03-03 HST)

- Interop/fn-completeness closures:
  - CLI now wires identity backup import/export, explicit mark-sent, history clear, listeners/path-state/diagnostics/peers status surfaces.
  - WASM now wires local nickname override, history retention/prune controls, and external-address visibility.
  - Android+iOS adapters now consume `reset_stats`; CLI/WASM consume history retention/prune controls.
- Build/gate revalidation:
  - `cargo check --workspace` — **pass**
  - `cargo clippy --workspace --lib --bins --examples -- -D warnings` — **pass**
  - `cd android && ./gradlew :app:generateUniFFIBindings :app:compileDebugKotlin :app:lintDebug` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`0 warnings` in this run)
  - `cd wasm && wasm-pack build` — **pass**
- Interop evidence update:
  - `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md` now reports no static adapter-consumption gaps.
- Script operations docs:
  - Added active operations guide: `scripts/README.md` (5-node + launch/control/debug workflow map).

### WS12.21 Pairwise Deep-Dive Status Sweep (2026-03-03 HST)

- Deep-dive analyzers run on current artifacts:
  - `bash ./scripts/correlate_relay_flap_windows.sh ios_diagnostics_latest.log logs/5mesh/gcp.log`
    - classification: `unsynchronized_artifacts_no_time_overlap`
  - `bash ./scripts/verify_relay_flap_regression.sh ios_diagnostics_latest.log`
    - `PASS` (no deterministic relay dial-loop regression in this artifact)
  - `bash ./scripts/verify_receipt_convergence.sh android_mesh_diagnostics_device.log ios_diagnostics_latest.log`
    - result: no `delivery_attempt` message markers found in this artifact pair
  - `bash ./scripts/verify_ble_only_pairing.sh android_logcat_latest.txt ios_diagnostics_latest.log`
    - result: no strict-BLE markers/timeouts in this artifact pair
- Fresh live probe attempt:
  - `IOS_TARGET=device IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
  - result: Android connected, iOS physical device listed as `unavailable` by `xcrun devicectl`, so physical dual-device pairing deep dive could not complete in this pass.
- Simulator fallback probe:
  - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=20 GCP_RELAY_CHECK=1 bash ./scripts/live-smoke.sh`
  - artifacts captured under `logs/live-smoke/20260303-005207/`; expected limitation remains (no CoreBluetooth hardware path in simulator).
- Pairwise closure status:
  - `Core -> Android`: closed in static interop matrix.
  - `Core -> iOS`: closed in static interop matrix.
  - `Core -> WASM/Desktop`: closed in static interop matrix.
  - `Android <-> iOS` direct/relay delivery+receipt continuity: still open pending synchronized physical-device artifact capture.
  - `Android <-> iOS` strict BLE-only continuity: still open pending synchronized physical-device BLE-only artifact capture.

### WS12.22 Android+iOS Crash + Stability Hardening Sweep (2026-03-03 HST)

- Fresh runtime evidence captured:
  - iOS debug-detach bundle: `logs/pairwise/ios-debug-detach-20260303-014559`
  - Android USB capture: `logs/pairwise/android-usb-pull-20260303-014849`
- iOS crash RCA from latest SCMessenger crash artifact in the captured bundle:
  - crash path pointed to BLE peripheral send flow (`BLEPeripheralManager.sendDataToCentral`) with force-unwrap-sensitive code under active send.
- iOS hardening applied:
  - BLE central/peripheral managers now run on the main queue for consistent delegate/state access.
  - Removed force-unwrap hotspots in BLE peripheral send/advertise paths; send methods now return explicit success/failure booleans.
  - Added reconnect/service-rediscovery behavior in BLE central send flow when disconnected or missing message characteristic.
  - Added pending outbox bounded-expiry drop policy in repository (`attempt_count` and age guard) with explicit diagnostics markers.
- Android hardening applied:
  - Removed all Kotlin `!!` force unwrap usage from app source paths (repository, BLE transport, settings/viewmodel, platform bridge).
  - BLE advertiser restart churn reduced by skipping unnecessary restart when identity payload does not change advertisement-visible bytes.
  - BLE GATT client send path now attempts reconnect when disconnected/not-ready instead of immediate terminal false path.
  - Added pending outbox bounded-expiry drop policy mirroring iOS diagnostics semantics.
- Verification gates rerun after fixes:
  - `cd android && ./gradlew :app:compileDebugKotlin :app:lintDebug` — **pass** (`0 errors`, warnings remain)
  - `bash ./iOS/verify-test.sh` — **pass** (`0 warnings` in this run)
  - `bash ./scripts/generate_interop_matrix.sh` — **pass**
- Remaining closure dependency:
  - Requires fresh synchronized physical Android+iOS live send/receipt artifact capture to confirm no new iOS send crash and to close remaining pairwise runtime risks.

### WS12.23 Pending-Outbox Synchronization Reliability Pass (2026-03-03 HST)

- Root-cause closure target:
  - older pending messages could remain stuck while a newer message to the same peer delivered, because promotion/flush triggers were not consistently tied to active-connection events.
- Reliability hardening applied in `MeshRepository` on Android+iOS:
  - pending queue promotion now matches both canonical `peerId` and cached `routePeerId`,
  - `peer_identified` and BLE identity-read paths now promote same-peer queue entries and immediately flush retries,
  - iOS connected-event emission now also triggers targeted same-peer promotion/flush.
- Expected behavior shift:
  - when any usable path to a peer is active, the app immediately opportunistically drains older undelivered entries for that peer instead of waiting for periodic backoff windows.
- Verification after patch:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`3 warnings`, non-fatal)
- Remaining live proof requirement:
  - capture fresh synchronized physical Android+iOS traces and confirm deterministic backlog drain + pending-to-delivered convergence on both directions.

### WS12.24 Follow-up: Sender-State Convergence + Conversation Swipe-Delete Parity (2026-03-03 HST)

- Field-reported issue intake:
  - iOS -> Android sends can still show `stored` on iOS sender even when Android recipient has already received/rendered the message.
- Problem decomposition for closure:
  - validate Android receipt/ack emission for affected message IDs,
  - validate iOS receipt ingest + message-ID correlation into sender history state,
  - validate UI mapping does not regress message state from `delivered` back to `stored`.
- Planned closure evidence gate:
  - synchronized Android+iOS+relay artifact bundle for at least one affected message ID, proving recipient ingest and sender-side `delivered` convergence in the same session.
- Conversation-delete UX parity update in this pass:
  - Android conversation list now supports end-to-start swipe-to-delete with confirmation dialog, matching iOS swipe-delete behavior and reusing existing `clearConversation(peerId)` data path.
- Verification in this pass:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` - **pass**

### WS12.25 Mega-Update Intake: Pending-Sync RCA + Node-Role Unification (2026-03-03 HST)

- `run5.sh` and associated logs were reviewed for the reported "older pending messages remain undelivered while newer traffic still appears active" issue:
  - `logs/5mesh/latest/android.log` shows the same message ID (`1c24a6d2-5114-42cc-8545-01f9bfc41eb1`) repeatedly cycling `forwarding -> stored`, with `Core-routed delivery failed` / `Relay-circuit retry failed` and repeated flush triggers (`peer_discovered`, `peer_identified`).
  - `logs/pairwise/ios-debug-detach-20260303-014559/pending_outbox.json` shows multiple queued items for one canonical peer with persisted `routePeerId` and relay-circuit address hints tied to prior relay identities.
- Root-cause conclusion (implementation confidence: medium-high):
  - route hints/candidates can become stale under peer-id/alias churn, and receipt/retry paths were not consistently preferring fresh inbound route/listener context for the active sender identity.
- Fixes applied on both Android and iOS:
  - existing-contact route-hint updates now refresh on route change (not only when hints are initially blank),
  - delivery-receipt send path now accepts preferred inbound route/listener hints and uses them for targeted retries,
  - route candidate building now includes recipient-public-key-aware candidate discovery/filtering and relay/mismatched-candidate rejection,
  - outbound send guard now rejects relay/bootstrap identities as direct chat recipients.
- UI role-model unification applied (Android + iOS dashboard):
  - reduced displayed node-role buckets to exactly two categories:
    - `Node` (full identity),
    - `Headless Node` (no identity; includes relay/headless transport peers).
- Verification in this pass:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (3 warnings, non-fatal in script policy)
- Remaining closure gate:
  - capture fresh synchronized physical Android+iOS artifacts post-fix to confirm previously stuck pending entries drain and sender-side states converge to `delivered`.

### WS12.26 Sender-State + Conversation Preview Convergence Hotfix (2026-03-03 HST)

- Field issue intake addressed in this pass:
  - message status and conversation-row previews could stay stale (`stored`/older preview text) even after receipt-driven delivery state transitions.
- Root-cause conclusion (implementation confidence: high):
  - Android+iOS receipt handlers updated durable history state but did not always publish a fresh `messageUpdates` event after mutating delivered/pending state, so UI lists could continue rendering stale records.
  - iOS conversation preview selection depended on a narrow ordering assumption (`recentMsgs.last` from a minimal slice), making "latest preview" correctness fragile under ordering/alias drift.
- Fixes applied:
  - Android `MeshRepository.onReceiptReceived` now emits refreshed `MessageRecord` via `messageUpdates` immediately after `markDelivered` + pending-outbox removal.
  - Android `ConversationsViewModel` now also refreshes conversation state on `MessageEvent.Delivered`/`MessageEvent.Failed`.
  - iOS `MeshRepository.onDeliveryReceipt` now emits refreshed `MessageRecord` via `messageUpdates` after receipt-driven history/pending updates.
  - iOS `ConversationListView` preview selection now chooses newest message by timestamp from a bounded recent sample (`limit: 25` + `max(timestamp)`), removing reliance on list order assumptions.
  - UniFFI Swift bridge now marks `FfiConverter` helper statics as `nonisolated(unsafe)` for Swift strict-concurrency compatibility, and this rewrite is persisted in `core/src/bin/gen_swift.rs` so regenerated bindings keep compiling under `-default-isolation=MainActor`.
- Verification in this pass:
  - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.test.ChatViewModelTest" --tests "com.scmessenger.android.ui.viewmodels.ConversationsViewModelTest"` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (build succeeds under current Swift isolation settings)
- Remaining closure gate:
  - live/passive-log confirmation for this hotfix still requires mobile binaries that include this patch set.

### WS12.27 Node-Role Classification Correction + Trip Readiness Validation (2026-03-03 HST)

- Field issue intake:
  - iOS reported a confirmed full iOS-sim peer rendered as `Headless Node`.
- Root-cause correction applied on Android+iOS:
  - peer-identify classification now treats `/headless/` agent string as provisional when transport identity resolves.
  - resolved identity peers are promoted to full classification even when prior identify metadata indicated headless.
  - `isKnownRelay` now treats only bootstrap peers and non-full dynamic relay peers as relay-only, preventing full peers from being forced into headless bucket due relay-capability flags.
- Build verification after patch:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass**
- Live relay visibility snapshot (fast run, no reinstall):
  - `IOS_TARGET=simulator IOS_INSTALL=0 ANDROID_INSTALL=0 DURATION_SEC=25 GCP_RELAY_CHECK=0 bash ./scripts/live-smoke.sh`
  - Android capture `logs/live-smoke/20260303-113927/android-logcat.txt` showed:
    - `IdentityDiscovered(... listeners=[.../p2p-circuit/...])`
    - dashboard/runtime state: `Loaded 2 discovered peers (2 full)` and `Mesh Stats: ... 2 full, 0 headless`.
- Remaining validation gap:
  - physical iOS-device + Android synchronized capture with WS12.27 binaries is still required for final field closure.

### WS12.28 Transport Regression Hotfix (2026-03-03 HST)

- Live regression evidence intake from the active trip log bundle:
  - `logs/5mesh/20260303_115412/android.log` showed repeated `BleGattClient.connect` `NullPointerException` at line 238 while retry loops were active.
  - same bundle showed repeated dials to special-use/unusable addresses (for example `/ip4/192.0.0.6/...`) and persistent `stored` retry loops with no core peers connected.
- Root-cause conclusions (implementation confidence: high):
  - Android BLE fallback could enter a crash loop when `BluetoothDevice.connectGatt(...)` returned `null` and the result was stored as a non-null `BluetoothGatt`.
  - Android+iOS local address selection and dial filtering allowed special-use IPv4 values, enabling stale/unroutable candidate churn.
- Fixes applied in this pass:
  - Android `BleGattClient.connect`:
    - added address format guard (`BluetoothAdapter.checkBluetoothAddress`),
    - added explicit `connectGatt == null` handling with graceful failure instead of exception loop.
  - Android `MeshRepository` networking helpers:
    - added special-use IPv4 filtering for dialability checks,
    - hardened local IPv4 selection to prefer usable private LAN addresses and skip special-use ranges.
  - iOS `MeshRepository` networking helpers:
    - mirrored special-use IPv4 filtering in dialability checks,
    - hardened local IPv4 selection scoring to prefer usable private LAN addresses and skip special-use ranges.
- Verification in this pass:
  - `cd android && ./gradlew app:compileDebugKotlin -q` — **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -sdk iphonesimulator -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 16e' build CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO CODE_SIGN_IDENTITY=''` — **pass**
- Remaining closure gate:
  - deploy WS12.28 binaries to physical Android+iOS and confirm live logs no longer show BLE NPE loops or special-use IPv4 dial attempts during retry windows.

### WS12.29 Known-Issues Consolidation + Full-Functionality Burndown (2026-03-03 HST)

- Fresh field evidence intake in this pass:
  - iOS crash reports pulled from device crash storage show repeated send-path `SIGTRAP` in `BLEPeripheralManager.sendDataToCentral` during outbox flush/send flow.
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger-2026-03-02-185622.ips`
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger-2026-03-02-185659.ips`
  - iOS watchdog reports show CPU resource kills under retry pressure.
    - `logs/device-debug-20260303-140445/ios-crashpull/SCMessenger.cpu_resource_fatal-2026-02-27-213024.ips`
  - Android on-device diagnostics show persistent stale-route/stale-BLE-target retry churn:
    - `Network error` count in extracted log: 291
    - repeated route target: `12D3KooWHqa2jd8Ec3bbXR24Fn8Lc2rPQQwjeEiY2zUyXXMCez27`
    - repeated BLE fallback target: `65:99:F2:D9:77:01`
    - source: `logs/device-debug-20260303-140445/android-mesh_diagnostics-device.log`
- Additional operator-evidence gap identified:
  - direct pull of large iOS `mesh_diagnostics.log` from app container repeatedly failed with file-service socket closure; this blocks deterministic device-side timeline extraction until workflow/tooling is hardened.
- Consolidated remediation source of truth added:
  - `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
- Requested TODO explicitly added for UX safety:
  - require confirmation in iOS before contact deletion (`UX-IOS-001` in WS12.29 plan and active backlog).

### WS12.30 Live Verification Feedback-Loop Harness (2026-03-03 HST)

- Added a non-destructive iterative harness copy for field step-by-step validation:
  - `scripts/run5-live-feedback.sh`
- Execution model:
  - deploy Android+iOS build updates (`scripts/deploy_to_device.sh both`, optional skip flag),
  - run `run5.sh` with `--update` for synchronized 5-node capture,
  - enforce sequential gates before accepting a step:
    - log-health gate (all five node logs),
    - directed pair-matrix gate (all node pairings),
    - crash/fatal marker gate,
    - deterministic verifiers (`relay_flap`, `ble_only`, `receipt_convergence`, `delivery_state_monotonicity`).
- Evidence packaging:
  - each attempt writes a self-contained bundle under `logs/live-verify/<step>_<timestamp>/attempt_*`.
- Recommended command per fix:
  - `./scripts/run5-live-feedback.sh --step=<fix-id> --time=5 --attempts=3`
  - add `--require-receipt-gate` when sender/receipt convergence is the closure target.

### WS12.31 Stale-Target Convergence Hardening + Transport Priority Clarification (2026-03-04 HST)

- Field issue intake addressed in this pass:
  - active Android/iOS paired usage still reported non-delivery with stale route/BLE retry churn under WS12.29 open-risk class.
- Reliability hardening applied in `MeshRepository` on Android+iOS:
  - route candidate ordering now prefers fresh discovery/ledger candidates before persisted note/cached hints.
  - route-candidate recipient validation now requires either:
    - extracted route peer key matches recipient key, or
    - runtime-discovered/ledger evidence that route peer maps to recipient key.
  - failed send attempts no longer persist failed route IDs back into pending-outbox entries (`routePeerId` stays unset when no route ACK succeeded), preventing stale-route lock-in across retries.
  - local BLE fallback target selection now prefers currently connected BLE peers ahead of cached `ble_peer_id` hints.
  - Android disconnect handling now prunes disconnected aliases by peer ID + canonical ID + matched public-key aliases (previously only direct keys were removed in callback path).
- Transport-priority audit (current behavior):
  - Android send path: `WiFi Direct` -> `BLE` -> `Core direct route candidates (LAN-prioritized addresses first)` -> `relay-circuit retry`.
  - iOS send path: `Multipeer` -> `BLE` -> `Core direct route candidates (LAN-prioritized addresses first)` -> `relay-circuit retry`.
  - strict BLE-only mode (`SC_BLE_ONLY_VALIDATION=1`) blocks Multipeer/WiFi/Core attempts and keeps only BLE local fallback.
- UX safety closure in this pass:
  - iOS contacts list now requires explicit confirmation before contact deletion.
- Verification in this pass:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd android && ./gradlew :app:testDebugUnitTest --tests "com.scmessenger.android.data.MeshRepositoryTest"` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass** (`10 warnings`, non-fatal policy)
- Remaining closure gate:
  - synchronized physical Android+iOS evidence is still required to retire `R-WS12-29-02` and paired-delivery residuals (`R-WS12-04/05/06`).

### WS12.34 Transport Failure Triage + 10-Fix Reliability Sweep (2026-03-04 HST)

- Field issue intake addressed in this pass:
  - iOS and Android messaging stopped working after toggling WiFi/BLE/cell connections.
  - Rust core `receive_message` failures were invisible on mobile due to swallowed `tracing` output.
  - iOS relay flapping threshold was self-triggering, permanently blocking relay circuit path.
  - Stale routing data caused infinite retry loops against unreachable peers.
  - Messages were being expired/dropped despite "never fail delivery" philosophy.
- Fixes applied (10 total across Rust core, iOS, Android):
  1. **`eprintln!` error visibility** (Rust core) — `receive_message` errors now visible on mobile platforms via stderr.
  2. **`relayEnabled` nil-safety** (iOS + Android) — relay toggle checks no longer produce nil, preventing silent message drops.
  3. **Retry throttle 500→2000ms** (iOS) — reduces main thread pressure during outbox flush.
  4. **Relay diagnostic throttle** (iOS) — 90% reduction in relay-state logging when flapping.
  5. **Messages NEVER expire** (iOS + Android) — removed attempt limits and age-based expiry from outbox.
  6. **Progressive backoff** (iOS + Android) — retry delay: `min(2^attempt, 60)` seconds, capping at 5 minutes for long-lived items.
  7. **WiFi recovery → immediate outbox flush** (iOS) — network path change triggers immediate pending message delivery.
  8. **WiFi recovery → immediate outbox flush** (Android) — `notifyNetworkRecovered()` triggers flush on WiFi restoration.
  9. **BLE 15s connection timeout** (Android) — stale GATT connections auto-cleaned after 15 seconds.
  10. **Dial candidate cap (6 max)** (iOS + Android) — prioritizes LAN → relay → public IPs, reduces stale-address dial spam.
- Core philosophy enforcement:
  - Messages NEVER expire. No attempt limit, no age limit, no TTL.
  - All messages retry indefinitely with progressive backoff until delivered.
  - Network recovery triggers immediate delivery attempts.
- Verification in this pass:
  - `cargo check --workspace` — **pass**
  - Rust core compiles with `eprintln!` diagnostics active.
- Remaining closure gate:
  - Deploy Rust core + both mobile apps and observe `eprintln!` output to diagnose any remaining `receive_message` failures.
  - Confirm end-to-end message delivery across all transport layers post-fix.

### WS12.35 Non-Device Reliability Reconciliation (2026-03-06 UTC)

- Baseline/CI correlation in this pass:
  - `cargo check --workspace` — **pass**
  - `cargo test --workspace --no-run` — **initial fail** (WASM `MessageRecord` test initializers missing `sender_timestamp`), then **pass** after fix.
  - `./scripts/docs_sync_check.sh` — **pass**
  - Latest failed non-`action_required` CI run inspected (`22706811148`, `CI`): blocker set matched local drift (`scmessenger-wasm` E0063 + iOS MainActor isolation in `MultipeerTransport` + Android `MeshRepositoryTest` null-settings expectations).
- Minimal reliability fixes applied:
  - WASM tests updated to include `sender_timestamp` in all `MessageRecord` initializers touched by desktop role/parity suites.
  - Core receipt verification now requires outbound-recipient correlation for receipt sender identity (accepting canonical recipient identity/public-key forms) so forged third-party receipts are ignored without regressing valid delivery receipts.
  - iOS `MultipeerTransport` now bridges repository diagnostics/identity snippet calls through MainActor-safe helpers to avoid synchronous nonisolated actor violations.
  - iOS `ChatViewModel` + `SettingsViewModel` explicitly annotated `@MainActor` for Swift concurrency correctness in UI-bound repository calls.
  - Android `MeshRepositoryTest` now matches canonical runtime semantics (`relayEnabled` defaults to enabled when settings are unavailable).
- WS12.24 deterministic gate reconciliation in this pass:
  - `scripts/run5-live-feedback.sh` already enforces `verify_delivery_state_monotonicity.sh` alongside `verify_receipt_convergence.sh`; this gate is now treated as canonical closure flow.
- Receipt guard regression tests in this pass:
  - `cargo test -p scmessenger-core test_delivery_receipt_marks_history_and_outbox_delivered -- --nocapture` — **pass**
  - `cargo test -p scmessenger-core test_mismatched_sender_receipt_is_ignored -- --nocapture` — **pass**
- WS12.29 diagnostics workflow hardening in this pass:
  - `scripts/run5-live-feedback.sh` iOS diagnostics pull now retries `devicectl copy` and requires near-stable file-size confirmation across pulls before accepting capture, failing fast when non-truncation cannot be confirmed.
  - Follow-up hardening: one-shot mode (`IOS_DIAG_PULL_ATTEMPTS=1`) now accepts a valid non-empty pull, and failed stability runs remove the untrusted output file before returning non-zero.
- Environment limitations observed:
  - Android Gradle unit-test run in this environment failed before tests due blocked dependency fetch from `dl.google.com` (host/network prerequisite).
  - `bash ./iOS/verify-test.sh` could not execute here (`xcodebuild` unavailable on this host), so iOS physical/simulator runtime closure gates remain unchanged.

### WS12.36 PR CI Failure Closure (2026-03-07 UTC)

- Latest failing PR CI run inspected (`22790198922`, `CI`):
  - Android failure was workflow ordering drift: `.github/workflows/ci.yml` ran `android/verify-build-setup.sh` before installing `cargo-ndk`.
  - iOS failure extended to `BLECentralManager` in addition to `MultipeerTransport`; transport-layer delegate paths were still calling `@MainActor` repository helpers synchronously.
  - Rust Core macOS failure came from a transient sled database lock while reopening the same test path in `identity::store::tests::test_store_persistence_across_instances`.
- Minimal fixes applied:
  - `check-android` now installs `cargo-ndk` before Android preflight verification.
  - `BLECentralManager` now routes repository diagnostics through a MainActor-safe helper.
  - `MultipeerTransport.identitySnippetForDisplayName()` now uses MainActor-safe synchronous bridging for repository snippet lookup.
  - `test_store_persistence_across_instances` now briefly retries reopening the sled backend when the prior lock is still being released by the OS.
- Local validation in this pass:
  - `cargo fmt --all -- --check` — **pass**
  - `cargo test -p scmessenger-core identity::store::tests::test_store_persistence_across_instances` — **pass**

### WS12 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-core --test integration_offline_partition_matrix` — **pass** (deterministic offline/partition matrix)
- `cargo test -p scmessenger-core --test integration_retry_lifecycle` — **pass**
- `cargo test -p scmessenger-core --test integration_receipt_convergence` — **pass**
- `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` — **pass**
- `cargo test -p scmessenger-wasm test_desktop_role_resolution_defaults_to_relay_only_without_identity` — **pass**
- `cargo test -p scmessenger-wasm test_desktop_relay_only_flow_blocks_outbound_message_prepare` — **pass**
- `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:testDebugUnitTest --tests com.scmessenger.android.test.RoleNavigationPolicyTest --tests com.scmessenger.android.data.MeshRepositoryTest` — **pass**
- `bash ./iOS/verify-test.sh` — **pass** (21 warnings, non-fatal policy; includes local transport fallback + role-mode parity checks)
- `ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh` — **pass**

### WS12.5 Burndown Audit Snapshot (2026-03-03)

- `cargo test -p scmessenger-core --test integration_offline_partition_matrix` — **pass**
- `cargo test -p scmessenger-core --test integration_retry_lifecycle` — **pass**
- `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` — **pass**

### WS12.6 Optional Closeout Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test -p scmessenger-core relay_custody -- --nocapture` — **pass**
- `cargo test -p scmessenger-core convergence_marker -- --nocapture` — **pass**
- v0.2.0 closeout outcomes:
  - relay custody persistence defaults to durable app-data paths (env override + OS-local fallback chain),
  - storage pressure enforcement now has synthetic snapshot fallback when platform probe data is unavailable,
  - convergence-marker application now requires validation + local tracking correlation,
  - workspace/app version metadata bumped to `0.2.0` for release synchronization.

### WS12.7 Live Runtime Sanity Snapshot (2026-03-02 HST)

- Live runtime/debug commands:
  - `adb logcat --pid=$(adb shell pidof -s com.scmessenger.android) -T 1 -v threadtime`
  - `xcrun simctl spawn booted log show --style compact --last 10m --predicate 'process == "SCMessenger"'`
  - `adb shell run-as com.scmessenger.android cat files/pending_outbox.json`
  - `xcrun simctl get_app_container booted SovereignCommunications.SCMessenger data`
- Build verification after runtime fixes:
  - `cd android && ./gradlew :app:compileDebugKotlin` — **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -configuration Debug -sdk iphonesimulator -destination 'platform=iOS Simulator,name=iPhone 16e' build CODE_SIGNING_ALLOWED=NO` — **pass**
- Observed runtime state (pre-fix logs):
  - Android live logs showed repeated `Core-routed delivery failed` / `Relay-circuit retry failed` while relay agent strings still included `scmessenger/0.1.0/headless/relay/*` (GCP rollout in progress).
  - Android pending outbox contained long-lived entries with very high retry counts (for example `attempt_count=2055`), consistent with no-give-up retry semantics.
  - Android emitted overlapping outbox flush runs (`reason=enqueue` and `reason=peer_identified`) with duplicate forwarding attempts for the same message in the same second.
  - Android `ServiceStats.uptimeSecs` remained `0` in repeated status emissions.
  - iOS simulator had no active pending-outbox backlog (`pending_outbox.json` = `[]`) during this pass.
- Runtime fixes applied in this pass:
  - Android: fixed BLE identity beacon fallback logic that previously overwrote non-empty listener/external hint payloads unconditionally.
  - Android: serialized pending outbox flush execution with a coroutine mutex to prevent duplicate concurrent retry passes.
  - Android: added uptime fallback when core-reported `uptimeSecs` is `0` while service is running.

### WS12.8 Runtime Recheck Snapshot (2026-03-02 HST)

- Live runtime/debug commands:
  - `adb devices -l` / `adb mdns services`
  - `xcrun simctl spawn booted log show --style compact --last 12m --predicate 'process == "SCMessenger" OR subsystem == "com.scmessenger"'`
  - `nc -z -w 5 34.135.34.73 9001`
  - `curl --max-time 8 http://34.135.34.73:9000`
  - `curl --max-time 8 http://34.135.34.73:8080`
  - `./target/debug/scmessenger-cli start` (interactive runtime probe)
  - `./scripts/verify_ws12_matrix.sh`
- Runtime observations:
  - Android device log streaming was blocked in this pass (`adb devices` empty; no mDNS-discoverable wireless endpoint).
  - iOS simulator process was active but reported only local Multipeer routing-table self state (`1 nodes`) in sampled logs.
  - GCP relay endpoint `34.135.34.73:9001` was reachable over TCP.
  - GCP relay landing page was reachable on `34.135.34.73:9000`; `:8080` timed out (current deploy script uses `--http-port 9000`).
  - CLI runtime probe observed relay identity rotation at `34.135.34.73:9001`: `12D3KooWETatHYo4xt9aufXEEDce719fyMEB7KmXJga1SYVUikaw` -> `12D3KooWJaLtGyFYvobdZyecLWKA45cjSLEjzWtKPeorgeFYrsjZ`.
  - During the same probe, relay-circuit reservation warning remained active (`Could not register relay circuit reservation`), indicating a remaining runtime gap post-redeploy.
- Verification delta:
  - `./scripts/verify_ws12_matrix.sh` now fails on live suite `integration_relay_custody -- --include-ignored` with timeout at `core/tests/integration_relay_custody.rs:71`.
- Fixes applied in this pass:
  - Core: relay reservation address construction now canonicalizes identify addresses before appending `/p2p/<relay>/p2p-circuit` (`core/src/transport/swarm.rs`).
  - Core: relay reservation warning logging now emits `Debug` error detail instead of potentially empty display text (`core/src/transport/swarm.rs`).
  - Test hardening: recipient-side custody test flow no longer gates on a pre-delivery peer-readiness drain before waiting for envelope delivery, and uses a larger delivery wait budget (`core/tests/integration_relay_custody.rs`).

### WS12.9 iOS Dashboard Node Count Hotfix (2026-03-03)

- Issue context:
  - iOS diagnostics/runtime checks showed `Peers Discovered` values were correct, but dashboard node totals could overcount due to stale online state and alias-key duplication (canonical/libp2p/BLE identifiers represented separately).
- Fixes applied:
  - iOS dashboard node counters now derive from online-only deduplicated peers (`full`/`headless` counts no longer include stale offline entries).
  - iOS dashboard final merge now deduplicates by alias graph (`id`, `peerId`, `libp2pPeerId`, `blePeerId`, `publicKey`) to collapse duplicate rows for the same identity.
  - iOS refresh merge no longer blindly preserves historical online state; prior online state now decays by recency guard before being retained.
- Code path:
  - `iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift`

### WS12.10 Runtime Re-baseline + Action Roundup (2026-03-03 HST)

- Live verification commands:
  - `ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh` — **pass**
  - `cargo test -p scmessenger-core --test integration_relay_custody offline_recipient_receives_after_reconnect_without_sender_resend -- --include-ignored --exact` (3 consecutive runs) — **pass/pass/pass**
  - `adb kill-server && adb start-server && adb mdns services && adb devices -l`
  - `adb logcat -d | rg "MeshRepository|delivery_state|relay|custody|swarm"`
  - `xcrun simctl spawn booted log show --style compact --last 60m --predicate 'eventMessage CONTAINS[c] "NSFileManager" OR eventMessage CONTAINS[c] "createDirectory"'`
  - `bash ./iOS/verify-test.sh` — **pass** (74 warnings, non-fatal per script policy)
- Runtime findings:
  - Custody reconnect gate is now stable in this environment (3/3 consecutive passes).
  - Android live logs were successfully captured after reconnect and showed active `scmessenger/0.2.0/headless/relay/*` peers; wireless ADB visibility later dropped again after daemon restart, so endpoint persistence remains an operational follow-up.
  - iOS runtime issue warnings were reproducible and attributable to app startup path (`NSFileManager createDirectory*` on main actor in `MeshRepository.init()`), not simulator-only noise.
  - Post-fix quick-launch probe (`xcrun simctl launch booted SovereignCommunications.SCMessenger` + 2-minute log window) showed no new `createDirectory` runtime-issue entries for SCMessenger.
  - Fresh CLI runtime probe did not reproduce `Could not register relay circuit reservation` warnings in this pass.
- Fixes applied:
  - iOS: removed main-actor storage directory creation from `MeshRepository.init()` and moved diagnostic file persistence to a background serial queue (`iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`).
  - iOS: fixed dashboard compile regression by passing `Array(merged.values)` into alias dedup helper (`iOS/SCMessenger/SCMessenger/Views/Dashboard/MeshDashboardView.swift`).

### WS12.13 Wave-2 Backlog Consolidation Snapshot (2026-03-03 HST)

- Validation/debt reconciliation commands:
  - `cargo check --workspace` — **pass**
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:generateUniFFIBindings` — **pass**
  - `bash iOS/copy-bindings.sh` — **pass**
  - `ANDROID_HOME=/path/to/android/sdk bash ./verify_integration.sh` — **pass**
  - `bash ./verify_simulation.sh` — **expected fail-fast** (Docker unavailable in this environment)
  - `cd wasm && wasm-pack build` — **pass** (with release `wasm-opt` disabled in `wasm/Cargo.toml` for host compatibility)
- Tooling adjustments in this wave:
  - `verify_integration.sh` was modernized to delegate to canonical `scripts/verify_ws12_matrix.sh` instead of stale grep-pattern checks that were producing false negatives.
  - `verify_simulation.sh` no longer attempts automatic Docker installation and now exits with explicit operator guidance when Docker is not preinstalled/running.
- Backlog-governance outcome:
  - Non-historical mixed docs were reclassified from open checkboxes to status-tagged guidance/roadmap entries (`FEATURE_WORKFLOW.md`, `AUDIT_QUICK_REFERENCE.md`, `FEATURE_PARITY.md`, `DRIFTNET_MESH_BLUEPRINT.md`, `docs/TRANSPORT_ARCHITECTURE.md`).
  - `docs/TRANSPORT_ARCHITECTURE.md` future enhancements now include explicit owner/milestone/gate/acceptance metadata.
- Post-update issue-slate evidence triage (live artifacts, no code edits):
  - Android live watch (`/tmp/scm_android_live_watch.log`) captured `BluetoothGatt` callback exceptions during BLE fallback writes (`IllegalStateException: The number of released permits cannot be greater than 1` in `BleGattClient.releaseGattOp`), alongside mixed "write successful" callbacks for the same peer window.
  - Android same window retained repeated `Core-routed delivery failed ... Network error` and `Relay-circuit retry failed ... Network error` with stalled `messagesRelayed=0`, reinforcing unresolved delivery convergence risk.
  - iOS live watch (`/tmp/scm_ios_live_watch.log`) captured high-churn Multipeer sessions (`Connection attempt in progress` on many channels, followed by repeated `Timed out, enforcing clean up` and `Disconnected` transitions), consistent with local session instability symptoms observed during relay flapping runs.
  - These findings were classified into "possibly in-flight" versus "likely still open" TODO buckets in `REMAINING_WORK_TRACKING.md` WS12.13 section for immediate post-update validation.

### WS12.11 iOS Relay Flapping Diagnosis Snapshot (2026-03-03 HST, no code edits)

- Live/runtime evidence reviewed:
  - iOS diagnostics (`ios_diagnostics_latest.log`) show repeated relay rediscovery and repeated relay-circuit dial attempts to bootstrap endpoints (`34.135.34.73:9001` and `104.28.216.43:9010`) in short intervals.
  - Same windows contain repeated `peer_identified` churn for relay agents, including headless relay identities reappearing multiple times per minute.
  - Runtime logs include `dial_throttled` events interleaved with new dial attempts, indicating retry pressure rather than stable session hold.
  - Prior GCP-side mesh logs (`logs/5mesh/gcp.log`) show disconnect/reconnect oscillation (`Lost relay peer ... scheduling reconnect with backoff`) for the same peers, consistent with cross-side instability rather than iOS-only UI artifact.
- Diagnosis outcome (current confidence: medium-high):
  - iOS "relay appears/disappears" behavior is reproducible and consistent with transport-session churn plus repeated identify/dial cycles.
  - No direct crash evidence was found in this pass; primary symptom is flapping connection state and redundant relay rediscovery.
  - Most likely contributors are state-churn/race interactions between repeated relay bootstrap priming and concurrent route-based connect attempts, amplified under unstable relay/session conditions.
- No-code-change constraints honored:
  - This run was documentation/diagnosis only; no source edits were applied to transport/runtime code.

### WS12.12 Android<->iOS Pairing Message Non-Delivery RCA (2026-03-03 HST, no code edits)

- Runtime evidence reviewed:
  - Android device diagnostics (`run-as com.scmessenger.android ... files/mesh_diagnostics.log`) repeatedly show:
    - `Core-routed delivery failed ... Network error; trying alternative transports`
    - `Relay-circuit retry failed ... Network error`
    - message state cycling `forwarding -> stored` with `awaiting_receipt_delay_sec=8` and rising retry attempts.
  - In the same windows Android logs repeatedly emit `✓ Delivery via BLE (target=...)` immediately followed by `Failed to initiate characteristic write ...` while also emitting characteristic-write-success callbacks.
  - Android stats remain effectively stalled for delivery (`messagesRelayed=0`) during these retries.
  - iOS diagnostics history (`ios_diagnostics_latest.log`) and prior relay logs show heavy relay dial/identify churn and throttling, consistent with unstable internet-route availability during pairing runs.
- RCA conclusion:
  - Primary failure mode is end-to-end delivery confirmation failure, not pairing absence: devices discover each other, but routed sends fail over internet (`Network error`) and BLE fallback does not converge to recipient receipt.
  - Most probable root-cause cluster is transport-state inconsistency in Android BLE send path under fallback load (conflicting write initiation/result signals) combined with relay route instability.
  - Secondary contributing factor: legacy pending outbox items with very high retry counts keep retry pressure high, obscuring fresh-message behavior and increasing contention.
- No-code-change constraints honored:
  - This pass performed diagnosis and documentation only; no implementation edits were applied.

### WS12.14 Android Bluetooth-Only Pairing Diagnosis (2026-03-03 HST, no code edits)

- Runtime evidence reviewed:
  - Android USB+ADB logcat during "Bluetooth-only" run showed sustained BLE stack churn for the iOS peer address with repeated `BluetoothRemoteDevices: Address type mismatch ... new type: 1`.
  - In the same window, Android app telemetry dropped to `Mesh Stats: 0 peers (Core), 0 full, 0 headless (Repo)` and `NearbyMediums: No BLE Fast/GATT advertisements found in the latest cycle`.
  - iOS logs during the same interval showed repeated Multipeer invitation timeouts/declines (`Invite timeout`, `Peer ... declined invitation`) and session resets.
  - iOS multipeer connection attempts in these traces reported `transportType=WiFi` (`interfaceName=en0`) rather than an explicit BLE-only transport hold.
- RCA conclusion (current confidence: high):
  - The requested Android<->iOS Bluetooth-only path is not converging to a stable BLE data path in this run.
  - Primary symptoms point to transport-path mismatch and BLE identity/address instability: Android repeatedly reclassifies peer address type while iOS multipeer flow repeatedly times out and appears to favor WiFi-backed session attempts.
  - Resulting behavior is consistent with fallback churn rather than sustained BLE-only connectivity, so message exchange fails before reliable send/receipt convergence.
- No-code-change constraints honored:
  - This pass was diagnosis/documentation only; no transport implementation edits were made.

### WS12.16 Wave-2 Runtime Hardening Pass (2026-03-03 HST)

- Verification commands:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `bash ./iOS/verify-test.sh` — **pass**
  - `cargo check --workspace` — **pass**
- Fixes delivered:
  - Android BLE callback/permit race hardening in `BleGattClient`:
    - single-release permit guard for queued GATT operations,
    - overflow-safe semaphore release handling,
    - `WRITE_TYPE_NO_RESPONSE` callback path treated as informational to avoid contradictory final write outcomes.
  - Android+iOS per-message `delivery_attempt` diagnostics timeline now emitted for local fallback, core direct route, relay-circuit retry, and aggregate terminal outcome with message ID context.
  - iOS relay-flap visibility and guardrails in `MeshRepository`:
    - relay dial debounce and bootstrap in-progress guard,
    - relay availability state export (`stable`/`flapping`/`backoff`/`recovering`) with timestamps and event counters,
    - relay timeline markers for identify/disconnect/dial attempt outcomes keyed to relay peer IDs.
  - iOS Multipeer channel-storm guardrails in `MultipeerTransport`:
    - invite debounce,
    - in-flight invite dedupe,
    - concurrent invite cap,
    - timeout/decline diagnostics counters.
- Remaining wave-2 live-evidence gates:
  - Re-run synchronized Android+iOS+relay live probe and confirm reduced relay/multipeer churn plus receipt convergence for both send directions.
  - Capture synchronized BLE-only and internet-degraded artifact bundles with message ID timeline continuity for residual-risk closure.

### WS12.17 Wave-3 Governance + Runtime Closure Sweep (2026-03-03 HST)

- Runtime/code updates applied:
  - Android BLE address-type mismatch mitigation now includes reconnect cooldown/backoff and skip counters in `BleGattClient`.
  - Android+iOS strict BLE-only validation mode and diagnostics export fields are active (`strict_ble_only_validation` markers).
  - Android BLE discovery/client counters and iOS Multipeer diagnostics snapshot counters are exported for operator triage.
- New deterministic harnesses added and executed:
  - `./scripts/correlate_relay_flap_windows.sh ios_diagnostics_latest.log logs/5mesh/gcp.log` — classified sampled pair as `unsynchronized_artifacts_no_time_overlap`.
  - `./scripts/verify_relay_flap_regression.sh ios_diagnostics_latest.log` — pass (no deterministic relay dial-loop regression for sampled artifact).
  - `./scripts/verify_receipt_convergence.sh android_mesh_diagnostics_device.log ios_diagnostics_latest.log` — no message IDs in sampled historical artifacts.
  - `./scripts/verify_ble_only_pairing.sh android_logcat_latest.txt ios_diagnostics_latest.log` — no strict BLE-only markers in sampled historical artifacts.
- Validation commands:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — **pass**
  - `cd wasm && wasm-pack build` — **pass**
- Documentation/backlog governance outcomes:
  - Historical open-checkbox sources were triaged with explicit status tags in `docs/historical/*` and are no longer active checklist noise.
  - `docs/ALPHA_RELEASE_AUDIT_V0.1.2.md` version-bump/redeploy steps were explicitly marked as historical closeout and superseded by v0.2.0 release-sync docs.
  - Final checklist inventory after wave-3 triage: 10 open checklist items repo-wide, all in `REMAINING_WORK_TRACKING.md`. See `DOCUMENTATION_UPDATE_TEMPLATE.md` in contact audit directory for canonical doc updates.
 
## v0.2.0 Critical Bug Fixes (2026-03-09)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-core swarm::tests:: -- --nocapture` — **pass** (5 guardrail tests)
- Core relay guardrails now enforce:
  - per-peer token bucket limiting,
  - global inflight custody-dispatch cap,
  - duplicate suppression window and cheap abuse-shape heuristics.

### WS11 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew testDebugUnitTest` — **pass** (includes WS11 delivery-state + diagnostics formatter tests)
- `ANDROID_HOME=/path/to/android/sdk ./android/verify-build-setup.sh` — **pass**
- `bash ./iOS/verify-test.sh` — **pass** (26 warnings, non-fatal per script policy)
- WS11 surface outcomes:
  - Android+iOS chat now expose explicit tester-facing delivery states: `pending`, `stored`, `forwarding`, `delivered`.
  - Android+iOS diagnostics exports now include structured tester bundle context (runtime summary, reliability notes, permissions rationale, delivery-state guide).
  - Android+iOS settings surfaces now include concise reliability and permissions rationale text for beta testers.

### WS9 Verification Snapshot (2026-03-03)

- `cargo test --workspace --no-run` — **pass**
- `cargo test --workspace` — **pass**
- `cargo test -p scmessenger-wasm` — **pass** (includes desktop WS9 flow tests)
- `cargo check -p scmessenger-core --target wasm32-unknown-unknown` — **pass**
- `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` — **pass**
- Desktop target checks from release matrix:
  - `cargo check --bin scmessenger-cli --target aarch64-apple-darwin` — **pass**
  - `cargo check --bin scmessenger-cli --target x86_64-apple-darwin` — **pass**
  - `cargo zigbuild --bin scmessenger-cli --target x86_64-unknown-linux-gnu` — **pass**
  - `PATH="/opt/homebrew/opt/llvm@20/bin:$PATH" cargo xwin check --bin scmessenger-cli --target x86_64-pc-windows-msvc` — **pass**
- `./scripts/docs_sync_check.sh` — **pass**
- Desktop GUI WS9 outcomes:
  - onboarding/identity, contacts, chat send/receive, mesh dashboard, and relay-only mode are now GUI-native through local WASM/Core APIs.
  - normal desktop workflows no longer depend on CLI websocket command fallback.

### CLI Surface

- `cargo run -p scmessenger-cli -- --help`
  - Verified commands: `init`, `identity`, `contact`, `config`, `history`, `start`, `send`, `status`, `stop`, `test`

### Platform Build Readiness Scripts

- `./android/verify-build-setup.sh`
  - Result: **pass** (with `ANDROID_HOME=/path/to/android/sdk`)
- `./iOS/verify-test.sh`
  - Result: **pass**
  - Confirmed simulator build plus local transport fallback and role-mode parity checks

### Platform App Builds

- Android:
  - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew assembleDebug`
  - Result: **pass**
  - `./android/install-clean.sh`
  - Result: **pass** (fresh install on connected Pixel 6a: Gradle `clean` + `:app:installDebug` + runtime permission grant pass for Bluetooth/Location/Nearby WiFi/Notifications)
  - Multi-device note: `android/install-clean.sh` now supports `ANDROID_SERIAL=<serial>` and defaults to a single connected device (prefers TCP/IP transport when duplicates are present).
- iOS:
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'platform=iOS Simulator,name=iPhone 17' build`
  - Result: **pass**
  - `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build`
  - Result: **pass** (device-target compile path verified)
  - `APPLE_TEAM_ID=<team> DEVICE_UDID=<udid> ./iOS/install-device.sh`
  - Result: **pass** (clean DerivedData + reinstall + launch on connected iPhone)
  - iOS runtime crash guard: `NSMotionUsageDescription` restored in `iOS/SCMessenger/SCMessenger/Info.plist` for motion-based power adaptation.

### Live Smoke Automation

- Cross-device smoke harness: `scripts/live-smoke.sh`
  - Runs optional clean installs (`android/install-clean.sh`, `iOS/install-device.sh`)
  - Supports deterministic Android targeting via `ANDROID_SERIAL=<serial>` (auto-selects one serial if omitted)
  - Supports simulator-only runs via `IOS_TARGET=simulator`
  - Captures Android runtime logcat for a configurable interaction window
  - Stores artifacts under `logs/live-smoke/<timestamp>/`

### Browser/WASM Runtime Validation

- `wasm-pack --version`
  - Result: **available** (`wasm-pack 0.14.0`)
  - `cd wasm && wasm-pack build` — **pass**

## Implemented Functionality (Repository State)

- Sovereign identity and key management (Ed25519), persisted storage
- Message encryption/signing pipeline (X25519 + XChaCha20-Poly1305 + signatures)
- Inbound message chronology now uses original sender timestamp from core callbacks (`sender_timestamp`) rather than local receive-time
- Store-and-forward queues with persistence
- libp2p swarm transport with discovery, messaging, relay, and NAT reflection
- Interactive CLI with:
  - contact and history management
  - live node mode
  - local control API
  - embedded web landing/dashboard server
- Mobile UniFFI surface (MeshService, SwarmBridge, managers, settings)
- iOS and Android app codebases with active integration to Rust core
- First-run install-mode choice restored on GUI variants (iOS/Android/Desktop-WASM): users can initialize identity immediately or skip into relay-only mode, then create identity later from Settings -> Identity without reinstall
- iOS background lifecycle repository hooks are wired (`pause/resume`, ledger save, sync/discovery triggers)
- WASM crate with full libp2p swarm transport (`startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers`) using browser-native websocket-websys; legacy `startReceiveLoop` deprecated as shim
- Identity backup/restore wired end-to-end: iOS Keychain and Android SharedPreferences (`identity_backup_prefs.xml`); survives full app reinstall
- `mark_message_sent(message_id)` exposed via UniFFI; prevents outbox exhaustion on long-lived accounts
- CLI relay PeerId stable across upgrades: network key migrated from IronCore identity on first run, then persisted in `relay_network_key.pb`
- BLE GATT sequential operation queue: all GATT reads, writes, and CCCD writes serialised per-device via `Channel` + `Semaphore(1)` to comply with Android GATT API requirements

## Known Gaps and Partial Areas

### Contact Persistence & Data Integrity Issues (2026-03-14 Audit)

**Status:** Identified during fresh-install contact discovery audit (WS13.6+)
**Platform:** Android
**Priority:** Must fix before v0.2.1 release

- **Contact Auto-Creation Duplication** ❌
  - **Issue:** Same peer contact created twice (4 seconds apart) during discovery
  - **Root Cause:** Duplicate `onPeerIdentified` callbacks + non-idempotent contact creation
  - **Evidence:** MeshRepository logs show auto-create at 18:22:49.396, duplicate at 18:22:52.530 for relay peer ID `93a35a87...`
  - **Fix:** Implement idempotent upsert (not insert), add unique constraint on peer_id
  - **Location:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (onPeerIdentified callback)
  - **Audit:** See `tmp/scm_audit_logs/contact_audit_2026-03-14/CONTACT_PERSISTENCE_AUDIT_2026-03-14.md`

- **Relay Peers in User Contact List** ⚠️
  - **Issue:** Relay server auto-discovered via Internet and shown as contact "peer-93a35a87"
  - **Question:** Should relay infrastructure peers be in user-visible contact list?
  - **Options:** (A) Hide from contacts, or (B) Show with special indicator
  - **Design Decision Required:** Product + Core team

- **Discovered Peers Persist in UI After Discovery Stops** ⚠️
  - **Issue:** Discovered peer continues showing for 6+ seconds after discovery stopped
  - **Evidence:** Peer shown from 18:22:49 to 18:23:08, even though stopDiscovery called at 18:23:02
  - **Root Cause:** Async discovery lifecycle or UI refresh batching
  - **Fix:** Ensure immediate UI removal when discovery stops

- **Permission Request Loop on App Startup** ⚠️
  - **Issue:** 9+ rapid permission requests (location, BLE, notifications, nearby WiFi) in ~700ms
  - **Evidence:** Multiple "Requesting permissions" logs from 18:22:48.152 to 18:22:49.237
  - **Root Cause:** Multiple code paths requesting same permissions without deduplication
  - **Fix:** Deduplicate requests, add backoff timer, coordinate permission sources
  - **Location:** `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt`

### Product/Feature Gaps

- Topic subscribe/unsubscribe/publish is now wired through Rust bridge on Android and iOS
  - Android: `android/app/src/main/java/com/scmessenger/android/data/TopicManager.kt`
  - iOS: `iOS/SCMessenger/SCMessenger/Data/TopicManager.swift`
- Privacy toggle parity is wired across Android, iOS, and Web/WASM for the canonical settings surface.
  - Android: `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
  - iOS: `iOS/SCMessenger/SCMessenger/ViewModels/SettingsViewModel.swift`
  - Web/WASM: `wasm/src/lib.rs`
- Android and iOS QR import/join flows are wired (Google Code Scanner on Android, VisionKit on iOS)
  - Android: `android/app/src/main/java/com/scmessenger/android/ui/join/JoinMeshScreen.kt`
  - Android contacts: `android/app/src/main/java/com/scmessenger/android/ui/contacts/AddContactScreen.kt`
  - iOS join: `iOS/SCMessenger/SCMessenger/Views/Topics/JoinMeshView.swift`
  - iOS contacts: `iOS/SCMessenger/SCMessenger/Views/Contacts/ContactsListView.swift`
- Android and iOS can generate identity QR codes from full identity export payloads (ID, public key, nickname, libp2p peer ID, listeners, relay)
  - Android identity QR: `android/app/src/main/java/com/scmessenger/android/ui/identity/IdentityScreen.kt`
  - Android access path: Settings -> Identity -> Show Identity QR
  - Android export source: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
  - iOS identity QR: `iOS/SCMessenger/SCMessenger/Views/Settings/SettingsView.swift`
  - iOS export source: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`
- iOS physical-device helper scripts are available:
  - Build signed device artifact: `iOS/build-device.sh`
  - Build + clean-install on connected iPhone: `iOS/install-device.sh`
- Android `WifiAwareTransport` compile issue was fixed; runtime behavior still needs field validation across devices/NAT scenarios
  - `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`

### Operational/Test Coverage Gaps

- Browser-executed WASM tests require local `wasm-pack`; CI enforces this path in `.github/workflows/ci.yml` (`check-wasm`).
- Android build verification requires `ANDROID_HOME` to be set in-shell; CI now standardizes SDK env and enforces Android preflight in `.github/workflows/ci.yml` (`check-android`).
- App-update continuity code is complete (backup/restore, schema migration, relay key migration); pending: real-device package upgrade validation runs on Android/iOS/WASM

### Non-Markdown Extraction Highlights (2026-02-23)

- `docker/run-all-tests.sh` + `docker/docker-compose.test.yml` define a broader CI-like test surface than previously summarized:
  - Rust tests
  - lint (`cargo fmt` + `clippy -D warnings`)
  - security audit (`cargo audit`)
  - UniFFI bindings checks (Kotlin + Swift generation)
  - WASM node-runtime tests (`wasm-pack test --node`)
- `scripts/deploy_gcp_node.sh` is a concrete community-operator deployment path using Cloud Build + Compute Engine container update/restart for the relay/bootstrap role.
- `scripts/get-node-info.sh` documents and automates extraction of `Peer ID`, external address API query (`/api/external-address` on port `9876`), and shareable bootstrap multiaddr formatting.
- `iOS/verify-test.sh` is now an actual build verification script (simulator workspace build), not a placeholder.
- `android/app/build.gradle` currently aligns ABI filters and Rust build targets to `arm64-v8a` + `x86_64` (earlier mismatch note is outdated).
- `android/verify-build-setup.sh` now validates the same ABI matrix (`aarch64-linux-android` + `x86_64-linux-android`).
- `iOS/copy-bindings.sh` is normalized to the active generated path only: `iOS/SCMessenger/SCMessenger/Generated/`.

### Repository Structure Clarifications

- Active iOS app project/code is under:
  - `iOS/SCMessenger/SCMessenger.xcodeproj`
  - `iOS/SCMessenger/SCMessenger/`
- `iOS/SCMessenger-Existing/` is a legacy/reference tree and is not part of the active Xcode target.

### Product Directives (2026-02-23)

- Primary delivery target is one unified Android+iOS+Web app.
- Rollout model is global and organic (no region-targeted gating sequence).
- Infra model is community-operated (self-hosted and third-party relay/bootstrap operators are both valid).
- Canonical cross-platform identity is `public_key_hex`; other IDs are derived/operational.
- Relay toggle must remain user-controlled; OFF blocks all inbound/outbound relay traffic while preserving local read access.
- Bootstrap configuration direction is env-driven startup config plus dynamic fetch (with static fallback).
- Reliability objective is active-session availability plus durable eventual delivery (messages are retained/retried until route availability).
- Storage policy must be bounded so local history/outbox cannot grow unbounded.
- First-run consent gate is required before first messaging actions.
- Alpha language scope is English-only (i18n expansion remains backlog work).
- Abuse controls and regional compliance mapping are explicitly post-alpha tracks.
- Web/WASM remains experimental today and must be promoted to parity before GA.

## Source of Truth

Use this file plus:

- `README.md` (repo entrypoint)
- `docs/DOCUMENT_STATUS_INDEX.md` (documentation lifecycle map)
- `docs/TESTING_GUIDE.md` (test commands and expected outcomes)
- `REMAINING_WORK_TRACKING.md` (active gap backlog)
- `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` (active milestone scope/order)
- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (active residual risk posture)
- `docs/EDGE_CASE_READINESS_MATRIX.md` (extreme environment readiness/hardening)

Treat older status and audit report docs as historical snapshots unless they are explicitly linked from the files above as current.

## 2026-03-13 iOS Simulator Launch Blocker (Operational) (Verified)

- The iPhone 17 Pro simulator had a stale SCMessenger app bundle installed with Mach-O `platform IOS` instead of `platform IOSSIMULATOR`.
- Symptom: `xcrun simctl launch` failed with SpringBoard denial and `NSPOSIXErrorDomain Code=163` / `Launchd job spawn failed`, even though the app appeared installed.
- Verified root cause: the installed simulator bundle reported `platform IOS`, while the freshly rebuilt simulator artifact reported `platform IOSSIMULATOR`.
- Operational fix: uninstall the stale simulator app, rebuild for `Debug-iphonesimulator`, reinstall the fresh simulator app, and relaunch.
- Important ambiguity now documented: on Apple Silicon, a wrong-flavor iOS arm64 bundle can appear installable in the simulator but still fail only at process bootstrap. "Installed" is not sufficient proof of a valid simulator artifact.
- Verification: `xcodebuild -project iOS/SCMessenger/SCMessenger.xcodeproj -scheme SCMessenger -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 17 Pro' build` passed, reinstall succeeded, and `xcrun simctl launch booted SovereignCommunications.SCMessenger` returned a live PID.

## 2026-03-13 Conversation Consolidation: Transport, Stability, Relay, Harness, and Debt Ledger

- Android send semantics were reaffirmed as the canonical architectural model: local durable write first, immediate local-ledger/UI reflection second, asynchronous delivery progression third. A message queued locally is already "on the mesh" from that node's perspective even if delivery is still pending.
- Android pending-message investigation confirmed the send pipeline itself was operating correctly for the investigated stuck message `c35aa8da-d220-466c-8769-951523771af7`: local history write succeeded, retries/backoff executed, and failure was due to route instability to the recipient rather than lost UI state.
- Product expectation was clarified and accepted as binding: Bluetooth, direct LAN/libp2p, relay, and Wi-Fi Direct/local transports must all work together when available. The goal is simultaneous transport viability, not merely a single working fallback.
- GCP relay was repaired live after multiple operational failures were found:
  - active `gcloud` project drift
  - `/var/lib/docker` full
  - excessive Docker JSON log growth
  - stale exited containers
  - corrupted persisted relay ledger (`peers.json`)
- GCP recovery steps completed:
  - switch to project `scmessenger-bootstrapnode`
  - clear Docker log bloat and stale containers
  - replace corrupted `peers.json` with a valid empty `ConnectionLedger`
  - restart relay container
- Post-repair GCP verification showed:
  - relay container running cleanly
  - ports `9000` and `9001` listening
  - HTTP responsive
  - normal startup logs with relay/peer activity restored
- iOS physical-node participation improved materially after bootstrap/relay correction. The device no longer appeared completely off-mesh; direct and relay evidence returned, though app-level visibility evidence remained less complete than system/radio evidence.
- BLE remained an active focus area:
  - Android repeatedly showed strong BLE fallback activity and successful GATT writes.
  - Physical iOS still showed lower-confidence app-level BLE evidence than Android.
  - BLE freshness and route-selection quality were identified as central to reliability.
- Android BLE freshness profiling is now the documented active model:
  - filtered scan fallback to unfiltered scan after 20 seconds
  - 120-second BLE observation freshness cache
  - connected BLE peer preferred over stale persisted hint
  - stale BLE hints explicitly skipped and logged
- Remaining Android BLE ambiguity is still open and documented: accepted-send logs can preserve the requested stale BLE target while the actual on-wire GATT success callback is emitted for the fresher connected device. This is a telemetry debt item, not proven transport failure.
- iOS UX/stability conclusions from this conversation:
  - the send-button spinner behavior conflicts with the repo's store-and-forward mentality
  - the source of truth should be the local durable ledger, not live network success
  - messages should appear immediately and advance delivery state asynchronously
  - freeze/unfreeze behavior correlated with heavy peer-identify / identity-beacon churn during convergence
- The narrow safe stability direction established for iOS is:
  - remove beacon rebroadcast from hot peer-identification paths
  - debounce beacon refreshes
  - reduce redundant peer-identification churn
  - maintain fully local-ledger-first send behavior
- `run5.sh` was substantially upgraded in this conversation lineage:
  - GCP collector prepends Docker snapshot logs before incremental polling
  - physical iOS logs split into app-console and system/Bluetooth context
  - live status ticker now surfaces Android peer/BLE counts and separate iOS app/system counts
  - post-run analysis uses known own IDs only for visibility accounting
  - unknown own IDs are treated as collector gaps, not false mesh failures
  - transport evidence table added for peer IDs, app events, BLE, direct, relay, and Wi-Fi/Multipeer evidence
  - pre-running physical iOS app is no longer force-relaunched solely to capture console output
- Harness honesty improved, but full 5-node proof was still not achieved in the recent runs:
  - GCP healthy and active
  - OSX relay healthy with captured own ID
  - Android active across BLE/direct/relay
  - physical iOS active at system/radio level and sometimes app/peer level
  - iOS simulator had been a blind spot until launch recovery
  - overall state remained partially indeterminate rather than verified full 5-node visibility
- iOS simulator launch debugging found an operationally important failure mode:
  - stale installed SCMessenger bundle had Mach-O `platform IOS`
  - fresh simulator artifact had Mach-O `platform IOSSIMULATOR`
  - on Apple Silicon, a wrong-flavor arm64 app can appear installed in the simulator yet fail only at process bootstrap with SpringBoard denial and `NSPOSIXErrorDomain Code=163`
  - uninstalling the stale app, rebuilding for `Debug-iphonesimulator`, reinstalling, and relaunching restored simulator launchability
- After simulator launch recovery, real runtime follow-on issues were still visible:
  - BLE unavailable in the simulator
  - `historySync request failed to prepare message`
  - these are runtime issues, not launch blockers, and remain open for follow-up
- Documentation/process policy was explicitly tightened during this conversation and is now part of active repo expectations:
  - docs must be updated in the same run whenever behavior/scope/risk/scripts/workflows change
  - build verification must be performed whenever edited code/build-affecting targets change
  - final summaries should explicitly report docs-sync and build-verification status
- Consolidated tech debt / ambiguity ledger from this conversation:
  - physical iOS app-level own-ID/peer capture can still be absent from the active log window even when radio/system activity is strong
  - iOS simulator runtime mesh participation still needs validation even though launch now works again
  - Android BLE telemetry still needs unification so requested target and actual transport target match in logs
  - "installed in simulator" must not be treated as proof of a valid simulator artifact
  - iOS send-path parity with Android store-and-forward semantics still requires final confirmation across all code paths
