# Android Message Persistence Investigation

**Status:** ONGOING
**Priority:** CRITICAL
**Last Updated:** 2026-03-10

## Problem Statement

Messages typed and sent on Android disappear from the chat thread immediately after sending, despite being saved to the history database. This creates a broken user experience where users cannot see their own sent messages.

## Root Causes Identified

### 1. Case-Sensitive Peer ID Matching (FIXED)
**Discovered:** 2026-03-10
**Status:** ✅ RESOLVED

**Problem:**
- History manager used `==` for peer ID comparison
- Messages saved with one case variant, queried with another
- Zero matches returned even though messages existed

**Solution:**
```rust
// Before:
if &record.peer_id == peer {

// After:
if record.peer_id.eq_ignore_ascii_case(peer) {
```

**Files Changed:**
- `core/src/store/history.rs` - Lines 132, 194

**Tests Added:**
- `test_case_insensitive_peer_id_matching()`
- `test_remove_conversation_case_insensitive()`

### 2. Replace vs. Merge Strategy (ATTEMPTED FIX)
**Discovered:** 2026-03-10
**Status:** ⚠️ PARTIALLY IMPLEMENTED

**Problem:**
- `loadMessages()` in ChatViewModel replaces entire message list
- Optimistic messages added to UI are destroyed when history reload happens
- Even though messages are in history, timing issues cause them to disappear

**Solution Attempted:**
```kotlin
// Changed loadMessages() from REPLACE to MERGE
val mergedMessages = mutableListOf<MessageRecord>()
mergedMessages.addAll(messageList)  // From history

// Keep optimistic messages not confirmed yet
for (optimistic in currentMessages) {
    if (mergedMessages.none { it.id == optimistic.id }) {
        mergedMessages.add(optimistic)
    }
}
```

**Files Changed:**
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt`

**Status:** Code deployed but messages still disappearing

### 3. Optimistic UI Updates (IMPLEMENTED)
**Implemented:** 2026-03-10
**Status:** ✅ CODE COMPLETE, ❌ NOT WORKING IN PRACTICE

**Implementation:**
1. Message added to UI with temporary UUID when send button pressed
2. `sendMessage()` saves to history with real ID
3. `observeMessageUpdates()` replaces optimistic with real message
4. Smart deduplication by content + timestamp (±2 seconds)

**Files Changed:**
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt`
  - Added `observeMessageUpdates()`
  - Modified `sendMessage()` to add optimistic message
  - Added deduplication logic

**Current Issue:**
Messages still disappearing despite this triple-guarantee system.

## Investigation Findings (2026-03-10)

### Log Analysis Results

**Messages ARE Being Saved:**
```
03-10 01:53:39.197 I/MeshRepository: delivery_state msg=f9fcd0d5 state=pending detail=message_prepared_local_history_written
03-10 01:53:53.574 I/MeshRepository: delivery_state msg=c281dd6a state=pending detail=message_prepared_local_history_written
```

**Messages ARE Being Sent:**
```
03-10 01:53:39.158 D/MeshRepository$sendMessage: SEND_MSG_START: peerId='12d3koowmdrhwp6civdhwswd2rnjnm9vdbtex8vktqn9yzxax198'
03-10 01:53:39.159 D/MeshRepository$sendMessage: SEND_MSG_START: normalized='12d3koowmdrhwp6civdhwswd2rnjnm9vdbtex8vktqn9yzxax198'
```

**ChatViewModel NOT Logging:**
- No `Timber.d("Loaded X messages")` entries for ChatViewModel
- Only ConversationsViewModel logs present
- Suggests ChatViewModel might not be instantiated or logging is filtered

**Peer ID Used:**
- Normalized: `12d3koowmdrhwp6civdhwswd2rnjnm9vdbtex8vktqn9yzxax198` (lowercase)
- Original: `12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198` (mixed case)
- Case-insensitive matching SHOULD handle this

## Hypotheses to Investigate

### Hypothesis 1: ChatViewModel Not Being Used
**Evidence:**
- No ChatViewModel logs in logcat
- ConversationsViewModel logs present
- Optimistic UI code may not be executing

**Action Required:**
- Verify ChatViewModel is instantiated for chat screen
- Add more aggressive logging
- Check if deprecated ViewModel is being used instead

### Hypothesis 2: Message List State Not Updating
**Evidence:**
- Merge logic implemented
- No logs showing message counts

**Action Required:**
- Add logging to every state change in ChatViewModel
- Log `_messages.value.size` before and after updates
- Verify LiveData/StateFlow observation

### Hypothesis 3: Query Mismatch Despite Case-Insensitive Fix
**Evidence:**
- History stores with normalized ID
- ChatViewModel queries with original ID
- Case-insensitive should work, but maybe not applied everywhere

**Action Required:**
- Log the exact peer ID used in `getConversation()`
- Compare with peer ID in stored records
- Verify core's conversation() method is being called

### Hypothesis 4: Timing Issue - Messages Expire Too Fast
**Evidence:**
- Optimistic messages added
- loadMessages() called immediately
- Merge should preserve, but maybe race condition

**Action Required:**
- Add timestamp logging for:
  - When optimistic message added
  - When loadMessages() called
  - When merge happens
  - When _messages.value updated

## Related Issues

### ID Mismatch Across Transports
**Status:** 🔴 OPEN
**Priority:** HIGH

**Problem:**
Multiple ID formats cause confusion:
- Identity ID (Blake3 hash): `f77690efd3e66f6b4551aa3c25cec073e787657e99af4ef5b451bb2eca9315a2`
- LibP2P Peer ID: `12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198`
- BLE Address: `6C:5E:E4:9E:6C:00`
- Public Key (hex): `357bed7e...`

**Impact:**
- Android confused about which ID to use for which operation
- Messages sent to one ID format, queried with another
- Contact lookups failing due to ID mismatch

**Action Required:**
- Implement unified ID resolution across all transports
- Ensure canonical ID used for all storage operations
- Document ID mapping and when each format is used

### iOS Conversation Deletion Not Persisting
**Status:** 🔴 OPEN
**Priority:** MEDIUM

**Problem:**
- User deletes conversation in iOS
- Conversation reappears immediately
- Deletion not persisting to storage

**User Request:**
"Allow a message to be deleted without deleting a contact"

**Action Required:**
- Implement `clearConversation(peerId)` on iOS
- Ensure it calls history manager's `remove_conversation()`
- Verify persistence after deletion
- Add UI for individual message deletion

### iOS Performance Issues
**Status:** 🔴 OPEN
**Priority:** MEDIUM

**Symptoms:**
- App freezing/hanging, especially while debugging
- UI unresponsive during operations
- Particularly bad during development

**Potential Causes:**
- Excessive logging in debug builds
- Main thread blocking
- SwiftUI state update issues
- Memory pressure

**Action Required:**
- Profile with Instruments
- Move heavy operations off main thread
- Reduce logging verbosity in debug builds
- Investigate SwiftUI state management

## Attempted Fixes Timeline

### 2026-03-10 Morning
1. ✅ Fixed case-sensitive peer ID matching in history manager
2. ✅ Added 2 regression tests for case-insensitive matching
3. ✅ Implemented optimistic UI updates in ChatViewModel
4. ✅ Added `observeMessageUpdates()` for real message confirmation
5. ✅ Changed `loadMessages()` to merge instead of replace
6. ❌ Messages still disappearing

### 2026-03-10 Afternoon
1. ✅ Created log capture scripts
2. ✅ Analyzed Android logs - confirmed messages being saved
3. ⚠️ Discovered ChatViewModel not logging
4. 📋 Documented hypotheses for further investigation

## Next Steps

### Immediate (Critical)
1. Add extensive logging to ChatViewModel
   - Every state change
   - Every method entry/exit
   - Message list size at each step
2. Verify ChatViewModel is actually being used
3. Test with single message send and capture full flow
4. Compare stored messages with queried messages

### Short Term (High Priority)
1. Implement unified ID resolution system
2. Document ID mapping across all transports
3. Fix iOS conversation deletion persistence
4. Add individual message deletion (without deleting contact)

### Medium Term
1. Profile and optimize iOS performance
2. Reduce debug logging overhead
3. Add integration tests for message persistence
4. Create automated test for send → persist → reload flow

## Testing Checklist

- [ ] Send message on Android
- [ ] Verify message appears in UI immediately
- [ ] Verify message persists after 1 second
- [ ] Verify message persists after screen rotation
- [ ] Verify message persists after app restart
- [ ] Verify message appears on other device
- [ ] Test with different peer ID case variants
- [ ] Test with multiple rapid sends
- [ ] Test with network disconnected
- [ ] Test with app in background

## References

**Code Files:**
- `core/src/store/history.rs` - Message storage
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ChatViewModel.kt` - UI layer
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` - Data layer

**Related Docs:**
- `ANDROID_SEND_QUEUEING_FIX.md` - Previous send issues
- `ANDROID_ID_MISMATCH_RCA.md` - ID confusion issues
- `MESSAGE_DELIVERY_RCA_2026-03-09.md` - Delivery problems

**Git Commits:**
- `e004a47` - Fix Android message disappearing (case-insensitive)
- `b8d9bec` - CRITICAL FIX: Guarantee message persistence with optimistic UI
- `5e615ba` - FINAL FIX: Android message persistence - merge optimistic with history

**Scripts:**
- `scripts/capture_both_logs.sh` - Capture Android + iOS logs
- `scripts/watch_message_send.sh` - Monitor message send events
- `scripts/diagnose_message_issue.sh` - Message persistence diagnostics
