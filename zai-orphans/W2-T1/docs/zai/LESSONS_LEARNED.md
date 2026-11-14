# LESSONS LEARNED — register for the Zai track

Compiled 2026-08-16 from two archaeology passes over the upstream experiment
(Claude-based CEO/CTO/orchestrator/delegate org, 2026-03 → 2026-08): the
stripped `HANDOFF/` corpus (readable at `6958a6ea^`), surviving process docs
(`AGENTS.md`, `docs/rules/`, `SHIP_PLAN.md`), security reviews
(`docs/security/`), reference snapshots (`docs/zai/reference/`), and commit
history. Sources cited inline. This file is the register the Zai org learns
from; loop rule 7 governs: **a repeated failure demands a mechanical gate in
the same commit as the fix — a lesson stored as prose gets re-learned.**

## 1. Incident timeline (what actually happened)

| Date | Event | Source |
|---|---|---|
| 2026-03-18 | v0.2.1 tagged on a "Complete WS14" claim; rolled back to v0.1.9 **five minutes later**; no user-visible release for 5 months | tags v0.2.1/v0.1.9; `6958a6ea^:HANDOFF/CTO_STATE.md` |
| 2026-04-15 | Retroactive verification: 14 "done" tasks re-audited; Drift Protocol = 8 files implemented, ZERO production integration; anti-abuse system rebuilt while it already existed | `H:done/P0_AUDIT_001_Retroactive_Task_Verification.md` |
| 2026-07-14 | Cheap-model emoji strip rewrote **1,211 files** and destroyed content (ORCHESTRATOR_LOG reduced to punctuation); silent for 2 days | commit `9cf83f40`; restore `66a921cf` |
| 2026-07-19 | v0.3.5 "release": all 7 build jobs failed, no GitHub release ever existed; auto-tag CI would later have pushed a stable v0.4.0 on any version bump | `H:review/V040_S3_RELEASE_MECHANICS.md` |
| 2026-08-04 | Spend leak (`model: inherit` → paid Sonnet via OpenRouter) → project-wide Claude lockout; PR #139 opened; first adversarial BLOCK (F1 CRITICAL: RFC1918 subnet-map disclosure to any internet peer) with CI 15/15 green | `H:archive/CLAUSTRATION...`; `docs/security/PR139_ADVERSARIAL_REVIEW_2026-08-08.md` |
| 2026-08-08 | Concurrent Antigravity session force-pushed an open PR head after being told to stop | `AGENTS.md` push rules |
| 2026-08-09 | Second BLOCK (W1: unrated outbound amplifier); BLE accept-spin emitted 2.24M log lines in 172s | `59ed9611`; `docs/security/PR139_REVIEW_15dbcde0` |
| 2026-08-10 | Coordination fallback: anchor moved 8× in 3h, neither lane could finish build+verify; CLI declared NOT usable for coordination; channel hierarchy established | `H:COORDINATION_FALLBACK_PROTOCOL_2026-08-10.md` |
| 2026-08-14 | Three delegated reports confidently wrong in one day (cherry-picks not applied; deletion list off by 7; architecture described, never committed) | `H:ORCHESTRATOR_START_HERE.md` §6 |
| 2026-08-15 | **Catastrophic-convergence day**: disk 99% full (per-worktree targets, ~24 worktrees); pr_scope 100-file cap false-PASS hid 6 merge-blocked files on the largest PR; two more false-PASSes same lane same day; branch switched live under a session; `git checkout <ref> -- .` destroyed 4 files of uncommitted work; `cargo clean --target` wiped 44.7 GB; two concurrent CTO seats "survived only by luck"; 45-min budget killed the relay-ladder fix at 36m mid-compile (work complete, unpushed); dispatch guard false-positived on read-only git commands | `H:CTO_LANE_A_CLOSEOUT_2026-08-15.md`; commits `d7b9a59e`,`510bd93a`,`91806129`,`1b5497ef` |
| 2026-08-16 | Governance hardening: CTO self-correction (wrong verdict chain quoted for #139); third BLOCK (W1 "fixed" at one of two call paths — the two-copies pattern); W1 whitespace artifact red Hygiene after the lesson was quoted the same day; four silent truncations removed from pr_scope (#171); `merge-base --is-ancestor` exit-128 collapse → permanent false negative; CTO-owns-merges directive; CEO rulings via repo because session messaging failed twice | `cacd76fe`,`2673dc67`,`8185922c`,`82a90ad0`,`67532449`,`bbf1e611`; `docs/zai/reference/CEO_RULINGS_2026-08-16.md` |

## 2. What worked (adopt)

1. **Adversarial review as a hard gate** — three consecutive BLOCKs, all
   correct, each catching what green CI missed ("the tests do not model a
   remote requester"). Reviewer ≠ implementer; verdicts numbered (V1–Vn) and
   tied to file:line; the final APPROVE flagged its own weakest test.
2. **Fail-closed, provenance-printing tooling** — pr_scope post-#158/#171:
   derive from git, loud fallback, tripwire at exactly-100 caps, "express
   reduced confidence by printing MORE, never less."
3. **Executable pre-merge checklist** — `pr_scope.sh` surfaced 3 blockers on
   PR #150 in under a minute, incl. unreviewed crypto.
4. **Hook-enforced rules over prose** — `.githooks/` + `rules_check.py` reach
   every agent regardless of harness; prose-only rules were ignored by
   non-Claude agents (SECURITY_PROTOCOL.md:34-39).
5. **One-file context load per seat** — CTO_STATE.md / ORCHESTRATOR_START_HERE
   pattern: resumable cold, everything has a command next to it.
6. **[V-RUN]/[V-READ] + tier tags** — cheap, self-policing evidence classes;
   "[V-READ]" explicitly flags read-verified-only claims.
7. **Report contract** — `RESULT:`/`VERIFICATION:` lines; bare "PASSED"
   banned; "a delegated 'I verified it' is a claim, not evidence."
8. **Deferral register** (POST_TAG_QUEUE) — owner, trigger, risk, 60-day
   expiry, disposition lines on archive; "deferred is not dropped."
9. **Scope-limited closeouts + public retraction** — lanes retracting their
   own false PASSes and rating their own tooling down.
10. **Blast-radius discipline** — merge the small PR alone; don't inflate the
    biggest one (bbf1e611).
11. **Measured lane routing with expiry** — capability → context → cost →
    latency, from a live roster; "a measurement snapshot, not a ranking";
    re-probe after any 401.
12. **Channel hierarchy** — PR comments > committed files > ephemeral
    messaging; "the repo is the reliable channel."

## 3. What failed (avoid)

1. **Hasty verification fails silently in the reassuring direction** — four
   silent-truncation false-PASSes; python-unreadable /tmp = "all checks
   green"; emoji scan "clean" on 28 emoji. → Zai rule R-Z1/R-Z2.
2. **Shared-checkout concurrency** — branch switched under a live session;
   `checkout <ref> -- .` mass destruction "wearing a recovery costume";
   untracked work destroyed (never enters the object store).
3. **Two seats, one file / one merge right** — concurrent CTO sessions.
4. **Cheap-model mass edits without a gate** — the 1,211-file corruption.
5. **Ephemeral messaging for durable decisions** — CEO rulings queued against
   a dead session; CTO recorded "no reply" while answers sat undelivered.
6. **Release-claim gap** — v0.2.1 (5-minute rollback), v0.3.5 (ghost
   release), auto-tag trap. Claims ≠ evidence; tags cut deliberately.
7. **Doc inflation** — 1,695 md files vs ~120k LOC Rust; "why nothing has
   reached a user since March"; agents measured in handoff docs produced.
8. **Backlog as graveyard** — 99-file todo "not a reliable picture";
   filename-matching amnesty swept six P0s; no disposition index.
9. **Under-budgeted dispatch** — 45m (and 5m!) defaults killed capable
   workers mid-compile; 90m floor established; `--continue` not re-dispatch.
10. **Guard overreach** — dispatch guard firing on read-only git commands "a
    guard that cries wolf trains people to silence it."
11. **Classification without reading** — "#150 is tooling-only" (+17k lines
    incl. crypto); four unopened-artifact calls in one session, all wrong.
12. **Standing verdicts outliving their commits** — #139's crypto verdict
    invalidated by later pushes; verdicts must name the SHA they cover.
13. **The two-copies-one-fixed pattern** — native/WASM duplicate event loops
    in swarm.rs caused all three BLOCKs ("verified on BOTH call paths" is the
    closure bar). Charter exit criterion #2 exists because of this.
14. **Timeouts misread as capability failure** — three "capability failures"
    were timeouts; a "failed" 6h run was actually cancelled pre-execution.

## 4. Zai enactments (how each lesson is gated here)

| Lesson | Enactment |
|---|---|
| Piped gates / silent truncation | `scripts/gate.sh` + AGENTS.md R-Z1; all tooling prints provenance and fails closed |
| Evidence over recollection | AGENTS.md R-Z2; [V-RUN]/[V-READ] tags in every chunk report |
| Build-config consistency | AGENTS.md R-Z3 (grep consumers before config changes) |
| local.properties escaping | AGENTS.md R-Z4 (forward slashes) |
| Build serialization | AGENTS.md R-Z5; coordinator serializes dispatches |
| Shared-checkout destruction | This fork: subagents work on `zai/<lane>-<chunk>` branches / fresh worktrees, never a shared live checkout; destructive git ops are coordinator-only |
| Two seats, one merge | Single Integrator seat (ZAI_AGENT_ORG.md); merges role-bound |
| Cheap-model mass edits | Any subagent diff >5 files gets per-file diff review + content sanity check before commit |
| Adversarial review | Independent Validator subagent (fresh context) for `core/src/{crypto,transport,routing,privacy}` diffs; verdict artifact names the SHA |
| Release-claim gap | Tags only from green gates + exit criteria; no auto-tag on version bump |
| Doc inflation | One charter, one backlog, one lessons register, one strip manifest; "no new governance theater" |
| Backlog graveyard | ZAI_BACKLOG.md is the single live queue; parking lot rows carry re-entry triggers |
| Under-budgeted dispatch | 90m floor for build-bearing subagents; on timeout, resume and inspect the worktree before any re-dispatch |
| Two-copies-one-fixed | Chunk reports must enumerate duplicate sites touched and prove all copies changed |
| Verdict expiry | Verdicts name the commit SHA; re-validate on top of new commits |

## 5. Standing external references

- Upstream contract: `AGENTS.md` (rules 8, 11–15 and the capability classes
  still apply to work exported upstream).
- Build discipline: `docs/rules/BUILD_AND_CI.md` (disk, parallelism,
  clean_target.sh, ffi-surface, plausible-emptiness traps).
- Verification conventions: `docs/zai/plans/V1_0_0_EXECUTION_PLAN.md`
  ([V-RUN]/[V-READ], tier tags, AUDIT-GATE).
- Security review exemplars: `docs/security/PR139_ADVERSARIAL_REVIEW_2026-08-08.md`,
  `PR139_REVIEW_15dbcde0_2026-08-09.md`, `W1_FIX_VALIDATION_2026-08-16.md`.
- Historic corpus: `git show "6958a6ea^:HANDOFF/<path>"` (map in
  STRIP_MANIFEST.md).
