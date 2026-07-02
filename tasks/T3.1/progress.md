# T3.1 — Promote phase2_apis into default build

**Status:** completed
**Track:** 3 (Mycorrhizal Routing & Hardware-Aware Heuristics)
**Dependencies:** T5.4
**Blocks:** T3.2, T3.3

## Technical Context
- `routing/reputation.rs` + `routing/multipath.rs` gated behind `phase2_apis` cargo feature
- `transport/mesh_routing.rs` (ReputationTracker, MultiPathDelivery) ships unconditionally — audit overlap between the two reputation systems (`abuse/reputation.rs` is a third, abuse-scoped one)

## Implementation
1. Run full suite with `--features phase2_apis`
2. Reconcile the routing-reputation vs mesh_routing-ReputationTracker duplication (pick the routing-layer one as authoritative for path choice, abuse one stays for blocking)
3. Move `phase2_apis` code into default features (delete the gate or invert to an opt-out)

## Edge Cases
- WASM build must still compile (check the feature isn't accidentally pulling tokio-full into wasm32)
- Multipath duplicate-send interacts with T2.5 ownership rule — multipath counts as ONE owner (the live path) with internal redundancy

## Verification
- [x] `cargo test --workspace --all-features` and `cargo build --target wasm32-unknown-unknown` both green
- [x] `grep -rn "phase2_apis" core/` only in CHANGELOG
- [x] One authoritative routing-reputation source asserted in ARCHITECTURE.md
