# SCMessenger Development Audit — Issues List

## Audit Summary

- **Repository:** Treystu/SCMessenger (branch `main`, HEAD `5faa751`)
- **Time window reviewed:** 2026-07-06 20:00 through 2026-07-08 21:00 (HST, UTC-10), i.e. the "past 24–48h" per the task.
- **Volume:** 50 commits (44 code/chore-tagged), 131 unique files touched. Sole author: `Treystu` (a mix of `native:`-tagged Claude Code sessions and `swarm:`-tagged automated-swarm completions).
- **What the work did:** The bulk of the window advanced v1.0.0 Phase 1 "transport parity" — adaptive port selection (listen-side P1-11, dial/advertise/remember P1-12, hardcode sweep P1-13), mDNS self-loopback fixes, build-provenance stamps, BLE MAC-rotation and BLE-TX path work, plus Android QR identity-export performance fixes. A large fraction of commits are HANDOFF bookkeeping (design plans added, todo docs closed) rather than code.
- **Issues found:** **1 High, 5 Medium, 4 Low** (10 total).

The single most important finding: the P1-12 "Advertise, Dial, **Remember**" feature ships with a hardcoded placeholder network fingerprint, which silently defeats the per-network correctness of the remembered-port cache.

---

## Issues

### 1. Network fingerprint is a hardcoded placeholder — defeats P1-12 "remember" correctness
- **Severity:** High
- **Category:** Implementation Gap
- **Location:** `core/src/store/transport_memory.rs:65-69` (`get_network_fingerprint`); consumers at `core/src/transport/swarm.rs:3816` (record) and the `Dial` handler (~`swarm.rs:4285+`, `get_last_good`).
- **Problem:** `get_network_fingerprint()` unconditionally returns the constant `"placeholder_network_fingerprint"`. The transport-memory key is `format!("tmem:{}:{}", peer_id, network_fingerprint)` (`transport_memory.rs:24`), so every network collapses to one bucket. The whole point of P1-12 ("remember the last-good port *for this peer on this network*") is nullified: a port learned while on home WiFi is replayed as the top dial candidate when the device later joins a cellular/foreign LAN where that peer is unreachable or the port maps to a different host.
- **Impact:** Wrong-network dial candidates are tried first on every reconnect, adding latency to the <500ms fallback budget and, worst case, dialing a stale port on a NAT where it now belongs to an unrelated host. The committed feature is functionally incomplete despite being marked "completed."
- **Suggested fix:** Implement the fingerprint the comment already specifies — hash of active interface MAC + subnet /24 (fall back to gateway MAC / SSID hash on mobile). Gate the cache read on a real fingerprint; until then, treat `get_last_good` as advisory only and do not promote it above the common-port ladder.

### 2. Transport-memory entries never expire and the store grows unbounded
- **Severity:** Medium
- **Category:** Optimization / Implementation Gap
- **Location:** `core/src/store/transport_memory.rs:44-63` (`record_success` writes `last_success_unix`; `get_last_good` ignores it).
- **Problem:** `TransportMemoryEntry.last_success_unix` is recorded but never read. `get_last_good` returns any stored entry regardless of age, and there is no pruning/TTL — one sled key accumulates per `(peer_id × network_fingerprint)` forever. (The unbounded growth is currently masked by issue #1 collapsing all networks to one key, but will surface the moment #1 is fixed.)
- **Impact:** Stale ports (peer rebinding, DHCP lease change) are dialed first indefinitely; sled key count grows without bound on long-lived nodes that meet many peers.
- **Suggested fix:** In `get_last_good`, reject entries older than a freshness window (e.g. 7 days) and return `None`. Add a periodic prune, or a bounded LRU, keyed off `last_success_unix`.

### 3. `ladder_rank` is a dead field (always written as 0, never consumed)
- **Severity:** Low
- **Category:** Implementation Gap
- **Location:** `core/src/transport/swarm.rs:3816` (call passes literal `0`); field defined `transport_memory.rs:12`.
- **Problem:** `ladder_rank` is persisted as a constant `0` and never used to reorder dial candidates. It reads as scaffolding for an intended "which rung of the port ladder succeeded" ranking that was never wired.
- **Impact:** Misleading data model; implies ranking behavior that does not exist.
- **Suggested fix:** Either populate it with the real rung index at record time and use it to order candidates in the `Dial` handler, or remove the field until the ranking logic exists.

### 4. Hardcoded dial port ladder `[443, 80, 8080]` contradicts the same window's P1-13 hardcode sweep
- **Severity:** Medium
- **Category:** Code Quality
- **Location:** `core/src/transport/swarm.rs` `Dial` handler (candidate synthesis loop, `for port in [443, 80, 8080]`, ~`swarm.rs:4330`).
- **Problem:** The dial-candidate ladder is a hardcoded literal that duplicates the first three entries of `COMMON_PORTS` (`core/src/transport/multiport.rs:12-17` = `[443, 80, 8080, 9090]`) but drops `9090` and does not reference the constant. Commit `1138611` ("P1-13 Hardcode Sweep — Retire 9001/9002/9010") landed in this very window with the goal of eliminating hardcoded ports.
- **Impact:** Listen side (`COMMON_PORTS`) and dial side will silently diverge whenever the constant changes; `9090` listeners are never dialed by the ladder. Exactly the drift P1-13 aimed to remove.
- **Suggested fix:** Iterate `multiport::COMMON_PORTS` in the dial handler instead of a literal array.

### 5. Commit `487945d` claims to complete "BLE GATT traits never implemented" but contains no implementation
- **Severity:** Medium
- **Category:** Code Quality (audit-trail integrity)
- **Location:** commit `487945d` — diff is 10 files, all deletions (9 HANDOFF `todo/*.md` + 1 `Cargo.lock` line); message: `swarm: completed [[NEEDS PLANNING] CORE_SWEEP_03_ble_gatt_traits_never_implemented.md]`.
- **Problem:** The commit adds zero code, and the referenced `ble_gatt` todo file is not even among the deleted files (it deletes the QR/MAC-rotation/smart-router/etc. todos already completed by earlier commits). `core/src/transport/ble/gatt.rs` provides fragmentation/characteristic *abstractions* only; there is no evidence in the diff that the "never implemented" server/client trait binding was addressed.
- **Impact:** The backlog now records CORE_SWEEP_03 as done while the underlying concern (GATT traits not wired to a real BLE backend) appears unresolved. Future planning will skip a real gap.
- **Suggested fix:** Re-open the ticket; either land the actual GATT trait implementation/wiring or re-scope the ticket honestly. Fix the misleading commit message convention so "completed" implies code, not doc deletion.

### 6. No tests for the new `transport_memory` module or the adaptive-port dial logic
- **Severity:** Medium
- **Category:** Testing
- **Location:** `core/src/store/transport_memory.rs` (0 `#[test]`), dial-ladder synthesis in `core/src/transport/swarm.rs`; no references in `core/tests/` or any `*test*` file (`git grep` for `transport_memory`/`get_network_fingerprint` in tests → empty).
- **Problem:** A new persisted store and non-trivial dial-candidate construction (parsing multiaddrs, filtering circuit/ws, dedup) shipped with zero coverage. Round-trip serialization, key formatting, dedup of `candidates`, and the `found_ip && target_peer_id` gating are all untested.
- **Impact:** Regressions in the reconnect path — the most-exercised transport path — would go undetected by CI.
- **Suggested fix:** Add unit tests for `record_success`/`get_last_good` round-trip against a mock `StorageBackend`, and a test asserting the dial handler produces a de-duplicated, correctly-ordered candidate list for a representative direct multiaddr.

### 7. Self-advertised peer entry unconditionally claims `full_relay` + reliability 1.0
- **Severity:** Low
- **Category:** Code Quality
- **Location:** `core/src/transport/peer_broadcast.rs` `create_peer_list_response` (self-injection block added in `8ce54e7`, ~lines 118-127).
- **Problem:** When injecting the local peer into the peer-list response, it hardcodes `reliability_score: 1.0` and `capabilities: RelayCapability::full_relay()` regardless of whether this node is actually configured/able to relay.
- **Impact:** Peers may select this node as a full relay when it is not, degrading relay selection quality.
- **Suggested fix:** Populate capabilities from the node's real relay config and use a neutral/self-unknown reliability rather than a perfect score.

### 8. `preferred_port` can duplicate a common port, producing redundant bind attempts
- **Severity:** Low
- **Category:** Optimization
- **Location:** `core/src/transport/multiport.rs:78-92` (`generate_listen_addresses` — `preferred_port` block then `COMMON_PORTS` loop, no dedup).
- **Problem:** If `preferred_port` equals a value in `COMMON_PORTS` (very likely, since the preferred port is often the p2p port which may be a common port), `add_port` is invoked twice for it, emitting duplicate listen multiaddrs.
- **Impact:** The second bind to the already-bound port fails as "in use" and is logged as a warning on every startup — noise plus wasted syscalls.
- **Suggested fix:** Track emitted ports in a `HashSet<u16>` inside `add_port` and skip duplicates.

### 9. Descriptive multiaddr panic messages downgraded to bare `.unwrap()`
- **Severity:** Low
- **Category:** Code Quality
- **Location:** `core/src/transport/multiport.rs:65-75` (`generate_listen_addresses`).
- **Problem:** `8ce54e7`/`81d0e90` replaced `.parse().expect("Valid multiaddr")` with `.parse().unwrap()` while adding the `/ws` variants. The inputs are constant format strings so a panic is not reachable today, but the diagnostic message was lost.
- **Impact:** If a future edit introduces a malformed format string, the panic will be an opaque `unwrap()` with no context.
- **Suggested fix:** Restore `.expect("valid constant multiaddr")` on each parse.

### 10. Android QR data recomputed on every `identityInfo` emission
- **Severity:** Low
- **Category:** Optimization
- **Location:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/IdentityViewModel.kt` (init block `_identityInfo.collect { … _qrCodeData.value = getQrCodeData() }`, added in `d4f8a00`).
- **Problem:** The QR export string is regenerated on *every* `identityInfo` emission, not only on meaningful identity changes. `getQrCodeData()` calls back into the Rust core (`getIdentityExportString`). If `identityInfo` re-emits on unrelated refreshes (service-state polling, address updates), the potentially expensive export runs repeatedly.
- **Impact:** Redundant core round-trips; partially re-introduces the very slowness P1_ANDROID_QR aimed to fix, on the collect path rather than the render path.
- **Suggested fix:** `distinctUntilChanged` on the identity key (e.g. `libp2p_peer_id`/`initialized`) before regenerating, so QR data is recomputed only when the identity actually changes.

---

*Methodology note:* the repo was delivered as a single-commit shallow sparse clone; history was recovered with `git fetch --unshallow` and file contents inspected via `git show HEAD:<path>` (working tree is sparse/unmaterialized). All line numbers are from `HEAD` (`5faa751`). No GitHub issues were opened.
