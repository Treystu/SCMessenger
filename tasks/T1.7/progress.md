# T1.7 — Transport escalation policy unification (SmartTransportRouter parity)

**Status:** completed
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** T1.1
**Blocks:** none

## Technical Context
- Escalation logic exists in THREE places: Rust `transport/escalation.rs`, Swift `SmartTransportRouter.swift`, Kotlin `SmartTransportRouter.kt`
- Risk: divergent decisions (e.g., Swift prefers Multipeer while Rust expects BLE), causing dual-send waste

## Implementation
1. Make Rust authoritative: expose `recommended_transport(peer_id) -> ProximityTransport` through the FFI (consumes `TransportCapabilities` + `TransportHealthMonitor` from `transport/health.rs` + DeviceProfile battery state)
2. Native routers demote to executors: they report link availability up (`on_network_changed` extension or new `on_transport_availability(transport, available)`) and obey downward picks

## Edge Cases
- Native layer has information Rust lacks mid-flight (e.g., L2CAP channel just died) — allow native veto with mandatory report-back so the health monitor learns
- Don't break existing BLE-only flows during migration: feature-flag via `MeshSettings`

## Verification
- [x] Rust unit tests: given (battery, link set, payload size) -> deterministic transport pick matching a documented decision table in `ARCHITECTURE.md`
- [x] Grep proves Swift/Kotlin routers no longer contain independent preference ordering (only availability checks)
