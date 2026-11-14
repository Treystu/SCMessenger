# ZAI AGENT ORG — subagent structure for completing v1.0.0

Design doc (adopted 2026-08-16). Adapts what worked in the upstream
Claude CEO/CTO/orchestrator experiment (see `LESSONS_LEARNED.md`) to the
ZCode harness + GLM models, and mechanically avoids its failure modes.
Authority chain: this doc implements `ZAI_CHARTER.md`; conflicts resolve
charter-first.

## 1. Harness facts we design around

- One main session (persistent context, all tools) — the only seat with
  merge authority.
- Subagents: `general-purpose` (all tools — can edit, build, commit) and
  `Explore` (read-only). They run in the SAME filesystem/checkout as the
  main session, with NO shared context — prompts must be self-contained.
- Practical concurrency: ~2 async subagents at once (3 hit the user limit).
  Background bash tasks (builds) run alongside.
- Subagent final message returns to the dispatcher; the repo is the durable
  record.

## 2. Roles

| Role | Instance | Powers | Never |
|---|---|---|---|
| **Integrator** | The main ZCode session (GLM-5.3 seat) | Sole committer/merger to `zai/main`; dispatches work; re-runs every gate before merge; owns charter/backlog/lessons | Push to upstream (fetch-only until reveal); self-certify merge-blocked dirs |
| **Worker** | async `general-purpose` subagent, one per chunk | Creates its own `git worktree` + `zai/<lane>-<chunk>` branch; edits, builds, tests, COMMITS to its branch; writes its report as a file | Touch `zai/main` or another worker's worktree; push anywhere; merge; run gradle (main-checkout-only) |
| **Validator** | fresh async `general-purpose` subagent, read-only posture | Adversarial review of diffs touching `core/src/{crypto,transport,routing,privacy}`; produces numbered verdict artifact naming the SHA; applies the canon enforcement rubric (`docs/zai/PHILOSOPHY_CANON.md` + `skills/philosophy-enforcer/references/enforcement-rubric.md`) to the diff | Share an instance with the author of what it reviews; approve prospective future commits |
| **Prospector** | async `Explore` subagent | Read-only recon, history mining, measurement | Write anything |

Role separation is structural, not aspirational: author ≠ reviewer ≠
merger, always. "A worker holding merge rights defeats every gate above it."

## 3. Isolation and build discipline (the hard-won rules)

1. **A Worker works in its own worktree** — `git worktree add
   ../.zai-worktrees/<chunk> -b zai/<lane>-<chunk>` from the main checkout.
   Never the shared checkout (upstream: branch switched live under a
   session; `checkout <ref> -- .` destroyed uncommitted work).
2. **Cargo in worktrees uses the shared zai target dir** — every cargo
   invocation in a worktree sets `CARGO_TARGET_DIR=C:/Users/SCM/Documents/
   GitHub/.scm-zai-target` (env overrides config). Per-worktree `target/`
   filled a 237 GB drive to 99% upstream. The cargo file lock serializes
   builds — that is a feature.
3. **Gradle/Android verification runs only in the main checkout** (gradle
   resolves the cdylib via repo-root `target/`); the Integrator runs it
   post-merge-candidate. Workers do cargo-only verification.
4. **One build tool per lock domain** (AGENTS.md R-Z5, amended 2026-08-16
   by operator parallelism directive): gradle runs only in the main
   checkout; at most 2 build-bearing workers run concurrently, each in its
   own worktree + target dir with `CARGO_BUILD_JOBS=4-6` while contended;
   prospectors don't build. Batch verification (below) is where serialized
   heavy gates run.
5. **All gates through `scripts/gate.sh`** (AGENTS.md R-Z1) — full logs to
   `tmp/gate-logs/`, real exit codes, quoted terminal lines in reports (R-Z2).
6. **Destructive git ops are Integrator-only** and restore forward, never
   discard. Worktrees are reaped only when clean (`scripts/reap_worktrees.sh`
   semantics: refuse dirty trees, treat ref errors as UNKNOWN).
7. **Diff >5 files from one worker** → Integrator does per-file diff review
   plus a content sanity check before merge (the 1,211-file corruption
   lesson: no mass mechanical edits without a gate).

## 4. Chunk lifecycle

```
ZAI_BACKLOG.md ──► 1. Integrator writes docs/zai/chunks/<chunk>.md
                      (self-contained brief: context, files, done-when,
                       verification commands, report contract)
                 2. Worker (async, worktree) implements + gates + commits
                 3. Worker returns RESULT/VERIFICATION/FILES/NOTES
                 4. Integrator re-runs gates on the branch in the main
                    checkout — a delegated "I verified it" is a claim
                 5. Merge-blocked dirs? → Validator verdict artifact first
                 6. Integrator merges to zai/main, reaps worktree, updates
                    ZAI_BACKLOG + chunk file with [V-RUN] evidence
```

- Branches live <48h; no long-lived integration branches (upstream's
  tracking branch "became a second main").
- On worker timeout (90m floor for build-bearing chunks): resume the agent
  or inspect the worktree — completed-but-unpushed work is the recurring
  salvage; never blind re-dispatch.
- Merge order: smallest blast radius first; never inflate the biggest PR.

### Parallel dispatch and batch verification (operator directive 2026-08-16)

Parallelism unit = the chunk. Two chunks may run concurrently iff their
file sets are disjoint (the brief's FILES list is the contract) and each
build-bearing worker owns its worktree + target dir. The Integrator does
NOT verify each worker live; instead it **batch-checks**: when a batch of
workers returns (or a slot frees), it re-runs the heavy gates serially —
cargo fmt/clippy/test per branch in the main checkout or a verification
worktree, gradle only in the main checkout and only when no other gradle
runs — then merges in blast-radius order. A worker's VERIFICATION lines
are claims until the batch check. Max 2 async subagents at once (harness
concurrency cap observed at 3).

## 5. The report contract (every Worker/Validator return)

```
RESULT: DONE|BLOCKED|FAILED
VERIFICATION: <exact commands + GATE_EXIT lines + quoted terminal lines>
FILES: <paths touched>
COPIES: <duplicate sites of any concept touched — and proof all changed>
CANON: <PHIL-ids checked against docs/zai/PHILOSOPHY_CANON.md, e.g. "PHIL-012 n/a — no decision authority touched">
NOTES: <max 8 lines: decisions, risks, what the Integrator must re-run>
```

`[V-RUN]` = command executed this session with output quoted;
`[V-READ]` = read-verified only, flagged as needing a real run. Bare
"PASSED" is banned. BLOCKED is a structured decline — never invent answers.

## 6. Cost tiering (GLM lane mapping)

Upstream tiered by model cost (Haiku/Sonnet/Opus/verdict budget). ZCode
runs one GLM family, so tiering maps to AGENT TYPE and prompt weight:

| Upstream tier | Zai equivalent | Use |
|---|---|---|
| [HAIKU] verbatim-diff mechanical | Worker with tightly-scoped brief | renames, registrations, generated-file updates |
| [SONNET] scoped implementation | Worker with standard brief | most lane chunks (W/U lanes) |
| [OPUS+] spec/diagnosis | Integrator session itself (+ Prospector recon) | design notes, architecture, dispatch planning |
| [AUDIT-GATE] verdict | Validator (fresh instance, read-only) | merge-blocked dirs, release gates — budgeted, not routine |
| [HUMAN] | The operator (you) | irreversible/scope decisions, upstream reveal |

Verdict budget discipline: Validators fire only for merge-blocked dirs and
release-critical chunks, not every diff.

## 7. Communication: the repo is the channel

- Chunk briefs, verdicts, and completion reports are COMMITTED FILES under
   `docs/zai/chunks/`. Session messages are dispatch nudges only (upstream:
  CEO rulings queued against a dead session while work blocked on a phantom
  non-reply).
- Open decisions live in the chunk file with an owner; ambiguity fails
  closed (BLOCKED), never resolves to yes.

## 8. What we deliberately did NOT adopt

- **Capability-class sprawl** (FULL/REMOTE/FORBIDDEN/MAC lanes): one
  harness, one org; the push tier collapses to "Integrator only."
- **lanes.json multi-provider routing**: single GLM family; type-based
  tiering replaces provider tiering.
- **The HANDOFF state machine**: one charter, one backlog, one lessons
  register, one strip manifest. "No new governance theater."
- **Session-messaging coordination** for anything durable.

## 9. First enactments (this org's chunk zero)

1. W1 AndroidManifest registrations — Integrator-executed pilot of the
   lifecycle (done when gradle manifest merge is green).
2. W2-M (measurement fix): re-point `build_wiring_graph.py` /
   `generate_wiring_burndown.py` at surviving `HANDOFF_AUDIT/REPO_MAP.jsonl`,
   drop dead-path inputs, regenerate `FFI_WIRING_BURNDOWN.md` (post-strip
   true count; current 162 includes 28 stripped ghosts), route generator
   output through `rules_check.py`. FIRST WORKER DISPATCH — scripts-only,
   low risk, unblocks the whole W lane's burndown measurement.
3. U1 escalation authority — first merge-blocked-dir chunk → also the
   Validator pilot.
