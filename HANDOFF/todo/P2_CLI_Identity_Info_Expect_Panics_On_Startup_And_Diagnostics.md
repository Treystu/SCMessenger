# P2_CLI_Identity_Info_Expect_Panics_On_Startup_And_Diagnostics

**Priority:** P2
**Platform:** CLI
**Status:** TODO
**Source:** native sweep 2026-07-04

## Problem

`IdentityInfo::identity_id` is `Option<String>` (`core/src/lib.rs:100` struct
def). Most call sites in `cli/src/main.rs` handle the `None` case defensively
with `.unwrap_or_default()` (e.g. `main.rs:593`, `:627`, `:1968`, `:1980`), but
two call sites use `.expect(...)` instead, which will panic the whole daemon
process if `identity_id` is ever `None` at that point:

- `cli/src/main.rs:1312-1314` — inside the daemon startup banner
  (`print!`/`println!` sequence right after `SCMessenger — Starting...`):
  ```rust
  info.identity_id
      .clone()
      .expect("Identity ID should be available")
      .bright_cyan()
  ```
  This runs on every daemon start, before the mesh is up — a panic here kills
  the process before it ever binds the control API or transport, with no
  graceful degradation path.

- `cli/src/main.rs:699-705` — inside `print_full_identity`, the `scm identity
  show`-style diagnostic print path:
  ```rust
  info.identity_id
      .expect("Identity ID should be available")
      .bright_cyan()
  ```

Both are plausible to hit with `None` if identity initialization is
in-progress, partially failed, or if `get_identity_info()` is called in a
state where the underlying identity hasn't finished loading/registering yet
(the existence of the `.unwrap_or_default()` call sites elsewhere in the same
file for the exact same field confirms `None` is a real, anticipated case for
this field elsewhere in the codebase — the `.expect()` sites are the
inconsistent outliers, not the norm).

This is not covered by any existing `HANDOFF/todo/*.md` file (checked via grep
for `identity_id`, `expect(`, `IdentityInfo` across the todo directory) and is
distinct from T1-T7/S-tasks in `docs/release-readiness-2026-07-02.md` (those
cover backup/contact-store bugs, not this).

## Fix Plan

Replace both `.expect(...)` call sites with the same `.unwrap_or_default()`
pattern already used elsewhere in `main.rs` for this field, or (preferably,
since these are user-facing diagnostic/startup prints) a short fallback string
like `"(pending)"` so the operator gets a signal that identity hasn't
finished initializing rather than a bare empty string. Example:

```rust
info.identity_id
    .clone()
    .unwrap_or_else(|| "(pending)".to_string())
    .bright_cyan()
```

No behavior change is needed beyond this — both sites are print-only, not
gating any control flow on the value.

## Files to Touch

- `cli/src/main.rs` [EDIT] — lines ~700-705 (`print_full_identity`) and
  ~1312-1314 (daemon startup banner)

## Verification

```bash
cargo build -p scmessenger-cli
cargo test -p scmessenger-cli
```
Manual: start the daemon in a state where identity registration hasn't
completed (or stub `get_identity_info()` in a test) and confirm no panic,
banner prints `(pending)` instead.

## Acceptance Criteria

- Neither call site can panic the CLI process when `identity_id` is `None`.
- Existing `.unwrap_or_default()` call sites for the same field are left
  unchanged (out of scope — already safe).
- `cargo build -p scmessenger-cli` and `cargo test -p scmessenger-cli` pass.
