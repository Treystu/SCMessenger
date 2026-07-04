# TASK: P0-COMPILE-GATE — Get a ground-truth full workspace compile/test result before dispatching any further wiring or feature work

## Why this is P0 and first

A comment surfaced (source: gemini.google.com web UI reading the GitHub repo
statically — **not** a live build, not the agy/Antigravity session that has a
real local toolchain and physical-device access) claiming:

> "a few critical architectural blockages and operational risks are flying
> under the radar. The main compile gate is fundamentally broken. There are
> currently 8 pre-existing compile errors from earlier agent wiring work.
> Before the agents can safely finish the rest of the SCMessenger wiring
> backlog, you need to clear the compile gate."

**This claim has NOT been independently verified, in either direction, as of
this writing.** Here is exactly what is and isn't known:

- The 350-task wiring backlog itself is genuinely closed (verified 2026-07-04:
  `HANDOFF/todo/` has 0 `task_wire_*` files, `HANDOFF/done/` has 351;
  `HANDOFF/WIRING_TASK_INDEX.md` / `WIRING_PATCH_MANIFEST.{json,md}` were
  regenerated and confirm `Total tasks: 0`). If Gemini's comment is about
  *that* backlog needing more wiring, that specific claim is contradicted by
  direct evidence and is very likely stale/wrong.
- The much more plausible source of the "8 errors" comment: a same-day
  sandbox session (no local Rust toolchain, could only run `cargo check`
  before hitting sandbox timeouts on the link step) implemented
  `HANDOFF/DESKTOP_BRIDGE_WIRING_SPEC.md` — wiring 6 previously-unreachable
  modules into the `scmessenger-desktop-bridge` crate (new `types.rs`, `pub
  mod` registration, a `uniffi::setup_scaffolding!()` call, fixes to a few
  real bugs found along the way: an `Arc<Option<...>>` that should have been
  `Arc<Mutex<Option<...>>>`, zbus 4.x API mismatches in `ble.rs`, a lifetime
  escape in `notification.rs`). That subagent reported `cargo check -p
  scmessenger-desktop-bridge` passing, but could **never get `cargo build
  --workspace` or `cargo test` to actually finish** — timeouts in a 2-CPU
  sandbox, not confirmed compile failures. Separately, a CLI transport bug
  task written the same day
  (`HANDOFF/todo/P1_CLI_Transport_Negotiation_Failure_On_Android_Inbound_Dial.md`)
  mentions in passing: "a related crate, `desktop_bridge`, was independently
  found to be **failing to build entirely** in this same session due to
  missing `zbus`/`web_time` dependencies" — but checking `desktop_bridge/
  Cargo.toml` right now shows both `zbus = "4"` and `web-time = "1.1"` ARE
  present under `[target.'cfg(target_os = "linux")'.dependencies]`, so either
  that observation predates the fix, or there's a different, still-real
  problem being described inexactly.
- A static read of all `desktop_bridge/src/*.rs` files found no obvious
  syntax errors (brace-matching is clean in every file), but this is not a
  substitute for an actual compiler run — proc-macro expansion, trait bound
  errors, and UniFFI codegen issues are invisible to a manual read.

**Bottom line: nobody has actually run a clean `cargo build --workspace` on
current `main` and reported the real, authoritative result.** Every session
so far has hit either a toolchain gap (sandboxed sessions) or hasn't been
asked to (agy was mid-PQC-01, not building the full workspace). This task
exists to close that gap before anyone dispatches more work on the assumption
that the tree either builds or doesn't.

## Acceptance Criteria

- [ ] A real `cargo build --workspace` (or at minimum `cargo check
      --workspace`) has been run to completion, on a machine with a working
      Rust toolchain, against current `main` (or whatever branch/commit this
      task is picked up on — record the exact commit hash in this file when
      you run it).
- [ ] If it's clean: this file is updated with the real output (or a
      trimmed summary + exit code), moved to `HANDOFF/done/`, and Gemini's
      "8 compile errors" claim is recorded here as investigated-and-not-
      reproduced, with the actual evidence, so it doesn't get treated as
      true by a future session just because it was said with confidence.
- [ ] If it's NOT clean: paste the exact compiler errors into this file
      (do not summarize/paraphrase them — the next agent needs the real
      rustc output to fix them correctly), and only then should follow-up
      fix tasks be written, scoped to the exact errors found — not a guess.
- [ ] Specifically confirm or refute whether `scmessenger-desktop-bridge`
      builds as part of the full workspace build (`cargo build -p
      scmessenger-desktop-bridge` alone, then `cargo build --workspace`) —
      isolate whether any failure is scoped to that one crate or is
      workspace-wide.
- [ ] Also run: `cargo test --workspace --no-run` (the repo's "compile gate"
      per CLAUDE.md — builds all tests without running them), `cargo fmt
      --all -- --check`, `cargo clippy --workspace -- -D warnings -A
      clippy::empty_line_after_doc_comments`. Record all four results.

## Implementation Plan

1. `git pull` / confirm you're on current `main`, record the commit hash.
2. `export CARGO_INCREMENTAL=0` (required per `.claude/rules/build.md` on
   Windows to avoid rlib corruption).
3. `cargo build -p scmessenger-desktop-bridge` — isolate this crate first,
   since it's the most recently and heavily modified area and the most
   likely source of any real failure.
4. `cargo build --workspace`.
5. `cargo test --workspace --no-run`.
6. `cargo fmt --all -- --check`.
7. `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments`.
8. Paste every command's real output (or the last ~50 lines if it's long,
   but keep any actual error text verbatim, never paraphrased) into this
   file under a new "## Results" heading.
9. Based on the real results:
   - **If clean:** mark this done, move to `HANDOFF/done/`. No further
     wiring-backlog work is needed — that backlog is already closed and
     verified separately. Resume normal priority order from
     `HANDOFF/V1_0_0_UNIFICATION_PLAN.md` / `HANDOFF/RESUME_STATE_2026-07-04.md`.
   - **If broken:** do NOT attempt speculative fixes in this same task.
     Write one new, precisely-scoped `HANDOFF/todo/` task per distinct
     error (or per tightly-related cluster of errors in one file), quoting
     the exact rustc error text, following the existing task file
     conventions (see any file in `HANDOFF/todo/` for the template). If any
     resulting fix touches `core/src/crypto|transport|routing|privacy`, flag
     the mandatory `crypto-security-auditor` review requirement in that
     task file per `.claude/rules/security.md`.

## Files to Touch

None directly by this task — it is a verification/triage task. Follow-up
fix tasks (if needed) will name their own files based on real compiler
output.

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build -p scmessenger-desktop-bridge
cargo build --workspace
cargo test --workspace --no-run
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

## Do NOT

- Do not write or land any speculative fix for "8 compile errors" without
  first seeing the real compiler output — the count, location, and nature
  of any actual errors is currently unknown; guessing wastes a whole
  implementation cycle if the guess is wrong or the errors don't exist.
- Do not treat this task as evidence the 350-task wiring backlog is
  reopened — that closure was independently verified via file counts in
  `HANDOFF/done/` vs `HANDOFF/todo/` and manifest regeneration, and stands
  regardless of this task's outcome.
- Do not skip straight to fixing `desktop_bridge` based on the prior
  session's partial `cargo check` pass — that was on one crate in isolation
  under `cargo check` (type-checking only, not full codegen/linking); a full
  workspace `cargo build` can still fail even when `cargo check` on one
  member passes, especially with proc-macro-heavy crates like this one using
  UniFFI.
