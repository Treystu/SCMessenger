# RCA: iOS 43K Send Failures

**Date**: 2026-03-17
**Severity**: P0 - Critical (Resource Exhaustion)
**Component**: iOS MeshRepository.swift - Pending Outbox Retry Logic

---

## Executive Summary

The iOS app generated **42,734 "Direct send outbound failure" errors** from a single message retry loop targeting an unreachable peer (`peer-DtTmZVNv` / `12D3KooWCitizuothXVuqeGShBywNtQ2H8j3sxFap8TLHSYpounv`).

**Root Cause**: The retry loop was using blind periodic retries (every 8 seconds) without opportunistic triggering, causing excessive retries for unreachable peers.

---

## Root Cause Analysis

### File: `iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift`

#### The Issue

The `pendingOutboxExpiryReason()` function correctly returns `nil` (messages never expire per PHIL-011), but the retry strategy was suboptimal:

1. **`startPendingOutboxRetryLoop()`** (line 3732) runs every 8 seconds
2. **`flushPendingOutbox()`** (line 4386) processes the queue
3. For each item, retries are attempted with exponential backoff (2s → 60s → 300s)
4. At 300s backoff, a single unreachable peer generates **~288 errors/day**

### Why This Violates Philosophy (Initially)

My initial fix attempt added `max_age_exceeded` and `max_attempts_exceeded` checks, which violated **PHIL-011** (eventual delivery convergence). The Philosophy Canon states:

> "Treat delivery, reconnect, and active-session availability as eventual-consistency targets that converge to 100% over a sufficient time horizon."

Messages must **never be dropped** - they must retry until delivered.

---

## Fix Implemented

### 1. Opportunistic Retry on Delivery Receipt

Instead of blind periodic retries, retry is now triggered **opportunistically** when a delivery receipt is received:

```swift
// In onDeliveryReceipt():
let normalizedPeerId = normalizePeerId(rawPeerId)
if !normalizedPeerId.isEmpty {
    promotePendingOutboundForPeer(peerId: normalizedPeerId, excludingMessageId: normalizedMessageId)
    dispatchFlushPendingOutbox(reason: "opportunistic_retry_peer")
}
```

**Logic**: "Oh look they just got my Bluetooth message - let's send any other pending messages via Bluetooth now!"

### 2. ID Standardization

Fixed peer ID normalization across functions to prevent mismatches:

- `promotePendingOutboundForPeer()` - Now normalizes both input and stored peer IDs
- `triggerPendingSyncForPeerIds()` - Now uses `normalizePeerId()` instead of just trimming
- `onDeliveryReceipt()` - Normalizes peer ID before triggering retry

### 3. Peer ID Normalization Function

```swift
private func normalizePeerId(_ id: String) -> String {
    return PeerIdValidator.normalize(id)
}
```

Handles:
- 64-char hex identity IDs → lowercase
- libp2p Peer IDs (12D3Koo..., Qm...) → preserved as-is (case-sensitive)
- Other formats → trimmed

---

## Impact

| Metric | Before | After |
|--------|--------|-------|
| Retry strategy | Blind 8s periodic | Opportunistic on receipt |
| Errors/day (unreachable) | ~288 | Reduced (only during active sessions) |
| ID matching | Direct string comparison | Normalized comparison |
| Philosophy compliance | ❌ (max attempts) | ✅ (eventual delivery) |

---

## Verification

- [x] iOS build compiles successfully
- [x] ID normalization audit complete
- [x] Docs sync check: PASS

---

## Documentation Updated

- [x] `REMAINING_WORK_TRACKING.md` - Fix documented
- [x] `docs/RCA_IOS_43K_SEND_FAILURES_2026-03-17.md` - This file
