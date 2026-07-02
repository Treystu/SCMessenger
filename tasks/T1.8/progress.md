# T1.8 — BLE peripheral advertising on desktop (investigation)

**Status:** deferred (documented limitation, not attempted)
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** none
**Blocks:** none

## Technical Context
- `cli/src/ble_mesh.rs::run_ble_peripheral_advertising` logs and sleeps for an hour; it has never done real advertising.
- `cli/src/ble_mesh.rs`'s own module doc already states the design intent: "btleplug is central-oriented on desktop OSes; the CLI does not expose a full peripheral GATT server here. Mobile/native peers remain peripherals; this node scans, connects, and ingests notify payloads." I.e. by design, mobile/native devices advertise (peripheral role) and the desktop CLI scans/connects (central role) — see `run_ble_central_ingress`, which is real and working. The gap this task is about is narrower than "BLE doesn't work on desktop": it's specifically that two desktop CLI instances can't discover each other over BLE, since neither one advertises.

## Investigation
Asked to investigate feasibility of a platform-specific fallback before attempting any implementation, given `btleplug` has no cross-platform peripheral/GATT-server API on desktop.

**Linux — BlueZ D-Bus (`org.bluez.GattManager1` + `LEAdvertisingManager1`)**
Most tractable of the three: the CLI already depends on `dbus`/`dbus-tokio`/`bluez-async` transitively (via btleplug's own Linux central-mode backend), so the D-Bus transport is already in the dependency tree. But BlueZ's GATT-server API itself is not a small surface — it requires implementing `org.freedesktop.DBus.ObjectManager` plus `org.bluez.GattService1`/`GattCharacteristic1`/`GattDescriptor1` object trees that BlueZ introspects and calls back into for read/write/notify, plus separately registering advertisement data via `LEAdvertisingManager1`. This is a genuine multi-hundred-line D-Bus server implementation, not a thin wrapper — BlueZ's GATT server API is widely described (e.g. in BlueZ's own `doc/gatt-api.txt`) as one of the fiddlier corners of the D-Bus API surface, and no existing Rust crate in this dependency tree provides it (`bluez-async` is scan/central-oriented like btleplug).

**macOS — CoreBluetooth (`CBPeripheralManager`)**
No existing Rust binding for macOS peripheral mode exists in the ecosystem (btleplug doesn't implement it either). Would require either raw Objective-C interop (`objc`/`objc2` + `block2` crates) to drive `CBPeripheralManager` directly, or a small compiled Swift/ObjC shim linked into the binary. More ecosystem work than Linux, since there's no existing lower-level protocol (like BlueZ's D-Bus surface) to bind against from Rust — it's Cocoa runtime interop from scratch.

**Windows — WinRT (`GattServiceProvider` + `BluetoothLEAdvertisementPublisher`)**
Feasible via Microsoft's official `windows` crate (WinRT projection), which does cover these APIs. Requires pulling in that (large) dependency and writing a third, separate platform-specific implementation, plus the async/WinRT-callback boilerplate typical of that crate.

## Recommendation
**Document as a known v1.0.0 limitation rather than implement.** Reasoning:

1. Each platform is an independent, substantial implementation (best-guess: low-to-mid multi-week effort per platform for a correct, robust GATT server + advertising registration, not a quick patch) — matches the "flag rather than guess at a full implementation if multi-week" instruction for this task.
2. None of the three could be verified in this environment: there is no physical BLE radio available here, and BLE peripheral-mode code (advertisement timing, GATT server callback correctness, OS-specific quirks around characteristic permissions/MTU) is exactly the kind of thing that looks plausible in a code review and fails silently on real hardware. Shipping unverified peripheral-mode code would be worse than the current honest stub — same category of hardware dependency as the physical two/three-device tests this pass was told not to attempt.
3. It doesn't block the actual mesh use case as designed: mobile/native devices already advertise and desktop CLI nodes already scan/connect to them (`run_ble_central_ingress` is real). The gap is desktop-to-desktop BLE discovery specifically, which is a narrower scenario than "BLE doesn't work."
4. If this becomes a real requirement later, start with Linux (BlueZ D-Bus): the transport dependency is already present, and it's the only one of the three with a documented (if verbose) protocol to implement against rather than needing OS-runtime FFI bridging.

## Implementation (if picked up later)
1. Linux first: implement a `GattApplication` D-Bus object tree (`ObjectManager` + `GattService1`/`GattCharacteristic1`) matching the UUIDs in `core/src/transport/ble/gatt.rs`, register it via `GattManager1.RegisterApplication`, and register advertisement data via `LEAdvertisingManager1.RegisterAdvertisement`.
2. Gate behind a runtime capability check (`GattManager1`/`LEAdvertisingManager1` availability varies by BlueZ version/experimental-features flags) with a graceful fallback to the current stub behavior + log message.
3. macOS/Windows as separate follow-up tasks given the larger, independent FFI surfaces each requires.

## Verification
- [ ] Not attempted — needs physical BLE hardware per platform to verify advertising actually works and is discoverable by a real central, which was unavailable in this environment.
