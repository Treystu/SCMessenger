# SCMessenger v0.1.2 Alpha Release Audit

**Audited:** 2026-02-26 | **Updated:** 2026-02-26
**Branch:** `claude/audit-alpha-release-plan-bq4sA` (merged from `origin/main` ce2a1ad)
**Method:** Full codebase audit — documentation review, code inspection, runtime log analysis (android_log.txt)

---

## Executive Summary

The codebase is in strong shape for alpha. The core Rust library is feature-complete with 274 passing unit tests, zero clippy warnings, clean `cargo fmt`, and all four documented hardening issues resolved. Both Android and iOS apps build and install. WASM swarm transport is implemented. All P0 code items are now resolved — remaining gates are live-device validation and the final version bump after testing passes.

**Update (post-audit, 2026-02-26):** All P0 code items are now complete:
- Items 1/4 from original audit fixed in main (ce2a1ad)
- Item 6 (message persistence) fixed in this branch
- iOS BLE delayed identity retry fixed in this branch (2c4436d)
- WASM `NatStatusChanged` match arm added + workspace `cargo fmt` applied

---

## Verified Stable (No Changes Needed)

| Area | Status |
|---|---|
| `cargo test --workspace --lib` | 274 passed, 0 failed, 7 ignored |
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
| BLE GATT sequential queue | All reads/writes/CCCD ops serialized per-device (Android + iOS) |
| iOS BLE delayed identity retry | T+900ms + T+2200ms re-reads in `BLECentralManager.swift` (mirrors Android) |
| Schema v2 migration | Identity/outbox/inbox sub-store layout with SCHEMA_VERSION guard |
| Relay PeerId stability | Persisted `relay_network_key.pb`, migrated from IronCore identity |
| Connection path state | `ConnectionPathState` + `export_diagnostics()` exposed in UniFFI + WASM |
| CI tri-platform gates | `check-android` + `check-ios` + `check-wasm` in `.github/workflows/ci.yml` |
| Message/contact persistence | iOS `isExcludedFromBackup` scoped to `identity/` only; history.db + contacts.db backed up |
| Reinstall detection | Post-start identity beacon on both platforms when reinstall with existing identity detected |

---

## Resolved Since Initial Audit

| # | Item | Resolution |
|---|---|---|
| 1 | `onPeerIdentified` identify storm (Android + iOS) | **Fixed in main** (ce2a1ad): 30s dedup cache on both platforms; 1s dedup on disconnect; 5-min dedup on dial-throttle log |
| 4 | `android_log.txt` in version control | **Fixed in this branch** (925f6e3): removed from tracking, added `*_log.txt` + `*_logs.txt` to `.gitignore` |
| 6 | Message history wiped on reinstall | **Fixed in this branch**: iOS backup exclusion moved from whole `mesh/` dir to `identity/` subdir only; Android backup rules already correct; reinstall detection + post-start beacon added to both platforms |
| 7 | iOS BLE missing delayed identity retry reads | **Fixed in this branch** (2c4436d): `scheduleIdentityRefreshReads()` added to `BLECentralManager.swift`, mirrors Android `IDENTITY_REFRESH_DELAYS_MS = [900, 2200]` |
| 8 | CI tri-platform gate alignment | **Already present**: `check-android`, `check-ios`, `check-wasm` jobs confirmed in `.github/workflows/ci.yml` |
| 9 | WASM `NatStatusChanged` unhandled match arm | **Fixed in this branch**: added `NatStatusChanged(_) => {}` arm in `wasm/src/lib.rs`; workspace formatted with `cargo fmt --all` |

---

## Open Items

### P0 — Validation Required (No Code Changes)

All code is complete. These items require live-device or live-network verification to close the alpha gate.

#### 1. Live network matrix validation

**Status:** Code complete. Validation evidence pending.

**Required runs:**
- GCP bootstrap connect from non-LAN device (cellular) — both Android and iOS
- Direct P2P probe with no relay (both on same LAN)
- Forced relay-only delivery (direct path blocked)
- Network switch mid-send (Wi-Fi → cellular) with no message loss
- Cross-version: `v0.1.2-alpha` browser ↔ native (per `docs/PARTNER_TEST_PLAYBOOK_V0.1.2_ALPHA.md`)

**Acceptance:** No message loss, no duplicates, relay fallback succeeds within 30s of direct path failure.

---

#### 2. ACK-safe path switching validation

**Status:** Code complete (stable UUIDs, idempotent receive apply, ACK reconciliation implemented).

**Required:** Induce a network-path switch (e.g., disable Wi-Fi while a message is in-flight) and verify:
- Message delivered exactly once
- No stuck "Sending..." state
- Receipt arrives on sender side

---

#### 3. App-update + reinstall continuity validation on real devices

**Status:** Code complete. iOS backup fix and reinstall detection landed in this branch. Android backup rules already correct.

**Required:** On a real Android device and iPhone:
1. Install the current build with existing identity, contacts, and chat history.
2. Fully uninstall and reinstall.
3. Verify: identity restored from Keychain/SharedPreferences, contacts + history restored via iCloud/Android Auto Backup.
4. Also test in-place update (install new build over existing): verify no data loss.

---

### P0 — Final Step (After Testing Passes)

#### 4. Version bump to 0.1.2

**Current:** `Cargo.toml` workspace `version = "0.1.1"`. `core/src/transport/behaviour.rs` agent version `"scmessenger/0.1.1/..."`.

**Required:** Both must be `0.1.2` — do this as the FINAL commit after all validation passes.

**Files:**
- `Cargo.toml` (workspace `[workspace.package].version`)
- `core/src/transport/behaviour.rs` (agent version string)

After version bump: rebuild and redeploy GCP relay node via `scripts/deploy_gcp_node.sh`.

---

### P1 — Important, Non-Blocking

#### 5. GCP relay server rebuild after version bump

After item 4 is merged, rebuild and redeploy the relay node so it reports `scmessenger/0.1.2/headless/relay/...` in identify messages.

**Current state:** Relay reports `scmessenger/0.1.0/headless/relay/...`.

Use `scripts/deploy_gcp_node.sh` for the deployment.

---

#### 6. iOS power settings runtime evidence

**Status:** Code is complete — `setAutoAdjustEnabled`, `applyPowerAdjustments`, and profile-application logs are all wired in `MeshRepository`.

**Remaining:** Capture device evidence from a real iPhone confirming power profile transitions (Standard → Low → Standard) under real battery/motion/network changes. Not blocking alpha, but needed for beta gate sign-off.

---

### Deferred to Beta/Post-Alpha

These items are explicitly out of scope for v0.1.2-alpha.

| Item | Policy | Notes |
|---|---|---|
| Anti-abuse controls | Mandatory before **beta** | Not blocking alpha |
| WASM WebRTC answerer + ICE trickle | ~150 LOC | WebSocket is primary path; WebRTC is experimental beta track |
| WASM IndexedDB persistence | Beta parity | `localStorage` fallback works for alpha |
| Auto-detect and resume on startup | Roadmap 1.0.0 | Not committed for alpha |
| Reset All Data / Manual Data Management | Roadmap 1.0.0 | Not committed for alpha |
| i18n beyond English | Roadmap post-1.0.0 | Alpha is English-only by policy |
| iOS historical artifact cleanup | Hygiene | `iOS/iosdesign.md` and `build_*.txt` can wait |

---

## Alpha Release Checklist

```
RESOLVED (all code complete):
[x] Fix onPeerIdentified identify storm (Android + iOS) — main ce2a1ad
[x] Remove android_log.txt from repo — this branch
[x] Fix iOS message/contact persistence: isExcludedFromBackup scoped to identity/ only — this branch
[x] Add reinstall detection + post-start recovery beacon (Android + iOS) — this branch
[x] iOS BLE delayed identity retry reads (T+900ms, T+2200ms) — this branch (2c4436d)
[x] CI tri-platform gates (Android + iOS + WASM) — already present in ci.yml
[x] WASM NatStatusChanged match arm — this branch
[x] cargo fmt clean across workspace — this branch

VALIDATION REQUIRED (no code changes):
[ ] Live network validation: GCP + direct P2P + relay fallback (Android + iOS)
[ ] ACK-safe path switching validation (both platforms)
[ ] App-update + reinstall continuity validation on real devices

FINAL STEP (do LAST after testing passes):
[ ] Version bump: Cargo.toml + behaviour.rs → 0.1.2
[ ] Rebuild and redeploy GCP relay after version bump

SHOULD DO SHORTLY AFTER ALPHA:
[ ] Capture iOS power settings runtime evidence for beta gate
```

---

## LOC Summary for Code Changes in This Branch

| Item | Est. LOC | Status |
|---|---|---|
| onPeerIdentified identify storm fix (Android + iOS) | 30–50 | ✅ Done (main ce2a1ad) |
| iOS backup exclusion fix (contacts + history) | 50–70 | ✅ Done (this branch) |
| Reinstall detection + post-start beacon (iOS + Android) | 40–60 | ✅ Done (this branch) |
| .gitignore + remove android_log.txt | 1 | ✅ Done (this branch) |
| iOS BLE delayed identity refresh reads | ~15 LOC | ✅ Done (this branch, 2c4436d) |
| WASM NatStatusChanged match arm + cargo fmt | ~3 LOC | ✅ Done (this branch) |
| Version bump (after testing) | 2 | ⏳ Deferred — do after validation |

---

## Definition of Done

v0.1.2-alpha ships when:
1. All P0 code items are merged — **COMPLETE**.
2. Live-device validation matrix (items 1–3) is complete with no P0 failures.
3. Version bump to 0.1.2 applied as the final commit before tagging.
4. Two real users can exchange messages across different networks with relay fallback, keeping identity/contacts/history through an app upgrade.

---

## Canonical Reference Documents

- `docs/CURRENT_STATE.md` — Verified state baseline (last: 2026-02-24)
- `REMAINING_WORK_TRACKING.md` — Active backlog
- `APP_VERSION_0.1.2_ALPHA_PLAN.md` — Release intent and scope
- `docs/PARTNER_TEST_PLAYBOOK_V0.1.2_ALPHA.md` — Partner validation scenarios
- `docs/RELEASE_NOTES_V0.1.2_ALPHA.md` — Release notes draft
