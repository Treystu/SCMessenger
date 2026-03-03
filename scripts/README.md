# SCMessenger Scripts Guide

Status: Active  
Last updated: 2026-03-03

This guide covers active launch/debug/verification scripts, with a focus on AI-assisted debugging workflows.

## Fast Start By Goal

1. Full WS12 baseline sanity:
   - `./scripts/verify_ws12_matrix.sh`
2. Live Android+iOS runtime smoke with logs:
   - `./scripts/live-smoke.sh`
3. Relay flap diagnosis:
   - `./scripts/correlate_relay_flap_windows.sh`
   - `./scripts/verify_relay_flap_regression.sh <ios_diagnostics_log>`
4. Delivery/receipt convergence diagnosis:
   - `./scripts/verify_receipt_convergence.sh <android_log> <ios_log>`
5. BLE-only pairing diagnosis:
   - `./scripts/verify_ble_only_pairing.sh <android_log> <ios_log>`
6. Interop/function completeness matrix refresh:
   - `./scripts/generate_interop_matrix.sh`

## 5-Node / Multi-Node Debug Stack

Primary scripts used during 5-node and relay continuity investigations:

1. `scripts/verify_ws12_matrix.sh`
   - Canonical multi-surface verification gate (Rust + Android + iOS parity checks).
2. `scripts/live-smoke.sh`
   - Live interaction harness for Android+iOS runtime checks with synchronized log capture.
3. `scripts/correlate_relay_flap_windows.sh`
   - Correlates iOS relay churn windows with GCP relay logs.
4. `scripts/verify_relay_flap_regression.sh`
   - Deterministic iOS relay dial-loop regression check.
5. `scripts/verify_receipt_convergence.sh`
   - Message-ID convergence validation across Android/iOS diagnostics.
6. `scripts/verify_ble_only_pairing.sh`
   - BLE-only strict-mode behavior and instability checks.

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

1. `scripts/docs_sync_check.sh`
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
