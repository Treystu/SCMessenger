# MICRO_RUST_RELAY_ONION_ENABLE_001

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 test completeness
**Source:** `core/tests/integration_relay_onion.rs.disabled`

---

## Verified Gap

An integration test for relay onion routing exists but is disabled (`.disabled` suffix), meaning it is never compiled or run by CI.

**Verified Code State:**
- `core/tests/integration_relay_onion.rs.disabled` exists but is excluded from test compilation
- The test likely has compile errors or was disabled during a previous refactoring
- Relay onion routing is part of the `privacy/` module and should have integration test coverage

## Scope

1. Rename `core/tests/integration_relay_onion.rs.disabled` to `core/tests/integration_relay_onion.rs`
2. Run `cargo test --test integration_relay_onion -p scmessenger-core` to identify compile errors
3. Fix all compile errors (likely API mismatches from transport or privacy module changes)
4. If the test requires significant rework (>300s), document blockers and re-disable with TODO comment

## File Targets

- `core/tests/integration_relay_onion.rs.disabled` -> `core/tests/integration_relay_onion.rs` [RENAME]
- `core/tests/integration_relay_onion.rs` [EDIT to fix compile errors]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo test --test integration_relay_onion -p scmessenger-core --no-run
cargo test --test integration_relay_onion -p scmessenger-core
```

## Acceptance Gates

1. File renamed from `.disabled` to `.rs`
2. `cargo test --test integration_relay_onion -p scmessenger-core --no-run` compiles
3. Test runs and passes (or passes after reasonable fixes)
4. If test requires major rework: document in file comment and re-disable with explanation

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
