# SCMessenger Testing Guide

Status: Active
Last updated: 2026-03-07

Current release line: `v0.2.0` is the active alpha baseline. Planned workstreams `WS13` and `WS14` remain `v0.2.1` follow-up scope.

## Standard Local Validation Ladder

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo build --workspace
cargo test --workspace
./scripts/docs_sync_check.sh
```

## WS12 Reproducible Validation Command

```bash
ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh
```

Notes:

1. Set `ANDROID_HOME` to your local SDK path before running Android parity checks.
2. Set `SCM_SKIP_ANDROID=1` to skip Android parity checks.
3. Set `SCM_SKIP_IOS=1` to skip iOS parity checks.
4. iOS parity checks require macOS when `SCM_SKIP_IOS` is not set.

## Mandatory Workspace Gates

```bash
cargo test --workspace --no-run
cargo test --workspace
```

## WS12 Deterministic Offline/Partition Suites

```bash
cargo test -p scmessenger-core --test integration_offline_partition_matrix
cargo test -p scmessenger-core --test integration_retry_lifecycle
cargo test -p scmessenger-core --test integration_receipt_convergence
cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored
```

## WS12 Role/Fallback Parity Suites

```bash
# Desktop/WASM role-mode parity
cargo test -p scmessenger-wasm test_desktop_role_resolution_defaults_to_relay_only_without_identity
cargo test -p scmessenger-wasm test_desktop_relay_only_flow_blocks_outbound_message_prepare

# Android role + fallback parity
cd android && ANDROID_HOME=/path/to/android/sdk \
  ./gradlew :app:testDebugUnitTest \
    --tests com.scmessenger.android.test.RoleNavigationPolicyTest \
    --tests com.scmessenger.android.data.MeshRepositoryTest

# iOS transport + role parity
bash ./iOS/verify-local-transport.sh
bash ./iOS/verify-role-mode.sh
```

## Latest Verified Results (2026-03-03)

From `cargo test --workspace`:

1. CLI: `13 passed`
2. Core unit: `265 passed`, `7 ignored`
3. Core integrations: `52 passed`, `10 ignored`
4. Mobile crate: `4 passed`
5. WASM crate (native mode): `33 passed`
6. Aggregate: **367 passed, 0 failed, 17 ignored**

Additional WS12 parity checks:

1. Android targeted parity tests: **pass**
2. iOS local transport fallback tests: **pass**
3. iOS role-mode parity source checks: **pass**

WS12.5 re-validation (2026-03-03):

1. `cargo test -p scmessenger-core --test integration_offline_partition_matrix` — **pass**
2. `cargo test -p scmessenger-core --test integration_retry_lifecycle` — **pass**
3. `cargo test -p scmessenger-core --test integration_relay_custody -- --include-ignored` — **pass**

WS12.6 closeout validation (2026-03-03):

1. `cargo test --workspace --no-run` — **pass**
2. `cargo test -p scmessenger-core relay_custody -- --nocapture` — **pass**
3. `cargo test -p scmessenger-core convergence_marker -- --nocapture` — **pass**

## CI Enforcement

Current GitHub Actions map for repository-controlled validation:

1. `.github/workflows/ci.yml`
   - repo hygiene/path governance
   - docs sync guard
   - Rust fmt/clippy/build/tests
   - deterministic core integration suites
   - WASM checks
   - Android targeted validation
   - iOS verification
2. `.github/workflows/docker-test-suite.yml`
   - heavy containerized validation
   - manual/scheduled + `main` push only
3. `.github/workflows/docker-publish.yml`
   - Docker image publish flow
   - `main` push + manual dispatch only
4. `.github/workflows/release.yml`
   - CLI-only release artifacts on tags/manual dispatch
