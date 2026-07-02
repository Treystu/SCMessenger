# T5.3 — Add rustfmt + clippy + cargo-deny baseline

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.2
**Blocks:** T5.4

## Technical Context
- No `rustfmt.toml`, `clippy.toml`, or `deny.toml` exist
- 53k LOC core

## Implementation
1. Add `rustfmt.toml` (default style, `edition = "2021"`)
2. Add workspace-level `[workspace.lints.clippy]` (warn-level: `all`)
3. Allow existing violations via one `cargo clippy --fix` pass or targeted `allow`s
4. Add `deny.toml` (advisories + licenses: MIT/Apache-2.0/BSD allowlist)

## Verification
- [x] `cargo fmt --check` exits 0
- [x] `cargo clippy --workspace --all-features -- -D warnings` exits 0
- [x] `cargo deny check` exits 0
