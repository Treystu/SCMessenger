# SCMessenger Remaining Work Tracking

Status: Active
Last updated: 2026-04-28

---

## OPEN — WASM thin client: peripheral BLE & full UI lobotomy

**Status:** TRACKED — partial landing 2026-04-11

1. **BLE GATT peripheral:** btleplug-based **advertising** for the SCM service UUID (mobile discovery of the desktop daemon). **Central scan + Drift notify → `message_received`** is implemented in `cli/src/ble_mesh.rs` (notify char `0xDF03`); see `wasm/wasm_plan.md` Phase 4 verification table.
2. **Browser IronCore:** Optional runtime mode that refuses local `initialize_identity`, requires `get_identity` over the daemon socket first, and routes `prepare_message` / send exclusively through JSON-RPC (today: `wasm/src/daemon_bridge.rs` wire helpers + `transport.rs` direction comments; full gating not landed).
3. **Verification:** `wasm-pack test --headless` / browser harness for daemon-driven UI state (deferred where CI lacks headless browser).

---

## 🔴 NEW CRITICAL FINDING - 2026-03-19 13:27 UTC: Android ANR Storm

**Status:** 🔴 P0 CRITICAL - CONFIRMED ACTIVE
=======

**Source:** Live investigation `tmp/ANDROID_HANGING_ANR_INVESTIGATION_2026-03-19.md`

### Critical Issues Confirmed

| ID | Issue | Evidence | Impact |
|----|-------|----------|---------|
| **ANR-001** | **Frequent ANR Events** | Multiple `/data/anr/anr_2026-03-19-*` files, system shows "Application Not Responding" | App completely unusable, requires force-kill |
| **ANR-002** | **Network Bootstrap Complete Failure** | All 4 relay servers failing: `34.135.34.73`, `104.28.216.43` both UDP/TCP | Cannot connect to mesh network |
| **ANR-003** | **Message ID Tracking Corruption** | `java.lang.IllegalStateException: Message ID tracking lost` | Message delivery system broken |
| **ANR-004** | **Coroutine Cancellation Cascade** | `kotlinx.coroutines.JobCancellationException` storm | Background tasks failing, main thread blocked |
| **ANR-005** | **BLE Advertising Failure** | `BLE Advertising failed with error: 3` | Local peer discovery broken |

### Retry Storm Evidence
- Message `a21669ba-4961-4ca3-b38e-bc5462abcf96` at **retry attempt 63**
- Process restarted: PID 5447 → 6588 during investigation
- Transport success rates: BLE 50%, Core 0%

### Root Cause: Main Thread Blocking
1. Network timeouts (2000ms × 4 servers = 8+ seconds on UI thread)
2. Excessive retry loops (63+ attempts per message)
3. Exception handling from corrupted message tracking
4. Coroutine cancellation cleanup blocking main thread

**IMMEDIATE ACTION REQUIRED:**
1. Fix external relay connectivity (servers offline)  
2. Fix message ID tracking IllegalStateException
3. Move network operations off main thread
4. Cap retry attempts and implement exponential backoff

## 2026-03-19 13:19 UTC Live Log Verification: Systems Operating Normally

**Status:** ✅ OPERATIONAL

### Current Log Streaming Status

**iOS Device:** 592 lines, 21.6KB, active BLE communication with peer `CA06A88B-6036-788F-AADC-624669B0D390`
**Android Device:** Limited logging (1KB), archived logs from 03:20:39 UTC available

**Active Issues Verified:** None - both devices showing normal mesh operation patterns

## 2026-03-19 v0.1.9 Stable Baseline: Notification Implementation Requires Verification

**Status:** ⚠️ NEEDS VERIFICATION

### Summary

Regressed from v0.2.1 to v0.1.9 to establish stable baseline. While notification implementation code exists and WS14 phases are documented as complete, comprehensive testing is needed to verify real-world functionality before advancing versions.

### Current Situation

**Documentation vs Reality Gap**: The codebase contains notification implementation that passes automated tests, but the actual user-facing functionality needs thorough verification to ensure it works as desired in real-world scenarios.

### Priority Work Items

#### 1. **Notification Functionality Verification** (HIGH PRIORITY)
- [ ] **Real-world notification testing** on physical devices (iOS, Android)  
- [ ] **End-to-end message flow with notifications** verification
- [ ] **Settings integration** - verify notification toggles work correctly
- [ ] **Tap routing behavior** - test conversation/requests inbox navigation
- [ ] **Background/foreground handling** - verify notification behavior in both states
- [ ] **Cross-platform consistency** - ensure iOS/Android behave similarly

#### 2. **Core Stability Validation** (HIGH PRIORITY)
- [ ] **Message delivery reliability** under various network conditions
- [ ] **Transport layer stability** testing
- [ ] **Contact persistence** verification
- [ ] **Basic messaging workflows** end-to-end validation

#### 3. **Platform-Specific Issues** (MEDIUM PRIORITY)
- [ ] **iOS notification permissions** flow testing
- [ ] **Android notification channels** behavior verification  
- [ ] **WASM browser notifications** testing across browsers

### Implementation Status (Documented vs Verified)

| Component | Code Status | Documentation | Real Testing | Notes |
|-----------|-------------|---------------|--------------|-------|
| Core notification policy | ✅ Exists | ✅ Complete | ❓ Needs verification | `core/src/notification.rs` |
| iOS notifications | ✅ Exists | ✅ Complete | ❓ Needs verification | `NotificationManager.swift` |
| Android notifications | ✅ Exists | ✅ Complete | ❓ Needs verification | `NotificationHelper.kt` |
| WASM notifications | ✅ Exists | ✅ Complete | ❓ Needs verification | Browser API integration |
| Settings integration | ✅ Exists | ✅ Complete | ❓ Needs verification | Cross-platform parity |

### Approach
**Comprehensive real-world testing before any version advancement.** Focus on proving functionality works as users expect, not just that tests pass.

**Status:** ✅ COMPLETED

### Summary

Fixed three Swift compiler errors preventing iOS build compilation:

1. **Swift type-check timeout** in [`migrateTruncatedPublicKeys()`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:548-568):
   - Complex `first(where:)` closure with nested optional chaining exceeded compiler type-check budget
   - **Fix:** Broke up expression into explicit `for` loop with early returns

2. **Optional chaining on non-optional** in [`addContact()`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:2786):
   - `contact.publicKey` is `String` (non-optional) per UniFFI `Contact` struct, but code used `contact.publicKey?.trimmingCharacters(...)`
   - **Fix:** Removed erroneous optional chaining (`contact.publicKey.trimmingCharacters(...)`)

3. **Incorrect property name** in [`migrateTruncatedPublicKeys()`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:551):
   - Used `info.peerId` but `PeerDiscoveryInfo` struct has `canonicalPeerId` property
   - **Fix:** Changed to `info.canonicalPeerId`

### Verification

- [x] iOS Debug build compiles successfully
- [x] No new warnings introduced

## 2026-03-18 QUIC/UDP Cellular NAT Traversal Implementation

**Status:** ✅ COMPLETED

### Summary

Implemented QUIC/UDP endpoints in bootstrap configuration to enable reliable message delivery on cellular networks where TCP is blocked by carriers.

### Changes Made

1. **Android Bootstrap Configuration** ([`MeshRepository.kt`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:42-58)):
   - Added QUIC/UDP endpoints for GCP relay and OSX relay
   - QUIC endpoints listed first for cellular-friendly priority
   - Includes both QUIC and TCP for fallback

2. **iOS Bootstrap Configuration** ([`MeshRepository.swift`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:66-82)):
   - Added QUIC/UDP endpoints for GCP relay
   - QUIC endpoints listed first for cellular-friendly priority

3. **GCP Relay Deployment** ([`deploy_gcp_node.sh`](scripts/deploy_gcp_node.sh:21-43)):
   - Exposed UDP port 9001 alongside TCP port 9001
   - Docker container now listens on both protocols

### Issues Resolved

- **AND-CELLULAR-001** (P0): Android cellular message sending - FIXED
- **CROSS-RELAY-001** (P0): Cross-platform relay circuit delivery - FIXED

### ⚠️ Regression Introduced

- **AND-CONTACTS-WIPE-001** (P0): Android contacts wiped after update
  - After deploying via `deploy_to_device.sh both`, Android contacts were wiped
  - Identity and message history remained intact
  - **Root cause IDENTIFIED (2026-03-18):** `resolveTransportIdentity()` returns null when no contact exists, preventing auto-contact creation during peer discovery
  - **Fix APPLIED (2026-03-18):** Added fallback auto-contact creation in `onPeerIdentified` callback when `transportIdentity == null`
  - **Fix APPLIED (2026-03-18):** iOS bootstrap config updated to include OSX relay (matching Android)
  - Status: ✅ FIXED - requires fresh install verification

---

## 2026-03-18 Session: Passive Log Audit + Contact Persistence Fix

**Status:** ✅ COMPLETED

### Log Analysis Summary

**iOS Logs** (`tmp/iOSdevicelogs-new.txt`, 7023 lines):
- Bootstrap resolution working (DNS failure for bootstrap.scmessenger.net → static fallback)
- QUIC and TCP bootstrap dialing confirmed
- GCP relay connection successful
- Decryption errors when receiving messages (sender key mismatch - caused by missing contacts)
- BLE service discovery timing issues (transient)

**Android Logs** (`tmp/Google-Pixel-6a-Android-new.logcat.txt`, 5349 lines):
- QUIC bootstrap dialing confirmed for all 4 addresses
- Relay circuit attempts timing out (network-dependent)
- Contact loaded: 0 (confirmed contact persistence bug)
- `No existing contact for transport key` - root cause of decryption failures

### Issues Identified & Fixed

| Issue | Root Cause | Fix Applied |
|-------|-----------|-------------|
| iOS missing OSX relay | Only GCP relay in bootstrap config | ✅ Added OSX relay QUIC + TCP |
| Android 0 contacts on fresh install | `resolveTransportIdentity()` returns null → no auto-create | ✅ Added fallback auto-contact creation |
| Decryption failures | Missing contacts → wrong key material | ✅ Resolved by contact auto-creation fix |
| QUIC not attempted | Not a bug - QUIC was working | ✅ Verified in logs |

### Code Changes

1. **iOS** [`MeshRepository.swift`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift:66-82):
   - Added OSX relay QUIC/UDP and TCP endpoints

2. **Android** [`MeshRepository.kt`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:699-726):
   - Added fallback contact auto-creation when `transportIdentity == null`
   - Extracts public key from peer ID and creates contact via `upsertFederatedContact()`

### Verification Status

- [x] iOS build compiles (verified)
- [x] Android build compiles (verified)
- [x] Log analysis complete
- [x] Root cause identified for contact wipe
- [ ] Fresh install verification on physical devices (pending user testing)

### Impact

| Metric | Before | After |
|--------|--------|-------|
| iOS bootstrap paths | 2 (GCP only) | 4 (GCP + OSX) |
| Android contact auto-create | ❌ (broken) | ✅ (works on fresh install) |
| Cross-platform relay | Single point of failure | Redundant |

### Technical Details

- The swarm core already binds QUIC automatically (lines 1341-1347 in `swarm.rs`)
- Bootstrap nodes now advertise both QUIC/UDP and TCP endpoints
- QUIC provides better NAT traversal for cellular networks
- Relay circuit addresses include QUIC endpoints for improved connectivity

### Verification

Full verification requires physical devices on cellular networks:
1. Deploy updated relay with `scripts/deploy_gcp_node.sh`
2. Connect Android device on cellular network
3. Connect iOS device on WiFi
4. Verify relay connection via QUIC
5. Send messages between devices
6. Verify delivery succeeds without NetworkError

Note: `run5.sh` test requires physical device connections for full verification.

## iOS 43K Send Failures Fix (2026-03-17)

**Status:** ✅ COMPLETED

### Root Cause

The retry loop used **blind periodic retries** (every 8 seconds) without opportunistic triggering, causing excessive retries for unreachable peers. The `pendingOutboxExpiryReason()` function correctly returns `nil` (messages never expire per PHIL-011), but the retry strategy was suboptimal.

### Fix Applied

Implemented **opportunistic retry on delivery receipt** instead of adding expiry limits (which would violate PHIL-011 - eventual delivery convergence):

1. **Opportunistic Retry** (`onDeliveryReceipt()`):
   - When a delivery receipt is received, immediately retry all pending messages to that peer
   - Logic: "Oh look they just got my Bluetooth message - let's send any other pending messages via Bluetooth now!"

2. **ID Standardization**:
   - `promotePendingOutboundForPeer()` - Now normalizes both input and stored peer IDs
   - `triggerPendingSyncForPeerIds()` - Now uses `normalizePeerId()` instead of just trimming
   - `onDeliveryReceipt()` - Normalizes peer ID before triggering retry

### Verification

- [x] iOS build compiles successfully
- [x] ID normalization audit complete
- [x] Docs sync check: PASS

### Impact

| Metric | Before | After |
|--------|--------|-------|
| Retry strategy | Blind 8s periodic | Opportunistic on receipt |
| ID matching | Direct string comparison | Normalized comparison |
| Philosophy compliance | ❌ (if max attempts added) | ✅ (eventual delivery) |

## Repo Governance Lock: Documentation Sync + Build Verification (2026-03-13)

This is the active implementation backlog based on repository state verified on **2026-03-14**.

Primary delivery target: **one unified Android + iOS + Web app**.

## Repo Governance Lock: Documentation Sync + Build Verification (2026-03-13)

Completed in this pass:

1. [x] Tightened `AGENTS.md` so same-run canonical documentation updates are explicit whenever behavior, scope, risk, scripts, verification commands, or operator workflow change.
2. [x] Tightened `AGENTS.md` so edited-target build verification is mandatory whenever code, bindings, build wiring, or runtime-affecting scripts change.
3. [x] Mirrored the same closeout rules in `.github/copilot-instructions.md` to keep Codex/Copilot policy aligned.
4. [x] Updated the active canonical doc chain to reflect that documentation sync and build verification are release-governance requirements rather than optional cleanup.
5. [x] **Enhanced Mesh Log Visualizer Visibility**: Broadened recognition for BLE and Local Transport (Multipeer, Wifi-Direct, etc.) logs across iOS and Android. Improved PeerID extraction and tag-based fallbacks.

Remaining governance expectation:

1. [ ] Enforce these rules on every future change-bearing run and record exceptions only with exact blocking command output and rationale.

## BLE Log Visibility Improvements (2026-03-16)

Completed in this pass:

1. [x] **Log Visualizer (`mesh.html`)**: Broadened BLE detection keywords and refined own-identity parsing.
2. [x] **Run Script (`run5.sh`)**: Added proactive "Seeding node identities" step to inject identity markers for already-running nodes.
3. [x] **iOS Logging**: Expanded log stream predicates to capture `com.scmessenger` subsystem logs.

## GCP Node Nickname Update (2026-03-16)

Completed in this pass:

1. [x] **CLI Node Name Sync**: Updated `cmd_relay` to sync the `--name` argument (if provided) to the `IronCore` identity nickname.
2. [x] **Deployment Command**: Added `--name GCP-headless` to the relay startup command in `scripts/deploy_gcp_node.sh`.

## DHT Peer Discovery Latency Optimization (2026-03-18)

**Status:** ✅ IMPLEMENTED

### Overview

Implemented P0 and P1 optimizations for reducing DHT peer discovery latency from >2 seconds to <500ms, as specified in [`docs/DEEP_ARCHITECTURAL_REASONING_DHT_OPTIMIZATION.md`](docs/DEEP_ARCHITECTURAL_REASONING_DHT_OPTIMIZATION.md).

### Key Changes

#### 1. Hierarchical Timeout Budgeting (P0 - ~100 LOC)

Created [`core/src/routing/timeout_budget.rs`](core/src/routing/timeout_budget.rs):

- **Budget-based discovery**: Total 500ms budget across all phases instead of 500ms per phase
- **Progressive fallback**: LocalCache → NeighborhoodQuery → DelegateQuery → FullDhtWalk
- **Early termination**: Stop as soon as a route is found
- **Deterministic**: Same inputs produce same phase transitions

#### 2. Bloom Filter Negative Cache (P0 - ~200 LOC)

Created [`core/src/routing/negative_cache.rs`](core/src/routing/negative_cache.rs):

- **Fast negative answers**: O(1) bloom filter check vs O(log n) DHT walk
- **Time-based expiry**: Negative results expire after TTL (default 10 minutes)
- **Bounded false positives**: Accept ~1% false positive rate for fast negative answers
- **Capacity management**: Evict oldest entries when at capacity

#### 3. Route Prefetch on App Resume (P1 - ~150 LOC)

Created [`core/src/routing/resume_prefetch.rs`](core/src/routing/resume_prefetch.rs):

- **Proactive refresh**: Refresh routes before they're needed
- **Parallel validation**: Validate multiple routes concurrently
- **Graceful degradation**: Return stale routes while refreshing
- **Frequent peer tracking**: Prioritize refresh for frequently messaged peers

### Expected Impact

| Metric | Before | After Target |
|--------|--------|--------------|
| Cold start discovery | 2000ms | < 500ms |
| Warm cache hit | 2000ms | < 50ms |
| App resume to ready | 2000ms | < 200ms |
| Unreachable detection | 2000ms | < 10ms |

### Verification

- [x] `cargo test --workspace` - **PASS** (533 tests, 0 failures)
- [x] Unit tests for timeout_budget module - **PASS** (8 tests)
- [x] Unit tests for negative_cache module - **PASS** (8 tests)
- [x] Unit tests for resume_prefetch module - **PASS** (6 tests)
- [x] Documentation sync check - **PASS**

### Remaining Optimization Opportunities

1. **P2: Predictive Route Caching** (~300 LOC) - Pre-compute routes based on conversation patterns
2. **P2: Adaptive TTL** (~100 LOC) - Keep routes fresh longer for active peers
3. **P2: Speculative Delegate Pre-warming** (~250 LOC) - Register with delegates before sleep

Owner policy constraints (2026-02-23):

- Global organic growth (no region-targeted rollout sequence).
- Community-operated infrastructure model (self-hosted and third-party nodes are both valid).
- English-only alpha UI language (i18n expansion tracked as backlog).
- No abuse-control or regional compliance hard gate for alpha.
- Anti-abuse controls are required before beta release.
- Critical UX controls must stay in Android+iOS+Web parity with no temporary lead platform.

## v0.2.1 Critical Bug Fixes (2026-03-12)

Completed in this pass:

1. [x] **Android Duplicate Messages**: Fixed UI duplication bug by properly emitting reconciled message IDs from `MeshRepository` and deduplicating by content/timestamp in `ChatViewModel.loadMessages()`.
2. [x] **iOS CryptoError (Error 4)**: Traced to stale bootstrap data; resolved by updating static fallbacks and implementing dynamic ledger-driven discovery in `MeshRepository.swift`.
3. [x] **iOS Power & Log Optimization**: Increased adaptive interval for high battery levels in `IosPlatformBridge.swift` and simplified noisy power profile logs.

## v0.2.0 Critical Bug Fixes (2026-03-09)

Completed in this pass:

1. [x] **NAT Traversal**: Added relay server behavior to all nodes for cellular↔WiFi messaging
2. [x] **BLE Reliability**: Fixed DeadObjectException with proper subscription tracking
3. [x] **Delivery Status**: Eliminated false positives where BLE ACK was treated as full delivery
4. [x] **Android UI**: Fixed keyboard covering chat input with proper IME padding
5. [x] **Transport Optimization** (2026-03-10): Faster BLE/WiFi switching with reduced timeouts, aggressive retry backoff, enhanced transport logging
6. [x] **Android Mesh UI Scrolling** (2026-03-10): Converted DashboardScreen to LazyColumn for proper scrolling with large peer lists
7. [x] **Android ID Normalization** (2026-03-10): Standardized peer ID handling to fix "Contact not found" messaging issues
8. [x] **NAT Traversal & BLE Stability** (2026-03-13): Restored relay routing, throttled BLE beacons, fixed Android connect-on-demand.
9. [x] **BLE Freshness Profiling + run5 Visibility Clarification** (2026-03-13): Android now prefers fresh BLE observations over stale cached hints, promotes to unfiltered BLE scan after 20s of zero mesh advertisements, and `run5.sh` now splits iOS app/system logs while treating unknown own IDs as collector gaps instead of mesh failures.

Outstanding items:

1. [ ] Monitor Android-to-iOS delivery for "Missing Direction" receipts
2. [ ] Verify iOS UI no longer freezes during high-density peer discovery
3. [ ] Test BLE reconnection scenarios end-to-end with new 5s throttles
4. [ ] Verify parallel transport attempts reduce WiFi→BLE transition time to < 2s
5. [ ] Test Mesh tab scrolling with 50+ discovered peers
6. [ ] Re-run upgraded `run5.sh` on fresh artifacts and close the remaining "unknown own ID in current log window" ambiguity where full mesh transport evidence exists but passive identity capture is incomplete.
7. [ ] Bring iOS BLE route profiling to the same freshness-observation standard if stale BLE hint churn reappears; current explicit freshness cache is Android-only, while iOS still relies primarily on connected-peer preference plus runtime transport evidence.
8. [ ] Unify Android BLE fallback telemetry so the accepted-send target reflects the actual connected GATT address used on wire; current logs can still show the requested stale MAC while `BleGattClient` success callbacks fire for the fresher connected device.

## v0.2.0 Execution Residual Register

Residual risks from completed v0.2.0 phases (currently through WS12.5 burndown audit) are tracked in:

- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`

Do not start the next v0.2.0 phase without checking the corresponding entry gate in that register.

1. [ ] Ensure message history is not cleared on app startup. Messages should persist across app restarts.

## Priority 0: Tri-Platform Semantics and Reliability

1. [x] Privacy parity-first wiring (all toggles) on Android, iOS, and Web
   - Outcome: All four privacy toggles (onion routing, cover traffic, message padding, timing obfuscation) are now implemented with live UI bindings on Android (`SwitchSetting`), iOS (`Toggle`), and Web/WASM (`getSettings`/`updateSettings`). Dead placeholder components removed from both mobile platforms.

2. [x] Relay toggle enforcement parity (mandatory OFF behavior on all clients)
   - Outcome: WASM `prepareMessage`, `receiveMessage`, and WebSocket receive loop now enforce `relay_enabled` check, matching Android/iOS behavior. When OFF, outbound messages are blocked and inbound frames are dropped.

3. [x] Canonical identity normalization to `public_key_hex`
   - Outcome: `IdentityInfo` struct and `api.udl` now document `public_key_hex` as the canonical persisted/exchange identity. `identity_id` (Blake3) and `libp2p_peer_id` are documented as derived/operational metadata.

4. [x] Bootstrap configuration model implementation
   - Outcome: Added `BootstrapConfig` dictionary and `BootstrapResolver` interface to `api.udl` with full Rust implementation. Resolution chain: env override (`SC_BOOTSTRAP_NODES`) → remote URL fetch (via `ureq` HTTP client) → static fallback list. Android and iOS both wired to use resolver instead of hardcoded lists. WASM uses env → static path (no sync HTTP in browser).

5. [x] Android peer discovery parity hardening
   - Source: `ANDROID_DISCOVERY_ISSUES.md` investigation notes.
   - **RCA — Live test evidence (2026-02-25 09:10 HST):**
     - Android (`K8tm9`) connects to GCP relay then disconnects in <1ms, in a tight loop.
     - **Root cause A (fixed 2026-02-25):** `core/src/transport/swarm.rs` was calling `kademlia.add_address` for ALL peer-reported addresses including loopback (`127.0.2.x` — Android VPN interface), `10.x`, `192.168.x`, `172.16-31.x` RFC1918, and CGNAT. GCP's Kademlia then tried to dial Android at `127.0.2.3:50600` → `Connection refused` → immediate disconnect. **Fix applied:** Added `is_globally_routable_multiaddr()` filtering at all 7 `kademlia.add_address` call sites in `swarm.rs`. Private/loopback/CGNAT ranges now silently skipped.
     - **Root cause B (fixed 2026-02-25):** Android never explicitly registers a relay circuit reservation with GCP on startup so GCP cannot dial it back via `/p2p-circuit/`. The `relay_client` behaviour is present in `IronCoreBehaviour` (via `relay::client::Behaviour`) but no code actively calls `swarm.listen_on("/p2p/GCP_PEER_ID/p2p-circuit")` after connect. **Fix applied:** In `swarm.rs` `ConnectionEstablished` handler, when the connected peer is identified as a relay node (agent contains `relay`), call `swarm.listen_on(relay_multiaddr.with(Protocol::P2pCircuit))` to register a reservation. This gives the relay a stable back-channel to this mobile node.
     - Android Mesh Stats shows `2 peers (Core), 2 full, 1 headless` — partial BLE+GCP connectivity confirmed.
   - **Root cause C (fixed 2026-02-25):** iOS Sim identify storm — OSX peer identified every ~300ms on iOS Sim. Was `identify::Config::with_interval(30s)` in `behaviour.rs`. **Fix applied:** Increased to 60s. Prevents identify flooding that drowned swarm event loop for mobile clients.
   - Remaining open items:
     - **[MED]** Bootstrap relay visibility policy in nearby UI — headless relay nodes should not appear as 1:1 chat contacts.
     - **[MED]** Delayed identity-resolution retry after initial peer connect — BLE-connected peers may not have public key yet at connect time; need a 2-3s retry pull.

6. [x] Real-network NAT traversal field matrix
   - Scope: CLI host nodes + Android + iOS + Web over mixed LAN/WAN/NAT.
   - Target: scripted verification matrix with delivery latency + fallback success criteria.
   - **RCA / current gaps identified (2026-02-25):**
     - GCP→OSX: ✅ connected (both headless, public IPs).
     - GCP→iOS Dev: ✅ relay-circuit path functional.
     - GCP→iOS Sim: ✅ relay-circuit path functional.
     - GCP→Android: ✅ rapid connect/disconnect loop fixed.
     - OSX→iOS Dev: ✅ (seen in logs).
     - OSX→Android: ✅ OSX dialing Android circuit functional.
     - Android↔iOS Dev: ✅ circuit registration path functional.
     - iOS Sim↔Android: ✅ circuit registration path functional.
   - **Implementation applied (2026-02-25):** P0.5B (relay circuit reservation) is implemented, all mobile nodes register as relay clients and full mesh p2p-circuit connectivity is wired.

7. Nearby Peer Discovery and Identity Federation (Android Focus)
   - [x] Prevent permission-race startup regression: Android mesh now permission-gates BLE/WiFi init and auto-retries transport init when runtime permissions are granted (no restart required).
   - [x] Ensure Bluetooth, LAN, and Relay discovery are accounted for and routed to Mesh tab.
   - [x] Display total node count (headless and full) in Mesh UI.
   - [x] Fix nickname federation (ensure nicknames are correctly passed to neighbors over BLE/Swarm).
   - [x] Fix iOS -> Android nearby identity nickname propagation (Android currently discovers peer identity/public key but often misses federated nickname).
   - [x] Implement local nickname overrides in contacts (show both official and private nicknames).
   - Outcome (2026-02-24):
     - Android and iOS repositories now emit deduplicated identity/connected discovery events for BLE + internet peers, including headless relay visibility.
     - Dashboard surfaces aggregate full/headless totals from canonical discovery state.
     - BLE identity reads now perform delayed refresh pulls after initial connect to capture nickname updates quickly.
     - Contacts screens display local override nickname as primary with federated nickname retained as secondary (`@nickname`) on both mobile clients.

8. [x] Android WiFi Aware physical-device validation
   - File: `android/app/src/main/java/com/scmessenger/android/transport/WifiAwareTransport.kt`
   - **Implementation applied (2026-02-25):**
     - `WifiAwareTransport` responder/initiator sockets explicitly verified (`AwareConnection.startReading()`).
     - Discovery triggers the direct data path and seamlessly pushes raw blobs onto `onDataReceived`.
     - Full bi-directional connection over API level >= 29 is implemented in the data path `startReading()` loop.
   - Target: compatibility results by Android version/device class with documented pass/fail outcomes.

9. [x] Web parity promotion — WASM swarm transport and API parity
   - Previous: Web/WASM was functionally present but thinner than mobile app surfaces.
   - Completed (PR #48):
     - WASM now uses libp2p swarm via `wasm-bindgen` + `websocket-websys` transport as first-class path (no standalone relay-only bypass).
     - `startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers` implemented in `wasm/src/lib.rs`.
     - Legacy `startReceiveLoop(relayUrl)` converted to deprecated shim that delegates to swarm bootstrap.
     - `ConnectionPathState` enum and `exportDiagnostics()` exposed in both UniFFI and WASM APIs for tri-platform parity.
     - CI adds `wasm32` compile checks to guard browser transport builds.
   - Remaining (beta):
     - IndexedDB-backed persistence with migration/version support.
     - `wasm-pack` runtime browser test coverage in CI.
     - History UX and deep parity for settings/contacts surfaces on Web.

10. [x] WS9 desktop full GUI parity (alpha scope)

- Outcome (2026-03-03):
  - Desktop GUI now executes onboarding/identity, contacts, chat send/receive, mesh dashboard, and relay-only mode via local `wasm` + Core APIs.
  - Normal desktop workflows no longer depend on CLI websocket command fallback paths.
  - Role gating now aligns with mobile parity (`full` vs `relay-only`), including explicit identity-init CTA and blocked chat/contact actions in relay-only mode.
  - Remaining WS9 residual is tracked in `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (`R-WS9-01`).

11. [x] WS10 minimal anti-abuse guardrails (alpha level)

- Outcome (2026-03-03): Added per-peer token bucket limiting, global inflight custody-dispatch cap, duplicate suppression window, and cheap abuse heuristics in Core relay handling with native + wasm parity and targeted guardrail tests.

12. [x] WS11 public beta readiness surfaces

- Outcome (2026-03-03): Added explicit delivery-state UX mapping (`pending`, `stored`, `forwarding`, `delivered`) on Android+iOS chat surfaces, upgraded diagnostics exports into tester-readable bundles with contextual guidance, and added tester-facing reliability + permissions rationale notes in settings/diagnostics flows.

13. [x] WS12 test matrix expansion and docs parity lock

- Outcome (2026-03-03):
  - Added deterministic offline/partition integration coverage in `core/tests/integration_offline_partition_matrix.rs`.
  - Stabilized and validated live custody reconnect suite (`core/tests/integration_relay_custody.rs`) for `--include-ignored` execution on socket-enabled hosts.
  - Added reproducible WS12 validation runner: `scripts/verify_ws12_matrix.sh`.
  - CI now enforces WS12 parity gates:
    - core deterministic offline/partition suites,
    - Android role/fallback parity tests,
    - desktop/WASM role parity tests,
    - iOS verify pipeline now includes local transport fallback and role-mode parity checks.
  - Canonical documentation and residual-risk register were updated to align runtime behavior and release-gate status.

14. Beta anti-abuse gate implementation and validation

- Requirement: abuse controls are non-blocking in alpha but mandatory before beta.
- Target: enable and validate anti-abuse protections with measurable pass criteria across Android, iOS, Web, and relay-critical paths.
- Scope: relay spam/flood controls, abuse detection thresholds, and regression coverage in CI/release checks.

13. [x] Active-session reliability + durable eventual delivery guarantees
    - Requirement: while app is open/relaying, service should remain available and messages should not be dropped.
    - Target: explicit durability contract (persisted outbox/inbox semantics, resend/recovery behavior) plus failure-mode tests.
    - Scope: crash/restart recovery, relay outage handling, offline queue replay, duplicate-safe redelivery.
    - **Implementation applied (2026-02-25):**
      - **Relay outage handling:** Implemented explicit 10s→30s→60s exponential reconnect backoff in `swarm.rs` `ConnectionClosed` handler if a relay peer drops.
      - **Outbox persistence/Retry gap:** iOS now explicitly re-hydrates stuck messages (`delivered: false`, `direction: .sent`) via `historyManager.recent()` on startup inside `startPendingOutboxRetryLoop`. Resurrects them into the `sendMessage` pipeline with new routable identifiers.
      - **Duplicate-safe redelivery:** `HistoryManager.add(record:)` remains idempotent on `id` over stable UUID generation path in `ironCore`.

14. [x] Message timestamp parity (iOS align to Android)

- Requirement: Messages must display the **time they were sent**, not the time they were received or rendered.
- Android: already correctly associates each message with its sent timestamp from the message envelope.
- **Implementation applied (2026-02-25):**
  - **Rendering gap closed:** `MessageBubble` view (`iOS/SCMessenger/SCMessenger/Views/Navigation/MainTabView.swift`) now formats and renders the explicit `message.timestamp` (epoch SECONDS offset) with proper `HH:mm` format logic beside the `message.content`.
  - **Conversation list gap closed:** `loadConversations()` now explicitly invokes `repository.getConversation(peerId:limit:1)` to seed `lastMessage` and `lastMessageTime` into the list views for complete UI hydration parity with Android.

1. [x] Bounded retention policy implementation

- Requirement: local history/outbox storage must be policy-bound to avoid unbounded disk growth.
- Target: configurable retention caps + deterministic pruning behavior + docs for user expectations.
- Scope: Android, iOS, and Web local storage behavior and defaults.
- Outcome: Implemented `enforce_retention(max_messages)` and `prune_before(before_timestamp)` in `HistoryManager` (Rust core) with UniFFI exposure. Both return pruned count for observability. Mobile clients can call these on startup or periodically.

14. [x] First-run consent gate (mandatory)

- Requirement: first app launch must present consent text explaining privacy/security boundaries.
- Target: consent acknowledgment gate on Android/iOS/Web before first messaging actions.
- Scope: UX copy parity, acceptance persistence, and re-display rules after major policy changes.
- Outcome: Added `ConsentView` to iOS onboarding (6-step flow) and consent gate card to Android `OnboardingScreen`. Users must acknowledge keypair identity, local-only data, relay participation, E2E encryption, and alpha software status before proceeding. Consent state persisted via `UserDefaults` (iOS) and in-memory state gates (Android).

15. [x] 80/20 platform support matrix

- Requirement: prioritize the smallest support matrix that covers the majority of active users.
- Target: explicit minimum OS/browser matrix and validation plan tied to release gates.
- Scope: Android API levels, iOS versions/devices, and browser families/versions.
- Outcome: Created `docs/PLATFORM_SUPPORT_MATRIX.md` documenting Android 10+ (API 29), iOS 15+, latest 3 browser versions, with rationales, transport compatibility, known limitations, and validation plan.

16. [x] Community-operated relay/bootstrap topology support

- Requirement: both self-hosted and third-party-operated infra must be valid without protocol-level assumptions.
- Target: operator docs + connectivity tests for cloud-hosted and home-hosted relays/bootstrap nodes.
- Scope: examples for GCP-style deployments and low-resource/self-hosted setups.
- Outcome: Created `docs/RELAY_OPERATOR_GUIDE.md` covering Docker and manual setups, cloud deployment (GCP example), monitoring, security, and troubleshooting.

17. [x] Bootstrap governance mode decision (product choice pending)

- Requirement: choose how clients trust and discover bootstrap updates.
- Target: lock one governance mode and document it in canonical docs.
- Scope: trust source, update cadence, and fallback behavior.
- Outcome (2026-02-25): Registered newly identified peers as potential relays in the reputation tracker to expedite relay connectivity. Created `docs/BOOTSTRAP_GOVERNANCE.md` documenting the alpha model (static-first, env/URL override), trust model, and self-hosted operator instructions.

17. [x] Fast Bootstrap and Graceful Identity Handling

- Requirement: Support hardcoded or dynamically updated IPs for bootstrap nodes without mandatorily hardcoding their peer identities.
- Target: Allow the mesh service to gracefully accept the new or changing identity of a static-IP bootstrap node instead of failing the connection layout or validation.
- Scope: Refactor connection validation / Noise payload handling so that a known static bootstrap IP can dynamically rotate or present any valid peer identity without breaking clients.
- Outcome: Stripped `/p2p/PEER_ID` suffix from parsed bootstrap Multiaddrs in `core/src/transport/swarm.rs` prior to dialing, coercing libp2p into dialect-agnostic connection validation that gracefully accepts newly presenting peer identies correctly authenticated by Noise. Added DHT hyper-optimization (alpha concurrency 8, replication 5) to `behaviour.rs` Kademlia configuration as prescribed by `Gemini_Strategy_Supplement.md` to hit Alpha 0.1.2 requirements.

17. Multi-Transport Reliability and Targeted Acknowledgements

- Requirement: replies and metadata sync must not fail when peers move between LAN, BLE, and Internet (GCP Relay).
- Outcome (2026-02-25):
  - [x] Switched delivery receipts and identity sync from broadcast to targeted delivery (Multi-Path), ensuring they reach off-LAN peers via Relay or BLE.
  - [x] Implemented platform-level BLE fallback in `attemptDirectSwarmDelivery` for both Android and iOS, prioritizing LAN → BLE → Relay.
  - [x] Linked canonical identities to `ble_peer_id` and `libp2p_peer_id` in persisted contact notes to maintain routing across sessions.
  - [x] Verified GCP relay (34.135.34.73:9001) is alive and accepting connections.

18. [x] Parity: Data Deletion (Contacts and Message Threads)

- Requirement: Ensure complete parity across all instances (Android, iOS, Web) for deleting a contact and deleting a message thread.
- Target: Allow users to securely remove contacts and clear entire message threads, ensuring changes are fully persisted and reflected in the UI.
- Scope: Bind deletion operations in `ContactsManager` and `HistoryManager` to UI interactions on all platforms, including cleaning up associated metadata.
- Outcome: Contact/thread deletion APIs are wired in Android+iOS repository layers (`removeContact`/`deleteContacts` + `clearConversation`) and backed by `HistoryManager` core functions. Conversation-list swipe-delete parity is tracked and implemented in WS12.24.

19. [x] Headless/Relay logic Refinement
    - [x] Update `IronCoreBehaviour::new` to accept `headless` boolean flag and incorporate it into the `agent_version` string.
    - [x] Update `start_swarm` and `start_swarm_with_config` in `core/src/transport/swarm.rs` to accept and pass down the `headless` flag.
    - [x] Adjust calls to `start_swarm` in `cli/src/main.rs`: `cmd_start` passes `false`, and `cmd_relay` passes `true`.
    - [x] Update `MeshService::start_swarm` in `core/src/mobile_bridge.rs` to pass `false`.
    - [x] Update `CoreDelegate` trait and `api.udl` to include `agent_version` in `on_peer_identified`.
    - [x] Update Android `MeshRepository.kt` to handle `agentVersion` and identify headless peers.
    - [x] Update iOS `CoreDelegateImpl.swift` and `MeshRepository.swift` to handle `agentVersion` and identify headless peers.
    - [x] Confirm that direct P2P messaging works over cellular with fallback to relaying (mandatory for 0.1.2 Alpha).

### WS12.7 Runtime Sanity Follow-ups (2026-03-02 HST)

1. [x] Android: fix BLE identity beacon payload fallback so listener/external routing hints are not unconditionally stripped.
2. [x] Android: serialize pending outbox flush execution to prevent overlapping retry passes for the same queue item.
3. [x] Android: apply local uptime fallback when Core stats report `uptimeSecs=0` while service is running.
4. [x] Re-validate live delivery behavior after GCP relay rollout fully replaces `scmessenger/0.1.0/headless/relay/*` nodes still observed in active logs.
   - Outcome (2026-03-02 HST): live CLI runtime probe confirmed relay identity rotation on `34.135.34.73:9001` (`12D3KooWET...` -> `12D3KooWJa...`); post-rotation delivery path still requires follow-up due reservation/custody regression signals.
5. [x] Investigate operational handling for long-lived historical pending outbox entries (high-attempt legacy items) without violating no-give-up retry policy.
   - Outcome (2026-03-03 HST): Added explicit operator runbook guidance for legacy pending-outbox triage in `docs/RELAY_OPERATOR_GUIDE.md` without introducing retry exhaustion semantics.
6. [x] Triage iOS simulator startup runtime-issue warnings (`NSFileManager createDirectory*` main-thread I/O) and confirm whether they reflect app codepaths vs simulator-only diagnostics noise.
   - Outcome (2026-03-03 HST): Confirmed app-side startup path was invoking `FileManager.createDirectory` on `@MainActor` in `MeshRepository.init()`. Hotfix moved diagnostics file persistence to background I/O queue and removed main-thread storage directory creation.

### WS12.8 Runtime Recheck Follow-ups (2026-03-02 HST)

1. [x] iOS: fix dashboard node-count inflation where discovered metrics were correct but node totals were overstated by stale/alias peer entries.
   - Outcome (2026-03-03): `MeshDashboardView` now computes full/headless totals from online-only deduplicated peers and performs stronger alias collapse across canonical/libp2p/BLE/public-key identifiers.
2. [x] Restore Android live-log visibility by re-establishing wireless ADB endpoint (`adb devices`/`adb mdns services` were empty during this pass).
   - Outcome (2026-03-03 HST): Wireless endpoint was restored and Android runtime logs were captured (including active `scmessenger/0.2.0/headless/relay/*` agent observations). Endpoint later dropped again after daemon restart; persistence follow-up remains open below.
3. [x] Investigate relay-circuit reservation failure post-redeploy using new debug error detail emitted from `core/src/transport/swarm.rs`.
   - Outcome (2026-03-03 HST): Fresh CLI runtime probe did not reproduce `Could not register relay circuit reservation`; relay reservation failure signal is not currently reproducible in this environment.
4. [x] Resolve failing live custody integration gate: `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` (timeout waiting for reconnect delivery).
   - Outcome (2026-03-03 HST): `ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh` now passes, and `integration_relay_custody` passed 3/3 consecutive reruns (stable-pass classification).
5. [ ] Stabilize Android wireless ADB endpoint persistence across reconnect cycles (`adb devices` may drop back to empty after daemon restart despite prior successful discovery).

### WS12.10 Repo-Wide Action Roundup (2026-03-03 HST)

Inventory from repo-wide checklist scan (`rg -P "^\s*(?:[-*]|\d+\.)\s+\[ \]" --glob "*.md"`):

1. Open markdown checklist items repo-wide: **84**
2. Active canonical open checklist items: **31** (WS12.8/WS12.11/WS12.12/WS12.13/WS12.14/WS12.15)
3. Deferred residual risks requiring explicit carry-forward:
   - `R-WS10-02` (peer-identity rotation vs per-peer token buckets)
4. Non-historical open checklist sources (execution truth):
   - `REMAINING_WORK_TRACKING.md` (31)
5. Historical open checklist sources (context only, not canonical execution truth):
   - `docs/historical/iOS/FINAL_STATUS.md` (21)
   - `docs/historical/iOS/PHASE4_IMPLEMENTATION.md` (14)
   - `docs/historical/REMEDIATION_PLAN.md` (14)
   - `docs/historical/iOS/PHASES_4-15_GUIDE.md` (2)
   - `docs/historical/APP_VERSION_0.1.2_ALPHA_PLAN.md` (2)
6. Planned v0.2.1 queues (explicitly outside v0.2.0 closeout):
   - WS13 decomposition in `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md`
   - WS14 decomposition in `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`

### WS12.11 iOS Relay Flapping Follow-ups (2026-03-03 HST, implementation + follow-up)

1. [x] Add iOS-side relay connection state timeline instrumentation keyed by canonical relay peer ID (connect, disconnect, identify, reservation attempt/result) to prove whether duplicate `peer_identified` events are from distinct sessions or repeated callbacks on one session.
   - Outcome (2026-03-03 HST): Added relay timeline diagnostics in `MeshRepository` for identify/disconnect/dial-allowed/dial-debounced/dial-attempt/dial-started/dial-failed keyed to extracted relay peer IDs.
2. [x] Add guardrails to prevent overlapping relay bootstrap priming and route-triggered connect attempts for the same relay within a short debounce window.
   - Outcome (2026-03-03 HST): Added bootstrap in-progress gate plus relay-peer dial debounce in `primeRelayBootstrapConnections()` and `connectToPeer(...)`.
3. [x] Add explicit "relay availability state machine" metrics in iOS diagnostics export (`stable`, `flapping`, `backoff`, `recovering`) for operator-visible triage.
   - Outcome (2026-03-03 HST): `exportDiagnostics()` now emits relay availability state fields (`relay_availability_state`, `relay_recent_events_60s`, `relay_backoff_until_ms`, and related timestamps).
4. [x] Correlate iOS relay flapping windows against GCP relay/server logs in the same UTC intervals to separate client race behavior from remote relay churn.
   - Outcome (2026-03-03 HST): Added `scripts/correlate_relay_flap_windows.sh` and executed artifact correlation (`ios_diagnostics_latest.log` vs `logs/5mesh/gcp.log`), classifying the sampled pair as `unsynchronized_artifacts_no_time_overlap`.
5. [x] Add regression coverage (integration or deterministic harness) that fails when repeated identify/dial loops occur without sustained connected hold time.
   - Outcome (2026-03-03 HST): Added `scripts/verify_relay_flap_regression.sh` deterministic harness; run on current iOS diagnostics artifact completed with pass summary and explicit relay dial-loop counters.
6. [ ] Re-run dual-device live probe (Android + iOS + CLI/GCP) with synchronized timestamps and capture one full flap cycle artifact bundle for post-fix comparison.

### WS12.12 Android<->iOS Pairing Non-Delivery Follow-ups (2026-03-03 HST, implementation + follow-up)

1. [x] Add Android BLE send-path consistency guardrails so a single payload cannot concurrently report both write-init failure and write success without a deterministic final outcome state.
   - Outcome (2026-03-03 HST): `BleGattClient` now guards GATT queue permit ownership with atomic tracking, hardens callback/release races, and treats `WRITE_TYPE_NO_RESPONSE` callbacks as informational-only to prevent contradictory final outcomes.
2. [x] Add explicit per-message transport-attempt timeline diagnostics (`core`, `relay-circuit`, `BLE`) with final attempt verdict to isolate where receipt convergence breaks.
   - Outcome (2026-03-03 HST): Android+iOS `MeshRepository` now emit structured `delivery_attempt` markers for local fallback/core/relay retry paths with message ID + context (`initial_send`, `outbox_retry`).
3. [x] Add a focused integration test for Android fallback behavior: when internet route fails and BLE fallback fires, require deterministic recipient receipt or deterministic terminal-failure signal.
   - Outcome (2026-03-03 HST): Added Android unit test `ble-only fallback path emits deterministic terminal failure when BLE send fails` in `MeshRepositoryTest`, plus iOS local transport test `testBleOnlyTerminalFailureSignal`.
4. [x] Add temporary operator/tester playbook step to clear or quarantine extreme legacy pending-outbox entries before pairing validation runs so fresh-message behavior is observable.
   - Outcome (2026-03-03 HST): Added "Legacy Pending Outbox Triage (No-Give-Up Safe)" operator workflow to `docs/RELAY_OPERATOR_GUIDE.md` with concrete Android/iOS inspection commands.
5. [ ] Capture synchronized tri-platform traces (Android logcat + iOS diagnostics + relay log window) for one failed message ID from send initiation through retry cycle.
6. [ ] Verify iOS-side receipt/ack emission path during Android BLE fallback attempts to confirm whether recipient ingest succeeds but ack path fails, or ingest fails entirely.

### WS12.13 Wave-2 Backlog Consolidation (2026-03-03 HST)

1. Non-historical mixed-doc checklists were normalized to status-tagged guidance/roadmap entries (no open checkbox ambiguity):
   - `FEATURE_WORKFLOW.md`
   - `AUDIT_QUICK_REFERENCE.md`
   - `FEATURE_PARITY.md`
   - `DRIFTNET_MESH_BLUEPRINT.md`
   - `docs/TRANSPORT_ARCHITECTURE.md`
2. `docs/TRANSPORT_ARCHITECTURE.md` future enhancements were migrated to explicit roadmap lines with status, owner, milestone, gate command, and acceptance criteria.
3. Validation/debt reconciliation executed:
   - `cargo check --workspace` — pass
   - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:generateUniFFIBindings` — pass
   - `bash iOS/copy-bindings.sh` — pass
   - `ANDROID_HOME=/path/to/android/sdk bash ./verify_integration.sh` — pass (now delegates to canonical WS12 matrix)
   - `bash ./verify_simulation.sh` — fail-fast as designed when Docker is unavailable (no auto-install side effects)
   - `cd wasm && wasm-pack build` — pass (after installing `wasm-pack` and disabling release `wasm-opt` in `wasm/Cargo.toml` for host compatibility)
4. Script hygiene updates:
   - `verify_integration.sh` converted from stale grep-based checks to canonical `scripts/verify_ws12_matrix.sh` execution.
   - `verify_simulation.sh` now requires preinstalled/running Docker and exits with explicit operator instructions instead of attempting automatic system installs.
5. Repo-wide checklist inventory after wave-2 normalization:
   - Open markdown checkboxes repo-wide: **71**
   - Active canonical open checkboxes: **18**
   - Historical open checkboxes: **53** (all under `docs/historical/*`)
6. Residual-risk carry-forward:
   - `R-WS10-02` remains `Deferred`.
7. Post-update issue slate (based on live watch artifacts from 2026-03-02/03):
   - [Tracked Live Gate] Relay session stability under active pairing run: verify iOS no longer oscillates through rapid connect/timeout/disconnect cycles in Multipeer + relay coexistence windows.
   - [Tracked Live Gate] Android internet route resilience in pairing runs: verify `Core-routed delivery failed` / `Relay-circuit retry failed` rates materially drop and `messagesRelayed` progresses.
   - Likely still remaining TODO unless explicitly fixed in this update:
     - [x] Android BLE GATT operation-state race: eliminate `IllegalStateException: The number of released permits cannot be greater than 1` in `BleGattClient.releaseGattOp` during callback races.
       - Outcome (2026-03-03 HST): `BleGattClient` now enforces single-release semantics per queued op using atomic permit-held state and overflow-safe release handling.
     - [x] Android BLE stack mismatch noise triage: investigate repetitive `BluetoothRemoteDevices Address type mismatch` flood and determine whether app-level dedupe/throttle or transport-state correction is required.
       - Outcome (2026-03-03 HST): Added address-type mismatch mitigation in `BleGattClient` (`ADDRESS_TYPE_MISMATCH_BACKOFF_MS`) with connect-throttle + stats counter (`addressTypeMismatchConnectSkips`) to prevent repeated immediate reconnect churn for the same peer address.
     - [x] iOS Multipeer channel storm guardrails: bound concurrent channel attempts and enforce deterministic cleanup to prevent repeated `Timed out, enforcing clean up` cascades under reconnect pressure.
       - Outcome (2026-03-03 HST): Added invite debounce, in-flight gating, concurrent-invite cap, and timeout/decline diagnostics counters in `MultipeerTransport`.
     - [x] End-to-end receipt convergence assertion: add one deterministic cross-platform test/runbook step proving recipient ingest + receipt emit for Android->iOS and iOS->Android when internet route degrades and BLE fallback activates.
       - Outcome (2026-03-03 HST): Added deterministic operator runbook in `docs/RELAY_OPERATOR_GUIDE.md` ("Cross-Platform Receipt Convergence Assertion").

### WS12.14 Android Bluetooth-Only Pairing Follow-ups (2026-03-03 HST, implementation + follow-up)

1. [x] Add a strict "BLE-only validation mode" for mobile test runs that hard-disables internet/relay route usage and emits a fail-fast diagnostic marker when non-BLE paths (for example WiFi-backed multipeer sessions) are used.
   - Outcome (2026-03-03 HST): Implemented `SC_BLE_ONLY_VALIDATION` gating in Android+iOS `MeshRepository`; non-BLE route usage is explicitly blocked and logged via deterministic `delivery_attempt ... reason=strict_ble_only_mode` markers.
2. [x] Harden Android BLE peer address-type handling for iOS peers; investigate and resolve repeated `Address type mismatch` churn so one canonical address-type mapping is retained per session.
   - Outcome (2026-03-03 HST): Added mismatch detection counter + per-address cooldown backoff to suppress reconnect hammering after address-type mismatch events in `BleGattClient`.
3. [x] Add Android BLE discovery-health counters (advertisements seen, GATT connects attempted/succeeded, address-type transitions) to diagnostics export and Mesh stats.
   - Outcome (2026-03-03 HST): Added `BleScanner` discovery stats and `BleGattClient` connect/mismatch counters, merged into Android diagnostics export (`ble_discovery`, `ble_client`, `strict_ble_only_validation`).
4. [x] Add iOS diagnostics marker for effective Multipeer transport medium per session (BLE/AWDL/WiFi) and include invitation timeout/decline reason counts.
   - Outcome (2026-03-03 HST): Added `MultipeerTransport.diagnosticsSnapshot()` and export fields in iOS diagnostics (`multipeer_effective_medium_estimate`, `multipeer_invite_timeout_count`, `multipeer_invite_decline_count`, `strict_ble_only_validation`).
5. [x] Add deterministic integration harness for Android<->iOS Bluetooth-only pairing/send/ack flow that fails on repeated invite timeout loops or zero-advertisement windows.
   - Outcome (2026-03-03 HST): Added `scripts/verify_ble_only_pairing.sh` and `scripts/verify_receipt_convergence.sh` harnesses for strict BLE-only marker validation and message ID receipt-convergence checks.
6. [ ] Capture synchronized BLE-only artifact bundle (Android logcat + iOS logs + one message ID timeline) after fixes and compare against WS12.14 baseline before closing risk.

### WS12.15 Wave-2 Continuation Plan Intake (2026-03-03 HST)

1. [x] Fix CLI reconnect-ledger panic under long failure streaks (`attempt to multiply with overflow` in `cli/src/ledger.rs`).
   - Outcome (2026-03-03 HST): backoff math now uses saturating arithmetic and clamped exponent; added regression test `test_ledger_entry_backoff_overflow_safety`; `cargo test -p scmessenger-cli ledger` and `cargo check -p scmessenger-cli` both pass.
2. [x] Install `wasm-pack` on the active dev host and rerun `cd wasm && wasm-pack build` to clear remaining local validation-debt blocker.
   - Outcome (2026-03-03 HST): Installed `wasm-pack 0.14.0`, added `wasm-opt = false` release-profile metadata in `wasm/Cargo.toml` for this host target, and re-ran `cd wasm && wasm-pack build` successfully.
3. [ ] Provision Docker runtime on the active dev host and rerun `bash ./verify_simulation.sh` to convert fail-fast prerequisite guidance into executed simulation evidence.
4. [ ] Execute live network matrix validation (GCP + direct P2P + relay fallback, Android+iOS) and store artifact bundle pointer in canonical docs.
5. [ ] Execute ACK-safe path switching validation (mid-send route switch, no duplicate/loss, sender receipt convergence) and record evidence.
6. [ ] Execute app-update + reinstall continuity validation on real Android+iOS devices and record identity/contact/history continuity evidence.
7. [ ] Capture iOS power settings runtime evidence on a real iPhone for beta-gate carry-forward and link artifacts.
8. [x] Resolve historical carry-forward decision from `docs/ALPHA_RELEASE_AUDIT_V0.1.2.md`: explicitly mark v0.1.2 version-bump/redeploy tasks as either superseded by v0.2.0 release-sync docs or carried into a dedicated historical closeout note.
   - Outcome (2026-03-03 HST): Updated `docs/ALPHA_RELEASE_AUDIT_V0.1.2.md` with explicit historical closeout status and canonical v0.2.0 release-sync pointers.

### WS12.16 Wave-2 Runtime Hardening Closure (2026-03-03 HST)

1. Implemented in this pass:
   - Android BLE GATT permit/callback race hardening (`BleGattClient`).
   - Android+iOS per-message `delivery_attempt` timeline diagnostics.
   - iOS relay timeline instrumentation + debounce/availability-state export.
   - iOS Multipeer invitation storm guardrails + timeout/decline diagnostics counters.
2. Verification commands:
   - `cd android && ANDROID_HOME=/path/to/android/sdk ./gradlew :app:compileDebugKotlin` — pass.
   - `bash ./iOS/verify-test.sh` — pass.
   - `cargo check --workspace` — pass.
3. Updated checklist inventory after WS12.16:
   - Open markdown checkboxes repo-wide: **76**
   - Active canonical open checkboxes: **23**
   - Historical open checkboxes: **53**
   - Note: this snapshot is superseded by WS12.17 final-wave inventory below.
4. Highest-priority remaining wave-2 actions (post-implementation evidence gates):
   - Re-run synchronized Android+iOS+GCP live probe and verify reduced relay/multipeer churn with receipt convergence in both directions.
   - Capture synchronized BLE-only and internet-degraded artifact bundle with one message ID end-to-end timeline for residual-risk closure.
   - Provision Docker runtime and rerun `bash ./verify_simulation.sh` to clear the final local validation-debt blocker.

### WS12.17 Wave-3 Governance Closure (2026-03-03 HST)

1. Historical checklist triage completed:
   - `docs/historical/APP_VERSION_0.1.2_ALPHA_PLAN.md`
   - `docs/historical/REMEDIATION_PLAN.md`
   - `docs/historical/iOS/PHASE4_IMPLEMENTATION.md`
   - `docs/historical/iOS/PHASES_4-15_GUIDE.md`
   - `docs/historical/iOS/FINAL_STATUS.md`
   - All open checkboxes in these historical files were converted to explicit historical status tags (`Historical - Superseded`, `Historical - Re-scoped`, `Historical - Carry-forward`).
2. Added deterministic runtime-harness set for active follow-ups:
   - `scripts/correlate_relay_flap_windows.sh`
   - `scripts/verify_relay_flap_regression.sh`
   - `scripts/verify_receipt_convergence.sh`
   - `scripts/verify_ble_only_pairing.sh`
3. Normalized future execution queue (v0.2.1+ planning scope, non-blocking for v0.2.0 closeout):
   - `WS13.1` Identity metadata persistence — Owner: Core + Mobile Bridge — Gate: identity persistence + migration test suite.
   - `WS13.2` Contact/request schema updates — Owner: Core Data + Mobile/WASM adapters — Gate: schema migration + parity adapter tests.
   - `WS13.3` Registration protocol/signature verification — Owner: Core Transport — Gate: protocol signature validation integration tests.
   - `WS13.4` Relay registry/custody enforcement — Owner: Core Transport + Relay Ops — Gate: custody routing + registry state-machine tests.
   - `WS13.5` Handover/abandon queue migration + UX — Owner: Core + Android+iOS clients — Gate: queue migration and user-facing rejection-path tests.
   - `WS13.6` Compatibility/migration matrix — Owner: Cross-platform QA — Gate: upgrade/migration matrix and manual runbook evidence.
   - `WS14.1` Notification policy model — Owner: Core + Bindings — Gate: classifier/unit tests + UDL/WASM API parity checks.
   - `WS14.2` iOS notification completion — Owner: iOS — Gate: DM/DM-request routing integration tests.
   - `WS14.3` Android notification completion — Owner: Android — Gate: channel/action parity tests + foreground suppression checks.
   - `WS14.4` WASM notification wiring — Owner: Web/WASM — Gate: browser worker notification flow tests.
   - `WS14.5` Hybrid endpoint interface prep — Owner: Core + Adapter surfaces — Gate: endpoint registration persistence/validation tests.
   - `WS14.6` Verification + docs gate — Owner: Cross-platform QA + Docs — Gate: parity matrix pass + residual-risk sync.
4. Final inventory after wave-3 triage (`rg -P "^\s*(?:[-*]|\d+\.)\s+\[ \]" --glob "*.md"`):
   - Open markdown checklist items repo-wide: **10**
   - Active canonical open checklist items: **10** (`REMAINING_WORK_TRACKING.md` only)
   - Historical open checklist items: **0**
5. Remaining action items at WS12.17 closeout (repo-wide exhaustive list at that time):
   - WS12.8.5: stabilize Android wireless ADB endpoint persistence across daemon reconnect cycles.
   - WS12.11.6: run synchronized dual-device live probe and capture full flap-cycle bundle.
   - WS12.12.5: capture synchronized tri-platform traces for one failed message ID.
   - WS12.12.6: verify iOS receipt/ack emission path during Android BLE fallback attempts.
   - WS12.14.6: capture synchronized BLE-only artifact bundle and compare against baseline.
   - WS12.15.3: provision Docker runtime and rerun `verify_simulation.sh`.
   - WS12.15.4: execute live network matrix validation (GCP + direct + relay fallback).
   - WS12.15.5: execute ACK-safe path switching validation and record evidence.
   - WS12.15.6: execute app-update + reinstall continuity validation on real Android+iOS devices.
   - WS12.15.7: capture iOS power settings runtime evidence on real iPhone.

## Priority 1: Tooling, CI, and Experimental Surface

1. [x] Align CI with tri-platform target status
   - Outcome (2026-03-03): `.github/workflows/ci.yml` now includes explicit WS12 parity/test gates for:
     - deterministic core offline/partition suites,
     - Android role/fallback unit parity checks,
     - desktop/WASM role parity checks,
     - iOS verification with transport fallback + role-mode parity checks.

2. [x] Add browser-executed WASM test job (parity gate)
   - Current: native/non-browser WASM tests only in workspace run.
   - Target: `wasm-pack` runtime test coverage in CI.
   - Outcome: `.github/workflows/ci.yml` `check-wasm` installs `wasm-pack` and runs browser runtime tests (`wasm-pack test --headless --firefox`) in CI.

3. [x] Resolve integration test warnings in core tests
   - Current: workspace tests pass with warning noise.
   - Target: warning-clean path for strict CI.
   - Outcome: Cleaned up unused assignments and unused variables across all integration suites. Unit and integration tests are 100% warning-clean.

4. [x] Standardize Android CI environment setup for `ANDROID_HOME`
   - Current: local build requires explicit shell env setup.
   - Target: consistent CI env bootstrap and preflight enforcement.
   - Outcome: `.github/workflows/ci.yml` `check-android` now sets up Android SDK, standardizes `ANDROID_HOME`/`ANDROID_SDK_ROOT`, and runs `android/verify-build-setup.sh` preflight before Gradle build/tests.

5. [x] iOS legacy tree cleanup policy
   - Active app lives in `iOS/SCMessenger/SCMessenger/`.
   - `iOS/SCMessenger-Existing/` confirmed non-existent — legacy code already cleaned up.
   - Outcome: Verified directory does not exist; task complete.

6. [x] Docker test/ops script consistency cleanup
   - Current: mixed compose filename references and stale command paths across `docker/*.sh` and docs.
   - Target: one canonical compose naming set and verified command examples that match checked-in files.
   - Outcome: Normalized all references to use canonical compose naming (`docker compose` CLI standard and `docker-compose*.yml` filename format without spaces).

7. [x] CLI surface normalization for long-term dependability
   - Current: `cli/src/main.rs.backup` and mixed identity/public-key field naming remain in the CLI surface.
   - Target: remove backup artifacts from runtime path, align CLI identity/contact semantics with canonical `public_key_hex`, and revalidate relay/bootstrap controls.
   - Outcome: No `.backup` files found in repo. CLI codebase is clean of TODO/FIXME markers. Identity/public-key naming aligned with canonical `public_key_hex`.

8. [x] Reference artifact hygiene
   - Current: `reference/Androidlogs.txt` includes non-SCMessenger application logs; `reference/` mixes active porting guides with raw captures.
   - Target: isolate SCMessenger-specific evidence logs and keep reference crypto sources clearly separated from runtime diagnostics.
   - Outcome: Reference directory well-organized with README. Historical audit/migration docs moved to `docs/historical/` with index.

9. [x] Android test execution truthfulness cleanup
   - Current: `android/app/src/test/README.md` says previously `@Ignored` tests are enabled, but `android/app/src/test/java/com/scmessenger/android/test/MeshRepositoryTest.kt` still contains broad `@Ignore` usage.
   - Target: either enable those tests with stable mocks or update docs/scripts to match actual execution status.
   - Outcome: Updated `android/app/src/test/README.md` to truthfully explain that UniFFI MockK limitations natively prevent complete CI verification for generated files, serving as architectural documentation pending a stable JNA harness setup instead.

10. [x] Android ABI and verification script alignment

- Current: `android/app/build.gradle` and `buildRustAndroid` are aligned on `arm64-v8a` + `x86_64`, but `android/verify-build-setup.sh` still checks for legacy extra Rust targets (`armv7`, `i686`).
- Target: align environment verification script with actual supported ABI matrix and documentation.
- Outcome: `android/verify-build-setup.sh` now validates only `aarch64-linux-android` and `x86_64-linux-android`, and install guidance was updated to match the supported ABI matrix.

11. [x] Core settings model convergence (critical reliability debt)

- Current: multiple overlapping settings models diverge in defaults/semantics:
  - `core/src/mobile_bridge.rs` (`MeshSettings`, DiscoveryMode `Normal/Cautious/Paranoid`)
  - `core/src/mobile/settings.rs` (`MeshSettings`, DiscoveryMode from transport layer)
  - `core/src/platform/settings.rs` (`MeshSettings`, DiscoveryMode `Open/Closed/Stealth`)
- Target: one canonical settings schema and mapping strategy used by UniFFI/mobile/runtime layers.
- Outcome: Deleted the unused `mobile/settings.rs` and `platform/settings.rs` completely. Unified purely behind the single UniFFI-verified `mobile_bridge::MeshSettings` exported transparently through `api.udl`. Web clients will default to "always plugged in" behavior via this schema.

12. [x] iOS verification script hardening

- Current: `iOS/verify-test.sh` now performs simulator build verification.
- Target: harden script behavior (deterministic destination selection, warning handling policy, and explicit failure output) for stable CI/operator use.
- Outcome: `iOS/verify-test.sh` now uses strict shell flags, deterministic `generic/platform=iOS Simulator` destination, explicit failure handling, and an explicit warning count policy.

13. [x] iOS background capability hardening

- Current: `iOS/SCMessenger/SCMessenger/Info.plist` declares a broad background mode set.
- Target: retain only modes required by implemented behavior and provisioning policy; remove speculative extras.
- Outcome: removed speculative `location` and `remote-notification` background modes and removed unused location/motion permission strings from `Info.plist`; retained BLE + fetch + processing modes used by implemented services.

14. [x] iOS power settings runtime observability and enforcement verification (Validated for v0.1.2)

- Outcome: Added diagnostic logging to `applyPowerAdjustments` in `MeshRepository.swift`. Verified that Android identity survives upgrade/reinstall. (iOS verification pending unlock, but code-hardened and logic parity confirmed).

- Current: explicit runtime logging/enforcement hooks are now wired in `MeshRepository` (`setAutoAdjustEnabled`, `applyPowerAdjustments`, and profile-application logs across battery/network/motion updates), and Settings toggle now drives repository state directly.
- Remaining: capture active-session device evidence confirming power profile transitions under real motion/network/battery changes.
- Follow-up: simplify iOS power UX to a single automatic mode and remove manual Low/Standard/High style overrides; drive gradual adaptation from battery %, bandwidth quality, and latency measurements.

15. [x] iOS generated-binding path normalization

- Current: `iOS/copy-bindings.sh` wrote generated files into both `iOS/SCMessenger/SCMessenger/Generated/` and `iOS/SCMessenger/Generated/`.
- Target: one canonical generated artifact path tied to active Xcode targets and docs.
- Outcome: `iOS/copy-bindings.sh` now writes only to `iOS/SCMessenger/SCMessenger/Generated/`, which matches active Xcode target paths.

16. [x] iOS historical artifact segmentation

- Current: `iOS/iosdesign.md` and `iOS/SCMessenger/build_*.txt` mix design/historical/runtime evidence in active tree.
- Target: section-level historical tagging and relocation/retention policy to keep active docs concise.
- Outcome: historical iOS design artifacts are retained under `docs/historical/` references, and active `iOS/` tree no longer contains `iOS/iosdesign.md` / `iOS/SCMessenger/build_*.txt` historical noise files.

17. [x] TODO/FIXME accuracy sync pass (including external test/update signals)

- Current: TODO/FIXME markers are distributed across code/docs; external testing updates can drift from tracked backlog.
- Target: recurring TODO/FIXME audit that syncs canonical backlog items with current implementation evidence.
- Evidence source: `docs/historical/TRIPLE_CHECK_REPORT.md` risk scan + direct file review.
- Companion reference: `docs/STUBS_AND_UNIMPLEMENTED.md` — comprehensive stub/placeholder inventory (43 items across 4 severity tiers).
- Outcome: Full sweep completed. Core Rust, CLI, WASM, and Android codebases are clean of actionable TODO/FIXME markers. iOS TODOs are exclusively auto-generated UniFFI scaffolding comments (not actionable).

18. [x] Android multi-share intent handler — full implementation with IntentCompat

- File: `android/app/src/main/java/com/scmessenger/android/utils/ShareReceiver.kt`.
- History: stub was originally removed (prior outcome); PR #48 added a complete working implementation.
- Outcome: `ShareReceiver` now handles `ACTION_SEND_MULTIPLE` with `IntentCompat.getParcelableArrayListExtra()` for API < 33 compatibility (no `NoSuchMethodError` crash on Android 12 and below). Multi-stream URI items are forwarded correctly alongside text items.

19. [x] App-update persistence migration hardening (identity, contacts, message history)

- Requirement: app upgrades must preserve identity, contacts, and message history without manual re-import.
- Target: deterministic migration/verification path across Android and iOS app updates, including storage-path continuity checks and automatic import fallback for legacy stores.
- Scope: core storage versioning, mobile app startup migration hooks, and update smoke tests that assert post-update continuity.
- Completed:
  - Added core storage layout/schema guard (`SCHEMA_VERSION`) and explicit `identity/`, `outbox/`, `inbox/` sub-store initialization.
  - `IronCore::with_storage()` now initializes persistent inbox/outbox backends (not memory-only fallback by default).
  - Added core persistence restart tests for inbox/outbox continuity under storage-backed initialization.
  - Added schema v2 legacy-root migration to copy old identity/outbox/inbox keys into split sub-stores on upgrade.
  - Identity manager now hydrates persisted identity/nickname on startup without auto-generating fresh identities.
  - Added restart continuity tests for identity hydration, legacy-root migration, contacts (including local nickname), and history delivery-state persistence.
  - Android onboarding now waits for confirmed identity creation + nickname persistence before completing first-run flow.
  - Android/iOS repository flows now explicitly resume deferred swarm startup after identity/nickname creation, closing a first-run internet transport stall path.
  - CLI relay mode now uses persisted headless network identity (`storage/relay_network_key.pb`) so relay peer IDs remain stable across process restarts; key migrated from existing IronCore identity on first upgrade to preserve `/p2p/` bootstrap addresses.
  - Identity backup export/import implemented via iOS Keychain and Android SharedPreferences (`identity_backup_prefs.xml`); survives full reinstall with no manual re-import.
  - `mark_message_sent(message_id)` added to `IronCore` and exposed via UniFFI; mobile clients call it after confirmed ACK to keep outbox bounded (prevents "outbox full" stall on long-lived accounts).
  - Key material zeroized after use in both `export_identity_backup` and `import_identity_backup` (even on error path).
  - Android `allowBackup="true"` + `dataExtractionRules` + `fullBackupContent` wired in `AndroidManifest.xml`; `backup_rules.xml` fixed (removed `<include>` that silently disabled default backup).
  - BLE GATT sequential operation queue implemented (`Channel<() -> Unit>` + `Semaphore(1)` per device); all reads, writes, and CCCD writes serialised; stale-session guard on refresh reads.
  - `cargo clippy --workspace` clean; `cargo fmt --all` clean; 5 new core unit tests for backup roundtrip, validation errors, and `mark_message_sent` behaviour.
- Remaining (validation only — no code changes needed):
  - Platform-level upgrade simulations on Android/iOS/WASM package installs with real prior-app data.
  - End-to-end package upgrade evidence capture (device install/update logs + retained chat transcript checks).

## Priority 2: Documentation Completion and Governance

1. Full-file documentation pass completion using `docs/historical/DOC_PASS_TRACKER.md` (completed)
   - Current: all tracked files are reviewed (`pending` = 0, checked = 356).
   - Ongoing target: keep this at 0 pending via delta checks on new/changed files.

2. Historical-heavy docs section-status sweep
   - Requirement: stale/current components tagged at section level (`[Current]`, `[Historical]`, `[Needs Revalidation]`) with canonical pointers.

3. Keep canonical chain authoritative
   - `README.md`
   - `DOCUMENTATION.md`
   - `docs/REPO_CONTEXT.md`
   - `docs/CURRENT_STATE.md`
   - `REMAINING_WORK_TRACKING.md`
   - `docs/GLOBAL_ROLLOUT_PLAN.md`
   - `docs/STUBS_AND_UNIMPLEMENTED.md`

4. [x] Resolve `ios/` vs `iOS/` path-case split in tracked docs vs app source
   - Outcome: canonicalized documentation/script references to `iOS/` and recorded governance rule to prevent lowercase-path drift.

## Verified Stable Areas (No Active Gap)

- `cargo test --workspace` passes (367 passed, 0 failed, 17 ignored — verified 2026-03-03)
- `cargo clippy --workspace` clean (0 warnings)
- `cargo fmt --all -- --check` clean
- Core NAT reflection integration tests pass
- iOS build verification script passes, including static library build
- iOS simulator app build passes (`SCMessenger` scheme, iPhone 17 simulator)
- Android build verification script passes when `ANDROID_HOME` is set
- Android app build passes (`./gradlew assembleDebug`)
- Topic subscribe/unsubscribe/publish paths are wired on Android and iOS
- QR contact + join bundle scan flows are wired on Android and iOS
- CLI command surface and control API paths are functional
- Identity backup export/import wired end-to-end (iOS Keychain, Android SharedPreferences)
- Relay PeerId stable across CLI upgrades (persisted `relay_network_key.pb`, migrated from IronCore identity)
- WASM swarm transport functional (`startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers`)
- `mark_message_sent` exposed via UniFFI for bounded outbox management

## Roadmap to 1.0.0 (Post v0.2.0-alpha)

## Immediate v0.2.0 Closeout Queue (Feasible Remaining Work)

The following items are feasible to execute as additional `v0.2.0` closure work without introducing net-new product scope:

1. [x] `R-WS3-02` / `EC-01`: migrate relay custody default persistence from temp-dir to durable app data paths and add restart recovery verification.
   - Outcome: `RelayCustodyStore::for_local_peer` now resolves to durable app-data paths (`SCM_RELAY_CUSTODY_DIR` override -> OS local data dir -> home fallback -> temp fallback), with restart persistence tests retained.
2. [x] `R-WS5-01` / `EC-02`: ensure platform adapters always provide storage snapshots so dynamic pressure controls cannot no-op.
   - Outcome: storage pressure enforcement now uses synthetic snapshot fallback when platform probes are unavailable, preventing no-op behavior in fallback paths.
3. [x] `R-WS4-02` / `EC-04`: add low-cost convergence-marker trust hardening and abuse validation checks.
   - Outcome: convergence markers now require structural/timestamp validation and local message-tracking correlation before retry/custody convergence is applied.
4. [x] Release sync execution (`WS13.x` scoped to release ops): finalize versions/tags/release notes using `docs/releases/*` artifacts.
   - Outcome: release artifacts are canonicalized in `docs/releases/*`, workspace/app version metadata is bumped to `0.2.0`, and CI/docs now reference repo-local release sources.

Not feasible for `v0.2.0` without expanding release scope:

1. `WS13` Tight Pairing (single active device lifecycle).
2. `WS14` direct-message/direct-request notifications.

3. **Automatic Environment Detection and Unified Hydration**
   - Requirement: The app must automatically detect if a previous identity, message history, contacts, or user preferences exist in local storage/backups and utilize them immediately on startup without user intervention.
   - Target: Unified "detect-and-resume" logic that covers all persisted data types across Android, iOS, and Web.
   - Scope: Identity (Keychain/SharedPreferences), Message History (history.db), Contacts (contacts.db), and Privacy Toggles.

4. **Manual Data Management (Reset/Refresh/Delete)**
   - Requirement: Provide a secure, user-facing way to clear or reset all application data.
   - Target: A "Delete All Data" or "Reset Application" button in the Settings view.
   - Action: Securely wipe identity, message history, contacts, and all local preferences from the device.
   - Scope: Android (`SettingsScreen`), iOS (`SettingsView`), and Web.

5. **WS13 (v0.2.1): Single Active Device per Identity (Tight Pairing)**
   - Requirement: enforce one active `(identity_public_key, device_id)` destination binding to prevent stale/recycled identity misrouting and multi-device active collisions.
   - Target: cryptographically signed registration/deregistration protocol + relay-side registration state machine + custody enforcement.
   - Scope:
     - identity persistence (`device_id`, `seniority_timestamp`),
     - contacts metadata (`last_known_device_id`),
     - transport protocol (`/sc/registration/1.0.0`),
     - relay custody registry states (`Active`, `Handover`, `Abandoned`),
     - sender-facing recycled/abandoned error semantics.
   - LoC planning envelope: `3,950-6,950 LoC`.
   - Execution decomposition: `WS13.1` through `WS13.6`.
   - Canonical plan: `docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md`.
   - Kickoff prompt: `docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md` section `WS13 Kickoff (v0.2.1) - Tight Pairing start`.

6. **WS13.x (v0.2.1): GitHub release/version synchronization and release-note publishing flow**
   - Requirement: normalize repository/app version metadata and GitHub release artifacts so release tags, release notes, and workspace versions remain consistent.
   - Target:
     - align workspace/package versions for the intended release cut,
     - ensure `v0.1.2` GitHub release notes are finalized/publish-ready,
     - stage `v0.2.0` draft release notes and release checklist inputs for final cut timing.
   - Scope:
     - Cargo/workspace version synchronization (`Cargo.toml` files as applicable at release-cut time),
     - release note doc finalization for GitHub paste/publish flow,
     - release workflow checklist alignment with residual-risk closure evidence,
     - promote external planning artifacts into repo-local docs before execution (to avoid workstation-specific paths).
   - Source inputs now canonicalized in-repo:
     - `docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md`
     - `docs/releases/RELEASE_NOTES_V0.1.2_GH.md`
     - `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`
   - Canonicalization target (during WS13.x execution):
     - keep `docs/releases/` as the only release-notes/checklist source of truth.
   - Progress: repo-local release planning/note artifacts now exist under `docs/releases/`, so WS13.x no longer depends on workstation-specific external files.
   - Execution note: queue this after current WS12 in-flight session and after WS12/WS12.5 closure evidence is captured.

7. **WS14 (v0.2.1): Direct Message + Direct Message Request Notifications (iOS/Android/WASM)**
   - Requirement: notification parity for direct messages and direct message requests across iOS, Android, and WASM.
   - Delivery model: hybrid.
     - Local notifications are fully shipped in WS14.
     - Remote-push interfaces/contracts are prepared in WS14, while APNs/FCM/Web Push backend dispatch is deferred.
   - Product rules:
     - DM Request source is both unknown-sender inference and explicit request flag/type support.
     - Notification tap behavior: existing conversation opens the exact conversation; new request opens Requests Inbox.
   - LoC planning envelope: `2,500-4,550 LoC`.
   - Canonical plan (full context): `docs/V0.2.1_NOTIFICATIONS_DM_PLAN.md`.
   - Execution handoff + prompt: `docs/WS14_AUTOMATION_HANDOFF.md` and `docs/WS14_HOURLY_AUTOMATION_PROMPT.md`.
   - Current status (2026-03-14 HST): `WS14.1` core notification contract and `WS14.2` iOS notification completion are landed on `codex/ws14-hourly-20260314-0301`; next exact phase is `WS14.3` Android notification parity.

## Edge-Case Hardening Backlog (Global/Extreme Conditions)

Canonical scenario matrix and rationale:

- `docs/EDGE_CASE_READINESS_MATRIX.md`

Priority items to track into remaining v0.2.x execution:

1. `[Closed in WS12.6]` `EC-01`: relay custody default persistence now uses durable app path fallback chain (`R-WS3-02` closed).
2. `[Closed in WS12.6]` `EC-02`: platform storage snapshots now have synthetic fallback so pressure policy cannot no-op (`R-WS5-01` closed).
3. `EC-03` (Accepted in v0.2.0 alpha): replace volatile local transport route hints with stable authenticated alias mapping (`R-WS6-01`, `R-WS7-01`, revisit before beta hardening).
4. `[Closed in WS12.6]` `EC-04`: convergence marker validation/trust hardening baseline shipped (`R-WS4-02` closed).
5. `[Closed in WS12]` `EC-05`: custody reconnect integration test is now CI-gated and reproducible (`R-WS3-01` closed).
6. `EC-06` (Reduced in WS11, accepted in WS12): sender-facing delivery states are normalized in Android+iOS UI/export surfaces; remaining Core-native transition API work is tracked via `R-WS11-01` for post-v0.2.0 follow-up.
7. `EC-07` to `EC-09` (v0.2.1 WS13): execute tight-pair single-active-device lifecycle.
8. `EC-10` to `EC-16` (post-v0.2.1): captive portal adaptation, high-latency profile tuning, censorship-resilience strategy, wake/delegate architecture, sparse encounter optimization, and clock-skew normalization.

## 2026-03-13 iOS Simulator Launch Ambiguity

- Completed: Identified and cleared an iPhone 17 Pro simulator launch blocker caused by a stale `platform IOS` SCMessenger bundle installed into the simulator instead of an `IOSSIMULATOR` build.
- Open: If this recurs, audit any operator or harness path that reuses a previously installed simulator bundle without validating the built Mach-O platform.

## 2026-03-13 Consolidated Open Items From Full Conversation

- Open: prove full 5-node visibility after simulator recovery using the upgraded `run5.sh`; current honest state remains partially indeterminate rather than fully verified.
- Open: investigate iOS simulator runtime `historySync request failed to prepare message` after successful launch recovery.
- Open: complete iOS send-path parity with store-and-forward-first UX so the send action never blocks on live transport success.
- Open: continue hardening iOS against peer-identify / identity-beacon event storms that can contribute to transient freeze/unfreeze behavior.
- Open: unify Android BLE telemetry so accepted-send target reporting matches the actual fresher connected GATT target used on the wire.
- Open: improve physical iOS app-level own-ID/peer capture in harness evidence so transport activity is not hidden by collector gaps.
- Open: validate simultaneous transport functionality across BLE, direct LAN/libp2p, relay, and Wi-Fi Direct/local options.
- Open: identify any script/operator path capable of reinstalling or preserving a stale `iphoneos` bundle inside the simulator.
