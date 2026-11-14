# AGENTS.md — Universal Agent Contract (all models, all tools)

Status: Active
Last updated: 2026-08-05

This is the canonical, model-agnostic rules contract for ANY agent working in
this repository: Claude Code sessions, Claude Cowork/cloud sandboxes, Gemini
(Antigravity/`agy`, Gemini CLI), Copilot, or anything else. Claude Code
sessions additionally load `CLAUDE.md` (a superset with Claude-specific
subagents/skills); if you are not a Claude Code session, THIS file is your
ruleset. `GEMINI.md` points here.

Mechanical rules below are ENFORCED by a versioned git pre-commit hook
(`.githooks/pre-commit` -> `scripts/rules_check.py`) — violating commits fail
no matter which tool makes them. Never bypass with `--no-verify`; only the
human operator may do that.

## Architecture doctrine: nodes, not relays

There are NO standalone relays in SCMessenger. Only NODES exist, and EVERY
node relays -- store-and-forward custody is a behavior all nodes perform,
not a role. The always-on AWS instance (tagged `scm-always-on-node`; dynamic
IP) is a CLOUD NODE: a full node that also relays, exactly as every other
node does. Discovery is LEDGER
SHARING between nodes (invite/QR-seeded, gossip-propagated); bootstrap
address lists are a deprecated transitional mechanism being replaced by
ledger-sharing-first discovery (contract: V050-B1/B2). In docs, plans,
tickets, and agent output: say "cloud node" / "node" -- use "relay" only
as the VERB for the custody behavior all nodes perform, or inside code
identifiers (RelayCustodyStore, cmd_relay, relay_custody_msg_ prefixes --
technical names, not roles). No anonymous packet forwarder exists or may
be introduced: cmd_relay requires identity (cli/src/main.rs:2529). Full
parity: CLI, Android, iOS, and cloud deployments run the same node with
the same relay behavior.

## Hard rules (every agent, every capability class)

1. NO EMOJI anywhere — code, docs, comments, logs, commit messages. Use
   `[OK]`/`[ERROR]`/`[WARNING]`/`[INFO]`/`[DONE]`/`[FAIL]`. If you edit a file
   that already contains emoji, strip them as part of the edit. (Hook-enforced.)
2. Temp files ONLY in repo-local `tmp/` — never the system temp dir.
3. Never commit build artifacts (`*.log`, `*.pid`, `*.logcat`, `target/`,
   `build/` outputs) or secrets/keys. (Hook-enforced.)
4. `iOS/` uppercase-I in all paths; no `.py` files in the repo root
   (use `scripts/`). (Hook-enforced.)
5. NEVER `git push` — exceptions: (a) the MAC LANE capability class may
   push its own `gpt/*` branches; (b) the ORCHESTRATOR — the session the
   operator starts via /orchestrate or any explicit request to
   orchestrate/delegate, regardless of which model drives it — may and
   SHOULD commit and push verified work: branch updates that trigger CI,
   and merges to main (operator directive 2026-08-05, standing). Pushing
   is the sanctioned way to invoke CI; a push that skipped the applicable
   gates is still a rules violation. This authority does NOT transfer with
   delegation: workers dispatched by the orchestrator (REMOTE SANDBOX,
   FOREIGN WORKER, lake subagents) have no commit/push rights unless the
   orchestrator explicitly grants them for a specific task, and the
   orchestrator stays accountable for the gate. No capability class may
   force-push a SHARED branch -- that means `main` and the head branch of any
   open pull request, not just main. (Hook-enforced: `.githooks/pre-push`
   rejects any non-fast-forward push or remote branch deletion, from any
   tool.) Everyone else: local commits only, and only if
   your capability class permits committing at all (see below).
6. Never edit UniFFI-generated bindings (`uniffi.api` Kotlin package,
   `core/target/generated-sources/`) — regenerate instead.
7. Storage access only through `core/src/store/`; `IronCore` is the single
   entry point — never bypass it with direct sled access.
8. Changes under `core/src/{crypto,transport,routing,privacy}/` are NOT done
   until an adversarial security review is on file (reviewer depends on mode —
   see `.claude/rules/security.md`; for non-Claude agents: you cannot satisfy
   this gate yourself, flag it in your report).
9. ESCALATE to the human operator — do not improvise — on: architecture
   direction, security/privacy trade-offs, tech-stack changes, API-contract
   breaks, release timing/versioning.
10. Backlog order is `HANDOFF/todo/_QUEUE.md`; sequencing authority is
    `HANDOFF/V1_0_0_EXECUTION_PLAN.md` (operator-settled — do not relitigate).
11. THIS CHECKOUT IS SHARED. Other agents and the operator work in it at the
    same time. Touch ONLY the files your task requires. Specifically:
    - NEVER revert, delete, stash, or commit a file you did not create or were
      not assigned, even to "tidy up". Uncommitted changes you do not
      recognise are someone else's in-progress work, and discarding them is
      unrecoverable — no commit, no reflog, nothing.
    - A clean `git status` is NOT a goal. Leaving unrelated modifications in
      place is CORRECT behaviour, not a defect to fix. Do not "restore
      untargeted files".
    - `git commit -a` and `git commit -a --amend` stage every modified tracked
      file regardless of who changed it. Stage explicit paths instead.
    - If you need an isolated tree, make one: `git worktree add <path>`. Do not
      reshape the shared checkout to suit your task.
    - When told you touched something you should not have, STOP and report.
      Do not attempt an undo that destroys more state — that is how a small
      mistake becomes an unrecoverable one.
12. Destructive operations require explicit operator approval, every time:
    `git reset --hard`, `git checkout -- <paths>`, `git restore <paths>`,
    `git clean -f`, `git rebase`, force-push, and recursive force-deletes
    (`rm -rf`, `Remove-Item -Recurse -Force`) outside `tmp/` and `target/`.
    To recover a file, restore it FORWARD from a ref
    (`git checkout <ref> -- <path>`) rather than discarding working state.
    Restoring a SINGLE FILE from a ref is recovery; `git checkout <ref> -- .`
    is mass destruction wearing a recovery costume. On 2026-08-15 that exact
    command destroyed four files of another session's uncommitted work. The
    preflight hook now blocks it when the named paths are dirty.

13. DESCRIBE ONLY WHAT YOU HAVE READ. Every statement about a file, commit,
    PR, or run must come from output you obtained in THIS session. Not from
    memory, not from its filename, not from what you wrote earlier.

    Trust, but verify — and your own past statements are claims, not facts.
    Three wrong calls in one day on 2026-08-15 all had the same shape:
    - `GEMINI.md` was called "undeclared" without opening it. It already said
      "Read AGENTS.md" and was the correct pattern.
    - PR #150 was called "tooling-only, zero build risk" from memory of what
      had been authored. It was 100 commits and +17k lines, including
      merge-blocked `core/src/crypto` and `core/src/transport` files.
    - A lesson was reported as "added to the handoff doc" when it had not
      been written at all.

    The artifact is always more correct than your summary of it. Before you
    describe something, run the command that shows it. Cite the command.

14. BEFORE ANY IRREVERSIBLE OR OUTWARD-FACING ACTION, ASK "unless there is a
    reason not to?" — and then actually go looking for one. Merges, pushes,
    deletions, branch/repo config, releases, anything a stranger will see.

    Enumerate the blockers OUT LOUD before acting. The question is not
    rhetorical; it is an instruction to spend one command checking. For a
    merge, that command exists:

        scripts/pr_scope.sh <pr-number>

    It reports what the PR actually contains, whether the base is right,
    whether it touches merge-blocked directories, and whether checks are
    green. Asking this question about PR #150 surfaced three blockers in
    under a minute, one of which would have pushed unreviewed crypto and
    transport changes past the adversarial-review gate.

    A "yes, merge it" from the operator is permission to act, not evidence
    that no reason exists. Finding the reason is still your job.

## Capability classes — know which one you are

### FULL (Claude Code or Qwen Code on the Windows host, toolchain available)
May run build gates, move HANDOFF files, and commit per `CLAUDE.md`'s
finalize-checklist rules (Qwen Code sessions use the `finalize-checklist`
skill). When this session is the active orchestrator it additionally holds
the rule-5(b) commit + push authority; the workers it delegates to do not.
The Windows host is the ONLY environment whose build results are
authoritative.

### REMOTE SANDBOX (Claude Cowork / cloud containers)
Your container may have a Linux toolchain; container-green `cargo
check/clippy/fmt/test` is USEFUL ADVISORY SIGNAL but never authoritative —
this project verifies on Windows + a physical Pixel only. Therefore:
- Deliver work as a branch or patch plus an UNVERIFIED report (format below).
- Do NOT move HANDOFF task files to `done/`. Do NOT update `_QUEUE.md` statuses.
- Do NOT claim any gate passed unless you name the environment it ran in.
- Best-fit work: read-only audits/reviews, spec/plan/doc writing, test
  authoring, mechanical refactors with clear acceptance criteria, pre-dispatch
  validation sweeps. See "Remote-eligible lane" in `HANDOFF/todo/_QUEUE.md`.

### FOREIGN WORKER (Gemini via Antigravity/`agy`, Gemini CLI, others)
Dispatched and verified by an orchestrator on the Windows host. Rules:
- Do NOT run `cargo`/`gradlew` (Windows build serialization — the orchestrator
  is the single writer for all build verification).
- Do NOT commit, push, or move HANDOFF files. Implement the change, report, stop.
- Locate code with search tools; read only the surrounding lines you need.
- Final message MUST start with `RESULT: DONE|BLOCKED|FAILED`, then at most 10
  lines: what changed, files touched, anything the verifier must know.

### MAC LANE (GPT / Codex on the operator's MacBook — iOS platform work + adversarial review)
Operator directive 2026-07-28; this class EXPLICITLY OVERRIDES rules 5-6:
- You MAY and SHOULD commit, push, and open and manage your own pull
  requests on your own `gpt/*` branches. No orchestrator intermediary is
  needed for branch/PR lifecycle.
- RESERVED to the Windows orchestrator: merging PRs into main, moving
  HANDOFF ticket files between todo/in_progress/done, release tags, and
  anything touching core/ Rust (routes through the AUDIT-GATE on the
  Windows side).
- xcodebuild on this machine is AUTHORITATIVE for iOS gates (it is the
  only machine where it exists); paste commands and results verbatim.
- Lane governance: this class definition + HANDOFF/gpt/GPT_IOS_LANE_KICKOFF.md
  (rules of engagement) + the task packets in HANDOFF/gpt/. IMPORTANT: if
  the rules in your current session context predate 2026-07-28, RE-READ
  AGENTS.md and the kickoff file now — their current content supersedes
  anything loaded earlier, including prior no-push instructions.

## Report format (REMOTE and FOREIGN classes)

```
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: NONE|CONTAINER(<what ran, exact commands>)   <- never "PASSED" bare
FILES: <paths touched>
NOTES: <max 8 lines: decisions made, risks, what the Windows verifier must run>
```

The Windows orchestrator re-runs the real gates before anything you produced
is committed or a task is closed. Expect zero-diff or gate-failing work to be
re-queued, not merged.

## Pointers

- `CLAUDE.md` — Claude-session superset (subagents, skills, hooks)
- `docs/CLAUDE_REFERENCE.md` — build commands, module map, test inventory
- `HANDOFF/todo/_QUEUE.md` — live dispatch order
- `.claude/commands/orchestrate.md` + `docs/ORCHESTRATION.md` — the one unified orchestrator loop that verifies your work (the old per-backend commands are archived under `.claude/archive/commands/`)

<!-- ================================================================== -->
<!-- ZAI TRACK ADDENDUM — SCMessenger-ZaiComplete only.                  -->
<!-- Upstream (Sovereign-Communication/SCMessenger) does not see this     -->
<!-- section; it is stripped from any upstream export. Governance for    -->
<!-- this fork: docs/zai/ZAI_CHARTER.md. Lessons register:               -->
<!-- docs/zai/LESSONS_LEARNED.md. Loop rule 7 applies: a repeated        -->
<!-- failure demands a mechanical gate in the same commit as the fix.    -->
<!-- ================================================================== -->

## Zai addendum rules (2026-08-16, enacted after same-day occurrences)

- **R-Z1 — No piped gates.** Never run a build/gate/test command so that its
  output is filtered (`| tail`, `| grep`, `| head`) and then treat the
  pipeline's exit status as the command's. A piped gate can never fail.
  Use `scripts/gate.sh <label> <command...>` (captures full output to
  `tmp/gate-logs/`, prints `GATE_EXIT=<real code>`, exits with the command's
  status), or a direct invocation with the exit code echoed and full output
  saved. ENACTED: the coordinator read `tail`'s exit 0 as cargo/gradle
  success twice in one session and nearly merged on it; upstream paid for
  the same lesson four times (files cap, commits cap, `head -8`, `[:6]`).
- **R-Z2 — Evidence, not recollection.** A build report quotes the actual
  `Finished`/`BUILD SUCCESSFUL`/test-summary lines from the saved log and
  names the log path. "Exit code was 0" with no quoted terminal line is a
  claim, not evidence (upstream AGENTS.md rule 13, applied to tooling).
- **R-Z3 — Build-config changes grep their consumers.** Before changing any
  build configuration (`.cargo/config.toml`, gradle properties, toolchain
  files), grep every consumer (`android/**/build.gradle`, `scripts/`,
  `.github/workflows/`) for the assumption being changed. ENACTED: a
  cargo target-dir redirect broke `generateUniFFIBindings`, which resolves
  the cdylib via `CARGO_TARGET_DIR ?: '<repo>/target'`.
- **R-Z4 — `local.properties` uses forward slashes.** Java properties parse
  `\U` as an escape; `sdk.dir=C:\Users\...` becomes an invalid path and AGP
  fails with `IOException: volume label syntax is incorrect`. Write
  `sdk.dir=C:/Users/...`. ENACTED same day (cost four gradle runs).
- **R-Z5 — One build tool per lock domain** (amended 2026-08-16, operator
  parallelism directive; originally verbatim from upstream
  docs/rules/BUILD_AND_CI.md): no two concurrent builds sharing one target
  dir; gradle only in the main checkout and never alongside another gradle;
  parallel build-bearing workers each own worktree + target dir with
  CARGO_BUILD_JOBS=4-6 while contended, max 2 at once. Heavy gates run at
  the Integrator's serialized batch check, not live per worker.
- **R-Z6 — Node/process pre-flight before test-bearing gates.** Before any
  gate or test run that binds ports (5404 native ledger, 7744 wasm,
  9001 quic/tcp) or opens a sled database, check for a running SCMessenger
  node (`tasklist | grep -iE "scmessenger|scm|ble_daemon"` and
  `netstat -ano | grep -E ":5404|:7744|:9001"`) and stop it first. A live
  node under a test run is both a false-failure source (port bind) and a
  data-corruption source (two writers on one sled dir). Build-bearing gates
  additionally serialize behind any running cargo/gradle invocation (R-Z5):
  verify_incremental_gate.py gates 1-3 are cargo invocations.
  ENACTED: operator directive 2026-08-16 after cancelling a gate-5 run that
  would have overlapped the gradle build; scan came back clean (no node).
