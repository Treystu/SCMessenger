# Urgent Testing & Fixes TODO
**Date:** 2026-03-10
**Status:** Critical - Testing Required

## Just Completed

✅ **ID Unification - Phase 1**
- Added `resolve_identity()` to core (Rust)
- Exposed via UniFFI API
- Integrated into Android `sendMessage()`
- Built and installed on device

## IMMEDIATE TESTING NEEDED

### 1. Test Android Send Functionality (RIGHT NOW)

**Setup:**
- Android device on cellular network
- USB connected for logcat
- At least one contact/peer visible

**Test Steps:**
1. Open SCMessenger Android app
2. Select a conversation or contact
3. Type a test message
4. Hit send button
5. Observe behavior

**Expected:**
- Message appears in conversation
- Send button clears text field immediately
- Message shows as sent/delivered
- No "Peer not found" errors

**Log Commands:**
```bash
# Clear logs and start fresh capture
adb logcat -c
adb logcat -v time | grep -E "SEND_MSG|resolv|IronCore" > android_send_test_$(date +%Y%m%d_%H%M%S).log
```

**What to Look For:**
```
✅ SEND_MSG_START: peerId='...'
✅ SEND_MSG: Core resolved '...' to publicKey='...'
✅ Message sent (encrypted) to ...
❌ SEND_MSG: Core resolution failed
❌ Peer not found
❌ IllegalStateException
```

### 2. Test Contact Visibility & Nicknames

**Current Issue:** Conversations showing IDs instead of nicknames

**Test Steps:**
1. Open Conversations tab
2. Check if nicknames display (not IDs)
3. Open a chat
4. Check if contact name shows at top

**Fix Required:**
- Ensure `Contact.nickname` populated from identity sync
- Use `Contact.displayName()` in UI
- Check `ConversationsViewModel` and `ChatScreen`

**Files to Check:**
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ConversationsTab.kt`
- `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`

### 3. Test Contact Persistence

**Test Steps:**
1. Add a new contact
2. Send a message to them
3. Force-close app
4. Reopen app
5. Check if contact still there

**Expected:**
- Contact persists
- Message history preserved
- Can send new messages

### 4. Add Blocking UI to Android

**Current Issue:** No way to block identities in Android app

**Implementation Required:**

**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`

Add to menu/options:
```kotlin
// In ChatScreen composable
IconButton(onClick = {
    viewModel.blockContact(peerId)
}) {
    Icon(Icons.Default.Block, "Block User")
}
```

**File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt`

Add function:
```kotlin
fun blockContact(peerId: String) {
    viewModelScope.launch {
        try {
            repository.blockPeer(peerId, "Blocked by user")
            // Remove from conversations
            // Show confirmation
        } catch (e: Exception) {
            Timber.e(e, "Failed to block peer")
        }
    }
}
```

**Estimated:** 20-30 lines of code

### 5. Test iOS Simulator Connection

**Setup:**
1. Start iOS simulator on laptop
2. Android device on cellular
3. Both connected to relay
4. Verify peer discovery

**Test Steps:**
1. Check if iOS sim appears in Android contacts/discovered peers
2. Try sending message from Android → iOS sim
3. Try sending message from iOS sim → Android
4. Verify both directions work

**Log Commands:**
```bash
# iOS logs
xcrun simctl spawn booted log stream --predicate 'subsystem == "com.scmessenger"' --level debug

# Android logs
adb logcat -v time | grep -i "scmessenger\|mesh"
```

### 6. iOS Crash/Hang Investigation

**Current Issues:**
- iOS crashing/hanging intermittently
- Performance degradation

**Debug Steps:**
1. Check iOS device logs for crashes
2. Run iOS sim for 15 minutes
3. Monitor memory usage
4. Check for deadlocks/hangs

**Log Collection:**
```bash
# Get crash logs
xcrun simctl diagnose --output /tmp/ios_diagnostics

# Monitor in real-time
xcrun simctl spawn booted log stream --level debug | tee ios_live_$(date +%Y%m%d_%H%M%S).log
```

## ANDROID SPECIFIC FIXES NEEDED

### Fix 1: Send Button UI Lag

**Issue:** Message stays in text field too long after send

**File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt`

**Fix:** Ensure text field clears immediately on send
```kotlin
onSendMessage = { text ->
    viewModel.sendMessage(text)
    messageText = "" // Clear immediately, don't wait for callback
}
```

### Fix 2: Message Queue Display

**Issue:** Queued messages may not show proper status

**Fix:** Ensure UI shows "Sending..." or "Queued" status
- Add message state field
- Display appropriate icon/color
- Update when delivery confirmed

### Fix 3: Discovered Peers Not Showing

**Issue:** Peers discovered by relay not visible

**Files to Check:**
- `MeshRepository.kt` - `_discoveredPeers` updates
- `PeerManager.kt` - Peer sync logic
- Check if identity sync is working

## CROSS-PLATFORM PARITY CHECK

### Required Functions

| Function | Android | iOS | WASM | Notes |
|----------|---------|-----|------|-------|
| resolve_identity() | ✅ | ❌ | ❌ | Just added |
| send_message() | ✅ | ⏳ | ⏳ | Android fixed |
| block_peer() | ⚠️ | ⚠️ | ❌ | Core exists, UI missing |
| contact persistence | ⚠️ | ⏳ | ❌ | Needs testing |
| nickname display | ❌ | ⏳ | ❌ | Broken on Android |
| peer discovery relay | ✅ | ✅ | ❌ | Should work |
| message queueing | ✅ | ⏳ | ❌ | Just improved |

Legend:
- ✅ Implemented and working
- ⏳ Implemented, needs testing
- ⚠️ Partial implementation
- ❌ Not implemented

## DOCUMENTATION UPDATES NEEDED

After testing confirms everything works:

1. **REMAINING_WORK_TRACKING.md**
   - Mark ID unification as complete
   - Update send issues status
   - Add blocking UI as in-progress

2. **DOCUMENTATION.md**
   - Add section on ID resolution
   - Document resolve_identity() API
   - Update mobile integration guide

3. **API.md** (if exists)
   - Document resolve_identity()
   - Show usage examples
   - Note ID format support

4. **Run doc verify script:**
```bash
./scripts/verify_docs.sh  # If this exists
```

## DEBUGGING REFERENCE

### Android Logcat Filters

```bash
# Send issues
adb logcat -v time | grep -E "SEND_MSG|prepare|encrypt"

# ID resolution
adb logcat -v time | grep -E "resolv|identity|peer.*id"

# Delivery tracking
adb logcat -v time | grep -E "delivery|receipt|queued"

# Errors only
adb logcat -v time '*:E' | grep scmessenger
```

### iOS Logging

```bash
# Real device
idevicesyslog | grep -i scmessenger

# Simulator
xcrun simctl spawn booted log stream --predicate 'subsystem == "com.scmessenger"'
```

### run5.sh Script

**Purpose:** Test multiple nodes with fresh install

```bash
# Run comprehensive test
./run5.sh

# Check all nodes are running
pgrep -l scmessenger

# Collect logs from all nodes
# (commands will depend on run5.sh implementation)
```

## SUCCESS CRITERIA

This issue is RESOLVED when:

1. ✅ Android send button works reliably (no "peer not found" errors)
2. ⏳ Messages queue properly when offline
3. ⏳ Nicknames display in conversations (not IDs)
4. ⏳ Contacts persist across app restart
5. ⏳ Android ↔ iOS messaging works 100%
6. ⏳ Block functionality accessible in UI
7. ⏳ iOS stable (no crashes/hangs)
8. ⏳ Documentation updated
9. ⏳ All tests pass

## AGENT COORDINATION

If using agent swarm:

1. **Android UI Agent** - Fix nickname display, add block button
2. **iOS Debug Agent** - Investigate crashes/hangs
3. **Core Test Agent** - Write unit tests for resolve_identity
4. **Integration Test Agent** - Run cross-platform messaging tests
5. **Documentation Agent** - Update all docs once testing complete

## ESTIMATED TIME (Lines of Code, Not Hours!)

- Android nickname display fix: 15-20 lines
- Android block UI: 25-30 lines
- iOS resolve_identity integration: 50-75 lines
- Core unit tests: 100-150 lines
- Documentation updates: N/A
- Manual testing/validation: N/A

**Total estimated:** ~200-275 lines of code

## NOTES

- No time estimates allowed per user requirement
- Focus on completing one task fully before moving to next
- Test thoroughly after each change
- Document issues as they're found
- Don't leave TODO comments in code

## PRIORITY ORDER

1. **TEST ANDROID SEND RIGHT NOW** ← START HERE
2. Fix nickname display
3. Add blocking UI
4. Test cross-platform messaging
5. Debug iOS issues
6. Update documentation

---

**Ready to test? Start with Android send functionality and capture logs!**
