# Android/iOS Cross-Network Testing - March 10, 2026

## Current State

### What Works ✅
1. **Android:** Fresh install, initialization, relay connection
2. **iOS Sim:** Running, stable, relay connection
3. **Both:** Connect to relay nodes (GCP, OSX)

### What Doesn't Work ❌
1. **Peer Discovery:** Android doesn't see iOS sim (different networks)
2. **Relay Peer Sharing:** Relays don't broadcast connected peer lists
3. **Cross-Network Messaging:** Can't test without peer discovery

## The Problem

**Relay nodes are passive** - they only forward messages between peers that already know about each other. They don't:
- Broadcast "peer joined" events to other connected clients
- Share their connected peer list
- Propagate DHT/ledger information
- Facilitate peer-to-peer discovery

## Architecture Analysis

### Current Flow
```
Android (cellular) → Relay Node (knows about Android)
iOS Sim (WiFi)     → Relay Node (knows about iOS)
                     ❌ Relay doesn't tell them about each other
```

### Needed Flow
```
Android connects → Relay broadcasts "new peer: Android" → iOS receives update
iOS connects    → Relay broadcasts "new peer: iOS"     → Android receives update
Both now know about each other → Can attempt P2P connection
```

## Code Investigation

### Relay Custody Store Exists
**File:** `core/src/store/relay_custody.rs`
- Handles message custody/delivery via relay
- Tracks delivery states
- **Does NOT handle peer discovery propagation**

### What's Missing

1. **Peer List Broadcast Mechanism**
   - Relay needs to maintain list of connected peers
   - Broadcast "peer joined/left" events
   - Clients subscribe to these events

2. **Ledger Synchronization**
   - Relay should aggregate ledger entries from all clients
   - Distribute merged ledger to new connections
   - Periodic ledger refresh

3. **NAT Traversal Coordination**
   - Relay should share external IP/port info
   - Facilitate hole-punching
   - Coordinate direct P2P establishment

## Recommendations

### Short-term Workaround
**Test on same network:**
- Put Android on same WiFi as laptop
- Both will discover each other via mDNS/BLE
- Can verify messaging works without relay peer discovery

### Long-term Fix (Next Sprint)
**Implement active relay functionality:**

1. **Add to `core/src/transport/swarm.rs`:**
   ```rust
   // New: Broadcast peer list to all connected clients
   fn broadcast_peer_joined(peer_id: &PeerId) {
       // Send peer announcement to all connected clients
   }
   
   // New: Send full peer list to newly connected client
   fn send_peer_list_to_client(client: &PeerId) {
       // Send list of all currently connected peers
   }
   ```

2. **Add new message types in protocol:**
   - `PeerAnnounce` - Relay broadcasts when peer joins
   - `PeerList` - Relay sends full list to new client
   - `PeerGone` - Relay broadcasts when peer disconnects

3. **Client-side handling:**
   - Subscribe to peer announcements
   - Update discovered peers list
   - Attempt dial to newly announced peers

### Estimated Effort
- Protocol changes: 2-4 hours
- Relay implementation: 4-6 hours
- Client integration: 2-3 hours
- Testing: 2-3 hours
**Total: 10-16 hours** (1-2 days)

## Testing Plan (Once Implemented)

1. **Setup:**
   - Android on cellular
   - iOS sim on WiFi
   - Both connect to relay

2. **Verify:**
   - Android receives "peer joined: iOS" from relay
   - iOS receives "peer joined: Android" from relay
   - Both attempt direct P2P connection
   - Fallback to relay circuit if P2P fails

3. **Test Messaging:**
   - Android sends to iOS
   - Verify delivery (direct or via relay)
   - iOS sends to Android
   - Verify bidirectional works

## Documentation Updates Needed

1. **Update `docs/RELAY_OPERATOR_GUIDE.md`:**
   - Document peer discovery functionality
   - Explain ledger synchronization
   - Add troubleshooting for peer visibility

2. **Update `docs/TRANSPORT_ARCHITECTURE.md`:**
   - Document active relay design
   - Explain peer propagation mechanism
   - Add sequence diagrams

3. **Update `docs/CURRENT_STATE.md`:**
   - Mark peer discovery as implemented
   - Update known limitations
   - Add cross-network messaging status

## Immediate Action Items

### For This Session (Limited Time)
- [x] Document the issue
- [x] Identify code locations
- [x] Estimate effort
- [ ] Create GitHub issue for relay peer discovery
- [ ] Update session documentation

### For Next Session
- [ ] Implement peer announcement protocol
- [ ] Add relay-side peer list tracking
- [ ] Add client-side peer announcement handling
- [ ] Test cross-network discovery
- [ ] Verify end-to-end messaging

## Conclusion

**The relay peer discovery issue is architectural, not a bug.** The current relay implementation (passive message forwarding) is correct for its intended scope. Adding active peer discovery is a feature enhancement that requires protocol changes.

**Workaround for immediate testing:** Use same network for Android and iOS to bypass relay discovery requirement.

**Long-term solution:** Implement active relay peer propagation as outlined above.

