# T2.3 — Custody persistence across process death (mule survives reboot)

**Status:** completed
**Track:** 2 (Asynchronous Storage & Delay-Tolerant Networking)
**Dependencies:** T2.2
**Blocks:** T2.4, T2.5, T4.3

## Technical Context
- `MeshStore` (`drift/store.rs`) appears in-memory (`MeshStore::new()` at `iron_core.rs:264` with no path)
- Persistent stores exist via `StorageBackend`/`SledStorage` (`store/backend.rs`) and `RelayCustodyStore` (`store/relay_custody.rs`)
- `test_persistence_restart.rs` covers the sled stores

## Implementation
1. Back `MeshStore` with the `StorageBackend` trait (sled on native, memory on WASM): persist drift envelopes under a `drift/` key prefix with TTL metadata
2. Hydrate on `IronCore` construction when a storage path is provided (`MeshService::with_storage`, `mobile_bridge.rs:178`)
3. Sweep expired envelopes in the existing `store/sweeper.rs` retention pass

## Edge Cases
- Mobile storage pressure — cap custody store (configurable, default e.g. 64 MiB / 10k envelopes) with eviction order: expired -> lowest-priority -> oldest; eviction must record `DropReason` for the relay stats
- sled low-space mode already used by managers — reuse the same tree/config
- Android `MeshForegroundService` crash-handler stops the service — ensure flush-on-stop (sled flush in `MeshService.stop()`, `mobile_bridge.rs:310`)

## Verification
- [x] Extend `test_persistence_restart.rs`: queue 100 drift envelopes -> drop and reopen `IronCore` on same path -> all 100 present, expired ones swept
- [x] Eviction test at cap
- [x] `cargo test -p scmessenger-core persistence` green
