# Contact Persistence Debug Report - Remaining Issues

**Date:** 2026-03-14 04:52 UTC  
**Status:** Investigation & Debug Plan for Remaining Issues  

---

## Issue #1: Contact Auto-Creation Duplication ⏳ REQUIRES DEBUG

### Current Status
🔴 NOT YET FIXED - Requires deeper investigation

### Evidence from Logs
```
03-13 18:22:49.396  MeshRepository: Auto-created/updated contact for peer: 93a35a87... [FIRST]
03-13 18:22:52.530  MeshRepository: Auto-created/updated contact for peer: 93a35a87... [DUPLICATE]
```

**Time gap:** 3.134 seconds between identical contact creates for same peer ID

### Root Cause Analysis

**Hypothesis:** `onPeerIdentified` callback is being fired multiple times for the same peer

**Investigation points:**

1. **Deduplication Cache Check**
   - Location: `MeshRepository.kt` lines 575-590
   - Mechanism: `peerIdentifiedDedupCache` with 5-second window
   - Issue: Second call at 3.134 seconds should be within window

2. **Duplicate Callback Source**
   - Could be: Rust core emitting duplicate callbacks
   - Could be: Multiple code paths triggering same event
   - Could be: Peer promotion triggering separate identify cycle

3. **Database Insert Behavior**
   - File: `core/src/contacts_bridge.rs` line 80-92
   - Method: `db.insert(key, value)` - This is sled DB upsert
   - Result: Should overwrite, not duplicate in database

**Key Question:** Are duplicates in the DATABASE or just in LOGS?
- If just logs: Dedup cache is working, just logging twice
- If in database: Cache not working or being bypassed

### Debug Steps Needed

1. Check if contact is actually duplicated in database
   ```bash
   adb shell run-as com.scmessenger.android sqlite3 files/contacts.db
   SELECT COUNT(*) FROM contacts WHERE peerId = '93a35a87...';
   ```

2. Add stack trace logging to `onPeerIdentified`
   ```kotlin
   Timber.i("onPeerIdentified called for $peerId")
   Timber.i("Stack: ${Thread.currentThread().stackTrace.joinToString("\n")}")
   ```

3. Check `peerIdentifiedDedupIntervalMs` value
   ```kotlin
   // Verify it's actually 5000ms or check if it's being overridden
   ```

4. Add concurrent access logging
   - Check if cache is being modified from multiple threads
   - Verify thread-safety of peerIdentifiedDedupCache

### Proposed Fix

**Option A: Strengthen Cache**
- Use ConcurrentHashMap for peerIdentifiedDedupCache
- Add stricter equality check (include listeners signature)
- Log cache hits/misses

**Option B: Investigate Source**
- Add tracing to find WHERE duplicate callback originates
- Check if it's Rust core or Android layer
- Fix at source if possible

**Option C: Accept & Ignore**
- If only in logs (not database), it's acceptable
- Sled insert is idempotent anyway
- Just clarify in documentation

---

## Issue #3: Discovered Peers Persist After Discovery Stops ⏳ REQUIRES DEBUG

### Current Status
🟡 PARTIALLY UNDERSTOOD - Needs timeline analysis

### Evidence from Logs
```
03-13 18:22:49.024  DashboardViewModel: Loaded 1 discovered peers (1 full)
[... peer shown in UI for 13+ seconds ...]
03-13 18:23:02.277  NearbySharing: stopDiscovery called
03-13 18:23:02.289  NearbySharing: DiscoveryController stopped
[... UI still shows peer for 6+ seconds ...]
03-13 18:23:08.727  DashboardViewModel: Loaded 0 discovered peers [FINALLY CLEARED]
```

**Delay:** 6.45 seconds from stopDiscovery to UI clear

### Root Cause Analysis

**Possible sources:**

1. **UI State Batching**
   - StateFlow updates might batch
   - Multiple emissions might be coalesced
   - Solution: Check if there's a debounce

2. **Discovery Map Cleanup Lag**
   - Location: `_discoveredPeers` StateFlow (line 283)
   - Function: `updateDiscoveredPeer()` (line 1857)
   - Check: When is peer removed from map?

3. **Async Job Processing**
   - Discovery might spawn background jobs
   - Jobs might complete after discovery stops
   - Solution: Add explicit cleanup on stopDiscovery

4. **Message Queue Backup**
   - Multiple onPeerIdentified callbacks might queue up
   - Processing delay could explain 6-second lag
   - Solution: Check job/coroutine queue

### Debug Steps Needed

1. Trace discovery lifecycle
   ```kotlin
   // Log exact timestamps
   Timber.i("startDiscovery called: ${System.currentTimeMillis()}")
   Timber.i("stopDiscovery called: ${System.currentTimeMillis()}")
   Timber.i("_discoveredPeers cleared: ${System.currentTimeMillis()}")
   ```

2. Monitor discoveredPeers updates
   ```kotlin
   _discoveredPeers.collect { peers ->
       Timber.i("discoveredPeers updated: ${peers.size} peers, ts=${System.currentTimeMillis()}")
   }
   ```

3. Check if cleanup job exists
   - Search for where discovered peers are removed
   - Check for scheduled cleanup tasks

### Proposed Fix

**Option A: Explicit Cleanup on Stop**
```kotlin
fun stopDiscovery() {
    // ... existing code ...
    _discoveredPeers.value = emptyMap()  // Explicit clear
    Timber.i("Cleared all discovered peers")
}
```

**Option B: Remove on Timeout**
- Track last seen for each peer
- Remove if not updated for N seconds after discovery stops

**Option C: Debounce UI Updates**
- Add explicit refresh on stopDiscovery
- Force immediate StateFlow emission

---

## Issue #4: Permission Request Loop on App Startup ⏳ REQUIRES INVESTIGATION

### Current Status
🟠 IDENTIFIED - Needs root cause clarification

### Evidence from Logs
```
03-13 18:22:48.152  MainActivity: Requesting permissions: [ACCESS_FINE_LOCATION, ...]
03-13 18:22:48.180  MainActivity: Requesting permissions: [ACCESS_FINE_LOCATION, ...] [REQUEST #2]
03-13 18:22:48.914  MainActivity: Permissions denied
03-13 18:22:48.916  MainActivity: Requesting permissions: [ACCESS_FINE_LOCATION, ...] [REQUEST #3]
[... continues to REQUEST #9+ ...]
```

**Timespan:** 9+ requests in ~700ms  
**Impact:** Dialog spam, UX degradation, possibly blocking mesh initialization

### Root Cause Analysis

**Multiple possible sources:**

1. **Multiple Permission Requesters**
   - Mesh service requesting permissions
   - Nearby API requesting permissions
   - BLE components requesting permissions
   - UI screens requesting permissions
   - Each requesting same permissions independently

2. **Retry Loop Without Backoff**
   - Permission denied → immediate retry
   - No exponential backoff
   - No rate limiting

3. **Race Condition on Startup**
   - Multiple coroutines all hitting permissions code simultaneously
   - Each thinks permissions not yet requested
   - All request at once

4. **Missing State Management**
   - No "permission request in progress" flag
   - No deduplication of requests

### Files to Check

- `android/app/src/main/java/com/scmessenger/android/ui/MainActivity.kt`
- Mesh initialization code (where permissions requested)
- BLE transport code
- Nearby API integration

### Debug Steps Needed

1. Find all permission request call sites
   ```bash
   grep -rn "Requesting permissions\|requestPermissions\|requestMultiplePermissions" \
     android/app/src/main/java --include="*.kt"
   ```

2. Add caller logging
   ```kotlin
   Timber.i("requestPermissions caller: ${Thread.currentThread().stackTrace[3]}")
   ```

3. Track request state
   ```kotlin
   private val permissionRequestInProgress = AtomicBoolean(false)
   
   fun requestPermissions(...) {
       if (!permissionRequestInProgress.compareAndSet(false, true)) {
           Timber.d("Permission request already in progress, skipping")
           return
       }
   }
   ```

### Proposed Fix

**Immediate (Quick Fix):**
- Add deduplication flag to prevent concurrent requests
- Add 500ms debounce before retrying

**Long-term (Architecture Fix):**
- Centralize permission management
- Use permission state machine
- Coordinate between components
- Add exponential backoff

---

## Debug Plan Priority

1. **High Priority:** Issue #1 (Duplication)
   - Could cause data inconsistency
   - Affects contact persistence reliability

2. **Medium Priority:** Issue #4 (Permission Loop)
   - Impacts UX but not data integrity
   - Possibly blocking mesh on some devices

3. **Low Priority:** Issue #3 (UI Lag)
   - Minor UX issue (6-second delay)
   - Acceptable if other issues fixed

---

## Testing Commands

### Monitor logcat for issues
```bash
adb logcat | grep -E "Auto-created|Requesting permissions|Loaded.*discovered"
```

### Check database for duplicates
```bash
adb shell run-as com.scmessenger.android find . -name "*.db" -type f
```

### Fresh install test
```bash
adb uninstall com.scmessenger.android
adb install -r app/build/outputs/apk/debug/app-debug.apk
adb shell am start -n "com.scmessenger.android/.ui.MainActivity"
sleep 5
adb logcat -d | grep -E "Issue patterns"
```

---

## Next Session Action Items

- [ ] Run full debug traces for Issues #1, #3, #4
- [ ] Identify exact source of permission requests
- [ ] Implement fixes in priority order
- [ ] Add test cases for each fix
- [ ] Verify no regressions

