# TASK: D-05 Unwrap/Panic Hardening (v1.0.0 Robustness)

Status: DISPATCH-READY (parallel x2: FFI/startup + crypto/storage)
Model: Qwen CODER tier
Scope: Bounded hardening (~60 sites, Result/logged-default pattern)

## Objective

Replace ~60 `unwrap()` / `panic!()` calls with proper error handling (Result types + logged defaults). Priority zones: FFI boundary, startup path, crypto, storage.

## Strategy

**Zone 1 (FFI boundary + Startup):** mobile_bridge.rs + iron_core.rs init paths
- FFI boundary violations = return error to caller (Result)
- Startup errors = log + default (non-critical) or bail (critical path)
- Use anyhow::Context for error wrapping

**Zone 2 (Crypto + Storage):** crypto module + store module  
- Critical invariant failures = keep panic, add `// SAFETY:` comment explaining why
- Non-critical decoding errors = Result + log
- Cryptographic operations with fallback logic = Result + default

## Implementation Pattern

```rust
// Before:
let key = something().unwrap(); // UNSAFE

// After (non-critical):
let key = match something() {
    Ok(k) => k,
    Err(e) => {
        tracing::warn!("failed to load key: {}", e);
        DEFAULT_KEY // or return Err
    }
};

// After (critical invariant):
let key = something().expect("SAFETY: key must exist at this point (verified by load_keys)")
// OR if panic truly required:
// SAFETY: This path is reached only after initialization_guard check (line 123)
panic!("key missing after guard");
```

## Scope

Identify all `unwrap()` / `panic!()` via grep:
```bash
grep -rn "unwrap()\|panic!" core/src/ --include="*.rs" | grep -v test | grep -v "// SAFETY"
```

Categorize by zone:
1. **mobile_bridge.rs** + iron_core.rs init (~20 sites) → Result/logged-default
2. **crypto/** (~15 sites) → Result or documented panic
3. **store/** (~10 sites) → Result or documented panic  
4. **Other** (~15 sites) → Context-dependent (read each)

## Success Criteria

- Diff applies cleanly via `--mode diff --apply --verify "cargo check --workspace"`
- `cargo test --workspace --no-run` green (compile gate)
- All existing tests pass (no functional regressions)
- Critical panics documented with `// SAFETY:` comment
- Non-critical paths return Result or logged-default

## Files to Modify

- `core/src/mobile_bridge.rs` (primary FFI boundary)
- `core/src/iron_core.rs` (startup/initialization)
- `core/src/crypto/*.rs` (as needed)
- `core/src/store/*.rs` (as needed)

## Estimate
600 LOC, ~90 min total (dispatch as 2x parallel if large)

## Parallel Sharding (Optional)
If diff becomes large (>300 lines):
- **Worker 1:** FFI boundary + startup (mobile_bridge.rs + iron_core.rs init)
- **Worker 2:** Crypto + storage (crypto/ + store/ modules)

Each worker operates independently on their zone; diffs merge cleanly (no overlap).

## Review Gate
None (hardening task). Verify compile + all tests pass.

## Execution
Dispatch to Qwen CODER with `--mode diff --apply --verify "cargo check --workspace"`.

Single dispatch or split if needed.

## Handoff
Move this file to `HANDOFF/done/` ON COMPLETION via `mv` command.
