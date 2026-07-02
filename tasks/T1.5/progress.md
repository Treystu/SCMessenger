# T1.5 — BLE L2CAP throughput & fragmentation hardening

**Status:** completed
**Track:** 1 (Native Hardware & Proximity Transport Layer)
**Dependencies:** T5.4
**Blocks:** none

## Technical Context
- Rust framing: `transport/ble/l2cap.rs` (framing/reassembly); Swift `BLEL2CAPManager.swift`; Kotlin `BLEL2CAPManager.kt`
- BLE capability says 512-byte payloads (`abstraction.rs`), but DriftFrames go to 65,536 (`drift/frame.rs` MAX_PAYLOAD) — fragmentation correctness under loss is the survival-critical path

## Implementation
1. Add to `l2cap.rs`: explicit reassembly timeout (drop partial after 30 s)
2. Per-peer reassembly memory cap (e.g. 256 KiB) with `DropReason` accounting into `drift/relay.rs` stats
3. CRC32 verification on reassembled frame (frame.rs already carries CRC — assert it's checked post-reassembly, add if not)
4. Property test: random fragment loss/reorder/duplication never panics and never yields a corrupt-but-accepted frame

## Edge Cases
- iOS L2CAP MTU negotiation differs from Android (iOS up to ~2048-byte SDU typical; Android `BluetoothSocket` L2CAP CoC API 29+)
- Peripheral-role L2CAP listen on Android requires API 29+ — Kotlin must gate with `Build.VERSION` and fall back to GATT characteristic writes (the `gatt.rs` path) below that

## Verification
- [x] `proptest` in `core/src/transport/ble/`: 10k randomized fragment streams, zero panics, corrupt frames always rejected (CRC)
- [x] Memory cap test: oversized partial stream evicted with logged DropReason
