# Android ID Mismatch Root Cause Analysis
**Date**: 2026-03-09  
**Status**: CRITICAL BUG  
**Impact**: Messages cannot be sent

---

## Issue Summary

**Symptom**: Android cannot send messages - "Contact not found for peer: f77690ef..."  
**User Report**: "These were already established contacts" but system can't find them  
**Root Cause**: **ID field naming inconsistency across the stack**

---

## Root Cause Analysis

### The ID Mismatch Chain

#### 1. **Rust Core (UDL)**
```rust
dictionary MessageRecord {
    string peer_id;  // ← snake_case
    ...
}

dictionary Contact {
    string peer_id;  // ← snake_case
    ...
}
```

#### 2. **Kotlin Auto-Generated Bindings**
The UniFFI binding generator converts snake_case to camelCase:
- `peer_id` → `peerId`

#### 3. **Android Code Usage**
```kotlin
// ConversationsViewModel.kt:36
messageList.groupBy { it.peerId }  // ✅ Correct - auto-converted

// MeshRepository.kt:2171
val contact = contactManager?.get(peerId)  // ✅ Correct call
```

#### 4. **The Actual Bug**
The issue is NOT in field naming - it's that **the contact doesn't exist in the database for that peer_id**.

---

## Log Evidence

```
Contact not found for peer: f77690efd3e66f6b4551aa3c25cec073e787657e99af4ef5b451bb2eca9315a2
```

This peer ID exists in **messages** but not in **contacts**.

---

## How This Happens

### Scenario 1: Deleted Contact with Orphaned Messages
1. User adds contact → chat → messages saved
2. User deletes contact (swipe-to-delete)
3. Messages remain in history
4. Conversation still shows in list (grouped by message.peerId)
5. User taps conversation → navigates to chat
6. User tries to send → **Contact not found**

### Scenario 2: Received Message from Unknown Peer
1. Peer broadcasts identity on mesh
2. Message received and saved (no contact required to receive)
3. Conversation appears in list
4. User taps → tries to send → **Contact not found**

### Scenario 3: Contact ID Changed (Edge Case)
1. Contact added with peer_id A
2. Peer identity updated → now has peer_id B
3. Old messages still reference peer_id A
4. Conversation shows peer_id A
5. Contact lookup fails

---

## Additional Issues Found in Logs

### Issue #1: BLE Address Type Mismatch (Minor)
```
BluetoothRemoteDevices: Address type mismatch for XX:XX:XX:XX:A3:62, new type: 1
```
**Impact**: Low - OS-level BLE scanning, not app-breaking  
**Action**: Monitor only

### Issue #2: WiFi Scan Errors (System)
```
wificond: Failed to get interface index from scan result notification
```
**Impact**: None - system WiFi, not app-related  
**Action**: Ignore

### Issue #3: Send Button Not Working (FIXED)
**Cause**: Input cleared before send completed  
**Fix**: Already implemented - only clear on success  
**Status**: ✅ Deployed

---

## Game Plan: Fix ID Mismatch

### Phase 1: Immediate Workaround (~30 LoC)
**Goal**: Prevent crashes, graceful degradation

**Changes**:
1. **ConversationsScreen.kt** - Filter out conversations without contacts
   ```kotlin
   items(conversations.filter { (peerId, _) -> 
       viewModel.getContactForPeer(peerId) != null 
   })
   ```
   **LoC**: +2

2. **ChatScreen.kt** - Show error if no contact
   ```kotlin
   LaunchedEffect(conversationId) {
       if (viewModel.getContactForPeer(conversationId) == null) {
           // Show "Add contact first" banner
       }
   }
   ```
   **LoC**: +10

3. **ConversationsViewModel.kt** - Better error message
   ```kotlin
   catch (e: IllegalStateException) {
       if (e.message?.contains("Contact not found") == true) {
           _error.value = "Please add this peer as a contact before sending messages"
       } else {
           _error.value = "Failed to send: ${e.message}"
       }
   }
   ```
   **LoC**: +8

**Total**: ~20 LoC  
**Result**: App won't crash, user knows what to do

---

### Phase 2: Proper Fix (~80 LoC)
**Goal**: Allow sending to any peer on mesh, not just contacts

**Changes**:
1. **MeshRepository.kt** - Lookup identity from mesh, not just contacts
   ```kotlin
   suspend fun sendMessage(peerId: String, content: String) {
       val contact = contactManager?.get(peerId)
       val publicKey = if (contact != null) {
           contact.publicKey
       } else {
           // Try to get from discovered peers
           meshManager?.getPeerInfo(peerId)?.publicKey 
               ?: throw IllegalStateException("Peer not found: $peerId")
       }
       // ... rest of send logic
   }
   ```
   **LoC**: +15

2. **Add Quick-Add Contact UI** - Banner in ChatScreen
   ```kotlin
   if (contact == null && peerOnMesh) {
       Row {
           Text("Not in contacts")
           Button("Add Contact") { /* quick add */ }
       }
   }
   ```
   **LoC**: +20

3. **Message Receive Handler** - Auto-create conversation stub
   - When message received from unknown peer
   - Create conversation entry (no contact required)
   - Store peer_id for later contact addition
   **LoC**: +25

4. **Contact Deletion** - Cascade options
   ```kotlin
   AlertDialog(
       "Delete contact? Keep messages?"
       - "Delete All" → delete contact + messages
       - "Keep Messages" → delete contact only
   )
   ```
   **LoC**: +20

**Total**: ~80 LoC  
**Result**: Full mesh messaging, contacts optional

---

### Phase 3: ID Standardization Audit (~50 LoC)
**Goal**: Ensure all ID usage is consistent

**Actions**:
1. **Create ID validator utility**
   ```kotlin
   object PeerIdValidator {
       fun validate(id: String): Boolean = 
           id.matches(Regex("[a-f0-9]{64}"))
       
       fun normalize(id: String): String = 
           id.trim().lowercase()
   }
   ```
   **LoC**: +10

2. **Audit all ID comparisons**
   - Search codebase for `peerId ==`, `peer_id ==`
   - Ensure normalization before comparison
   - **Locations**: ~15 files
   **LoC**: +30 (add normalize calls)

3. **Add ID logging**
   ```kotlin
   Timber.d("Sending to peerId: ${PeerIdValidator.normalize(peerId)}")
   ```
   **LoC**: +10

**Total**: ~50 LoC  
**Result**: Prevent future ID mismatch bugs

---

## Implementation Priority

### Critical (Deploy Now)
- ✅ **Send button fix** - Already deployed
- ⏳ **Phase 1: Workaround** - Prevents crashes

### High (Next Build)
- **Phase 2: Proper fix** - Full mesh messaging

### Medium (v0.2.1)
- **Phase 3: ID audit** - Long-term stability

---

## Testing Checklist

After Phase 1:
- [ ] Open chat with deleted contact → see error banner
- [ ] Cannot send message → see helpful error
- [ ] App doesn't crash

After Phase 2:
- [ ] Receive message from unknown peer → conversation appears
- [ ] Tap conversation → quick-add contact button shows
- [ ] Send message to non-contact (if on mesh) → works
- [ ] Delete contact with "Keep Messages" → messages remain

After Phase 3:
- [ ] All peer IDs normalized (lowercase, trimmed)
- [ ] ID comparisons work across case variants
- [ ] Logs show normalized IDs

---

## Estimated LoC Total
- Phase 1: 20 LoC
- Phase 2: 80 LoC
- Phase 3: 50 LoC
- **Grand Total**: ~150 LoC

---

## Recommendation

**Implement Phase 1 immediately** (20 LoC):
- Graceful degradation
- Clear error messages
- No crashes
- User can recover (re-add contact)

**Deploy Phase 2 in next session** (80 LoC):
- Real mesh messaging (send to any peer)
- Better UX
- Matches app philosophy

**Schedule Phase 3 for v0.2.1** (50 LoC):
- Preventive maintenance
- Code quality
- Long-term stability

---

## Next Steps

1. Approve game plan
2. Implement Phase 1 (20 LoC)
3. Build & deploy
4. Validate with live devices
5. Document in session report

**Ready to proceed?**
