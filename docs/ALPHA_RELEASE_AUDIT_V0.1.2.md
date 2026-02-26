# SCMessenger v0.1.2 Alpha Release Audit

**Audited:** 2026-02-26
**Branch audited:** `claude/audit-alpha-release-plan-bq4sA` (merged from `origin/main` de2188b)
**Method:** Full codebase audit — documentation review, code inspection, runtime log analysis (android_log.txt)

---

## Executive Summary

The codebase is in strong shape for alpha. The core Rust library is feature-complete with 343 passing tests, zero clippy warnings, and all four documented hardening issues resolved. Both Android and iOS apps build and install. WASM swarm transport is implemented. The remaining work before alpha is **small and well-scoped**: two specific code fixes, a version bump, one repo hygiene item, and live-network validation runs.

---

## Verified Stable (No Changes Needed)

| Area | Status |
|---|---|
| `cargo test --workspace` | 343 passed, 0 failed, 7 ignored |
| `cargo clippy --workspace` | 0 warnings |
| `cargo fmt --all -- --check` | Clean |
| Core Rust modules (12 of 12) | Complete — identity, crypto, message, store, transport, drift, routing, relay, privacy |
| All 4 hardening issues | Resolved — resume storm, zombie loop, slow loris, key leak |
| All stubs cleared | Per `docs/STUBS_AND_UNIMPLEMENTED.md` |
| Android build + install | Passes (`./gradlew assembleDebug` + `install-clean.sh`) |
| iOS build (sim + device) | Passes (`xcodebuild` sim and device-target) |
| WASM swarm transport | `startSwarm`, `stopSwarm`, `sendPreparedEnvelope`, `getPeers` all functional |
| Delivery receipts | `MessageType::Receipt` + `DeliveryStatus` wired end-to-end |
| Identity backup/restore | iOS Keychain + Android SharedPreferences, survives reinstall |
| Privacy parity | All 4 toggles (onion, cover, padding, timing) on Android + iOS + Web |
| Relay toggle enforcement | Inbound/outbound blocking on OFF verified on all 3 platforms |
| First-run consent gate | Implemented on Android + iOS (6-step flow) |
| Bounded retention policy | `enforce_retention(max_messages)` + `prune_before(before_timestamp)` via UniFFI |
| Relay visibility (contacts) | Headless/relay nodes blocked from contact DB and Contacts nearby list |
| Bootstrap relay UI | Relay peers shown distinctly in Dashboard/Mesh tab with "Relay Node" label |
| BLE GATT sequential queue | All reads/writes/CCCD ops serialized per-device (Android) |
| Schema v2 migration | Identity/outbox/inbox sub-store layout with SCHEMA_VERSION guard |
| Relay PeerId stability | Persisted `relay_network_key.pb`, migrated from IronCore identity |
| Connection path state | `ConnectionPathState` + `export_diagnostics()` exposed in UniFFI + WASM |

---

## Open Items

### P0 — Must Fix Before Alpha Release

#### 1. `onPeerIdentified` identify storm causes excessive background work (ANDROID)

**Source:** `android_log.txt` (live device, 2026-02-25 19:21 session)
**Observed:** The GCP relay node (`12D3KooWET...`) triggers **20+ `onPeerIdentified` callbacks within 1 second** on connect. Each callback fires:
- `flushPendingOutbox(...)` → loads `pending_outbox.json` from disk (file I/O)
- `updateBleIdentityBeacon()` → queues a GATT write (GATT op)
- `primeRelayBootstrapConnections()` → coroutine launch

With 20+ identify events per second, this results in ~20 redundant file I/O cycles, ~20 queued GATT operations, and excessive coroutine activity on every relay connection.

**Root cause:** The libp2p `identify` protocol fires once per open stream to the relay. Multiple transport connections (TCP, QUIC, relay circuit) all fire independently, and the relay's 60s interval does not prevent burst-on-connect.

**Fix:** Add per-peer deduplication for post-identify side effects in `onPeerIdentified`. Track `lastOutboxFlushByPeer` and `lastBeaconUpdateByPeer` sets; only fire once per peerId per minimum window (e.g., 5 seconds).

**File:** `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`
**LOC estimate:** ~30–50 LOC

**Verify iOS too:** iOS has the same `onPeerIdentified` handler in `MeshRepository.swift`. Check if the same storm occurs and apply equivalent deduplication if needed.

---

#### 2. iOS missing delayed BLE identity retry reads

**Source:** Code inspection of `iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift` vs Android `BleGattClient.kt`

**Gap:** Android schedules BLE identity re-reads at **T+900ms** and **T+2200ms** after connect (`IDENTITY_REFRESH_DELAYS_MS = listOf(900L, 2200L)`), handling the case where a peer's identity GATT characteristic is not yet populated at connection time. iOS reads the identity characteristic exactly once, immediately after `didDiscoverCharacteristicsFor`. No retry is scheduled.

**Impact:** BLE-connected iOS peers may be discovered without a public key if the peripheral's GATT server is not yet ready at characteristic discovery time. The missing identity causes the peer to appear in the mesh dashboard without nickname or identity, and prevents messaging.

**Fix:** In `BLECentralManager.swift`, after the first `peripheral.readValue(for: identityCharacteristic)` in `didDiscoverCharacteristicsFor`, schedule two delayed re-reads at T+900ms and T+2200ms using `Task { try? await Task.sleep(nanoseconds: ...); peripheral.readValue(for: char) }`, guarded by `connectedPeripherals[peripheral.identifier] != nil`.

**File:** `iOS/SCMessenger/SCMessenger/Transport/BLECentralManager.swift`
**LOC estimate:** ~30–50 LOC

---

#### 3. Version numbers need bumping

**Current:** `Cargo.toml` workspace has `version = "0.1.1"`. `core/src/transport/behaviour.rs` has agent version `"scmessenger/0.1.1/..."`.

**Required:** Both must be `0.1.2` for the alpha release.

**Note:** The live GCP relay still reports `scmessenger/0.1.0/headless/relay/...` in the android log — the relay server needs to be rebuilt and redeployed after the version bump lands.

**Files:**
- `Cargo.toml` (workspace `[workspace.package].version`)
- `core/src/transport/behaviour.rs` (agent version string)

**LOC estimate:** 2 LOC

---

#### 4. `android_log.txt` should not be in version control

**Source:** File committed in latest merge from main (26,448 lines, raw Android `adb logcat` output).

**Fix:** Add `android_log.txt` to `.gitignore` and remove from tracking. These device logs belong in `logs/` (which is already gitignored) or handed off as external artifacts.

**Files:** `.gitignore`, remove `android_log.txt` from git index
**LOC estimate:** 1 line in `.gitignore`

---

### P0 — Validation Required (No Code Changes)

These items have complete code implementations. They require live-device or live-network verification to close the alpha gate.

#### 5. Live network matrix validation

**Status:** Code complete. Validation evidence pending.

**Required runs:**
- GCP bootstrap connect from non-LAN device (cellular) — both Android and iOS
- Direct P2P probe with no relay (both on same LAN)
- Forced relay-only delivery (direct path blocked)
- Network switch mid-send (Wi-Fi → cellular) with no message loss
- Cross-version: `v0.1.2-alpha` browser ↔ native (per `docs/PARTNER_TEST_PLAYBOOK_V0.1.2_ALPHA.md`)

**Acceptance:** No message loss, no duplicates, relay fallback succeeds within 30s of direct path failure.

---

#### 6. ACK-safe path switching validation

**Status:** Code complete (stable UUIDs, idempotent receive apply, ACK reconciliation implemented).

**Required:** Induce a network-path switch (e.g., disable Wi-Fi while a message is in-flight) and verify:
- Message delivered exactly once
- No stuck "Sending..." state
- Receipt arrives on sender side

---

#### 7. App-update continuity validation on real devices

**Status:** Code complete (backup/restore, schema migration, relay key migration all implemented and unit-tested).

**Required:** On a real Android device and iPhone:
1. Install the `v0.1.1` build with existing identity, contacts, and chat history.
2. Update to `v0.1.2-alpha` build.
3. Verify: identity preserved, contacts intact, chat history readable, no startup crash.

---

### P1 — Important, Non-Blocking

These items do not block the alpha release but should be scheduled immediately after.

#### 8. CI tri-platform gate alignment

**Gap:** Primary CI (`.github/workflows/ci.yml`) validates Rust workspace on Linux/macOS only. No Android build check, no iOS build check, no WASM browser runtime test.

**Risk:** Platform regressions are caught only by manual build verification scripts.

**Target:** Add CI jobs for:
- Android: `./gradlew assembleDebug` (with mocked `ANDROID_HOME`)
- iOS: `xcodebuild -scheme SCMessenger -destination 'generic/platform=iOS Simulator'`
- WASM compile check: `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown`

---

#### 9. GCP relay server rebuild after version bump

After item 3 (version bump) is merged and the Rust workspace is rebuilt, the GCP relay node needs to be rebuilt and redeployed so it reports `scmessenger/0.1.2/headless/relay/...` in identify messages.

**Current state:** Relay reports `scmessenger/0.1.0/headless/relay/...` per `android_log.txt`.

Use `scripts/deploy_gcp_node.sh` for the deployment.

---

#### 10. iOS power settings runtime evidence

**Status:** Code is complete — `setAutoAdjustEnabled`, `applyPowerAdjustments`, and profile-application logs are all wired in `MeshRepository`.

**Remaining:** Capture device evidence from a real iPhone confirming power profile transitions (Standard → Low → Standard) under real battery/motion/network changes. Not blocking alpha, but needed for beta gate sign-off.

---

### Deferred to Beta/Post-Alpha

These items are explicitly out of scope for v0.1.2-alpha per the policy decisions in `REMAINING_WORK_TRACKING.md` and `docs/GLOBAL_ROLLOUT_PLAN.md`.

| Item | Policy | Notes |
|---|---|---|
| Anti-abuse controls | Mandatory before **beta** | Not blocking alpha |
| WASM WebRTC answerer + ICE trickle | ~150 LOC | WebSocket is primary path; WebRTC is experimental beta track |
| WASM IndexedDB persistence | Beta parity | `localStorage` fallback works for alpha |
| WASM browser runtime tests in CI | Beta gate | `wasm-pack` not installed in CI env |
| Auto-detect and resume on startup | Roadmap 1.0.0 | Not committed for alpha |
| Reset All Data / Manual Data Management | Roadmap 1.0.0 | Not committed for alpha |
| i18n beyond English | Roadmap post-1.0.0 | Alpha is English-only by policy |
| iOS historical artifact cleanup | Hygiene | `iOS/iosdesign.md` and `build_*.txt` can wait |

---

## Alpha Release Checklist

```
MUST COMPLETE BEFORE SHIPPING:
[ ] Fix onPeerIdentified identify storm (Android + verify iOS)
[ ] Implement iOS BLE delayed identity refresh reads (T+900ms, T+2200ms)
[ ] Bump version: Cargo.toml + behaviour.rs agent string → 0.1.2
[ ] Remove android_log.txt from repo, add to .gitignore
[ ] Rebuild and redeploy GCP relay after version bump
[ ] Live network validation: GCP + direct P2P + relay fallback (Android + iOS)
[ ] ACK-safe path switching validation (both platforms)
[ ] App-update continuity: install v0.1.1 → upgrade to v0.1.2, verify data retained

SHOULD DO SHORTLY AFTER ALPHA:
[ ] Add Android+iOS+WASM CI gates to primary workflow
[ ] Capture iOS power settings runtime evidence for beta gate
[ ] Revalidate all docs/CURRENT_STATE.md fields after code changes above
```

---

## LOC Estimate for Open Code Items

| Item | Est. LOC |
|---|---|
| onPeerIdentified identify storm fix (Android) | 30–50 |
| iOS BLE delayed identity refresh reads | 30–50 |
| Version bump | 2 |
| .gitignore + remove android_log.txt | 1 |
| **Total code** | **~63–103 LOC** |

---

## Definition of Done

v0.1.2-alpha ships when:
1. All four P0 code items above are merged and passing CI.
2. Live-device validation matrix (items 5–7) is complete with no P0 failures.
3. Two real users can exchange messages across different networks with relay fallback, keeping identity/contacts/history through an app upgrade.

---

## Canonical Reference Documents

- `docs/CURRENT_STATE.md` — Verified state baseline (last: 2026-02-24)
- `REMAINING_WORK_TRACKING.md` — Active backlog
- `APP_VERSION_0.1.2_ALPHA_PLAN.md` — Release intent and scope
- `docs/PARTNER_TEST_PLAYBOOK_V0.1.2_ALPHA.md` — Partner validation scenarios
- `docs/RELEASE_NOTES_V0.1.2_ALPHA.md` — Release notes draft
