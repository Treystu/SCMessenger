# BATCH: Core Rust + WASM + CLI Wiring (C4)

You are a worker implementing wiring tasks. Each task requires you to:
1. Find the target function
2. Identify where it should be called
3. Wire it into the production call path
4. Verify compilation with `cargo check --workspace`
5. Move the task file from HANDOFF/todo/ to HANDOFF/done/

CRITICAL: You are forbidden from considering a task 'complete' until you execute the mv or Rename-Item command to move the task markdown file from todo/ (or IN_PROGRESS/) to done/. If you do not move the file, the Orchestrator assumes you failed.

## Build Verification
After wiring, run: `cargo check --workspace`

## Tasks — Group A: Core Infrastructure Wiring (routing, relay, transport)


## Sub-batch 2 of 7

1. **read_with_timeout** — core/src/routing/ or store — Wire into timeout read path
2. **refresh_delegate_routes** — core/src/routing/ — Wire into delegate refresh
3. **register_endpoint** — core/src/routing/ — Wire into endpoint registration
4. **register_path** — core/src/routing/ — Wire into path registration
5. **resolveDeliveryState** — core/src/store/ or drift — Wire into message delivery resolution
6. **run_optimization** — core/src/routing/ — Wire into optimization trigger
7. **send_message_status** — core/src/drift/ — Wire into message status reporting
8. **should_advance** — core/src/routing/ — Wire into ratchet advancement check
9. **start_refresh** — core/src/routing/ — Wire into refresh initiation
10. **timeout_budget_summary** — core/src/routing/ — Wire into TTL budget diagnostics
11. **touch_endpoint** — core/src/routing/ — Wire into endpoint liveness update
12. **unregister_endpoint** — core/src/routing/ — Wire into endpoint removal
13. **update_keepalive** — core/src/routing/ — Wire into keepalive update path
14. **transport_type_to_routing_transport** — core/src/routing/ — Wire into transport type mapping

## Tasks — Group B: Identity & Contact Wiring

15. **contact_new_has_no_last_known_device_id** — core/src/store/contacts — Wire into new contact creation validation