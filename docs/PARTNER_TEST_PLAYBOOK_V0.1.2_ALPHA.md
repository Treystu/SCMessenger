# SCMessenger v0.1.2-alpha Partner Test Playbook

## Objective
Validate upgrade continuity and messaging reliability across direct and relay paths.

## Required Topology
- One browser client (WASM build).
- Two native clients (iOS/Android/CLI).
- At least one WSS-reachable bootstrap relay.

## Test Matrix
1. Browser (`v0.1.2-alpha`) <-> Native (`v0.1.0`)
2. Browser (`v0.1.2-alpha`) <-> Native (`v0.1.1`)
3. Browser (`v0.1.2-alpha`) <-> Native (`v0.1.2-alpha`)

## Scenarios
1. Upgrade continuity: install previous build, create identity/contacts/messages, upgrade, verify data preserved.
2. Cross-network messaging: sender and receiver on different networks.
3. Relay-only messaging: block direct path and validate fallback delivery.
4. Mid-send interruption: cut connectivity during send and verify eventual delivery/no duplicates.
5. Suspend/resume: background/suspend browser tab and validate reconnect/recovery.

## Required Evidence
- Export diagnostics from each participant:
  - Mobile: `MeshService.export_diagnostics()`
  - Web: `IronCore.exportDiagnostics()`
- Capture message IDs, receipt IDs, and timestamps for both directions.
- Record path-state transitions (`ConnectionPathState`) across each scenario.

## Pass Criteria
- No identity/contact/history loss after upgrade.
- No message loss and no duplicate user-visible message entries.
- Relay fallback succeeds when direct path fails.
- Diagnostics artifacts captured for every failed run.
