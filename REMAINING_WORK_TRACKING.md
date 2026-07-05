# SCMessenger Remaining Work Tracking

Status: Active
Last updated: 2026-07-05 (Phase 1 Stage A compile-gate fixes landed; P1-15 transport-matrix
audit and P1-10 adaptive-port design complete — see "Phase 1 Stage A/C/D progress" note below.
Windows/Android parity effort remains top priority; full v1.0.0 sequencing governed by
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` — two-phase DAG, Phase 1 = parity effort + adaptive ports,
Phase 2 = all remaining ship-blocking work)

---

## 2026-07-04 WINDOWS <-> ANDROID PARITY EFFORT (TOP PRIORITY, operator-directed)

**Status:** OPEN — this is the current #1 priority. Everything else in this
file is explicitly deprioritized until this effort's blockers clear, except
work already in flight from another session/agent (do not interrupt that;
simply do not dispatch *new* non-parity work from native `/scm` sessions).

**Why:** A live interop test on 2026-07-04 between a Windows CLI daemon
(`192.168.0.121`) and a physical Pixel 6a (`192.168.0.148`, live-connected
via adb over WiFi at time of writing) on the same LAN surfaced multiple real
bugs preventing the two clients from reliably working together. Operator
directive: fix these before any other feature/workstream, and use this same
test as the "networking fundamentals" sanity check — it already found real
gaps, not hypothetical ones.

**Priority order (dependency-aware):**

0. **CLOSED 2026-07-05 — `HANDOFF/done/P1_ANDROID_TransportManager_LAN_Discovery_Never_Starts.md`**
   Root cause confirmed and fixed: `stopMeshService()` nulls `transportManager`
   to release BLE/WiFi Aware cleanly, but nothing recreated it on a later start
   (unlike BLE components, which self-heal) — any stop, including an internal
   failure-recovery stop during startup, permanently disabled `MdnsServiceDiscovery`
   + `SubnetProbe` for the rest of the process. Fixed via a lazy
   `ensureTransportManager()` at all 3 call sites (`ef2869b1`).
   `./gradlew assembleDebug` PASS. **Live-device retest not yet done this
   session (budget-constrained) — that's the next session's first move**:
   confirm `MdnsServiceDiscovery`/`SubnetProbe` actually log starting and find
   the CLI's open ports, then re-attempt #1 below.
1. **P0 — `HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`**
   (re-ranked P1->P0 on 2026-07-04; superseded in immediate priority by #0 above
   as of 2026-07-05 — retesting this before #0 lands will just reproduce the same
   silent-discovery stall, see that ticket's 2026-07-05 progress note). Windows
   CLI fails `Failed to negotiate transport protocol(s)` on both raw-TCP and WS
   inbound dials from the Android device — originally documented 2026-07-04 when
   mDNS discovery reportedly succeeded first; not reproduced in the 2026-07-05
   session because discovery itself never completed that run. Touches
   `core/src/transport/` -> mandatory `crypto-security-auditor` review before done.
2. **P0 — `HANDOFF/todo/P0_ANDROID_ANR_BatteryReceiver_Synchronous_FFI_Call.md`**
   Reproducible ANR (app killed/relaunched) from a synchronous FFI call on
   the main thread in a battery-change BroadcastReceiver. Independent of
   #1 (Kotlin-only fix), can run in parallel.
3. **P1 — `HANDOFF/todo/P1_ANDROID_mDNS_Self_Loopback_Discovery.md`**
   Phone's mDNS resolves its own broadcast as a "peer." Must land before
   #4 per that ticket's own acceptance criteria (self-loopback contaminates
   the peer-count signal #4 needs to verify against).
4. **P1 — `HANDOFF/todo/P1_ANDROID_LAN_Discovery_Not_Feeding_Bootstrap_Peer_Count.md`**
   LAN-resolved peers don't visibly increment `MeshRepository`'s
   `peersDiscovered` stat. Depends on #3 landing first (needs a real,
   non-self peer to test against).
5. **P2 — `HANDOFF/todo/P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity.md`**
   BLE MAC rotation (Android privacy feature, ~15min interval) forces the
   Windows CLI's `ble_mesh` to treat the same phone as a new peripheral
   every rotation. Continuity/robustness issue, not a hard connectivity
   blocker like 1-2. Touches `core/src/transport/ble/` -> mandatory
   `crypto-security-auditor` review (DarkBLE rotation material has direct
   privacy implications).

**Compile-gate note (2026-07-04, CLOSED 2026-07-05):** ground-truth
`cargo build --workspace` run (`HANDOFF/done/P0_COMPILE_GATE_VERIFICATION.md`)
found 2 real, independent compile bugs, tracked as their own P0 follow-ups.
**Both fixed and gate-verified 2026-07-05** (native `/scm` session, P1-01/P1-02
in `HANDOFF/V1_0_0_EXECUTION_PLAN.md`): `cargo build --workspace`, `cargo test
--workspace --no-run`, and `cargo test -p scmessenger-core --lib` all pass.
One unrelated pre-existing failure surfaced (`transport::bootstrap::tests::
test_bootstrap_manager_creation` — `discover_hardcoded_backup_relays()`
returns an empty list, so `BootstrapManager::with_defaults()` seeds zero
nodes) — not a regression from these fixes, flagged separately as its own
follow-up. Ticket files moved to `HANDOFF/done/`.

**Phase 1 Stage A/C/D progress (2026-07-05, native `/scm` session):**
- P1-01, P1-02: DONE (see compile-gate note above).
- **P1-15** (transport-matrix ground-truth audit): DONE. Full findings in
  `HANDOFF/plans/P1-15_transport_matrix_audit.md`. Two genuine implementation
  gaps surfaced — CLI has no BLE TX path (Android->CLI BLE works, CLI->Android
  does not), and Windows has no WiFi Direct implementation at all (Android-only
  today). New tickets: `P1_CLI_BLE_Outbound_TX_Path_Missing.md`,
  `P1_CORE_BLE_GATT_Traits_Dead_And_Malformed_UUID.md` (also fixes a malformed
  `GATT_SERVICE_UUID` constant, currently unused so harmless),
  `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md`. The release-readiness T12c
  finding (WiFi Aware `send()` hardcoded `false`) is **stale** — it's a
  deliberate documented no-op, not a missing write path; ledger-only fix in
  `P1_DOCS_WiFi_Aware_T12c_Ledger_Correction.md`.
- **P1-10** (adaptive port selection design): DONE. Spec at
  `HANDOFF/plans/P1-10_adaptive_port_selection_design.md`, decomposed into
  `HANDOFF/todo/P1-11_Listen_Side_Adaptive_Port_Selection.md`,
  `P1-12_Advertise_Dial_Remember_Adaptive_Port_Selection.md`,
  `P1-13_Hardcode_Sweep_Retire_9001_9002_9010.md`. **Flags 3 items requiring
  operator sign-off before P1-11/12 implement them** (peer_exchange
  self-address semantics change, a new `GroupInfo.port` field crossing the
  Kotlin FFI boundary, and a new sled `transport_memory` schema including a
  privacy-adjacent network-fingerprint definition) — see spec section 4.
- **Next gating item: P1-04** (root-cause the Android<->Windows transport
  negotiation failure) — [OPUS+][AUDIT-GATE][DEVICE], requires live Pixel 6a
  access. P1-11/12 (adaptive ports) and P1-16/17 (BLE/WiFi Direct gap closure)
  all queue behind it per the plan's `transport/` hotspot-lane rule (§1.4).

**Explicitly deprioritized pending this effort (not cancelled, not to be
newly dispatched from native `/scm` sessions):**
- PQC-01..14 post-quantum migration workstream (below) — note PQC-01
  appears already landed per commit history and may still be actively
  worked by a separate (non-`/scm`) session; do not interrupt that, just
  don't open new PQC task dispatch from here.
- `*_SWEEP_*` dead-code/panic-audit tasks (`ANDROID_SWEEP_*`,
  `CORE_SWEEP_*`) — general robustness hygiene, not parity blockers.
- `TASK_KMP_*` (Kotlin Multiplatform desktop architecture) — separate,
  longer-horizon initiative.
- All standalone P2/P3 iOS/WASM/CLI items in `HANDOFF/todo/`.
- `HANDOFF/IN_PROGRESS/IN_PROGRESS_P0_ANDROID_024_IDENTITY_GENERATION_REGRESSION.md`
  — dated 2026-06-05 (~1 month stale), marked OPEN but the 2026-07-04 live
  test session got through onboarding into a working foreground mesh
  service with no identity-generation symptoms reported. Needs a quick
  re-verification pass (not full triage) before either closing as stale or
  re-activating — flagged here so it isn't silently lost, not treated as
  an active parity blocker right now.

---

## 2026-07-03 POST-QUANTUM MIGRATION WORKSTREAM (PQC-01..14)

**Status:** OPEN — task files staged in `HANDOFF/todo/`, human-approved for implementation
**Reference:** `docs/QUANTUM_READINESS_AUDIT.md` (verdict: not quantum-proof; all asymmetric crypto is Curve25519) and `HANDOFF/todo/PQC_00_MASTER_PLAN.md` (dependency graph, suite registry, global rules, standard gates)

Goal: hybrid X25519+ML-KEM-768 for all new-session confidentiality (closes harvest-now-decrypt-later), Ed25519+ML-DSA-65 dual signatures for identity operations. Symmetric layer (XChaCha20-Poly1305 / Blake3 / Argon2id) is already quantum-safe — unchanged.

| Wave | Tasks | Notes |
|------|-------|-------|
| 0 | PQC-01 (ML-KEM dep), PQC-02 (Envelope v2), PQC-03 (Identity v2 bundle) | Parallelizable; PQC-03 needs PQC-01 |
| 1 | PQC-04 (suite negotiation), PQC-05 (hybrid KEM module) | PQC-05 requires adversarial review |
| 2 | PQC-06 (hybrid session init) | Adversarial review |
| 3 | PQC-07 (PQ ratchet — Sonnet-tier only), PQC-09 (hybrid onion), PQC-10 (ML-DSA) | PQC-07 is highest-risk; auditor + gatekeeper |
| 4 | PQC-08 (legacy path retirement), PQC-11 (relay/invite dual-sig), PQC-12 (TLS PQ groups) | |
| 5 | PQC-13 (Kani/proptest/cross-version matrix), PQC-14 (docs + risk register closure) | Workstream exit gates |

Standing rules for all PQC tasks: hybrid never pure; never remove legacy decrypt/verify paths; bincode format-tag discipline for any wire/sled struct change; adversarial review for `crypto/`/`privacy/` changes per `.claude/rules/security.md`. Per-task Definition of Done includes the standard build gates and moving the task file to `HANDOFF/done/`.

---

## 2026-07-02 V1.0.0 RELEASE READINESS ASSESSMENT

**Status:** IN PROGRESS
**Reference:** `docs/release-readiness-2026-07-02.md`

Based on the latest PR merge (`cbec1f4`), the following tasks are the final remaining items for v1.0.0 perfect code:

### Human-only / Infrastructure Blockers
- **H1:** Restore GitHub Actions runners (Runners failing immediately without logs due to billing/quota issues). This blocks all CI validation.
- **H2:** Physical-device procedures (WiFi Aware/Direct, BLE tests, DTN mule test). Requires hardware.

### Completed Code & Script Fixes (Verified 2026-07-02)
- [OK] **S-Tasks (S2-S8):** All core automation and script tasks have been resolved in the codebase.
- [OK] **T-Tasks (T1-T17):** All Rust, CLI, Android, and iOS codebase bug fixes have been completed and merged.

*Note: S9 (Cross-platform workflow validation) is still pending, blocked by H1.*

---

For historical entries prior to 2026-07-02, see docs/historical/REMAINING_WORK_TRACKING_ARCHIVE_2026.md
