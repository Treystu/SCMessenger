# BATCH: Core / WASM / CLI Wiring (Priority 1)

Complete all tasks below. Process sequentially. After each task, run `cargo check --workspace` to verify. If a task cannot complete due to missing dependencies, document blockers and move on.

## Task 1: jsonrpc_get_identity
- File: `HANDOFF/todo/task_wire_jsonrpc_get_identity.md`
- Target: `core/src/wasm_support/rpc.rs`

## Task 2: derive_key_always_32_bytes
- File: `HANDOFF/todo/task_wire_derive_key_always_32_bytes.md`
- Target: `core/src/crypto/kani_proofs.rs`

## Task 3: relay_discovery_mut
- File: `HANDOFF/todo/task_wire_relay_discovery_mut.md`
- Target: `core/src/relay/discovery.rs`

## Task 4: transport_type_to_routing_transport
- File: `HANDOFF/todo/task_wire_transport_type_to_routing_transport.md`
- Target: `core/src/routing/engine.rs`

## Task 5: recordTransportEvent
- File: `HANDOFF/todo/task_wire_recordTransportEvent.md`
- Target: `core/src/routing/telemetry.rs`

## Task 6: get_forwarding_capability
- File: `HANDOFF/todo/task_wire_get_forwarding_capability.md`
- Target: `core/src/transport/capability.rs`

## Task 7: drain_received_messages
- File: `HANDOFF/todo/task_wire_drain_received_messages.md`
- Target: `core/src/store/inbox.rs`

## Task 8: send_prepared_envelope
- File: `HANDOFF/todo/task_wire_send_prepared_envelope.md`
- Target: `core/src/outbox/prepare.rs`

## Task 9: getNetworkDiagnosticsSnapshot
- File: `HANDOFF/todo/task_wire_getNetworkDiagnosticsSnapshot.md`
- Target: `core/src/transport/diagnostics.rs`

## Task 10: get_healthy_connections
- File: `HANDOFF/todo/task_wire_get_healthy_connections.md`
- Target: `core/src/transport/health.rs`

## Task 11: custody_deduplicates_same_destination_and_message_id
- File: `HANDOFF/todo/task_wire_custody_deduplicates_same_destination_and_message_id.md`
- Target: `core/src/store/relay_custody.rs`

## Task 12: relay_request_carries_ws13_metadata_when_set
- File: `HANDOFF/todo/task_wire_relay_request_carries_ws13_metadata_when_set.md`
- Target: `core/src/relay/protocol.rs`

## Verification
After all tasks: run `cargo build --workspace` and `cargo test --workspace --no-run`
Report: STATUS: SUCCESS_STOP or list blockers.
