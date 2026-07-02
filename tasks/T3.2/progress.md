# T3.2 — Hardware-aware routing cost function (battery/charging/motion → route choice)

**Status:** completed
**Track:** 3 (Mycorrhizal Routing & Hardware-Aware Heuristics)
**Dependencies:** T3.1
**Blocks:** T3.3, T3.4

## Technical Context
- `DeviceProfile` (battery_pct, is_charging, has_wifi, MotionState) reaches Rust via `update_device_state` (`mobile_bridge.rs:839`) and feeds `AutoAdjustEngine` (BLE scan intervals, relay budgets) — but NOT the `RoutingEngine`'s next-hop choice (`routing/engine.rs:128`, `route_message`)
- Peers' device states partially propagate via gossip (`neighborhood.rs` gateway info)

## Implementation
1. Extend `RoutingDecision` scoring: when choosing among `alternatives`, weight gateway/relay candidates by advertised energy class
2. Add a 2-bit energy class (Charging/High/Low/Critical) to the neighborhood gossip record (`NeighborhoodGossip` — bump gossip schema version with backward-compat decode)
3. Cost function: `cost = base_hop_cost * energy_multiplier[class] * (1/confidence)` with multipliers {Charging:0.5, High:1.0, Low:2.0, Critical:8.0}
4. A Critical-battery peer is chosen only when it is the sole route

## Edge Cases
- Gossip schema versioning — old peers send records without energy class: default to High (neutral), never reject
- Energy class is adversarially spoofable — cap its influence (multiplier bounds above) and let `ReputationTracker` delivery-failure feedback dominate over time
- Do not leak precise battery % over the mesh (privacy) — 2-bit class only

## Verification
- [x] Unit tests in `routing/engine.rs`: given equal-hop alternatives, charging peer wins; critical peer chosen only as sole route
- [x] Gossip decode of old-schema record defaults correctly (round-trip test both directions)
- [x] `integration_mycorrhizal_routing.rs` extended with an energy-skewed topology asserting route selection
