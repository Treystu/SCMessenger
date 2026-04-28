# Local Progress Checkpoint - 2026-04-27

## Status
- Core and CLI successfully compiling.
- Wiring for B1 (Core API) batch is nearly complete.
- Successfully implemented and wired:
    - `active_paths` (UniFFI + Core)
    - `get_available_paths` (CLI + API + Bridge)
    - `random_port` (UniFFI + Core)
    - `can_bootstrap_others` (Swarm + Core)
    - `best_relays` (Swarm + Core)

## Changes
- `core/src/lib.rs`: Added `multipath` manager, wired `active_paths`, `random_port`, and fixed `history_manager`.
- `core/src/api.udl`: Added `DeliveryPath` dictionary and updated `IronCore` interface.
- `core/src/mobile_bridge.rs`: Updated `SwarmBridge` and `MeshService` initialization.
- `core/src/store/relay_custody.rs`: Implemented `RelayRegistry::list_active`.
- `core/src/transport/swarm.rs`: Added `CanBootstrapOthers` command and handle methods.
- `cli/src/api.rs`: Added `get_transport_paths_via_api` and JSON RPC handler.
- `cli/src/main.rs`: Added `transport-paths` command and updated `status` display.
- `cli/src/server.rs`: Added `/api/transport/available-paths` endpoint.

## Next Tasks
- [ ] `task_wire_get_history_manager.md`
- [ ] `task_wire_get_history_via_api.md`
- [ ] `task_wire_add_discovered_peer.md`
