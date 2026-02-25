# SCMessenger v0.1.2-alpha Parity Audit

## Scope
- Core parity baseline on shared protocols and topics.
- API parity contract for connection-path state + diagnostics.
- Browser transport parity through wasm libp2p swarm.

## Locked Contracts Verified In-Repo
- Protocol IDs remain unchanged (`/sc/message/1.0.0`, `/sc/address-reflection/1.0.0`, `/sc/relay/1.0.0`, `/sc/ledger-exchange/1.0.0`, `/sc/id/1.0.0`).
- Default topics remain unchanged (`sc-lobby`, `sc-mesh`).
- Web path now uses libp2p swarm runtime with wasm transport parity.

## API Contract Parity
- `ConnectionPathState` is now part of the UniFFI contract.
- `MeshService::export_diagnostics()` provides structured JSON for partner support and issue triage.
- WASM exports include `getConnectionPathState()` and `exportDiagnostics()`.

## Test Evidence
- `cargo test --workspace` passes locally.
- `cargo check -p scmessenger-core --target wasm32-unknown-unknown` passes.
- `cargo check -p scmessenger-wasm --target wasm32-unknown-unknown` passes.

## Remaining External Validation (Release Signoff)
- Browser/native matrix runs against `v0.1.0`, `v0.1.1`, and current head over live infra.
- Relay-only and cross-network runbook evidence from partner environments.
- ACK-safe path-switching verification under induced network transitions.
