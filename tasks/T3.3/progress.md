# T3.3 — StoreAndCarry decision → Drift handoff wiring (close routing→DTN loop)

**Status:** completed
**Track:** 3 (Mycorrhizal Routing & Hardware-Aware Heuristics)
**Dependencies:** T2.5, T3.1
**Blocks:** T3.4

## Technical Context
- `NextHop::StoreAndCarry` (`routing/engine.rs:18-36`) exists, and drift is live (Track 2) — verify the actual code path from `route_message` returning StoreAndCarry to envelope landing in `MeshStore` custody
- Grep suggests `iron_core.rs:602` converts envelopes but the decision->custody linkage needs proof

## Implementation
1. Trace and (if missing) implement: send path consults `RoutingEngine.route_message()`
2. On `StoreAndCarry`, invoke the T2.5 custody handoff
3. On `RouteDiscovery{hint}`, trigger neighborhood route request (`routing/global.rs` `request_route`) with `timeout_budget.rs` phase budget, falling back to StoreAndCarry on exhaustion
4. This task is *verification-first*: write the failing test, then add only the missing glue

## Edge Cases
- Routing engine optional (`iron_core.rs:3277` shows `Option<RoutingEngine>`) — when None (e.g., WASM minimal), send path must default to direct-or-custody, never panic
- Priority field (u8) must map to drift `RelayProfile` priority thresholds consistently (one mapping table, tested)

## Verification
- [x] Integration test: node with zero peers sends -> message in drift custody with correct TTL/priority
- [x] Peer appears later -> delivered (this is T2.2's scenario driven through the public send API rather than drift internals — both must pass)
