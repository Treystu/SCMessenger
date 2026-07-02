# T1.2 — Wire WifiAwareTransport into the live core (de-orphan, G1)

**Status:** partial
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** T1.1
**Blocks:** T1.3

## Technical Context
- `transport/wifi_aware.rs` is complete (state machine, data paths, RSSI bandwidth model, `wire_discovery_callback`) but unreferenced
- Consumer seam: `MeshService.start()` (`mobile_bridge.rs:227`) and the swarm event loop (`transport/swarm.rs`)
- Settings flag already exists: `MeshSettings.wifi_aware_enabled` (api.udl:222 block)

## Implementation
1. Implement a production `WifiAwarePlatformBridge` whose methods forward over the T1.1 FFI plane (publish/subscribe/data-path requests become `PlatformBridge` calls; preferred: fold into generic `transport_control(transport, op, payload)` to keep the surface small)
2. Instantiate `WifiAwareTransport` inside `MeshService.start()` when `wifi_aware_enabled && bridge.is_available()`
3. On `DataPathInfo` confirmation (IP+port), dial that socket via the existing libp2p TCP transport (`SwarmHandle.dial` path used by `SwarmBridge::dial`, `mobile_bridge.rs:2428`) so Noise/Yamux/Gossipsub ride the Aware data path with zero new protocol code
4. PMK derivation: blake3-derive a 32-byte PMK from the DarkBLE group key (`transport/ble/beacon.rs`) so only mesh members can join data paths

## Edge Cases
- Android-only (iOS has no Wi-Fi Aware API — bridge `is_available()` must return false on iOS; `MultipeerTransport.swift` is the iOS analog)
- Android requires `NEARBY_WIFI_DEVICES` (API 31+) / fine-location (<=30) at runtime — already in manifest, but Kotlin bridge must check grant state before `is_available()=true`
- Aware sessions die on Wi-Fi toggle/Doze: `on_network_changed` (existing PlatformBridge callback) must tear down `DataPathActive` state

## Verification
- [x] `cargo test -p scmessenger-core wifi_aware` (existing 15 tests still pass)
- [x] New integration test with `MockWifiAwareBridge` proving: discovery event -> `create_data_path` -> dial issued to `SwarmHandle` (asserted via real mutual `get_peers()` connectivity on both swarms, mDNS disabled so only the deliberate dial can connect them)
- [ ] Kotlin unit test (Robolectric) for permission-gated availability

## Update (2026-07-01)
Adding real assertions to `integration_wifi_aware.rs` exposed two genuine
bugs that made the discovery->dial path silently non-functional:
1. `on_wifi_aware_peer_discovered`'s spawned task called the sync
   `SwarmBridge::dial()` (which does `rt.block_on`) from within an
   already-running tokio task — a guaranteed "Cannot start a runtime from
   within a runtime" panic on every real discovery event, swallowed because
   the task's `JoinHandle` was never awaited.
2. The confirmation channel used a blocking `std::sync::mpsc::Receiver::
   recv_timeout()` inside an async fn, which could starve the shared tokio
   runtime under concurrent discovery.

Fixed both: added `SwarmBridge::dial_async` (used from
`on_wifi_aware_peer_discovered` and `on_wifi_direct_connection_info`'s
spawned tasks instead of the sync `dial()`), and switched
`PlatformWifiAwareBridge`'s data-path channel to `tokio::sync::oneshot`
with `tokio::time::timeout`. The test now enables `wifi_aware_enabled`
(off by default) and runs a second real libp2p swarm with mDNS disabled
so the assertion can only pass via the deliberate dial path.
