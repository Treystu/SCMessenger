# Sprint Plan: Optimization Bundle

**Status:** Draft
**Created:** 2026-03-18
**Sprint Goal:** Optimize expensive queries and transport selection for global mesh performance

---

## Executive Summary

This sprint bundles 5 optimization items that collectively address the most expensive operations in SCMessenger's mesh networking stack. The items are interrelated and should be implemented together to maximize synergies.

**Estimated Total LOC:** ~800-1200 LOC across all items
**Risk Level:** Medium (mostly additive optimizations, minimal risk to existing functionality)
**Dependencies:** Some items depend on others (noted below)

---

## Item 1: Contact Lookup & Deduplication

### Problem Statement
Contact lookup uses linear search with `firstOrNull` for peer matching. While n < 100 typically, this becomes a bottleneck as the mesh grows. Additionally, contact deduplication is inconsistent across platforms.

### Current State Analysis
- **File:** [`core/src/routing/local.rs`](core/src/routing/local.rs:86-97) - `LocalCell` uses `HashMap<PeerId, PeerInfo>` (good)
- **File:** [`core/src/contacts_bridge.rs`](core/src/contacts_bridge.rs:95-108) - Uses sled database with key lookup (good)
- **File:** [`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2268-2297) - `canonicalContactId()` uses `contactManager?.list()` for linear search (bad)

### Optimization Strategy

**Phase 1: Add HashMap index in Android/iOS (Low Risk)**
- Add `HashMap<String, Contact>` cache in MeshRepository
- Update cache on add/remove operations
- Use cache for O(1) lookup instead of linear search

**Phase 2: Normalize peer ID matching (Medium Risk)**
- Ensure all platforms use `canonicalContactId()` consistently
- Add index on `public_key` in sled database
- Implement upsert logic to prevent duplicates

### LOC Estimate
- Android: ~80 LOC (cache + lookup optimization)
- iOS: ~80 LOC (mirroring Android)
- Core: ~40 LOC (index helper functions)
- **Total: ~200 LOC**

### Risk Assessment
- **Risk:** Low - Additive changes, existing lookups remain as fallback
- **Testing:** Unit tests for cache consistency, integration tests for lookup speed

### Dependencies
- None (can be implemented independently)

---

## Item 2: Smart Transport Selection

### Problem Statement
The 500ms parallel racing algorithm evaluates transport health scores (success rate + latency weighted) across BLE/WiFi/Relay. This is already implemented but may need tuning.

### Current State Analysis
- **File:** [`android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt`](android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt) - Already implemented
- **File:** [`iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift`](iOS/SCMessenger/SCMessenger/Transport/SmartTransportRouter.swift) - Already implemented

### Optimization Strategy

**Phase 1: Tune existing parameters (Low Risk)**
- Adjust timeout from 500ms to 300ms for faster fallback
- Tune health score weights (currently 70% success, 30% latency)
- Add circuit breaker for consistently failing transports

**Phase 2: Add predictive transport selection (Medium Risk)**
- Track transport success patterns by time of day
- Pre-select likely transport based on historical data
- Implement transport affinity (prefer last successful transport)

### LOC Estimate
- Android: ~60 LOC (parameter tuning + circuit breaker)
- iOS: ~60 LOC (mirroring Android)
- **Total: ~120 LOC**

### Risk Assessment
- **Risk:** Low - Tuning existing algorithm, not changing core logic
- **Testing:** Integration tests for transport fallback timing

### Dependencies
- None (can be implemented independently)

---

## Item 3: Drift Protocol Sync (IBLT)

### Problem Statement
The IBLT (Invertible Bloom Lookup Table) set reconciliation for message sync between peers may be inefficient for large message sets or high-frequency sync.

### Current State Analysis
- **File:** [`core/src/drift/sync.rs`](core/src/drift/sync.rs) - IBLT implementation exists
- **File:** [`core/src/drift/sketch.rs`](core/src/drift/sketch.rs) - Sketch implementation

### Optimization Strategy

**Phase 1: Optimize IBLT parameters (Low Risk)**
- Tune cell count based on expected message set size
- Implement adaptive cell sizing based on sync history
- Add compression for IBLT transmission

**Phase 2: Implement delta sync (Medium Risk)**
- Track last sync timestamp per peer
- Only sync messages since last successful sync
- Fall back to full IBLT sync only when delta fails

### LOC Estimate
- Core: ~150 LOC (parameter tuning + delta sync)
- **Total: ~150 LOC**

### Risk Assessment
- **Risk:** Medium - Changes to sync protocol could affect message delivery
- **Testing:** Extensive integration tests for sync correctness

### Dependencies
- None (can be implemented independently)

---

## Item 4: Multi-Transport Peer Discovery

### Problem Statement
BLE scanning + WiFi Direct + mDNS + relay discovery all run in parallel with deduplication. This can cause resource contention and duplicate processing.

### Current State Analysis
- **File:** [`android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt`](android/app/src/main/java/com/scmessenger/android/transport/SmartTransportRouter.kt) - Handles transport coordination
- **File:** [`core/src/transport/manager.rs`](core/src/transport/manager.rs:969-982) - Has deduplication tests

### Optimization Strategy

**Phase 1: Implement discovery coordination (Medium Risk)**
- Add discovery coordinator to prevent redundant scans
- Implement backoff for failed discovery attempts
- Add discovery budget (max concurrent discoveries)

**Phase 2: Optimize deduplication (Low Risk)**
- Use bloom filter for fast duplicate detection
- Implement discovery result caching
- Add peer deduplication at transport layer

### LOC Estimate
- Android: ~100 LOC (coordinator + deduplication)
- iOS: ~100 LOC (mirroring Android)
- Core: ~50 LOC (bloom filter helper)
- **Total: ~250 LOC**

### Risk Assessment
- **Risk:** Medium - Changes to discovery could affect peer visibility
- **Testing:** Integration tests for discovery timing and deduplication

### Dependencies
- Item 2 (Smart Transport Selection) - should be implemented first

---

## Item 5: Message History Pagination

### Problem Statement
Loading conversation history with potential for large datasets can cause memory pressure and slow UI rendering.

### Current State Analysis
- **File:** [`core/src/store/history.rs`](core/src/store/history.rs) - History storage
- **File:** [`android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:2885-2892) - `getConversation()` loads all messages

### Optimization Strategy

**Phase 1: Implement pagination API (Low Risk)**
- Add `getConversationPaginated(peerId, offset, limit)` to core
- Implement cursor-based pagination for consistent ordering
- Add total count for UI progress indicators

**Phase 2: Implement lazy loading (Medium Risk)**
- Load only visible messages in UI
- Implement message prefetching for scroll
- Add message caching for recently viewed conversations

### LOC Estimate
- Core: ~80 LOC (pagination API)
- Android: ~60 LOC (lazy loading)
- iOS: ~60 LOC (lazy loading)
- **Total: ~200 LOC**

### Risk Assessment
- **Risk:** Low - Additive changes, existing API remains
- **Testing:** Unit tests for pagination correctness, UI tests for scroll performance

### Dependencies
- None (can be implemented independently)

---

## Implementation Order

Based on dependencies and risk assessment:

1. **Item 1: Contact Lookup & Deduplication** (Independent, Low Risk)
2. **Item 2: Smart Transport Selection** (Independent, Low Risk)
3. **Item 5: Message History Pagination** (Independent, Low Risk)
4. **Item 3: Drift Protocol Sync** (Independent, Medium Risk)
5. **Item 4: Multi-Transport Peer Discovery** (Depends on Item 2, Medium Risk)

---

## Sprint Capacity

Assuming 2-week sprint with 2 developers:
- **Week 1:** Items 1, 2, 5 (low risk, independent)
- **Week 2:** Items 3, 4 (medium risk, some dependencies)

---

## Verification Plan

### Unit Tests
- Contact lookup cache consistency
- Transport selection algorithm correctness
- IBLT sync correctness
- Discovery deduplication accuracy
- Pagination API correctness

### Integration Tests
- End-to-end message delivery with optimized components
- Transport fallback timing
- Discovery timing and deduplication
- History pagination performance

### Performance Benchmarks
- Contact lookup latency (target: < 1ms)
- Transport selection latency (target: < 100ms)
- IBLT sync time (target: < 500ms for 100 messages)
- Discovery time (target: < 2s for all transports)
- History load time (target: < 100ms for 50 messages)

---

## Risk Mitigation

### Rollback Strategy
- All changes are additive or tunable
- Feature flags for new optimizations
- Existing code paths remain as fallback

### Monitoring
- Add metrics for optimization effectiveness
- Track cache hit rates
- Monitor transport selection patterns
- Measure sync performance

---

## Success Criteria

1. **Contact Lookup:** < 1ms for 99% of lookups
2. **Transport Selection:** < 100ms for 95% of selections
3. **IBLT Sync:** < 500ms for 100 messages
4. **Discovery:** < 2s for all transports combined
5. **History Load:** < 100ms for 50 messages

---

## Appendix: No-Go Items

All items are feasible for this sprint. No items should be excluded.

---

*Document generated for Sprint Planning - SCMessenger Optimization Bundle*
