# T1.1 — Generalize the FFI proximity-data plane (G3)

**Status:** completed
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** T5.7
**Blocks:** T1.2, T1.4, T1.7

## Technical Context
- `core/src/api.udl` `PlatformBridge` callback interface; `mobile_bridge.rs:1126` (`on_ble_data_received`), `:1249` (`send_ble_packet`)
- Swift `SmartTransportRouter.swift`; Kotlin `SmartTransportRouter.kt`
- Today only BLE bytes can cross the FFI

## Implementation
1. Add to UDL: `enum ProximityTransport { Ble, WifiAware, WifiDirect, Multipeer }`
2. Add `on_proximity_data_received(string peer_id, ProximityTransport transport, bytes data)` and `send_proximity_packet(string peer_id, ProximityTransport transport, bytes data)`
3. Keep the BLE-named methods as thin delegating wrappers (FFI surface gate T5.7 gets a snapshot update)
4. Internally route by `TransportType` (`transport/abstraction.rs:11-22`) so `TransportCapabilities::max_payload_size` per transport (BLE 512 vs WiFiDirect 4096) is enforced at the bridge with explicit `IronCoreError::InvalidInput` on oversize

## Edge Cases
- UniFFI 0.31 callback interfaces are sync and must not block — dispatch ingest onto the global runtime (`mobile_bridge.rs:2267-2297` pattern)
- Duplicate delivery when a peer is reachable over two transports — dedup already exists (`store/dedup.rs`), but verify message-id dedup fires before decrypt cost

## Verification
- [x] New Rust unit tests: oversize payload per transport rejected
- [x] Round-trip via a mock `PlatformBridge` for each enum variant
- [x] T5.7 snapshot updated in same change
- [x] Both binding generators succeed
