# iOS Issues - 2026-03-10

**Status:** ONGOING  
**Last Updated:** 2026-03-10T12:02:00Z

## Critical Issues

### 1. Conversation Deletion Not Persisting
**Status:** 🔴 OPEN  
**Priority:** HIGH  
**User Impact:** SEVERE

**Problem:**
- User deletes a conversation in iOS app
- Conversation reappears almost immediately
- Deletion does not persist to storage

**Expected Behavior:**
- User deletes conversation
- Messages for that peer are removed from history
- Conversation stays deleted
- Contact remains (if exists)

**Current Behavior:**
- Deletion appears to work momentarily
- Conversation reappears within seconds
- User cannot permanently delete conversations

**Root Cause (Hypothesis):**
- History sync from other device restores messages
- Deletion not calling `remove_conversation()` on history manager
- UI reload fetching from history without checking deletion flag
- No persistent deletion marker

**Code Locations:**
```swift
// iOS/SCMessenger/SCMessenger/Views/Conversations/ConversationsListView.swift
// Look for swipe-to-delete or delete action

// iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift
// clearConversation() method
```

**Action Required:**
1. Find conversation deletion code in iOS
2. Verify it calls `historyManager.removeConversation(peerId)`
3. Add logging to track deletion flow
4. Check if history sync is re-adding deleted messages
5. Implement deletion marker or suppress deleted conversations

### 2. Contact Deletion vs. Conversation Deletion
**Status:** 🟡 FEATURE REQUEST  
**Priority:** MEDIUM

**Current Behavior:**
- Deleting a contact also deletes conversations with that contact
- This is working correctly

**User Request:**
"Allow a message to be deleted without deleting a contact"

**Interpretation:**
User wants to be able to:
- Delete conversation history with a peer
- Keep the peer in contacts list
- Start fresh conversation with same contact

**Implementation Required:**
```swift
// Separate actions in UI:
1. "Delete Conversation" → Remove messages, keep contact
2. "Delete Contact" → Remove contact and messages
3. "Delete Message" → Remove individual message (new feature)
```

**Files to Modify:**
- `iOS/SCMessenger/SCMessenger/Views/Conversations/ConversationsListView.swift`
- `iOS/SCMessenger/SCMessenger/Views/Chat/ChatView.swift`
- Add swipe actions or context menu

### 3. App Freezing and Hanging
**Status:** 🔴 OPEN  
**Priority:** HIGH  
**User Impact:** SEVERE

**Symptoms:**
- iOS app becomes unresponsive
- UI freezes during operations
- Particularly bad while debugging
- Hangs especially during:
  - Message sends
  - Peer discovery
  - Contact operations

**Observed Contexts:**
- Development builds with Xcode attached
- During heavy logging
- When multiple operations happening simultaneously

**Potential Causes:**

#### A. Main Thread Blocking
**Evidence Needed:**
- Instruments trace showing main thread waits
- Long-running operations on @MainActor

**Likely Culprits:**
```swift
// Heavy operations that might block:
- historyManager?.conversation() - queries entire database
- contactManager?.list() - scans all contacts
- Synchronous file I/O
- SwiftUI state updates during iteration
```

**Solution:**
```swift
// Move to background:
Task.detached {
    let messages = await repository.loadMessages(peerId)
    await MainActor.run {
        self.messages = messages
    }
}
```

#### B. Excessive Debug Logging
**Evidence:**
- 60+ warnings in iOS build
- Console spam in Xcode
- SwiftUI debug description generation

**Solution:**
- Conditional logging in debug builds only
- Reduce log verbosity
- Disable SwiftUI debug instruments

#### C. SwiftUI State Thrashing
**Evidence Needed:**
- View recreation count
- State update frequency

**Possible Causes:**
```swift
// Problematic patterns:
@Published var messages: [MessageRecord] = []
// Updated 60+ times per second during operations

// Observers triggering cascade updates:
.onChange(of: messages) { ... }
.onChange(of: peers) { ... }
.onChange(of: contacts) { ... }
```

**Solution:**
- Debounce state updates
- Use `@State` for local UI state
- Batch updates together
- Reduce observation granularity

#### D. Memory Pressure
**Evidence Needed:**
- Memory graph in Xcode
- Allocation instruments

**Possible Causes:**
- Large message histories loaded into memory
- Peer discovery creating many temporary objects
- String allocations in tight loops

**Solution:**
- Pagination for large conversations
- Lazy loading of old messages
- Object pooling for frequent allocations

## Investigation Steps

### Step 1: Profile with Instruments
```bash
# Run iOS app from Xcode
# Product → Profile (⌘I)
# Choose "Time Profiler"
# Reproduce freeze
# Find hot paths and blocking calls
```

### Step 2: Add Performance Logging
```swift
let start = Date()
// Operation
let duration = Date().timeIntervalSince(start)
if duration > 0.1 {
    logger.warning("Slow operation: \(duration)s")
}
```

### Step 3: Main Thread Checker
```bash
# Enable in Xcode scheme:
# Product → Scheme → Edit Scheme
# Run → Diagnostics → Main Thread Checker
# Will catch UI updates on background threads
```

### Step 4: Memory Graph
```bash
# While app running in Xcode:
# Debug → View Memory Graph
# Look for retain cycles and large allocations
```

## Quick Fixes to Try

### Fix 1: Async Message Loading
```swift
// In ChatView.swift or ChatViewModel
func loadMessages() async {
    await Task.detached {
        let msgs = await repository.getConversation(peerId)
        await MainActor.run {
            self.messages = msgs
        }
    }.value
}
```

### Fix 2: Reduce Logging in Release
```swift
#if DEBUG
    logger.debug("Heavy debug info: \(expensiveOperation)")
#endif
```

### Fix 3: Batch State Updates
```swift
// Instead of:
for message in newMessages {
    messages.append(message)  // Triggers 100 updates
}

// Do:
var updated = messages
updated.append(contentsOf: newMessages)
messages = updated  // Single update
```

### Fix 4: Limit Conversation Size
```swift
// Only load recent messages by default
let conversation = repository.getConversation(peerId, limit: 100)
// Load more only when user scrolls up
```

## Action Items

### Immediate
- [ ] Profile iOS app with Instruments during freeze
- [ ] Enable Main Thread Checker
- [ ] Add performance logging to suspect methods
- [ ] Test conversation deletion persistence

### Short Term
- [ ] Implement async message loading
- [ ] Add individual message deletion
- [ ] Separate conversation delete from contact delete
- [ ] Reduce debug logging overhead

### Medium Term
- [ ] Optimize database queries
- [ ] Implement message pagination
- [ ] Add caching layer for frequent queries
- [ ] Memory optimization pass

## User Experience Impact

**Current State:**
- ❌ Cannot reliably delete conversations
- ❌ App becomes unresponsive frequently
- ❌ Debugging is painful due to freezes
- ❌ Professional development workflow impaired

**After Fixes:**
- ✅ Conversation deletion persists
- ✅ Contact deletion separate from conversation deletion
- ✅ Individual message deletion available
- ✅ App remains responsive during all operations
- ✅ Smooth debugging experience

## Related Documentation

- `IOS_CRASH_AUDIT_2026-03-10.md` - Previous crash investigations
- `ANDROID_MESSAGE_PERSISTENCE_INVESTIGATION.md` - Parallel Android issues
- `ID_UNIFICATION_AUDIT_2026-03-10.md` - ID confusion root causes

## Testing Checklist

**Conversation Deletion:**
- [ ] Delete conversation
- [ ] Verify it's removed from UI
- [ ] Background app
- [ ] Foreground app
- [ ] Verify conversation still deleted
- [ ] Kill and restart app
- [ ] Verify conversation still deleted
- [ ] Send new message to same peer
- [ ] Verify new conversation created (old messages not restored)

**Performance:**
- [ ] Send 10 messages rapidly
- [ ] App remains responsive
- [ ] Delete conversation
- [ ] App remains responsive
- [ ] Load conversation with 100+ messages
- [ ] Scroll performance acceptable
- [ ] Switch between conversations rapidly
- [ ] No freezes or hangs

**Contact vs. Conversation:**
- [ ] Delete conversation only
- [ ] Verify contact remains
- [ ] Can send new message to contact
- [ ] Delete contact
- [ ] Verify conversation also deleted
