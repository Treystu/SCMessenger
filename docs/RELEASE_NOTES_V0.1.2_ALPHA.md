# SCMessenger v0.1.2-alpha Release Notes

## Highlights
- Browser networking now uses libp2p swarm on wasm (`with_wasm_bindgen` + websocket-websys transport).
- Deprecated web relay receive-loop path is retained only as a compatibility shim.
- Added canonical connection-path state contract (`ConnectionPathState`) to UniFFI API.
- Added structured diagnostics export for support and partner bug reports.
- Hardened persistence setup with explicit storage schema marker and sub-store layout.

## Compatibility
- Protocol IDs and default topics unchanged to preserve interoperability.
- Target matrix for partner validation:
  - Browser `v0.1.2-alpha` <-> Native `v0.1.0`
  - Browser `v0.1.2-alpha` <-> Native `v0.1.1`
  - Browser `v0.1.2-alpha` <-> Native current head

## Upgrade and Data Continuity
- Core storage now initializes dedicated `identity/`, `outbox/`, and `inbox/` stores under app storage root.
- Storage schema marker file (`SCHEMA_VERSION`) is written/read for forward migration safety.

## Developer Notes
- New mobile API surface:
  - `MeshService.get_connection_path_state()`
  - `MeshService.export_diagnostics()`
- New WASM API surface:
  - `getConnectionPathState()`
  - `exportDiagnostics()`

## Known Remaining Release Signoff Work
- External live-infra matrix signoff is still required for final alpha acceptance.
