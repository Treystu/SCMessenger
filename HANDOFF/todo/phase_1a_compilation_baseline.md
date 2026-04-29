# Phase 1A: Compilation Baseline

**Priority:** P0
**Assigned Agent:** rust-coder (glm-5.1:cloud)
**Fallback:** implementer (qwen3-coder-next:cloud)
**Status:** TODO

## Objective
Get `cargo check --workspace` passing with zero errors.

## Pre-Work
Merge conflict markers have been resolved. Verify no remaining conflicts:
```bash
grep -rl "<<<<<<< HEAD" --include="*.rs" --include="*.kt" --include="*.toml" --include="*.gradle" --include="*.swift" .
```
If any found, resolve them before proceeding.

## Tasks
1. Run `cargo check --workspace` and capture all errors
2. Fix each compile error, starting with the most fundamental (lib.rs, module declarations)
3. Verify `cargo test --workspace --no-run` compiles all tests
4. Run `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` and fix all warnings
5. Run `cargo fmt --all -- --check` and fix any formatting issues

## Success Criteria
- `cargo check --workspace` exits 0
- `cargo test --workspace --no-run` exits 0
- `cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments` exits 0
- `cargo fmt --all -- --check` exits 0

## Rules
- All state behind `Arc<RwLock<...>>` (parking_lot), not std::sync
- `IronCore` is the single entry point — do not bypass it
- Error handling: `anyhow` for app errors, `thiserror` for library errors — never `unwrap()` in production paths
- Module boundaries are strictly enforced (see .claude/rules/rust.md)
