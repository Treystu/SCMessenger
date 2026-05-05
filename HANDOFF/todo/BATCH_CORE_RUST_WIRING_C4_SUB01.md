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


## Sub-batch 1 of 7

1. **blake3_hash** — core/src/dspy/signatures.rs — Wire into DSPy signature verification path
2. **can_forward_for_wasm** — core/src/routing/ — Wire into WASM forwarding decision
3. **can_reach_destination** — core/src/routing/ — Wire into routing reachability check
4. **create_basic** — core/src/routing/ — Wire into default route creation
5. **create_cot** — core/src/routing/ — Wire into chain-of-thought route creation
6. **create_multihop** — core/src/routing/ — Wire into multipath route builder
7. **create_optimizer** — core/src/routing/ — Wire into routing optimization init
8. **evaluate_all_tracked** — core/src/routing/ — Wire into routing evaluation loop
9. **isAtMaxDelay** — core/src/routing/ — Wire into retry delay check
10. **list_endpoints** — core/src/routing/ — Wire into endpoint enumeration
11. **mark_path_failed** — core/src/routing/ — Wire into path failure handler
12. **mark_refresh_failed** — core/src/routing/ — Wire into refresh failure path
13. **negative_cache_stats** — core/src/routing/ — Wire into routing diagnostics
14. **next_refresh_hint** — core/src/routing/ — Wire into refresh scheduler
15. **prune_below** — core/src/routing/ — Wire into path pruning