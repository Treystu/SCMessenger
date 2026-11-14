# Mission Brief for Bob: Drive V1.0.0 to Completion (Lucas/Josh Alpha Test + Farm-Sim)

Status: ACTIVE
Written by: Claude (native session), for Bob (foreign orchestrator)
Date: 2026-07-20
Operator: Lucas

## Read this first

You already know the protocol -- `docs/ORCHESTRATION.md` is the canonical,
model-agnostic cross-mode reference and your own `.bob/skills/{orchestrate,
scmorc,scm,scmqwen,swarm,gemini-orchestrator}/SKILL.md` files mirror the
routing tables, worker contracts, and state machine. This brief does not
repeat that mechanics -- it tells you WHAT the goals are and WHY, in priority
order, plus what's changed since the HANDOFF docs were last written and what
authority you're operating under. Figure out the HOW yourself using your
existing playbook (pre-dispatch validation, free-lanes-first dispatch ladder,
Fusion judgement, adversarial review gates, HANDOFF state machine).

This brief is itself a backlog input, not a replacement for `HANDOFF/todo/
_QUEUE.md` or `scm_v1_farm_queue.jsonl`. Fold the new tickets below into
those the normal way.

## Your toolset for this run

Lucas has asked specifically for the free/cheap lanes: Qwen (DashScope),
Groq, OpenRouter free-tier models, Ollama cloud, Fusion Lite (planning/
verification panels, cost-capped) and Morph Lite (surgical diff application).
This is a foreign-orchestrator run -- no Claude subscription workers unless
you hit a genuine structural deadlock or an [AUDIT-GATE] item that your own
protocol says needs top-tier adversarial judgment, per the escalation ladder
you already have in `docs/ORCHESTRATION.md` Section 2.1 and your `scmorc`
skill. Use `scripts/delegate_task.py` (providers: qwen, groq, openrouter,
ollama, gemini), `scripts/fusion_lite.py`, `scripts/morph_lite.py` as you
already know how.

## Authority granted

Lucas is granting you broad execution authority for this run so you don't
need to stop and ask for routine things:

- Dispatch freely across the free/cheap lanes, choose models/tiers, retry
  and escalate per your existing ladder.
- Make implementation-detail decisions within a ticket's stated goal --
  you were told to figure out the HOW, this is that permission made explicit.
- Commit locally and move HANDOFF tickets through the state machine
  (todo -> IN_PROGRESS -> review -> done) as you already do.
- Spend within the EXISTING cost ceilings (Fusion Lite: $0.01 default / $0.05
  hard cap per `docs/ORCHESTRATION.md` Section 2.1 and Section 10; OpenRouter
  free-only key stays free-only).
- Write new HANDOFF ticket files for sub-tasks you decompose from the goals
  below, same as any orchestrator would.

What is NOT included in that grant -- these stay exactly as your existing
rules already say, this is a reminder, not a new restriction:

- Never `git push` to `origin` without Lucas's explicit go-ahead. Commit
  locally, checkpoint often, but he pushes (or explicitly tells you to).
- Never cut a tagged release or publish a GitHub Release without Lucas's
  go-ahead (see `V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md` below -- prepare
  it, don't publish it unsupervised).
- Never raise a cost ceiling (Fusion Lite cap, OpenRouter key scope, etc.)
  yourself -- ask.
- The mandatory Security Gates table (`docs/ORCHESTRATION.md` Section 4) is
  not negotiable: any diff in `core/src/{crypto,transport,routing,privacy}/`
  gets adversarial review before commit, regardless of how confident the
  implementation felt. Same for the WS-A delivery-logic triangulation
  requirement (outbox/receipt/custody/retry) and the PQC sequencing freezes.
- Escalate to Lucas before: architecture-direction changes, security/privacy
  tradeoffs, tech-stack changes, API-contract breaks, or release/versioning
  decisions. This is the same standing rule your `scm`/`scmorc` skills
  already state -- it still applies here.
- If a task's requirements are ambiguous in a way that risks real rework or
  a security-relevant judgment call, stop and ask Lucas rather than guessing.
  Silent, confident guessing on the actually-uncertain stuff is the one
  thing NOT wanted here -- goal-directed autonomy on everything else is.

## What's changed since the HANDOFF docs were last written (assume nothing -- verify these yourself, don't just trust this list either)

1. **GitHub Actions is live, not blocked.** The repo is on the
   `Sovereign-Communication` org with an active GitHub Enterprise trial
   (operator-stated, ~20 days remaining as of today). This contradicts
   `HANDOFF/V1_0_0_EXECUTION_PLAN.md`'s "No CI" assumption and
   `HANDOFF/todo/WAVE_H_HUMAN_GATES.md`'s H-01 gate -- both are stale on this
   point (H-01 was actually already marked resolved once, in
   `HANDOFF/GITHUB_CI_CD_AUDIT_FINDINGS.md` Section 1.1, but that correction
   never propagated everywhere). Treat H-01 as CLOSED. This also means the
   iOS CI lane, XCFramework regen, and PQC-13 interop-matrix testing are no
   longer gated on billing -- only on the actual workflow defects and
   PQC-10 sequencing, which are separately tracked.
2. **CI is currently RED on `main`, and has been all day, unnoticed** -- see
   the new ticket `CI_RED_ON_MAIN_ALL_FEATURES.md`. This is priority 0,
   below.
3. Everything from today's Lucas/Josh alpha-test session (dial fix, graceful-
   AF dial policy, farm-sim bootstrap wiring, iOS CI fixes, ratchet/PQ
   wiring) IS committed AND pushed to `origin/main` already -- verified via
   `git log`/`git status` directly, not just HANDOFF claims. You don't need
   to re-push any of it.
4. The ratchet/PQ subsystem wiring (`E-00`,
   `HANDOFF/done/CRITICAL_RATCHET_SUBSYSTEM_NOT_WIRED_INTO_IRONCORE.md`) is
   DONE, contrary to how urgently `REMAINING_WORK_TRACKING.md`'s 2026-07-17
   entry frames it -- that entry is now stale too, it was resolved the same
   day it was filed.
5. Two background dispatches launched at the end of today's session (A-04
   Android receipt unification, D-05 unwrap/panic hardening) produced EMPTY
   logs (`tmp/a04_dispatch.log`, `tmp/d05_dispatch.log` are both 0 bytes) --
   they did not run to completion. Both tickets are still sitting in
   `HANDOFF/IN_PROGRESS/` and need to be re-dispatched or picked up fresh.

## Priority order

Sequencing logic: fix the thing that's silently broken and blocks trusting
anything else (0), then close the gap between "the fix works" and "the thing
Lucas actually asked for is true" (1), then the larger Farm-Sim validation
track which is real but not urgent for the Josh test specifically (2), then
the rest of the standing backlog opportunistically (3). Human-only items (4)
you cannot act on beyond flagging them.

### 0. CI is red -- fix it first

`CI_RED_ON_MAIN_ALL_FEATURES.md` (new ticket, drafted alongside this brief).
Get `ci.yml` green on `main`: Lint (`cargo clippy --workspace --all-features
-- -D warnings`), FFI Surface Contract (`scripts/ffi_surface.sh`), and the
full Test matrix (`cargo test --workspace --all-features` on ubuntu/windows/
macos). This has been failing since at least this morning's first commit --
it's not a regression from anything you'll be doing, but nothing after this
should be trusted as "verified" via CI until it's green, and you now have a
real regression gate once it is.

### 1. Close the actual gap for the Josh install-link goal

Three new tickets, drafted alongside this brief -- read each in full, they
have the evidence and open questions:

- `PROVE_SECOND_REAL_ENDPOINT_DELIVERY.md` -- the real remaining gap. Every
  success so far is Lucas's own two clients. Prove two INDEPENDENT
  identities can exchange contacts and a delivered+acknowledged message
  through the alpha relay, using whatever substitute for "a second real
  device" actually works (your choice -- options listed in the ticket).
- `V1_INSTALL_ARTIFACT_FOR_ALPHA_TESTERS.md` -- Lucas asked for "a link to
  install," and today there isn't one (just a local debug APK and a
  build-from-source path for the CLI). Figure out whether `release.yml`
  already covers this or needs extending, and get to a real, shareable
  download link built from a CI-green commit. Do not publish/tag without
  Lucas's go-ahead -- prepare it, then ask.
- Also still open and already well-specified, pick up as-is:
  `HANDOFF/todo/GRACEFUL_AF_DIAL_POLICY.md` items 3-4 (per-peer backoff +
  concurrent-dial cap; prefer relay-circuit routing over promiscuous direct
  dial), and `HANDOFF/IN_PROGRESS/A-04_ANDROID_RECEIPT_UNIFICATION.md`
  (re-dispatch -- last attempt produced no output).
- Small, not-yet-filed item: the swarm's adaptive port listener should
  exclude the control-API port (9876) from its own port range so the
  earlier port-collision bug class can't recur (`core/src/transport/`,
  security-review-gated). File the ticket, then fix it.
- Design question, still genuinely open, don't guess: whether/how to
  reframe "bootstrap" as "relay seed" per the operator's 2026-07-20
  directive ("bootstrap deprecated in favor of relays") -- see the note in
  `HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md` under "Known
  issues filed this session." If it's genuinely a judgment call with product
  implications, ask Lucas rather than deciding it yourself.

### 2. Farm-Sim validation track

This is real V1.0.0 scope (the 28-acre farm deployment validator) but not on
the critical path for the Josh test. Work it in parallel/opportunistically,
not ahead of priority 1.

- Re-validate the contact-provisioning fix end-to-end. The fix (deterministic
  identity fetch via `/api/identity` + provisioning via `/api/contacts`, plus
  the stress-test binary now built into the Docker image) is committed
  (`docker/bootstrap-topology.sh`, commit `2eaad174`), but nobody has re-run
  Phase 2 (progressive load 10->100 msg/sec) or Phase 3 (failure injection)
  against it to confirm real message delivery now actually works on the
  7-node topology. See `HANDOFF/todo/FARM_SIM_PHASE_2_3_FINDINGS.md` and
  `HANDOFF/todo/ORCHESTRATE_FARM_SIM_FIX_AND_RETEST.md` for the full test
  plan already written -- execute it, don't re-design it.
- Only after that retest passes: the 6-hour stability run, the cross-variant
  test (real Android/iOS builds joining the topology, not just the CLI
  container), and scaling to the planned 12-node/3-group topology
  (`HANDOFF/plans/FARM_FINAL_PLAN.md` has the topology spec).
- `HANDOFF/todo/A-09_RELAY_DISCOVERY_DIAL_AMPLIFICATION.md` -- HIGH-severity
  DoS ticket, partially mitigated twice already. The remaining items
  (`connection_limits`, authenticating relay-discovery messages, dial dedup,
  input-size guard) are well-specified in the ticket. Mandatory adversarial
  review before this closes.
- `HANDOFF/todo/C-05_P1_14_hostile_network_test_lo.md` -- explicitly needs a
  design decision before it's safe to dispatch (the ticket says so itself:
  decide what "test passes" means concretely, wire the existing orphaned
  `docker/docker-compose.network-test.yml` into a real test runner with an
  assertion, THEN decide if code changes are even needed). Do the design
  step, don't skip straight to code.
- iOS CI: now that H-01 is resolved (see above), figure out which of the 7
  documented defects in `HANDOFF/GITHUB_CI_CD_AUDIT_FINDINGS.md` Section 3.1
  / `TASK_CI_IOS_MACOS_RUNNER_FIX.md` are still actually open after commit
  `1950c374` ("iOS Swift compile errors, NDK env, ffi exec bit...") -- some
  may already be fixed, don't assume the list is still accurate as written.
  Close what's left. This unblocks the farm's iOS-parity requirement
  (half+ of farm users carry iPhones, per the operator's own farm-plan
  decision).

### 3. Standing backlog -- pick up opportunistically, lower urgency

- `E1` / `PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY.md` -- the hardest open
  item in the whole backlog. Two prior design attempts both failed
  adversarial review for two different desync failure modes. This needs
  real protocol design work, not a quick dispatch -- run it through your
  full THINK-design -> Fusion-judgement -> CODER-implementation ->
  adversarial-panel pipeline (`docs/ORCHESTRATION.md` Section 10), same as
  any other AUDIT-GATE crypto item. Don't rush it just because it's been
  open a while.
- `HANDOFF/IN_PROGRESS/D-05_UNWRAP_PANIC_HARDENING.md` -- already re-scoped
  to file-by-file, judgment-reviewed dispatches (whole-file bulk dispatch
  was tried three times and failed safely each time -- don't repeat that).
  Confirmed non-blocking for Lucas/Josh or Farm-Sim core functionality.
- `HANDOFF/review/U7_SCHEMA_DRIFT_AUDIT_REPORT.md` exists -- read it for
  what it actually found before assuming its status; not verified in this
  brief.
- Doc hygiene: now that GitHub Actions' real status is confirmed, sweep
  `HANDOFF/` and `docs/` for other places that assume CI is unavailable
  (starting with `HANDOFF/V1_0_0_EXECUTION_PLAN.md` Section 1.2/2.4 and
  `HANDOFF/todo/WAVE_H_HUMAN_GATES.md` H-01) and correct them. This repo has
  a recurring pattern of a correction landing in one doc and never
  propagating to the others that reference the same claim -- worth a
  general pass, not just these two.

### 4. Human-only -- flag and wait, don't attempt

- H-02 (physical two-device WiFi Aware / BLE field trials) -- needs real
  hardware in Lucas's or a farm-mate's hands.
- Apple Developer account decision ($99/yr, needed before iOS TestFlight).
- Actually getting Josh's real phone connected and tested -- that's Lucas's
  and Josh's step once priority 1 above is done. Your job is to make sure
  everything up to that point is proven and ready, not to somehow do that
  step yourselves.
- Final v1.0.0 release sign-off (`HANDOFF/todo/WAVE_H_HUMAN_GATES.md` H-05).

## Definition of done for this run

Not "queue empty" -- that's a lot of backlog and some of it (E1, farm 12-node
scaling) is genuinely multi-session work. Done for THIS run means: priority 0
green, priority 1's tickets each either closed or reduced to a clearly-stated
human decision point, and visible, committed (not pushed) progress on
priority 2. Write a session handoff doc when you stop, same format as
`HANDOFF/SESSION_HANDOFF_2026-07-20_LUCAS_JOSH_ALPHA.md`, so the next session
(you or anyone else) doesn't have to re-derive any of this.
