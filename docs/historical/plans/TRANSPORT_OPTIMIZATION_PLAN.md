# Transport Optimization & Multi-Path Delivery Plan

**Status:** Implementation In Progress
**Date:** 2026-03-10
**Impact:** Critical - Cellular↔WiFi, WiFi↔BLE transitions

## Root Causes Identified

### 1. Slow BLE Transition
- **Current:** Sequential transport selection with long timeouts (5000ms WiFi, 3500ms BLE)
- **Issue:** Waits for WiFi timeout before trying BLE
- **Impact:** 5+ second delays when switching from cellular/WiFi to BLE-only

### 2. No Parallel Transport Attempts
- **Current:** `best_transport_for_peer()` picks ONE transport, waits for timeout
- **Issue:** If WiFi NAT fails, must wait full timeout before BLE attempt
- **Impact:** Missed delivery windows, poor UX

### 3. Limited Transport Visibility
- **Current:** Minimal logging of transport selection rationale
- **Issue:** Cannot diagnose why BLE vs WiFi chosen
- **Impact:** Debugging difficulty

### 4. Android Mesh UI Not Scrolling
- **Current:** `Column` with `verticalScroll` + `forEach` for peer list
- **Issue:** Large peer lists cause layout issues
- **Impact:** Cannot see all discovered nodes

## Implementation Plan

### Phase 1: Parallel Transport Delivery (Core)
**LoC:** ~80

**Changes:**
1. Add `send_to_peer_parallel()` in `transport/manager.rs`
   - Try all available transports simultaneously
   - Return on first ACK
   - Cancel others on success
2. Add transport selection logging with scores
3. Reduce initial timeouts: WiFi 2000ms → 1500ms, BLE 3500ms → 1000ms

**Files:**
- `core/src/transport/manager.rs` (~40 LoC)
- `core/src/api.udl` (~10 LoC - expose new API)
- `core/src/mobile_bridge.rs` (~30 LoC - wire up)

### Phase 2: Fast Retry with Exponential Backoff Optimization
**LoC:** ~40

**Changes:**
1. Reduce initial backoff from 2^n to aggressive: [500ms, 1s, 2s, 4s, 8s, then 60s]
2. Add "fast lane" for BLE when WiFi times out
3. Enhance retry logging with transport type + reason

**Files:**
- `android/app/.../MeshRepository.kt` (~40 LoC)

### Phase 3: Android UI - LazyColumn for Mesh Tab
**LoC:** ~50

**Changes:**
1. Replace `Column` + `forEach` with `LazyColumn` + `items()`
2. Add header/footer composables
3. Ensure proper scroll state management

**Files:**
- `android/app/.../DashboardScreen.kt` (~50 LoC)

### Phase 4: Enhanced Transport Logging
**LoC:** ~60

**Changes:**
1. Log transport selection with scores in `MeshRepository`
2. Log each parallel attempt with result/latency
3. Add transport type to delivery status UI

**Files:**
- `android/app/.../MeshRepository.kt` (~40 LoC)
- `iOS/SCMessenger/.../MessageRepository.swift` (~20 LoC)

### Phase 5: Message Status Synchronization Fix
**LoC:** ~30

**Changes:**
1. Ensure delivery receipts properly update both sender and receiver
2. Fix iOS "forwarding" status lag
3. Fix Android message disappearing bug

**Files:**
- `android/app/.../MeshRepository.kt` (~15 LoC)
- `iOS/SCMessenger/.../MessageRepository.swift` (~15 LoC)

## Total Estimated LoC: ~260

## Success Criteria

1. **BLE Transition < 2s** when WiFi unavailable
2. **Parallel Delivery** attempts all transports, succeeds on first ACK
3. **Mesh Tab Scrolling** works with 100+ peers
4. **Complete Transport Logs** showing selection rationale, all attempts, latencies
5. **Accurate Message Status** on both iOS and Android

## Testing

1. WiFi→BLE transition: Disable WiFi mid-conversation, verify < 2s to BLE delivery
2. Cellular↔WiFi NAT: Send from cellular to WiFi device, verify parallel attempts in logs
3. Large mesh: Add 50+ peers to Dashboard, verify scrolling to bottom
4. Status accuracy: Send 10 messages iOS↔Android, verify all show "delivered" when acked

## Risks

- **Parallel sends** may increase battery drain (mitigated: cancel on first ACK)
- **Faster retries** may cause network congestion (mitigated: still exponential after attempt 6)
- **LazyColumn** may have edge cases with dynamic peer lists (mitigated: testing)

## Dependencies

- None - all changes within existing transport/UI layers

---
*This optimization directly addresses user-reported slow BLE transitions and NAT traversal issues.*
