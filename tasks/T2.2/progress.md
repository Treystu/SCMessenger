# T2.2 — Drift sync end-to-end verification under partition (prove the DTN claim)

**Status:** completed
**Track:** 2 (Asynchronous Storage & Delay-Tolerant Networking)
**Dependencies:** T2.1
**Blocks:** T2.3

## Technical Context
- `drift/sync.rs` (SyncSession state machine), `drift/sketch.rs` (IBLT), `drift/store.rs` (CRDT MeshStore), swarm activation at `transport/swarm.rs:3331+`
- Existing test `integration_offline_partition_matrix.rs`

## Implementation
1. New integration test `core/tests/integration_drift_mule.rs` simulating the canonical sneakernet scenario with three in-process nodes A, M(ule), B and a partition harness (no common connectivity between A and B ever)
2. A queues messages for B while only A<->M connected
3. A<->M disconnect, M<->B connect
4. Assert B receives, decrypts (Double Ratchet out-of-order tolerance — `MAX_SKIP_KEYS=64`), and dedup holds when M later re-syncs with A
5. Include IBLT failure path: difference count exceeding sketch capacity must fall back to full-list sync, not silently lose messages

## Edge Cases
- TTL expiry during custody (envelope `expires_in_seconds` from `TtlConfig`) — expired messages must be dropped by M with `DropReason` recorded, never delivered stale
- `SyncRateLimiter` (`drift/rate_limit.rs`) must not starve a short BLE contact window — verify a 10 s contact transfers >= N messages

## Verification
- [x] `cargo test --test integration_drift_mule` green
- [x] Test asserts: delivery, decryption, dedup count == 0 duplicates surfaced to history
- [x] Expired-message drop with reason
- [x] Sync completes within simulated 10 s contact
