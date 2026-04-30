# Agent Task: Test Compilation and Execution Gate

**Model:** deepseek-v4-flash:cloud (fast reasoning)
**Priority:** P1 — Verify test suite compiles and basic tests pass

## Context
The workspace compiles cleanly (`cargo check --workspace`, `cargo clippy`, `cargo fmt` all pass). Integration tests were recently fixed for the `prepare_message` signature change (String args). The test build requires dlltool on PATH for the GNU toolchain.

## Steps

1. Set PATH for the GNU toolchain:
   ```bash
   export PATH="/c/msys64/mingw64/bin:$PATH"
   ```

2. Compile all tests:
   ```bash
   PATH="/c/msys64/mingw64/bin:$PATH" cargo test --workspace --no-run
   ```
   If this fails due to dlltool, try the MSVC toolchain:
   ```bash
   cargo +stable-x86_64-pc-windows-msvc test -p scmessenger-core --no-run
   ```

3. Run the core unit tests:
   ```bash
   PATH="/c/msys64/mingw64/bin:$PATH" cargo test -p scmessenger-core --test integration_ironcore_roundtrip
   PATH="/c/msys64/mingw64/bin:$PATH" cargo test -p scmessenger-core --test integration_contact_block
   ```

4. If tests pass, report success. If any test fails, investigate and fix.

## Key Notes
- `prepare_message` now takes `(String, String, MessageType, Option<TtlConfig>)` not `(&str, &str, ...)`
- The test files have been updated to match but verify anyway
- Do NOT modify `core/src/` files unless a test failure requires it

## File Domains
- `core/tests/` (test files only)
- `.cargo/config.toml` (if toolchain changes needed)

## Completion
Write COMPLETION marker to `.claude/agents/<your_id>/COMPLETION` with STATUS, CHANGED_FILES, BUILD_STATUS.