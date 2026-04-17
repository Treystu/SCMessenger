# P0_CORE_001: Drift Protocol Completion

**Priority:** P0 (Critical Performance)
**Platform:** Core/Rust
**Status:** Open
**Source:** Completed task audit - Drift Protocol not wired to production

## Problem Description
The Drift Protocol (8 files, all unit-tested) is COMPLETELY DORMANT and not wired to any production path. Current implementation uses legacy bincode format instead of optimized Drift format.

## Missing Integration Points
1. **Message send/receive path wrapping** - Not using DriftEnvelope/DriftFrame
2. **SyncSession triggering** - Not activated on PeerDiscovered events  
3. **PolicyEngine integration** - Not connected to device state
4. **Drift compression** - LZ4 compression not applied in send path

## Performance Impact
- ❌ No message compression (bandwidth inefficient)
- ❌ No binary encoding (larger message sizes)
- ❌ No efficient sync protocol (poor mesh performance)
- ❌ No adaptive policies (static behavior)

## Implementation Required
1. Replace `message::encode_envelope()` with `DriftEnvelope` serialization
2. Add DriftFrame wrapping in transport layer
3. Integrate LZ4 compression for large payloads
4. Trigger SyncSession on PeerDiscovered events
5. Wire PolicyEngine to device state monitoring
6. Update all message preparation and reception paths

## Key Files to Modify
- `core/src/lib.rs` - prepare_message() and receive_message() paths
- `core/src/transport/swarm.rs` - Message send/receive handling
- `core/src/message/codec.rs` - Replace bincode with Drift encoding
- `core/src/drift/mod.rs` - Integration entry points

## Expected Outcome
- 40-60% bandwidth reduction from compression
- Fixed 186-byte envelope overhead (vs variable bincode)
- Efficient mesh synchronization with IBLT
- Adaptive policies based on device state
- Complete Drift Protocol activation

## Verification Requirements

### MUST PASS Before Marking Complete:
- [ ] `scripts/verify_task_completion.sh drift` returns SUCCESS
- [ ] All 9 Drift files integrated into production code
- [ ] Legacy bincode encoding completely replaced
- [ ] Compression active in message preparation  
- [ ] SyncSession triggered on PeerDiscovered events
- [ ] Cross-platform consistency verified (Android, iOS, WASM, CLI)
- [ ] Performance benchmarks show 40-60% bandwidth reduction
- [ ] Integration tests verify actual Drift format usage

### Verification Script:
```bash
# Run this to verify complete integration
./scripts/verify_task_completion.sh drift
```