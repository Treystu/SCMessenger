# BATCH: Core / WASM / CLI Wiring (Priority 2)

Complete all tasks below. Process sequentially. After each task, run `cargo check --workspace` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

## Task 1: task_cli_swarm_stats
- File: `HANDOFF/todo/task_cli_swarm_stats.md`
- Target: `cli/src/main.rs`

## Task 2: jsonrpc_send_message_roundtrip
- File: `HANDOFF/todo/task_wire_jsonrpc_send_message_roundtrip.md`
- Target: `core/src/wasm_support/rpc.rs`

## Task 3: get_settings
- File: `HANDOFF/todo/task_wire_get_settings.md`
- Target: `wasm/src/lib.rs`

## Task 4: get_identity_from_daemon
- File: `HANDOFF/todo/task_wire_get_identity_from_daemon.md`
- Target: `wasm/src/lib.rs`

## Task 5: start_receive_loop
- File: `HANDOFF/todo/task_wire_start_receive_loop.md`
- Target: `wasm/src/lib.rs`

## Task 6: validate_settings
- File: `HANDOFF/todo/task_wire_validate_settings.md`
- Target: `wasm/src/lib.rs`

## Task 7: custody_audit_persists_across_restart
- File: `HANDOFF/todo/task_wire_custody_audit_persists_across_restart.md`
- Target: `core/src/store/relay_custody.rs`

## Task 8: custody_transitions_are_recorded
- File: `HANDOFF/todo/task_wire_custody_transitions_are_recorded.md`
- Target: `core/src/store/relay_custody.rs`

## Task 9: blocked_only_peer_ids
- File: `HANDOFF/todo/task_wire_blocked_only_peer_ids.md`
- Target: `core/src/store/blocked.rs`

## Task 10: evaluate_all_tracked
- File: `HANDOFF/todo/task_wire_evaluate_all_tracked.md`
- Target: `core/src/abuse/auto_block.rs`

## Task 11: get_all_connection_stats
- File: `HANDOFF/todo/task_wire_get_all_connection_stats.md`
- Target: `core/src/transport/health.rs`

## Task 12: cleanup_stale_connections
- File: `HANDOFF/todo/task_wire_cleanup_stale_connections.md`
- Target: `core/src/transport/health.rs`

## Verification
After all tasks: run `cargo build --workspace` and `cargo test --workspace --no-run`
Report: STATUS: SUCCESS_STOP or list blockers.
