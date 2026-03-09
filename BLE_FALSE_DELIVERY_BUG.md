# BLE False Delivery Status Bug
**Date**: 2026-03-09 14:36 UTC  
**Status**: 🔴 CRITICAL - False Delivery Confirmations  
**Priority**: P0 - Data Integrity

## Problem

Android marks messages as "delivered" based on BLE transport ACK, even when core mesh network delivery fails. iOS never receives these messages.

## Evidence

Message `71a748f5-20bf-453e-b4f9-b7481a80f3a1`:
```
✓ BLE delivery succeeded (outcome=accepted)
✓ Receipt received via BLE
✓ Marked as "delivered"
✗ Core network delivery FAILED
✗ Relay circuit delivery FAILED
→ iOS NEVER received the message via mesh
```

## Root Cause

**File**: `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt:3319-3322`

```kotlin
outcome = if (localAcked) "local_accepted_no_core_ack" else "failed",
return DeliveryAttemptResult(acked = localAcked, routePeerId = null)
```

BLE transport ACK (`localAcked`) is treated as full delivery acknowledgment.

## Fix Required

### 1. Don't mark as delivered until BOTH transports confirm:
```kotlin
if (localAcked && coreAcked) {
    return DeliveryAttemptResult(acked = true)
} else {
    return DeliveryAttemptResult(acked = false)
}
```

### 2. Keep retrying core even after BLE succeeds:
```kotlin
if (bleAcked && !coreAcked) {
    // Keep in retry queue for core delivery
    enqueuePendingOutbound(messageId, bleSatisfied = true)
}
```

### 3. Distinguish delivery methods in UI:
- Single checkmark: BLE only
- Double checkmark: Core mesh confirmed

## Impact

**Critical**: Users cannot trust delivery status. Messages appear delivered but recipient never sees them.

**Must fix before release.**
