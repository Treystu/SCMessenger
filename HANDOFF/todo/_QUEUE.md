# _QUEUE — Dispatch Order for the Full v1.0.0 Backlog

Status: Active
Last updated: 2026-07-06 (post Fable 5 stabilization sprint)
Owner: the acting orchestrator (any mode). Sequencing authority:
`HANDOFF/V1_0_0_EXECUTION_PLAN.md` (operator-settled two-phase DAG — do not
relitigate). This file is the LIVE pick list: pull from the top, respect the
dependency notes, update statuses as tasks move to done/.

Rules of engagement:
- Phase 1 (Windows/Android transport parity) fully drains before any Phase 2
  fine-planning. Phase 2 tasks (PQC_*, TASK_KMP_*, WS-A/B items) are NOT
  dispatchable until P1-19 exit review passes — treat them as frozen.
- [DEVICE] tasks need the physical Pixel + operator; queue them, prep their
  playbooks, but do not block the lane — work the next non-device item.
- Anything touching core/src/{crypto,transport,routing,privacy} carries the
  mandatory adversarial-review gate before done.
- Tier tags ([HAIKU]/[SONNET]/[OPUS+]) come from the execution plan and map to
  the /scmorc routing table. Escalate effort before model.

## NOW — sprint verification chain (blocks everything else)

1. `NEXT_ITER_01_Compile_Gates_And_Test_Triage.md` [SONNET/high]
   All build gates + Android test triage after the 2026-07-06 Fable sprint.
   Contains the exact 7-failure inventory and quarantine restoration work.
2. `NEXT_ITER_02_Adversarial_Review_Sprint_Diff.md` [FABLE/high, read-only]
   MANDATORY audit of the sprint's transport/crypto diff (commit c76bd897).
   The sprint is not mergeable-quality until this verdict is on file.
   Critical/High findings escalate to the operator/Fable.

## Phase 1 Stage A residue (parallel-safe with the above where lanes differ)

3. `NEXT_ITER_03_Docs_Sync_And_Residual_Debt.md` section A only [HAIKU/low]
   (Section B — runBlocking-getter debt — queues after NEXT_ITER_01.)
~~4. P1-05 Build-provenance stamps [HAIKU/low]~~ (Completed via Qwen swarm)
~~5. IN_PROGRESS/ triage [HAIKU/low, orchestrator-local]~~ (Completed, folder empty)

## Phase 1 Stage B/device — needs operator + Pixel (schedule with operator)

6. `NEXT_ITER_04_Live_Device_Retest_Pairing.md` [DEVICE][OPUS+ driving]
   Subsumes the P1-04 rebuild-and-retest arm (fresh APK from current main via
   install-clean.sh — that alone may close the negotiation-failure story) and
   the P1-09 LAN E2E validation pass. Follow the ticket's test matrix.
7. `P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`
   [OPUS+][AUDIT-GATE][DEVICE] — P1-04 proper. Only if the rebuilt-APK retest
   in item 6 still fails: trace-log capture per ticket, then root-cause fix.
~~8. P1_ANDROID_mDNS_Self_Loopback_Discovery.md [SONNET/medium] — P1-06.~~ (Completed via Qwen swarm)
9. `P1_ANDROID_LAN_Discovery_Not_Feeding_Bootstrap_Peer_Count.md` [SONNET/medium] — P1-07.
10. `P1_ANDROID_Inbound_Libp2p_Listener_Not_Externally_Reachable.md` [SONNET/high]
    — verify against the sprint's listener-zombie fix first; may already be
    resolved (pre-dispatch validation should check for ALREADY_WIRED).

## Phase 1 Stage C — adaptive ports (P1-10 spec is DONE; implementation open)

11. `P1-11_Listen_Side_Adaptive_Port_Selection.md` [SONNET][AUDIT-GATE]
12. `P1-12_Advertise_Dial_Remember_Adaptive_Port_Selection.md` [SONNET][AUDIT-GATE]
13. `P1-13_Hardcode_Sweep_Retire_9001_9002_9010.md` [HAIKU] — after 11/12.
14. P1-14 Hostile-network validation [DEVICE] — after 11-13.

## Phase 1 Stage D — remaining transport cells (P1-15 matrix audit is DONE)

15. `P1_CLI_BLE_Outbound_TX_Path_Missing.md` [SONNET/high][AUDIT-GATE] — feeds P1-16.
16. `P2_ANDROID_BLE_MAC_Rotation_Breaks_Session_Continuity.md` [SONNET][AUDIT-GATE]
    — part of P1-16's worst-case BLE cell.
17. `P1-17_Windows_WiFi_Direct_Legacy_Client.md` [SONNET][AUDIT-GATE][DEVICE]
    + `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md` (same cell).
18. `P1_CORE_Rate_Limited_Negotiation_Failure_Signal.md` [SONNET/medium].
19. P1-18 Relay cells (LAN custody 3-node, then [HUMAN] WAN endpoint decision) [DEVICE].
20. P1-19 Phase 1 exit review [OPUS+/FABLE][HUMAN] — release-gatekeeper over the
    whole phase; operator signs waivers. GATE TO PHASE 2.

## Phase 1 filler lane (independent, any idle capacity; never blocks the DAG)

- `P1_ANDROID_QR_Code_Identity_Export_Extremely_Slow.md` [SONNET/medium]
- `P2_ANDROID_IDENTITY_QR_PRERENDER.md`, `P2_ANDROID_IDENTITY_SCROLL_FIX.md` [SONNET/low-medium]
- `P2_ANDROID_HARDCODED_STRINGS_CONTACTS_SETTINGS.md` +
  `ANDROID_SWEEP_01_hardcoded_strings_contacts_settings.md` [HAIKU/low]
  (check overlap — likely the same work, close one as duplicate)
- `P2_CORE_Bootstrap_Test_Depends_On_Unset_Env_Var.md` [HAIKU/low]
- `P3_*` NEEDS_PLANNING items: pre-dispatch validation only; file findings, don't implement.
- `P1_ANDROID_FABLE_5_DISCOVERY_REPORT.md` is EVIDENCE, not a task — leave in
  todo/ as reference for device sessions, or move to docs/historical/ at P1 exit.

## Remote-eligible lane (Claude Cowork / cloud sandboxes — 2x-usage window)

Tasks safe to farm out to REMOTE SANDBOX workers (AGENTS.md class rules:
deliver branch/patch + UNVERIFIED report; Windows orchestrator runs the real
gates and commits). Best fits, in value order:

- R1. `NEXT_ITER_02_Adversarial_Review_Sprint_Diff.md` — read-only review;
  needs no toolchain at all. Ideal first Cowork task.
- R2. Quarantined-test rework (the fix-authoring half of NEXT_ITER_01's
  quarantine item): rewrite `IdentityViewModelTest.kt` against the current
  ViewModel API and draft the `IdentityCreationFlowTest.kt` androidTest/
  Robolectric decision as a patch. Windows side compiles/executes.
- R3. Test authoring for Phase 1 tickets (P1-06 self-loopback unit test,
  P1-11/12 acceptance tests from the P1-10 spec) — delivered as patches.
- R4. P1-05 build-provenance stamps implementation as a patch (small,
  self-contained Rust; container cargo check is good advisory signal here).
- R5. Pre-dispatch validation sweeps of `P3_*_NEEDS_PLANNING_*` items and the
  IN_PROGRESS/ staleness triage — produce findings reports, no repo mutations.
- R6. `NEXT_ITER_03` section A docs work — delivered as a patch.

NOT remote-eligible: anything [DEVICE]; final AUDIT-GATE sign-off (review
drafts are fine, the gating verdict runs under the orchestrator); any task
whose acceptance is "gate passes on Windows" with no separable authoring half.

## Phase 2 — FROZEN until P1-19 passes

PQC_00..14 (master plan `PQC_00_MASTER_PLAN.md`), TASK_KMP_*, WS-A
release-readiness T/S items, WS-B crypto/transport hygiene trio (backup.rs KDF
gap, WiFi Aware orphan, escalation-authority consolidation), WS-F close-out.
Fine-planning happens as P2-00 after Phase 1 exit, per the execution plan.
