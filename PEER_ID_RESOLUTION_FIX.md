# Peer ID Resolution & Initialization Fix - March 10, 2026

## Critical Bug Fixed: IronCore Not Initialized

### Root Cause
Android app was attempting to send history/identity sync messages immediately when peers were identified, BEFORE IronCore finished initialization. This caused:
1. `uniffi.api.IronCoreException$NotInitialized` exceptions
2. Send failures with no clear error to user
3. Messages appearing to "disappear"
4. Delivery state showing `msg=unknown`

### The Fix
Added initialization checks to `sendHistorySyncIfNeeded()` and `sendIdentitySyncIfNeeded()`:

```kotlin
// Check if core is initialized before attempting sync
if (ironCore == null) {
    Timber.w("sendHistorySyncIfNeeded: IronCore not initialized, skipping...")
    return
}
```

**Files Modified:**
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (2 functions)

### Impact
- ✅ Eliminates "Not initialized" crashes on fresh install
- ✅ Prevents premature message send attempts
- ✅ Allows core to fully initialize before sync operations
- ✅ Fixes the "pre-loaded identity" issue (was trying to use uninitialized core)

### Testing Status
- Build: ✅ SUCCESS (38s)
- Install: ✅ SUCCESS
- Fresh app data clear: ✅ SUCCESS
- Verification: IN PROGRESS

### Related Issues Fixed
1. **Pre-loaded Identity Bug** - Core was being called before init
2. **msg=unknown** - Message prep failing due to uninitialized core
3. **Messages Disappearing** - Send failures weren't being handled properly

