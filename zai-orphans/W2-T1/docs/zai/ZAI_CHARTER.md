# ZAI CHARTER — SCMessenger-ZaiComplete

**Mission**: drive this codebase to a *completed* v1.0.0 — every interface
function wired, the "two-copies-one-fixed" duplication eliminated, one
authority per decision, workflows/CI dependable end to end — as a silent
parallel track that never touches the upstream release train.

**Authority documents** (conflicts resolve in this order): this charter →
`docs/zai/PHILOSOPHY_CANON.md` (product/engineering canon, completion
edition) → `docs/zai/ZAI_AGENT_ORG.md` (subagent org) →
`docs/zai/ZAI_BACKLOG.md` (live queue) → `LESSONS_LEARNED.md` (register).
The charter's exit criteria cannot be closed in a way that fails a canon
rule (mapping at the canon's tail).

## Relationship to upstream

- **Origin**: cloned from `Sovereign-Communication/SCMessenger`
  `tracking/pre-v040-tag-work` @ `0aac48b2` on 2026-08-16 (the trunk tip;
  `main` is 209 commits behind it). See `STRIP_MANIFEST.md` for every
  divergence introduced at clone time.
- **Upstream remote is fetch-only** (`git remote -v` shows
  `DISABLE_ZAI_NO_PUSH` as the push URL). Nothing here is ever pushed to
  upstream — no branches, no PRs, no issues — until the operator explicitly
  declares the reveal.
- **Upstream is untouched**: the main checkout, its 26 worktrees, and shared
  build targets are never written from this repo. All sync comes from
  `git fetch upstream`.
- **Sync-out protocol** (only on explicit operator go-ahead): cherry-pick or
  rebase `zai/*` commits onto the then-current upstream base (`v0.4.0` tag or
  later), push to a proper fork branch, and open scoped PRs. To keep that
  mechanical:
  - commits stay small and Conventional-Commits shaped (`feat(w1): …`),
  - changes to upstream merge-blocked dirs
    (`core/src/{crypto,transport,routing,privacy}`) carry audit-style notes
    in the commit body (what changed, threat-reasoning, test evidence),
    mirroring upstream's `[AUDIT-GATE]` expectations,
  - `STRIP_MANIFEST.md` maps every path that only exists on one side.

## Branch model

- `zai/main` — integration trunk. Everything lands here green.
- `zai/<lane>-<chunk>` — short-lived work branches, one per chunk
  (e.g. `zai/w1-manifest-registrations`, `zai/u1-escalation-authority`).
- No force-push anywhere. No long-lived integration branches (that is the
  upstream disease this repo exists to avoid repeating).

## Verification bar (per chunk, Windows dev box)

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -p scmessenger-core -p scmessenger-cli`
4. Wiring-lane chunks additionally regenerate the wiring manifest
   (`scripts/generate_wiring_patch_manifest.py`) and show the burndown delta.
5. Android chunks: `gradle lint test` when the SDK is available.
6. **Known gap**: iOS/macOS targets cannot be compiled or tested on this
   machine. Every iOS-touching chunk must flag this in its commit body and
   lean on `scripts/build_xcframework.sh` + CI (`GITHUB_V1_0_0_RUNNER_SETUP.md`)
   when a remote is added.

## Definition of "completed v1.0.0" (exit criteria)

1. `FFI_WIRING_BURNDOWN.md` reads zero unwired functions (honest baseline 788
   post-W2-M; the historical 162 was a dead-corpus artifact — see
   `docs/zai/chunks/W2-M-burndown-pipeline.md`).
2. `docs/zai/plans/CODEBASE_UNIFICATION_PLAN.md` ranked list fully retired;
   no concept implemented at more than one site unless platform-mandated.
3. Single escalation authority (core `EscalationEngine`), no Kotlin
   shadow-router, no dead-parameter routers.
4. Dual-pass audit CRITICAL findings (128) dispositioned: fixed or formally
   waived with reasoning; audit coverage advanced beyond 73/5,391 functions.
5. WS-A verification debt recorded; WS-C PQC tail (PQC-09 onion, PQC-08
   legacy retirement, PQC-13 matrix) closed; WS-D KMP desktop real (not a
   5-file skeleton); CI lanes green and honest (no `continue-on-error`
   crutches left untracked).
6. `CHANGELOG.md` truthful, version bumped to 1.0.0, full device-matrix
   regression pass recorded.

## Non-goals

- No rewrites for taste. Unification must be justified by the ranked plan or
  a defect, not aesthetics.
- No new governance theater: this repo replaced 1,233 HANDOFF files with this
  charter, `STRIP_MANIFEST.md`, and `docs/zai/`. Keep it that way.
- No work on the upstream v0.4.0 tag effort from here (ignored by design);
  absorb its results via `git fetch upstream` + rebase when the tag lands.
