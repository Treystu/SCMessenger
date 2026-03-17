# SCMessenger Testing Guide

Status: Active
Last updated: 2026-03-16

Current release line: `v0.2.0` is the active alpha baseline. Planned workstreams `WS13` and `WS14` remain `v0.2.1` follow-up scope.

## Testing Pyramid

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          TESTING PYRAMID                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 4: Full Mesh Integration (run5.sh)                                  │
│    └── 5-node live topology: GCP + OSX + Android + iOS Device + iOS Sim   │
│         Tests: peer discovery, relay circuits, transport diversity          │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 3: Live Verification Loop (run5-live-feedback.sh)                   │
│    └── Iterative build/deploy + run5 + verifier gates                      │
│         Tests: fix validation, regression gates, crash markers             │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 2: Platform Smoke Tests (live-smoke.sh, verify_ws12_matrix.sh)      │
│    └── Android+iOS runtime checks, WS12 parity validation                  │
│         Tests: install, launch, basic connectivity                         │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 1: Unit/Integration Tests (cargo test --workspace)                  │
│    └── Core crypto, transport, receipts, offline/partition matrices        │
│         Tests: algorithm correctness, state machines, edge cases           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Standard Local Validation Ladder

```bash
# Level 1: Core validation
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo build --workspace
cargo test --workspace

# Level 2: Platform parity (optional, requires devices)
./scripts/verify_ws12_matrix.sh

# Documentation sync
./scripts/docs_sync_check.sh
```

## Level 2: Platform Validation

### WS12 Reproducible Validation Command

```bash
ANDROID_HOME=/path/to/android/sdk ./scripts/verify_ws12_matrix.sh
```

Notes:

1. Set `ANDROID_HOME` to your local SDK path before running Android parity checks.
2. Set `SCM_SKIP_ANDROID=1` to skip Android parity checks.
3. Set `SCM_SKIP_IOS=1` to skip iOS parity checks.
4. iOS parity checks require macOS when `SCM_SKIP_IOS` is not set.

### Live Smoke Test (Android + iOS)

```bash
./scripts/live-smoke.sh                    # Default: 300s, both devices
DURATION_SEC=60 ./scripts/live-smoke.sh    # 60 second smoke
IOS_TARGET=simulator ./scripts/live-smoke.sh  # Simulator only
```

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

## Level 2: WS12 Role/Fallback Parity Suites

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

## Level 3: Live Verification Loop

For validating fixes with strict phase gates:

```bash
./scripts/run5-live-feedback.sh --step=fix-123 --time=5 --attempts=3
```

**Phase Gates:**
1. Mobile build/deploy (optional: `--skip-mobile-deploy`)
2. 5-node run with `--update`
3. Log health gate (all logs present, minimum line counts)
4. Pair matrix gate (all 20 directed visibility edges)
5. Crash/fatal marker scan
6. Deterministic verifiers:
   - `relay_flap_regression`
   - `ble_only_pairing`
   - `receipt_convergence` (warn-only by default, `--require-receipt-gate`)
   - `delivery_state_monotonicity`

## Level 4: Full 5-Node Mesh Test

```bash
./run5.sh --time=5           # Run for 5 minutes (default)
./run5.sh --time=10 --update # 10 minutes, rebuild headless nodes
./run5.sh --restore-on-exit  # Stop nodes we launched on exit
```

**Nodes:**
1. GCP — headless relay (Docker)
2. OSX — headless relay (local cargo)
3. Android — full node (via adb)
4. iOS Device — full node (physical)
5. iOS Simulator — full node (simulator)

**Post-run analysis includes:**
- Own ID extraction per node
- Transport evidence (BLE, direct, relay, WiFi)
- Visibility matrix (directed peer discovery)
- Log file health summary

## Deterministic Verifier Scripts

| Script | Purpose | Input |
|--------|---------|-------|
| `verify_relay_flap_regression.sh` | iOS relay dial-loop regression | iOS device log |
| `verify_ble_only_pairing.sh` | BLE-only strict-mode checks | Android + iOS logs |
| `verify_receipt_convergence.sh` | Message-ID convergence | Android + iOS diagnostics |
| `verify_delivery_state_monotonicity.sh` | Delivery state ordering | Android + iOS diagnostics |
| `correlate_relay_flap_windows.sh` | Relay churn correlation | iOS + GCP logs |

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
