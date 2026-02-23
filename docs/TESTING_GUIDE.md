# SCMessenger Testing Guide

This guide reflects the repository state verified on **2026-02-23**.

## Primary Commands

```bash
# Full workspace verification
cargo test --workspace

# Build only
cargo build --workspace

# CLI command surface
cargo run -p scmessenger-cli -- --help
```

## Package-Level Tests

```bash
# Core crate (unit + integration targets declared under core/tests)
cargo test -p scmessenger-core

# CLI crate
cargo test -p scmessenger-cli

# Mobile bindings crate
cargo test -p scmessenger-mobile

# WASM crate (native mode tests in this environment)
cargo test -p scmessenger-wasm
```

## Integration Suites in `core/tests`

```bash
cargo test --test integration_all_phases
cargo test --test integration_e2e
cargo test --test integration_ironcore_roundtrip
cargo test --test integration_nat_reflection
cargo test --test test_address_observation
cargo test --test test_mesh_routing
cargo test --test test_multiport
cargo test --test test_persistence_restart
```

## Latest Verified Results

From `cargo test --workspace`:

- CLI: `17 passed`
- Core unit: `227 passed`, `7 ignored`
- Core integrations (all `core/tests` files): `52 passed`
- Mobile crate: `4 passed`
- WASM crate (native/non-browser path): `24 passed`
- Total: **324 passed, 0 failed, 7 ignored**

## Platform Setup Checks

```bash
./android/verify-build-setup.sh
./iOS/verify-build-setup.sh
```

### Latest verification notes

- Android script passed most checks but failed in this environment because `ANDROID_HOME` is not set.
- iOS script passed, including UniFFI generation and static library compilation.

## Browser/WASM Runtime Tests

Browser-executed tests were not run in this environment because `wasm-pack` is not installed.

Typical command (when toolchain is present):

```bash
wasm-pack test --node wasm/
```

## Warnings and Quality Follow-Ups

- Core integration tests compile with some unused import/variable warnings.
- No failing tests were observed in the verified run.

## Recommended CI Baseline

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
