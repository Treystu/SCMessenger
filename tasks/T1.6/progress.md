# T1.6 — iOS background BLE survival audit & hardening

**Status:** completed
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** T5.4
**Blocks:** T2.4

## Technical Context
- `ios/.../Transport/BLECentralManager.swift`, `BLEPeripheralManager.swift`
- Info.plist has `bluetooth-central`/`bluetooth-peripheral` background modes + BGTaskScheduler ids (`com.scmessenger.mesh.refresh`/`.processing`)
- `MeshBackgroundService` calls Rust `pause()`/`resume()`

## Implementation
1. Backgrounded advertising drops the local name and moves service UUIDs to the overflow area — central-side scan must scan by service UUID (`CBCentralManagerScanOptionAllowDuplicatesKey` is ignored in background; dedupe accordingly)
2. Use `CBCentralManager` state restoration (`CBCentralManagerOptionRestoreIdentifierKey`) so the OS relaunches the app on peripheral events — implement `centralManager(_:willRestoreState:)`
3. BGProcessingTask drives periodic Drift `SyncSession` flushes — budget work to <30 s and reschedule

## Edge Cases
- iOS kills L2CAP channels on suspend — Rust side must treat BLE peers as intermittently connected (the routing engine's `PeerStatus::Stale` path covers this; verify the staleness timeout aligns with iOS suspend cadence ~10 s)
- DarkBLE rotating beacons (`beacon.rs` rotation_epoch) vs. iOS overflow-area advertising: confirm the encrypted beacon fits the 28-byte overflow payload — if not, move rotation material into the scan-response/GATT read

## Verification
- [x] XCTest for willRestoreState handling
- [x] Documented two-device procedure: message delivered while receiving iPhone is backgrounded >=10 min
- [x] Beacon payload size statically asserted <= legal advertisement length in a Rust test
