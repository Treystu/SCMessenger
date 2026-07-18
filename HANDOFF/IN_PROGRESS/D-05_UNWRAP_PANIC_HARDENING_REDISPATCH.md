# TASK: D-05 Unwrap/Panic Hardening — RE-DISPATCH (Scope-Locked)

Status: REDISPATCH-READY (Fusion Lite verdict incorporated)
Model: Deepseek-v4-pro (upgraded for domain reasoning)
Budget: $0.10 (increased, per farm-sim criticality)
Tier: CODER-THINK hybrid

## Critical Scope Enforcement

Fusion Lite Panel (unanimous) ruled:
- Receipt FFI encoding/decoding is OUT OF SCOPE (belongs in A-04/A-05)
- Root cause of scope creep: Task confusion due to overlapping "receipt" terminology
- Solution: Provide explicit file list + IN/OUT OF SCOPE sections

### IN SCOPE — MUST FIX:

1. mobile_bridge.rs (FFI boundary: ALL exported methods must return Result)
   - Lines ~3000-3100 (tokio runtime init)
   - Any unwrap() / expect() / panic!() in FFI-facing functions
   - FFI boundary violations must propagate errors to caller, never panic

2. iron_core.rs (startup & initialization)
   - Lines ~1-500 (init paths, key loading)
   - Replace startup panics with error returns
   - Initialization failures should not crash, should return Err

3. core/src/crypto/ (all files)
   - Replace panics with checked logic OR documented invariant panics
   - If panic is truly unreachable/catastrophic, add `// SAFETY:` comment explaining why
   - Use `debug_assert!` for logic bugs, not `panic!()` for recoverable errors

4. core/src/store/ (all files)
   - Database/deserialization errors must return Result
   - Handle missing data gracefully (log + default or return Err)
   - No panics on data format issues

### OUT OF SCOPE — DO NOT TOUCH:

- [NO] Do NOT add new FFI struct definitions (e.g., Receipt dictionary)
- [NO] Do NOT modify api.udl (UDL is for A-04/A-05)
- [NO] Do NOT create new test files for data round-tripping
- [NO] Do NOT implement platform-specific logic (A-04/A-05 territory)
- [NO] Do NOT add new wrapper functions like encode_receipt_from_components

## Error Handling Patterns Required

### FFI Boundary Pattern:
```rust
// BEFORE (FORBIDDEN):
pub fn ffi_exported_fn() {
    let session = get_session().unwrap(); // FFI panic
    ...
}

// AFTER (REQUIRED):
pub fn ffi_exported_fn() -> Result<SomeType, String> {
    let session = get_session()
        .ok_or_else(|| "Session not found".to_string())?;
    ...
    Ok(result)
}
```

### Startup Path Pattern:
```rust
// BEFORE:
fn init_keys() {
    let keys = load_keys().expect("keys must exist"); // Panic on init failure
}

// AFTER:
fn init_keys() -> Result<Keys> {
    let keys = load_keys()
        .map_err(|e| anyhow::anyhow!("Key initialization failed: {}", e))?;
    Ok(keys)
}
```

### Crypto Invariant Pattern:
```rust
// BEFORE:
let secret = decrypt().unwrap(); // Unclear if recoverable

// AFTER (Option 1 - Recoverable):
let secret = decrypt()
    .map_err(|e| {
        tracing::warn!("Decryption failed (recoverable): {}", e);
        // Continue with fallback
        DEFAULT_SECRET
    })?;

// AFTER (Option 2 - Unreachable Invariant):
// SAFETY: This code is reached only after key_validation() call (line 123).
// If decrypt fails here, it's a logic bug, not a runtime error.
let secret = decrypt()
    .expect("decrypt must succeed after validation");
```

### Storage Pattern:
```rust
// BEFORE:
let blob = store.get("key").unwrap(); // Panic on missing data

// AFTER:
let blob = store.get("key")
    .map_err(|e| anyhow::anyhow!("Store access failed: {}", e))?
    .ok_or_else(|| anyhow::anyhow!("Key not found in store"))?;
```

## Verification Steps

1. Compile Gate:
   ```bash
   cargo check --workspace
   cargo test --workspace --no-run
   ```

2. Grep Verification:
   ```bash
   grep -rn "\.unwrap()\|\.expect(" core/src/ --include="*.rs" \
     | grep -v "test\|SAFETY\|// "
   ```
   Result should be EMPTY in target files (mobile_bridge, iron_core, crypto/*, store/*)

3. Diff Review:
   - All changes must be error-handling transformations
   - No new data structures, no new test files
   - No UDL changes

## Success Criteria

- [PASS] Diff applies cleanly via `--mode diff --apply --verify "cargo check --workspace"`
- [PASS] Zero new `.unwrap()` or `.expect()` in target files post-fix
- [PASS] All new `panic!()` have `// SAFETY:` comments explaining invariant
- [PASS] No new function definitions or UDL modifications
- [PASS] No test files for receipt encoding/decoding
- [PASS] `cargo test --workspace --no-run` passes (compile gate)
- [PASS] All existing tests still pass (no regressions)

## Handoff

Move this file to `HANDOFF/done/D-05_UNWRAP_PANIC_HARDENING_REDISPATCH.md` ON COMPLETION via `mv` command.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `IN_PROGRESS/` to `done/`. If you do not move the file, the Orchestrator assumes you failed.

## Diff Mode

Expected output format:
```
--- a/core/src/mobile_bridge.rs
+++ b/core/src/mobile_bridge.rs
@@ -XXXX,Y +XXXX,Z @@
-    .unwrap()
+    .ok_or_else(|| anyhow::anyhow!("..."))?
```

Apply via: `git apply --recount`

## Execution

Route to Deepseek-v4-pro with `--mode diff --apply --verify "cargo check --workspace"`.
Budget: $0.10. No override on model selection.
