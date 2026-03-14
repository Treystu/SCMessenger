# Contact Persistence Issues - Comprehensive Fix Plan

**Date:** 2026-03-14  
**Status:** Plan for systematic resolution  
**Priority:** v0.2.1 blocker issues  

---

## FIXED ✅

### 1. Android Identity Modal Keyboard Issue ✅
**Status:** FIXED  
**Files:** ContactsScreen.kt, ContactDetailScreen.kt  
**Solution:** Added FocusRequester + LaunchedEffect for keyboard focus management  
**Impact:** Low - UX issue, no data impact  
**Effort:** 10 minutes (completed)  

### 2. Relay Peers Auto-Created as Stale Contacts ✅
**Status:** FIXED  
**File:** MeshRepository.kt  
**Solution:** Modified `resolveTransportIdentity()` to reject peers with no existing contact  
**Impact:** Medium - Prevents stale contact pollution on fresh installs  
**Effort:** 30 minutes (completed)  

---

## TO FIX - PRIORITY ORDER

### PRIORITY 1: Contact Auto-Creation Duplication

**Severity:** 🔴 HIGH - Data integrity concern  
**Impact:** Duplicate contacts in database, potential merge conflicts  
**Affected:** All users with multiple discovery cycles  

#### Problem Statement
Same peer contact created multiple times (4+ seconds apart) during single discovery event. Logs show duplicate "Auto-created contact" messages for identical peer ID.

#### Investigation Required
1. Determine if duplicates are in DATABASE or just LOGS
   - If just logs: dedup cache is working correctly
   - If in database: sled insert isn't idempotent or cache broken

2. Find root cause of duplicate onPeerIdentified callbacks
   - Check if Rust core emits duplicates
   - Check if Android layer triggers callbacks multiple times
   - Check peer promotion logic

3. Verify dedup cache functionality
   - Is peerIdentifiedDedupIntervalMs correct (5000ms)?
   - Is cache thread-safe?
   - Is signature matching accurate?

#### Proposed Solution

**If duplicates are just in logs:**
```kotlin
// Existing cache is working, just add clarity to logging
// Change from multiple info logs to single log on first create
// Only log if cache miss (actual create), not on duplication
```

**If duplicates are in database:**
```kotlin
// Option A: Make ContactManager.add() truly idempotent
pub fn add(&self, contact: Contact) -> Result<(), IronCoreError> {
    // db.insert() should be idempotent already
    // Verify sled semantics or switch to explicit upsert
}

// Option B: Strengthen Android-side dedup
peerIdentifiedDedupCache[trimmedPeerId] = signature to timestamp
// Add: synchronized HashMap or proper concurrency control
```

#### Test Plan
```bash
# Fresh install test
1. Clear app data: adb shell pm clear com.scmessenger.android
2. Install app: adb install app-debug.apk
3. Launch app: adb shell am start ...
4. Wait 30 seconds for discovery
5. Query database for duplicate peer_ids
6. Verify logs don't show duplicates

# Acceptance criteria:
- Max 1 "Auto-created contact" log per unique peer
- Database has max 1 row per peer_id
- No contacts created during subsequent re-discovers
```

#### Effort Estimate
- Investigation: 30 min
- Implementation: 30-60 min  
- Testing: 15 min  
- Total: 1.5-2 hours

---

### PRIORITY 2: Permission Request Loop on Startup

**Severity:** 🟠 MEDIUM - UX degradation, potential mesh initialization block  
**Impact:** Dialog spam on startup, may block BLE/discovery initialization  
**Affected:** Android devices that require multiple permissions  

#### Problem Statement
9+ rapid permission requests in ~700ms span on app startup. Multiple code paths requesting same permissions without coordination or deduplication.

#### Investigation Required
1. Locate all permission request call sites
2. Trace call stack for each request
3. Identify which are intentional vs redundant
4. Check for concurrent requests (race condition)

#### Proposed Solution

**Immediate fix (5 min):**
```kotlin
// Add global permission request state machine
private val permissionRequestInProgress = AtomicBoolean(false)
private val permissionRequestDebounceMs = 500L

fun requestPermissions(vararg permissions: String) {
    if (!permissionRequestInProgress.compareAndSet(false, true)) {
        Timber.d("Permission request already in progress")
        return
    }
    
    try {
        activityResultLauncher.launch(permissions)
    } finally {
        // Reset after delay to allow retry if denied
        Handler(Looper.getMainLooper()).postDelayed({
            permissionRequestInProgress.set(false)
        }, permissionRequestDebounceMs)
    }
}
```

**Long-term fix (2 hours):**
- Centralize permission management
- Create permission coordinator between:
  - Mesh service
  - BLE transport  
  - Nearby API
  - Location service
- Batch all required permissions into single request

#### Test Plan
```bash
# Fresh install permission test
1. Clear app + cache: adb shell pm clear com.scmessenger.android
2. Revoke all dangerous permissions first
3. Install app
4. Monitor logcat: adb logcat | grep "Requesting permissions"
5. Count total requests in first 2 seconds

# Acceptance criteria:
- Max 2 permission prompts (first request + one retry if denied)
- No rapid-fire dialog spam
- Mesh initialization unblocked
```

#### Effort Estimate
- Investigation: 20 min
- Immediate fix: 10 min
- Testing: 10 min
- Long-term refactor: 1.5 hours
- Total: 30 min quick fix, 2+ hours for full fix

---

### PRIORITY 3: Discovered Peers UI Lag After Discovery Stops

**Severity:** 🟡 LOW - Minor UX issue  
**Impact:** 6-second delay before peer disappears from UI  
**Affected:** Visual feedback timing only  

#### Problem Statement
After calling `stopDiscovery()`, discovered peers remain in UI for 6+ seconds before clearing. Expected behavior: immediate removal from UI.

#### Investigation Required
1. Trace discovery lifecycle from start to stop
2. Find where discovered peers are cleared from state
3. Identify source of 6-second delay
4. Check if UI batching or debouncing involved

#### Proposed Solution

**Quick fix:**
```kotlin
fun stopDiscovery() {
    // ... existing stop logic ...
    
    // Immediately clear from UI
    _discoveredPeers.value = emptyMap()
    Timber.i("Cleared all discovered peers on discovery stop")
}
```

**Better fix:**
```kotlin
// Remove peers with no recent updates after discovery stops
private fun pruneStaleDiscoveredPeers() {
    val now = System.currentTimeMillis()
    _discoveredPeers.update { peers ->
        peers.filter { (_, info) ->
            // Keep peers updated in last 5 seconds
            (now - (info.lastSeen.toLong() * 1000)) < 5000
        }
    }
}
```

#### Test Plan
```bash
# Discovery lifecycle test
1. Start app
2. Wait for peer discovery (observe 1 peer appears)
3. Note timestamp when discovered
4. Stop discovery (UI action)
5. Note timestamp when peer disappears
6. Verify delay < 1 second

# Acceptance criteria:
- Peer removed from UI within 1 second of stopDiscovery
- No 6+ second lag
- No visual jank or multiple refreshes
```

#### Effort Estimate
- Investigation: 15 min
- Implementation: 15 min
- Testing: 10 min
- Total: 40 min

---

## IMPLEMENTATION ROADMAP

### Session 1 (Now) - Completed ✅
- [x] Fix keyboard input in contact edit dialogs
- [x] Fix relay peer auto-creation as stale contacts
- [x] Document all findings

### Session 2 - Next Steps
- [ ] **Debugging Phase:** Determine if Issue #1 duplicates are in DB or logs
- [ ] **Quick Wins:** Fix Permission loop with atomic flag + debounce
- [ ] **Testing:** Comprehensive fresh install test for both fixes

### Session 3 - Final Polish
- [ ] **Root Cause Fix:** Implement full solution for Issue #1 if needed
- [ ] **Refactor:** Centralize permission management (if needed)
- [ ] **Cleanup:** Remove stale UI lag with immediate clear
- [ ] **Regression Testing:** Full integration test pass

---

## BUILD & DEPLOYMENT CHECKLIST

Before each fix:
- [ ] Run `./gradlew assembleDebug -x lint`
- [ ] Verify no new compiler warnings
- [ ] Test on actual device (not emulator if possible)
- [ ] Check logcat for new errors

After each fix:
- [ ] Fresh install test
- [ ] Contact operations test
- [ ] Permission flow test
- [ ] Discovery lifecycle test
- [ ] Update CURRENT_STATE.md with resolution status

---

## DOCUMENTATION UPDATES REQUIRED

After fixes are implemented:
1. Update `CURRENT_STATE.md` - Remove from "Known Issues"
2. Update `V0.2.0_RESIDUAL_RISK_REGISTER.md` - Mark as Closed
3. Update `REMAINING_WORK_TRACKING.md` - Move to completed
4. Add to `CHANGELOG.md` if one exists

---

## RISK ASSESSMENT

| Issue | Risk | Mitigation |
|-------|------|-----------|
| Duplication Fix | Could break discovery if too aggressive | Test with mesh of 5+ peers |
| Permission Fix | Could block mesh if state stuck | Add timeout + reset |
| UI Lag Fix | Could clear too early (edge case) | Add 1-second buffer |

---

## Success Criteria

✅ All 4 issues debugged and understood  
✅ Root causes documented in code comments  
✅ Fixes implemented without breaking existing functionality  
✅ Fresh install test passes (0 stale contacts)  
✅ Keyboard input works smoothly (no dialog jank)  
✅ No permission spam (max 2 dialogs)  
✅ UI lag < 1 second  
✅ All tests pass  
✅ Documentation updated  

---

**Status:** Ready for implementation  
**Next Action:** Start Session 2 debugging  

