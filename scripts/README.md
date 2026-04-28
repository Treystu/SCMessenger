# SCMessenger Scripts Guide

Status: Active
Last updated: 2026-04-11

This guide covers active launch/debug/verification scripts, with a focus on AI-assisted debugging workflows.

## Testing Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          TESTING PYRAMID                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 4: Full Mesh Integration (run5.sh)                                  │
│    └── 5-node live topology: GCP + OSX + Android + iOS Device + iOS Sim   │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 3: Live Verification Loop (run5-live-feedback.sh)                   │
│    └── Iterative build/deploy + run5 + verifier gates                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 2: Platform Smoke Tests (live-smoke.sh, verify_ws12_matrix.sh)      │
│    └── Android+iOS runtime checks, WS12 parity validation                  │
├─────────────────────────────────────────────────────────────────────────────┤
│  Level 1: Unit/Integration Tests (cargo test --workspace)                  │
│    └── Core crypto, transport, receipts, offline/partition matrices        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## CLI user install (daemon)

| Goal | Command |
|------|---------|
| Copy release CLI to `~/.local/bin` + emit systemd (Linux) or LaunchAgent (macOS) unit | `./scripts/install.sh` (after `cargo build --release -p scmessenger-cli`) |
| Windows: copy to `%USERPROFILE%\.local\bin` | `powershell -File scripts/install.ps1` |

Optional: `SCMESSENGER_BIN=/path/to/scmessenger-cli` (bash) or `$env:SCMESSENGER_BIN` (PowerShell) to install a non-default artifact.

## Fast Start By Goal

| Goal | Command |
|------|---------|
| Full WS12 baseline sanity | `./scripts/verify_ws12_matrix.sh` |
| Live 5-node mesh capture | `./run5.sh --time=5` |
| Live 5-node with strict gates | `./scripts/run5-live-feedback.sh --step=<id> --time=5` |
| Live Android+iOS smoke | `./scripts/live-smoke.sh` |
| Relay flap diagnosis | `./scripts/correlate_relay_flap_windows.sh` |
| Delivery convergence check | `./scripts/verify_receipt_convergence.sh <android_log> <ios_log>` |
| BLE-only pairing diagnosis | `./scripts/verify_ble_only_pairing.sh <android_log> <ios_log>` |
| Interop matrix refresh | `./scripts/generate_interop_matrix.sh` |

## 5-Node / Multi-Node Debug Stack

### Primary: `run5.sh` (Unified 5-Node Harness)

The canonical 5-node mesh test harness combining the best features from all variants:

**Nodes:**
1. GCP — headless relay (Docker on scmessenger-bootstrap)
2. OSX — headless relay (local cargo binary)
3. Android — full node (via adb)
4. iOS Device — full node (physical device via devicectl)
5. iOS Simulator — full node (simulator via simctl)

**Usage:**
```bash
./run5.sh --time=5           # Run for 5 minutes (default)
./run5.sh --time=10 --update # 10 minutes, rebuild headless nodes
./run5.sh --restore-on-exit  # Stop nodes we launched on exit
```

**Key Features:**
- `set -euo pipefail` for safe error handling
- `--restore-on-exit` flag to control cleanup behavior
- Passive log collection (never force-stops pre-existing apps)
- Comprehensive post-run mesh analysis with transport evidence
- Live status ticker showing peer discovery and relay activity
- Timestamped logs preserved in `logs/5mesh/<timestamp>/`

**Log Outputs:**
- `gcp.log` — GCP relay docker logs (streamed via SSH)
- `osx.log` — OSX relay stdout
- `android.log` — Android logcat (filtered for mesh components)
- `ios-device.log` — iOS app console output
- `ios-device-system.log` — iOS system logs (BLE, Multipeer)
- `ios-sim.log` — iOS simulator logs

### Iterative: `scripts/run5-live-feedback.sh`

Wraps `run5.sh` with strict phase gates for fix validation:

```bash
./scripts/run5-live-feedback.sh --step=fix-123 --time=5 --attempts=3
```

**Phase Gates:**
1. Mobile build/deploy (optional, `--skip-mobile-deploy`)
2. 5-node run with `--update`
3. Log health gate (all logs present, minimum line counts)
4. Pair matrix gate (all 20 directed visibility edges)
5. Crash/fatal marker scan
6. Deterministic verifiers:
   - `relay_flap_regression`
   - `ble_only_pairing`
   - `receipt_convergence` (warn-only by default, `--require-receipt-gate`)
   - `delivery_state_monotonicity`

**Output:** Per-attempt evidence bundles in `logs/live-verify/<step>_<timestamp>/`

### Platform Smoke: `scripts/live-smoke.sh`

Quick Android+iOS runtime validation without full 5-node overhead:

```bash
./scripts/live-smoke.sh                    # Default: 300s, both devices
DURATION_SEC=60 ./scripts/live-smoke.sh    # 60 second smoke
IOS_TARGET=simulator ./scripts/live-smoke.sh  # Simulator only
```

### WS12 Matrix: `scripts/verify_ws12_matrix.sh`

Canonical multi-surface verification (Rust + Android + iOS parity):

```bash
./scripts/verify_ws12_matrix.sh
SCM_SKIP_ANDROID=1 ./scripts/verify_ws12_matrix.sh  # Skip Android
SCM_SKIP_IOS=1 ./scripts/verify_ws12_matrix.sh      # Skip iOS
```

## Deterministic Verifier Scripts

| Script | Purpose | Input |
|--------|---------|-------|
| `verify_relay_flap_regression.sh` | iOS relay dial-loop regression | iOS device log |
| `verify_ble_only_pairing.sh` | BLE-only strict-mode checks | Android + iOS logs |
| `verify_receipt_convergence.sh` | Message-ID convergence | Android + iOS diagnostics |
| `verify_delivery_state_monotonicity.sh` | Delivery state ordering | Android + iOS diagnostics |
| `correlate_relay_flap_windows.sh` | Relay churn correlation | iOS + GCP logs |

## Launch / Control Scripts

1. `scripts/scm.sh`
   - Local process lifecycle helper (`start|stop|restart|status|logs`)
2. `verify_integration.sh`
   - Alias wrapper that executes `scripts/verify_ws12_matrix.sh`
3. `verify_simulation.sh`
   - Docker-based simulation verifier with prerequisite fail-fast checks
4. `run_comprehensive_network_tests.sh`
   - Extended Docker/NAT/traffic-control simulation workflow
5. `clean_all_devices.sh`
   - Device/simulator cleanup helper used before fresh multi-device runs

## iOS Device/Install Helpers

1. `iOS/build-device.sh`
2. `iOS/install-device.sh`
3. `iOS/install-sim.sh`
4. `iOS/verify-test.sh`
5. `iOS/verify-local-transport.sh`
6. `iOS/verify-role-mode.sh`

## Infrastructure / Deployment Helpers

1. `scripts/deploy_gcp_node.sh`
2. `scripts/test_gcp_node.sh`
3. `scripts/get-node-info.sh`
4. `scripts/deploy_to_device.sh`

## Repo / Governance Helpers

1. `scripts/docs_sync_check.sh` (Unix / Git Bash) or `scripts/docs_sync_check.ps1` (Windows PowerShell)
2. `scripts/repo_audit.sh`
3. `scripts/verify_branch_merges.sh`
4. `scripts/delete_merged_branches.sh`
5. `scripts/generate_interop_matrix.sh`

## Python Log Analysis

1. `analyze_mesh.py`
   - Live mesh monitor for 5-node test logs (`logs/5mesh/*`).

## Historical / Deprecated

Moved to `scripts/archive/`:
- `run5_trip.sh.deprecated` — Superseded by unified `run5.sh` with `--restore-on-exit` flag

Historical debug parsers in `reference/historical/`:
- `parse_connections.py`, `snapshot_mesh.py`, `snapshot_mesh2.py`
- Treat as reference only; promote to `scripts/` if needed again.

## Launch / Control Scripts

1. `scripts/scm.sh`
   - Local process lifecycle helper (`start|stop|restart|status|logs`).
2. `verify_integration.sh`
   - Alias wrapper that executes `scripts/verify_ws12_matrix.sh`.
3. `verify_simulation.sh`
   - Docker-based simulation verifier with prerequisite fail-fast checks.
4. `run_comprehensive_network_tests.sh`
   - Extended Docker/NAT/traffic-control simulation workflow.
5. `clean_all_devices.sh`
   - Device/simulator cleanup helper used before fresh multi-device runs.

## iOS Device/Install Helpers

1. `iOS/build-device.sh`
2. `iOS/install-device.sh`
3. `iOS/install-sim.sh`
4. `iOS/verify-test.sh`
5. `iOS/verify-local-transport.sh`
6. `iOS/verify-role-mode.sh`

## Infrastructure / Deployment Helpers

1. `scripts/deploy_gcp_node.sh`
2. `scripts/test_gcp_node.sh`
3. `scripts/get-node-info.sh`
4. `scripts/deploy_to_device.sh`

## Repo / Governance Helpers

1. `scripts/docs_sync_check.sh` (Unix / Git Bash) or `scripts/docs_sync_check.ps1` (Windows PowerShell)
2. `scripts/repo_audit.sh`
3. `scripts/verify_branch_merges.sh`
4. `scripts/delete_merged_branches.sh`
5. `scripts/generate_interop_matrix.sh`

## Python Log Analysis

1. `analyze_mesh.py`
   - Live mesh monitor for 5-node test logs (`logs/5mesh/*`).

## Historical Debug Parsers / One-Off Artifacts

These were intentionally moved out of repo root to reduce active-surface noise:

1. `reference/historical/parse_connections.py`
2. `reference/historical/snapshot_mesh.py`
3. `reference/historical/snapshot_mesh2.py`
4. `reference/historical/control_android_logs.txt`
5. `reference/historical/android_panics.txt`
6. `reference/historical/android_crash_logs_buffer.txt`
7. `reference/historical/continue_working_on_this.md`

Usage rule:

- Treat historical files as reference only.
- If a historical helper becomes active again, promote it into `scripts/` with a clear name and usage section in this README.
