# _QUEUE -- Dispatch Order for the Full v1.0.0 Backlog

Status: Active
Last updated: 2026-07-09 (Qwen session audited, emulator validated, P1-17 waived. NEXT_ITER_04 ready for emulator retest.)
Owner: the acting orchestrator (any mode). Sequencing authority:
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` (operator-settled two-phase DAG -- do not
relitigate). This file is the LIVE pick list: pull from the top, respect the
dependency notes, update statuses as tasks move to done/.

Rules of engagement:
- Phase 1 (Windows/Android transport parity) fully drains before any Phase 2
  fine-planning. Phase 2 tasks (PQC_*, TASK_KMP_*, WS-A/B items) are NOT
  dispatchable until P1-19 exit review passes -- treat them as frozen.
- [DEVICE] tasks now run on the Android emulator (operator's phone is broken,
  see memory `project_verification_via_emulator.md`). Orchestrator drives
  emulator-based tests; the operator's Pixel can be substituted if available.
- Anything touching core/src/{crypto,transport,routing,privacy} carries the
  mandatory adversarial-review gate before done.
- Tier tags ([HAIKU]/[SONNET]/[OPUS+]) come from the execution plan. For
  /scmqwen dispatch: [HAIKU]->[FLASH], [SONNET]->[CODER], [OPUS+]->[THINK/MAX].
- Orchestrator modes available: `/scmorc` (Claude native), `/scmqwen`
  (Qwen/DashScope, zero Anthropic cost, round-robin model selection).

## NOW -- Active Phase 1 items (ordered by dependency)

1. **Emulator validation** [INFRA] **COMPLETE 2026-07-09** — AVD `scm_pixel_34` (API 34, Google APIs, x86_64, Pixel 6a) boots with `-gpu swiftshader_indirect -no-audio -no-boot-anim`. `adb devices` shows `emulator-5554 device`. `adb shell getprop sys.boot_completed` returns `1`. Ready for APK install and NEXT_ITER_04 test matrix.

2. `ESC_ANDROID_DNS_RESOLVER_FIX.md` [ESCALATION]
   Hickory DNS resolver crashes on Android emulator due to missing /etc/resolv.conf.
   **Priority: P0 -- BLOCKS emulator execution. Requires validation/planning before implementation.**

3. `NEXT_ITER_04_Live_Device_Retest_Pairing.md` [DEVICE][OPUS+ driving]
   Subsumes the P1-04 rebuild-and-retest arm and the P1-09 LAN E2E
   validation pass. Can now run against the emulator once item 2 lands.
   **Priority: P0 -- this is the Phase 1 gating validation.**

~~3. `P1-17_Windows_WiFi_Direct_Legacy_Client.md` [SONNET][AUDIT-GATE][DEVICE]~~ WAIVED (Emulator HW restriction)
~~   + `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md` (same cell).~~
~~   **[HUMAN-gated]:** operator must decide build-vs-waive per design note~~
~~   Section 2. If waived, narrow matrix cell to Android<->Android~~
~~   [BLOCKED-HW] and close this ticket.~~
**WAIVER CONFIRMED 2026-07-09**: WiFi Direct cell narrowed to Android<->Android [BLOCKED-HW]. Android<->Windows equivalent covered by mDNS/LAN + TCP. Deferred to v1.1.

4. `P1-14 Hostile-network validation` [DEVICE] -- firewall profiles on
   Windows, test the adaptive port ladder. After emulator validates
   baseline LAN connectivity (item 2).

5. P1-18 Relay cells (LAN custody 3-node, then [HUMAN] WAN endpoint
   decision) [DEVICE]. Needs a second CLI instance on the Windows box
   in relay mode, then phone offline-then-returning custody test.

6. P1-19 Phase 1 exit review [OPUS+][HUMAN] -- release-gatekeeper over
   the whole phase; operator signs waivers. GATE TO PHASE 2.

## Phase 1 -- COMPLETED (swarm/native since 07-06)

~~P1-01 Fix swarm.rs test-module imports~~ DONE
~~P1-02 desktop_bridge ble cfg gate~~ DONE
~~P1-03 Working-tree triage~~ DONE (operator)
~~P1-04 Transport negotiation failure~~ DONE (investigation; rebuild-arm
  covered by NEXT_ITER_04 retest; root cause was artifact skew)
~~P1-05 Build-provenance stamps~~ DONE (Qwen swarm)
~~P1-06 mDNS self-loopback filter~~ DONE (Qwen swarm)
~~P1-07 LAN peers feed MeshRepository~~ DONE (Qwen swarm)
~~P1-08 ANR BatteryReceiver synchronous FFI~~ DONE (Fable sprint)
~~P1-09 LAN E2E validation pass~~ Covered by NEXT_ITER_04 retest
~~P1-10 Port-strategy design note~~ DONE
~~P1-11 Listen-side adaptive port~~ DONE (swarm, commit 81d0e909)
~~P1-12 Advertise/Dial/Remember adaptive ports~~ DONE (swarm, commit 8ce54e73)
~~P1-13 Hardcode sweep retire 9001/9002/9010~~ DONE (swarm, commit 1138611b)
~~P1-15 Transport-matrix ground-truth audit~~ DONE
~~P1-16 BLE TX path (CLI outbound)~~ DONE (commit c8b7a2f8, passed
  adversarial audit from qwen3-235b-a22b-thinking-2507)
~~P1-16 BLE MAC rotation~~ DONE (swarm, commit e90b7f6e)
~~P1_CORE_Rate_Limited_Negotiation_Failure_Signal~~ DONE
~~NEXT_ITER_01 Compile gates + test triage~~ DONE
~~NEXT_ITER_02 Adversarial review sprint diff~~ DONE
~~NEXT_ITER_03 Docs sync + residual debt~~ DONE
~~CLIPPY_DEBT_cli_desktop_bridge_dwarnings~~ DONE (commit dd52e75c)
~~Adaptive TTL test fix~~ DONE
~~TASK_INFRA_CLAUDE_PATH_FIX~~ DONE (C:\Users\SCM\.local\bin added to User PATH)

## Phase 1 filler lane (independent, idle capacity only)

- `P1_ANDROID_FABLE_5_DISCOVERY_REPORT.md` is EVIDENCE, not a task --
  move to docs/historical/ at P1 exit.
- All `[VALIDATED]_*` items in todo/ (~55 files) are historical records
  from prior sessions that were validated-complete but never moved to
  done/. Batch-move to `HANDOFF/done/[VALIDATED]/` for a clean signal.

## Phase 2 -- FROZEN until P1-19 passes

PQC_00..14 (master plan `PQC_00_MASTER_PLAN.md`), TASK_KMP_*, WS-A
release-readiness T/S items, WS-B crypto/transport hygiene trio (backup.rs KDF
gap, WiFi Aware orphan, escalation-authority consolidation), WS-F close-out.
Fine-planning happens as P2-00 after Phase 1 exit, per the execution plan.

## Open decision points for operator

1. **WiFi Direct scope** -- P1-17 needs build-vs-waive decision.
2. **Internet relay live proof** -- need a public endpoint (home-router
   port-forward to CLI relay, or waive WAN-live with LAN-relay evidence).
3. **Second Android device** -- WiFi Aware cell is [BLOCKED-HW] with one
   phone. Acquire or waive.
4. **WSL2 for KMP Linux validation** -- accepted with BlueZ caveat, or
   name real Linux hardware later.
