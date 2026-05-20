# P0_JSONRPC_PARITY_EXPANSION_001

**Status:** VERIFIED REMAINING WORK
**Agent:** rust-coder
**Budget:** 5400s (EXECUTE tier)
**Phase:** v0.2.1 cross-platform parity
**Source:** AUDIT_ANDROID_WINDOWS_INTEROP_PARITY_2026-05-20.md

---

## Verified Gap

The JSON-RPC bridge (`core/src/wasm_support/rpc.rs`) exposes only 25 methods, while Android via UniFFI accesses ~120+ methods from `IronCore` and `MeshRepository`. This creates a severe parity gap where the WASM/web thin-client cannot perform basic operations that Android handles natively.

**Missing JSON-RPC Methods (verified against Android MeshRepository.kt and CLI commands):**

| Missing Method | Android Equivalent | CLI Equivalent | Priority |
|----------------|-------------------|----------------|----------|
| `export_identity_backup` | `MeshRepository.exportIdentityBackup()` | `identity export` | P0 |
| `import_identity_backup` | `MeshRepository.restoreIdentityFromBackup()` | `identity import` | P0 |
| `set_nickname` | `MeshRepository.setNickname()` | `identity set-name` | P0 |
| `get_audit_log` | `MeshRepository.exportAuditLog()` | `audit export` | P0 |
| `set_privacy_config` | `MeshRepository.applyPrivacyConfig()` | `config privacy` | P0 |
| `get_pending_message_requests` | `MeshRepository.getPendingMessageRequests()` | N/A | P0 |
| `accept_message_request` | N/A (new) | N/A | P0 |
| `reject_message_request` | N/A (new) | N/A | P0 |
| `search_messages` | N/A (Android missing too) | `history --search` | P1 |
| `get_history_stats` | `MeshRepository.getHistoryStats()` | `history-stats` | P1 |
| `mark_message_delivered` | `MeshRepository.markMessageDelivered()` | `history-mark-delivered` | P1 |
| `delete_message` | `MeshRepository.getMessage()` + manual | `history-delete` | P1 |
| `get_diagnostics` | `MeshRepository.exportDiagnostics()` | N/A | P1 |
| `run_self_test` | N/A | `test` | P2 |

## Scope

### Part A: Add ClientIntents

Extend `ClientIntent` enum in `core/src/wasm_support/rpc.rs` with the missing intents above.

### Part B: Add parse_intent branches

Extend `parse_intent()` match block to deserialize each new method's params.

### Part C: Add dispatch handlers in CLI server

Extend `cli/src/server.rs::handle_jsonrpc_request()` to dispatch each new intent to the appropriate `IronCore` method.

### Part D: Verification

1. `cargo check --workspace` passes
2. `cargo test --workspace --no-run` passes
3. WASM target check: `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes

## File Targets

- `core/src/wasm_support/rpc.rs` [EDIT]
- `cli/src/server.rs` [EDIT]
- `core/src/wasm_support/` (may need new response types) [MAY EDIT]

## Build Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo check --workspace
cargo check -p scmessenger-wasm --target wasm32-unknown-unknown
cargo test --workspace --no-run
```

## Acceptance Gates

1. All P0 intents listed above have `ClientIntent` variants
2. All P0 intents have `parse_intent` deserialization branches
3. All P0 intents have server-side dispatch handlers
4. Build verification commands pass

## CRITICAL

You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.
