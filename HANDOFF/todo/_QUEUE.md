# _QUEUE -- Dispatch Order for the Full v1.0.0 Backlog

Status: Active
Last updated: 2026-07-11 (backlog groomed for execution: done-files swept out of
todo/, review checkpoint now a dispatchable task, decision points refreshed.
Delegation protocol: docs/ORCHESTRATION.md.)

## EXECUTION ORDER -- next dispatches (pull top-down)

1. `PQC_REVIEW_CHECKPOINT_05_06_07.md` [THINKING][AUDIT-GATE][BLOCKING] --
   read-only Qwen thinking dispatch; command inside the task file. Gates
   everything PQC below.
2. `PQC_08_LEGACY_PATH_RETIREMENT.md` [QWEN max, --verify compile gate] --
   IN PROGRESS, encrypt.rs gating landed; finish remaining scope. Can run
   in parallel with item 1 (already-written code; review covers it).
3. `TASK_CI_IOS_MACOS_RUNNER_FIX.md` [SONNET or QWEN max] -- independent of
   PQC; workflow FIX is dispatchable now, the RUN needs the operator's
   GitHub billing unlock first.
4. After checkpoint PASS: `PQC_09_HYBRID_ONION.md` and
   `PQC_10_MLDSA_IDENTITY_SIGNATURES.md` (parallel lanes, different files);
   then `PQC_11` (after 10) -> `PQC_12` -> `PQC_13` -> `PQC_08` final
   compat pass -> `PQC_14`.
5. `TASK_KMP_RUST_UNIFFI_LINUX.md` [QWEN max] anytime (verify-only start);
   `TASK_KMP_COMPOSE_ARCHITECT.md` is [HUMAN-gated] -- spec targets Compose
   on linuxX64 native but Compose Desktop is JVM-only; operator must settle
   the stack before any dispatch.
6. [NEEDS PLANNING] pair (BLE GATT traits, CLI orphaned modules) --
   [OPUS+/decision] items; hold for a Claude session or operator call.
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

2. `ESC_ANDROID_DNS_RESOLVER_FIX.md` [ESCALATION] **COMPLETE 2026-07-10** — Solved hickory-dns crash on Android by implementing custom DNS fallback configuration (Option A) with Google Public DNS nameservers. Bypasses file system lookup while preserving WAN-dial capability.

3. `NEXT_ITER_04_Live_Device_Retest_Pairing.md` [DEVICE][OPUS+ driving] **COMPLETE 2026-07-10** — Android APK built successfully, deployed to emulator, and paired with Windows CLI daemon over port-forwarding. Full E2E synchronization and messaging verified.

~~3. `P1-17_Windows_WiFi_Direct_Legacy_Client.md` [SONNET][AUDIT-GATE][DEVICE]~~ WAIVED (Emulator HW restriction)
~~   + `P1_CORE_WINDOWS_WIFI_DIRECT_Peer_Absent.md` (same cell).~~
~~   **[HUMAN-gated]:** operator must decide build-vs-waive per design note~~
~~   Section 2. If waived, narrow matrix cell to Android<->Android~~
~~   [BLOCKED-HW] and close this ticket.~~
**WAIVER CONFIRMED 2026-07-09**: WiFi Direct cell narrowed to Android<->Android [BLOCKED-HW]. Android<->Windows equivalent covered by mDNS/LAN + TCP. Deferred to v1.1.

4. `P1-14 Hostile-network validation` [DEVICE] -- **POST-EXIT VERIFICATION
   DEBT**: not individually closed before the P1-19 sign-off. Planned closure:
   AWS docker rig netem/firewall profiles (operator approved AWS 2026-07-11,
   reversing the plan's AWS-excluded note; rig prerequisites in the 2026-07-11
   deploy-infra audit: --http-bind flag + /health route + cloud/mesh compose
   repair).

5. P1-18 Relay cells [DEVICE] -- **POST-EXIT VERIFICATION DEBT**, same
   vehicle: 3-node custody chain + WAN relay live proof on the AWS rig
   (public P2P port vs home Windows CLI + emulator).

6. `P1-19_Phase_1_Exit_Review.md` [OPUS+][HUMAN] **COMPLETE 2026-07-10** -- Checklist for the
   operator's sign-off, listing completed milestones, waivers, and exit check
   steps. GATE TO PHASE 2.

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
~~ESC_ANDROID_DNS_RESOLVER_FIX~~ DONE (Custom DNS fallback on Android target)
~~NEXT_ITER_04_Live_Device_Retest_Pairing~~ DONE (E2E pairing verification PASS)

## Phase 1 filler lane (independent, idle capacity only)

Both filler items CLOSED 2026-07-11: FABLE_5 discovery report moved to
docs/historical/; the [VALIDATED]_* sweep was completed in a prior session.
todo/ now contains only live tasks (verified: 16 files + REJECTED/ + this
queue).

## NOW -- Active Phase 2 items (ordered by dependency)

1. `PQC_02_ENVELOPE_V2.md` [SONNET] **COMPLETE 2026-07-10** -- Envelope v2 wire format with suite tag and PQ fields.

2. `PQC_03_IDENTITY_V2_KEYBUNDLE.md` [SONNET] **COMPLETE 2026-07-10** -- Identity v2 key bundle with ML-KEM-768 public key. All gates pass, migration tested.

3. `PQC_04_SUITE_NEGOTIATION.md` [SONNET] **COMPLETE 2026-07-10** -- Suite negotiation logic for hybrid X25519+ML-KEM-768.

4. `PQC_05_HYBRID_KEM_MODULE.md` [QWEN] **COMPLETE 2026-07-11** -- Hybrid KEM module (libcrux-ml-kem).

5. `PQC_06_HYBRID_SESSION_INIT.md` [QWEN] **COMPLETE 2026-07-11** -- Hybrid Session Establishment.

6. `PQC_07_PQ_RATCHET.md` [QWEN] **COMPLETE 2026-07-11** -- PQ ratchet steps. Compile
   gate green 2026-07-11 (`cargo test --workspace --no-run` exit 0 after api.udl
   `LegacyStaticEcdhSend` fix). All three *_COMPILE_FIX tasks verified applied and
   moved to done/.

7. **[AUDIT-GATE] PQC-05/06/07 adversarial review checkpoint** -- COMPLETE
   2026-07-11. Verdict: FAIL (one CRITICAL, confirmed) at
   HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md. CRITICAL: PQC-07's PQ
   ratchet step is implemented but never called from the live
   encrypt/decrypt path (ML-KEM secret fixed at bootstrap, never refreshed).
   Follow-up: `PQC_07_WIRE_RATCHET_STEP.md` [QWEN max][AUDIT-GATE][BLOCKING]
   -- must land + pass a re-review before PQC-11/13 (PQC-09/10 may proceed
   in parallel, unaffected by this gap).

7a. ~~[AUDIT-GATE][BLOCKING] PQC-05/06/07 adversarial review checkpoint~~ -- the
   master-plan rule "auditor pass after PQC-05 before waves 2+ stack up" has NOT
   run: HANDOFF/review/ has no PQC verdicts. Must complete before PQC-09+ work.
   NOW DISPATCHABLE: task file `PQC_REVIEW_CHECKPOINT_05_06_07.md` contains the
   exact read-only Qwen thinking command; verdict lands at
   HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md. Zero Anthropic cost.

8. `PQC_08_LEGACY_PATH_RETIREMENT.md` [QWEN] IN PROGRESS -- encrypt.rs suite
   gating + LegacyStaticEcdhSend audit event landed; remaining scope per task file.

9. **Orchestration unification COMPLETE 2026-07-11** -- `docs/ORCHESTRATION.md`
   is now the master protocol (all modes); GEMINI.md + ORCHESTRATION_PLAYBOOK.md
   rewritten consistent (via Qwen dispatch); delegate_task.py gained
   `--verify`/`--max-rounds` auto-fix loop (live-tested: happy path exit 0,
   bounded failure exit 2) and a language-tag-agnostic parser.
   `TASK_DELEGATE_VERIFY_LOOP.md` + `UNIFY_GEMINI_DOCS.md` +
   `TASK_DELEGATE_DIFF_MODE.md` all in done/. Diff mode (`--mode diff`)
   live-tested: model diff applied via git apply --recount, verify green.
   Parser handles fenced (any language tag), filename-before-block, and
   raw-unfenced response formats; vacuous-success guarded (exit 3).

10. `TASK_CI_IOS_MACOS_RUNNER_FIX.md` [SONNET][INFRA] -- repo is PUBLIC so
   GitHub macOS runners are free; fix ios-build-test.yml (failure masking,
   lowercase paths, missing -project, no triggers) + bindings-drift gate.
   Unblocks the iOS parity lane without local Mac hardware.

PQC_09..14 (master plan `PQC_00_MASTER_PLAN.md`), TASK_KMP_*, WS-A
release-readiness T/S items, WS-B crypto/transport hygiene trio (backup.rs KDF
gap, WiFi Aware orphan, escalation-authority consolidation), WS-F close-out.
Fine-planning happens as P2-00 after Phase 1 exit, per the execution plan.

## Open decision points for operator (refreshed 2026-07-11)

1. ~~WiFi Direct scope~~ RESOLVED: waived 2026-07-09, v1.1.
2. **Internet relay live proof** -- AWS approved 2026-07-11; needs the rig
   built (P1-14/P1-18 entries above). Record the plan revision in
   HANDOFF/V1_0_0_EXECUTION_PLAN.md when the rig lands.
3. **GitHub billing unlock** -- Actions jobs are created but blocked:
   "account is locked due to a billing issue" (personal account governs the
   public repo; the Enterprise trial does not cover it). Fix billing or
   transfer the repo into the trial org. Unblocks free macOS runners ->
   the whole iOS lane.
4. **iOS scope** -- operator wants iOS parity, but iOS is NOT in the settled
   v1.0.0 scope list. Decide: amend the execution plan (in-scope v1.0) or
   declare it the v1.1 headline. Either way: single Swift bindings +
   XCFramework regen cycle AFTER PQC-10 lands.
5. **KMP D2 stack correction** -- TASK_KMP_COMPOSE_ARCHITECT targets Compose
   UI on linuxX64 native; Compose Desktop is JVM-only. Pick: JVM desktop
   target (recommended) or a different UI stack. Blocks D2 dispatch.
6. **Second Android device / WiFi Aware cell** -- still [BLOCKED-HW];
   acquire or record the waiver in the exit matrix.
7. **WSL2 for KMP Linux validation** -- accepted with BlueZ caveat, or name
   real Linux hardware later.
