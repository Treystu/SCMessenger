# MICRO_RUST_CLIPPY_CLEANUP_001

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder / triage-router
**Budget:** 300s (MICRO tier)
**Phase:** v0.2.1 polish
**Source:** `cargo clippy --workspace` warnings

---

## Verified Gap

`cargo clippy --workspace` generates warnings that should be cleaned up before release.

**Verified Code State:**
- `core/tests/integration_offline_partition_matrix.rs:1` — unused import `libp2p::PeerId`
- `core/tests/integration_offline_partition_matrix.rs:2` — unused import `scmessenger_core::store::backend::SledStorage`
- `core/tests/integration_offline_partition_matrix.rs:4` — unused imports `CustodyState`, `Outbox`, `RelayCustodyStore`
- `core/tests/integration_offline_partition_matrix.rs:9` — unused import `std::sync::Arc`
- `core/src/transport/swarm.rs:4929` — unused import `CustodyCompatMode` in test module
- `core/tests/integration_mycorrhizal_routing.rs:507` — useless comparison `stats_after.negative_checks >= 0` (type is unsigned)

## Scope

1. Remove all unused imports from `integration_offline_partition_matrix.rs`
2. Remove unused import `CustodyCompatMode` from `transport/swarm.rs` test module
3. Fix useless comparison in `integration_mycorrhizal_routing.rs:507` (remove or change to `> 0` if intent is to assert non-empty)

## File Targets

- `core/tests/integration_offline_partition_matrix.rs` [EDIT]
- `core/src/transport/swarm.rs` [EDIT]
- `core/tests/integration_mycorrhizal_routing.rs` [EDIT]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

## Acceptance Gates

1. `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` passes with zero warnings
2. `cargo test --workspace --no-run` still passes

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
