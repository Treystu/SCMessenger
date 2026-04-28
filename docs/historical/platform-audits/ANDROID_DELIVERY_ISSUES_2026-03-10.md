# Android Delivery & UI Issues - March 10, 2026

## STATUS: CRITICAL ISSUES IDENTIFIED

### Issue 1: Send Button Not Responding ❌ BLOCKING

**Symptoms:**
- User reports clicking send button 100+ times with no response
- No `SEND_BUTTON_CLICKED` log entries detected in logcat
- App is running (PID 28898) but UI not responding to clicks

**Evidence:**
```
- Logcat filter for "SEND_BUTTON": 0 results during test period
- Logcat filter for "ChatScreen": 0 results during test period
- App process confirmed running
- No crash detected
```

**Possible Causes:**
1. UI thread blocked/frozen
2. Compose recomposition issue preventing button from being clickable
3. Button disabled state not visible to user
4. Input validation preventing sends (empty text field?)
5. Coroutine scope cancellation

**Investigation Needed:**
- Check if text input field has content
- Verify button enabled state
- Check for UI thread ANR (Application Not Responding)
- Review recent Compose state changes

### Issue 2: Delivery State Tracking Broken ⚠️ HIGH

**Symptoms:**
- Multiple log entries show `msg=unknown` instead of actual message IDs
- Messages failing to send with "Network error"
- Delivery attempt count at 169 for one message (excessive retry)

**Evidence:**
```
delivery_attempt msg=unknown medium=core phase=direct outcome=failed
delivery_attempt msg=unknown medium=relay-circuit phase=retry outcome=failed
delivery_state msg=15496c61-c13a-49d9-9016-6fd32ad3cf97 state=stored detail=retry_backoff_sec=300 attempt=169
```

**Impact:**
- Messages may be sent but not tracked properly
- User cannot see delivery status
- Messages may "disappear" from UI due to missing state updates
- Retry logic may be broken (169 attempts is excessive)

### Issue 3: Network Connectivity Issues ⚠️ MEDIUM

**Symptoms:**
- All delivery attempts failing with "Network error"
- Multiple transport methods tried (core, relay-circuit, BLE)
- Peer `12D3KooWDWQm...` discovered but unreachable

**Evidence:**
```
Core-routed delivery failed: Network error
Relay-circuit retry failed: Network error
delivery_attempt medium=final phase=aggregate outcome=failed
```

**Possible Causes:**
1. Target peer actually offline/unreachable
2. Network connectivity issue on Android device
3. Firewall blocking outbound connections
4. Relay nodes not functioning

## Timeline of Events

**19:37:59** - Periodic outbox flush (2 pending items)
**19:38:06** - History sync triggered for peer
**19:38:06-08** - Multiple delivery attempts, all failing
**19:38:10** - Mesh stats: 1 peer core, 2 full (discrepancy?)

## Messages "Disappearing" Root Cause

**Theory:** Messages are being written to local history but:
1. Delivery state is not being tracked (`msg=unknown`)
2. UI is filtering out messages without proper delivery state
3. Retry logic is excessive, causing messages to be stuck in "stored" state
4. UI may only show "delivered" messages, hiding "pending" ones

## Immediate Actions Required

### 1. Fix Send Button UI (CRITICAL)
- [ ] Check for UI thread blocking
- [ ] Verify Compose button click handler
- [ ] Add defensive logging before/after sendMessage call
- [ ] Check for coroutine scope cancellation

### 2. Fix Delivery State Tracking (HIGH)
- [ ] Find why `msg=unknown` is appearing
- [ ] Ensure message ID is properly propagated
- [ ] Fix delivery state updates to UI
- [ ] Implement max retry limit (currently at 169!)

### 3. Investigate Network Issues (MEDIUM)
- [ ] Verify peer is actually online
- [ ] Check Android network connectivity
- [ ] Test with different peer
- [ ] Verify relay nodes are accessible

### 4. Fix Message Visibility (HIGH)
- [ ] Ensure "pending" messages show in UI
- [ ] Add delivery state indicator to message list
- [ ] Fix filter logic that may hide unsent messages
- [ ] Implement message state persistence

## Test Plan

1. **UI Test:**
   - Fresh app restart
   - Type message in input field
   - Click send once
   - Verify button click logged
   - Verify message appears in chat

2. **Delivery Test:**
   - Send message to known-good peer
   - Verify message ID in logs
   - Verify delivery state updates
   - Verify message persists in UI

3. **Network Test:**
   - Check device connectivity
   - Ping relay nodes
   - Test with iOS peer on same network
   - Verify BLE transport working

## Related Files to Audit

- `android/app/src/main/java/com/scmessenger/android/ui/screens/ChatScreen.kt` (send button)
- `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/ConversationsViewModel.kt` (sendMessage)
- `android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt` (delivery tracking)
- `android/app/src/main/java/com/scmessenger/android/ui/chat/DeliveryStateSurface.kt` (UI state)

## Status

**Blocking:** Send button not responding - user cannot send messages
**High Priority:** Delivery state tracking broken - messages disappearing
**In Progress:** Audit and fix...

