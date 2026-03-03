# SCMessenger Testing Guide

Status: Active  
Last updated: 2026-03-03

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

## CI Enforcement

Primary CI gates for WS12 parity lock:

1. `.github/workflows/ci.yml` `check-core`: deterministic offline/partition matrix tests.
2. `.github/workflows/ci.yml` `check-wasm`: desktop role-mode parity tests.
3. `.github/workflows/ci.yml` `check-android`: Android role/fallback parity tests.
4. `.github/workflows/ci.yml` `check-ios`: `iOS/verify-test.sh` now runs both local transport and role-mode parity checks.
