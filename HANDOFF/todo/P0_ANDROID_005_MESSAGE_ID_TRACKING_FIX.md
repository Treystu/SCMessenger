# P0_ANDROID_005: Message ID Tracking Fix

**Priority:** P0 CRITICAL
**Platform:** Android
**Status:** REJECTED - QA Gatekeeper returned to todo
**Routing Tags:** [REQUIRES: TECH_DEBT] [REQUIRES: FINALIZATION]
**QA Review Date:** 2026-04-20

## QA Gatekeeper Findings

### COMPILATION ERRORS (FIXED by Gatekeeper)
1. `MeshRepository.kt`: Missing `Permissions` import (removed but referenced at 4 sites) — restored
2. `MeshRepository.kt`: `FallbackTransport` import removed (class doesn't exist) — replaced with `Permissions`
3. `MeshRepository.kt`: `handleCancellationCascade()` reassigning read-only `coroutineContext` — fixed to cancel-only

### IMPLEMENTATION GAPS (Still need fresh coder)
1. **`executeWithCancellationHandling()` and `executeNonCancellable()` tagged P0_ANDROID_006, not P0_ANDROID_005** — cross-task contamination from autonomous agent
2. **`MessageTracking.isCorrupted()` detection logic unclear** — method exists but criteria not documented; fresh coder should verify corruption detection is meaningful
3. **`detectAndRecoverMessageTracking()` never called from any periodic/trigger path** — recovery only happens if explicitly invoked, not automatically
4. **`logRetryStormDetection()` never called from any periodic path** — storm detection is passive
5. **Multiple P0 task code intermixed in MeshRepository.kt** — P0_ANDROID_004, 005, 006, 007, NETWORK_001 all added code to the same file, creating coupling and confusion

### What IS Working
- ✅ `MessageTracking` data class with corruption detection and recovery
- ✅ `ConcurrentHashMap` for thread-safe tracking cache
- ✅ `Mutex`-protected `incrementAttemptCount()`
- ✅ `getMessageIdTracking()` gracefully recreates missing entries
- ✅ Exponential backoff with 30s cap in `getRetryDelay()`
- ✅ `MAX_RETRY_ATTEMPTS = 12` with `shouldRetryMessage()`
- ✅ `logMessageDeliveryAttempt()` and `logRetryStormDetection()` present

### What STILL NEEDS Work
- Wire `detectAndRecoverMessageTracking()` into a periodic health check
- Wire `logRetryStormDetection()` into delivery loop
- Verify `isCorrupted()` detection criteria are meaningful
- Clean up cross-task code contamination in MeshRepository.kt

## Files to Modify (for fresh coder)
1. `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` — Wire recovery and detection into operational paths, clean up cross-task code

## Estimated Remaining LOC: ~50-80 LOC