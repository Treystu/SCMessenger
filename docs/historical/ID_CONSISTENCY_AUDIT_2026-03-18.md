# ID Consistency Audit Report - 2026-03-18

## Executive Summary

This audit identifies the root cause of the Android contact addition issue where two separate message threads have spawned due to ID inconsistencies: one thread uses ID `f669fb0f...` and the other uses ID `12D3KooW...`.

## Issue Description

**User Report:** Christy is seeing two separate message threads for the same contact on Android:
- Thread 1: Uses ID `f669fb0f43df2947d4129f553e3fb785bb82feccbec4e36a79af6dda5ba1edad` (64-char public key)
- Thread 2: Uses ID `12D3KooWN8kn7CzkY2KGWQfEMD3FJsLw25JpWvvNUDKGW8yNroKz` (libp2p peer ID)

## Root Cause Analysis

### The Two IDs

1. **`f669fb0f...`** - A 64-character hex string (the full Ed25519 public key)
2. **`12D3KooW...`** - A base58-encoded libp2p peer ID (derived from the public key)

These two IDs represent the **same peer** but are being treated as separate contacts.

### How the Bug Occurs

#### Step 1: Contact Creation with Full Public Key

When a contact is created (e.g., from a message or manual addition), the `peerId` field can be set to either:
- The full 64-char public key (e.g., `f669fb0f...`)
- The libp2p peer ID (e.g., `12D3KooW...`)

The inconsistency arises because:
- Some code paths use the full public key as the `peerId`
- Other code paths use the libp2p peer ID as the `peerId`

#### Step 2: Identity Discovered Event Emission

When a peer is identified via the transport layer, the `resolveTransportIdentity` function is called:

**Android (`MeshRepository.kt:5014-5074`):**
```kotlin
private fun resolveTransportIdentity(libp2pPeerId: String): TransportIdentityResolution? {
    // Extract public key from libp2p peer ID
    val extractedKey = ironCore?.extractPublicKeyFromPeerId(libp2pPeerId)
    val normalizedKey = normalizePublicKey(extractedKey)
    
    // Find contact by matching public key
    val keyMatches = contacts.filter { normalizePublicKey(it.publicKey) == normalizedKey }
    val canonicalContact = routeLinked ?: keyMatches.firstOrNull()
    
    // BUG: If contact has peerId = "f669fb0f...", this returns that ID
    return TransportIdentityResolution(
        canonicalPeerId = canonicalContact.peerId,  // <-- This could be "f669fb0f..."
        publicKey = normalizedKey,
        nickname = canonicalContact.nickname
    )
}
```

**iOS (`MeshRepository.swift:2526-2563`):**
```swift
private func resolveTransportIdentity(libp2pPeerId: String) -> TransportIdentityResolution? {
    // Same logic as Android
    let canonicalContact = routeLinked ?? keyMatches.first
    
    return TransportIdentityResolution(
        canonicalPeerId: canonicalContact?.peerId ?? libp2pPeerId,  // <-- Same bug
        publicKey: normalizedKey,
        nickname: canonicalContact?.nickname
    )
}
```

The `IdentityDiscovered` event is then emitted with:
- `peerId = canonicalContact.peerId` (could be `f669fb0f...`)
- `libp2pPeerId = 12D3KooW...`

#### Step 3: Message Received - Canonical Peer ID Resolution

When a message is received, `resolveCanonicalPeerId` is called:

**Android (`MeshRepository.kt:4546-4628`):**
```kotlin
private fun resolveCanonicalPeerId(senderId: String, senderPublicKeyHex: String): String {
    val normalizedIncomingKey = normalizePublicKey(senderPublicKeyHex)
    
    // Try to find contact by matching public key
    val keyedContacts = contacts.filter {
        normalizePublicKey(it.publicKey) == normalizedIncomingKey
    }
    
    if (keyedContacts.size == 1) {
        return keyedContacts.first().peerId  // <-- Returns "f669fb0f..." if that's the contact's peerId
    }
    
    // ... more logic ...
}
```

If a contact exists with `peerId = f669fb0f...`, the function returns that ID instead of the libp2p peer ID.

#### Step 4: Duplicate Thread Creation

The UI uses the `canonicalPeerId` to look up conversations. If:
- Thread 1 was created with `peerId = f669fb0f...`
- Thread 2 was created with `peerId = 12D3KooW...`

Then two separate conversation threads exist for the same peer.

### Evidence from Logs

From `docs/roo_task_mar-16-2026_5-17-28-pm.md`:
```
"message": "PeerEvent emitted: IdentityDiscovered(
    peerId=f669fb0f43df2947d4129f553e3fb785bb82feccbec4e36a79af6dda5ba1edad, 
    publicKey=b702433766a558af5c8bd4fd1b4d53f126296d74ff40c3ec7252fcfbeccca6fd, 
    nickname=Christy, 
    libp2pPeerId=12D3KooWN8kn7CzkY2KGWQfEMD3FJsLw25JpWvvNUDKGW8yNroKz, 
    listeners=[...]
)"
```

The event shows:
- `peerId` = `f669fb0f...` (full public key)
- `libp2pPeerId` = `12D3KooW...` (libp2p peer ID)

The chat screen then uses `f669fb0f...` as the conversation ID:
```
"message": "CHAT_SCREEN: conversationId=f669fb0f43df2947d4129f553e3fb785bb82feccbec4e36a79af6dda5ba1edad, normalizedPeerId=f669fb0f43df2947d4129f553e3fb785bb82feccbec4e36a79af6dda5ba1edad, displayName=Christy..."
```

## ID Types in the System

| ID Type | Format | Example | Purpose |
|---------|--------|---------|---------|
| `public_key_hex` | 64-char hex | `f669fb0f...` | Canonical identity, encryption key |
| `libp2p_peer_id` | Base58 (12D3KooW...) | `12D3KooW...` | Transport routing, network identity |
| `identity_id` | 64-char hex (Blake3 hash) | Legacy format | Deprecated, being phased out |
| `device_id` | UUID | Planned | Device-specific identifier |

## Affected Code Paths

### Android
- [`MeshRepository.kt:5014-5074`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt#L5014-L5074) - `resolveTransportIdentity`
- [`MeshRepository.kt:4546-4628`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt#L4546-L4628) - `resolveCanonicalPeerId`
- [`MeshRepository.kt:3325-3374`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt#L3325-L3374) - `emitIdentityDiscoveredIfChanged`
- [`MeshRepository.kt:881-1175`](android/app/src/main/java/com/scmessenger/android/data/MeshRepository.kt#L881-L1175) - `onMessageReceived`

### iOS
- [`MeshRepository.swift:2526-2563`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift#L2526-L2563) - `resolveTransportIdentity`
- [`MeshRepository.swift:1901-1958`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift#L1901-L1958) - `resolveCanonicalPeerId`
- [`MeshRepository.swift:3602-3658`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift#L3602-L3658) - `emitIdentityDiscoveredIfChanged`
- [`MeshRepository.swift:1178-1553`](iOS/SCMessenger/SCMessenger/Data/MeshRepository.swift#L1178-L1553) - `onMessageReceived`

## Proposed Fix Strategy

### Option 1: Always Use libp2p Peer ID as Canonical (Recommended)

**Rationale:**
- The libp2p peer ID is the stable, transport-layer identifier
- It's what the network uses for routing
- It's what `ironCore.extractPublicKeyFromPeerId()` expects as input

**Implementation:**
1. Modify `resolveTransportIdentity` to always return the `libp2pPeerId` as the `canonicalPeerId`, even if a contact exists with a different `peerId`
2. Modify `resolveCanonicalPeerId` to prefer the libp2p peer ID when available
3. Add a migration step to merge duplicate contacts (same public key, different peer IDs)

### Option 2: Always Use Full Public Key as Canonical

**Rationale:**
- The public key is the cryptographic identity
- It's what's stored in the contact database

**Cons:**
- The public key is 64 chars, which is unwieldy for display
- The libp2p peer ID is already used extensively in the network layer

### Option 3: Use a Derived Canonical ID

**Rationale:**
- Create a new canonical ID format that's derived from the public key
- This would be a new field in the contact schema

**Cons:**
- Requires schema changes
- More complex migration

## Recommended Fix (Option 1)

### Changes Required

1. **Android `resolveTransportIdentity`:**
   - Always use `libp2pPeerId` as the `canonicalPeerId` in the return value
   - Even if a contact exists with a different `peerId`

2. **iOS `resolveTransportIdentity`:**
   - Same change as Android

3. **Contact Merge Logic:**
   - When upserting a contact, check if another contact exists with the same public key but different `peerId`
   - If so, merge them (keep the libp2p peer ID as the primary `peerId`)

4. **Migration Script:**
   - Scan all contacts for duplicates (same public key, different `peerId`)
   - Merge duplicates, keeping the libp2p peer ID as the primary
   - Update message history to use the canonical `peerId`

### Code Changes

#### Android

```kotlin
// In resolveTransportIdentity, change:
return TransportIdentityResolution(
    canonicalPeerId = canonicalContact.peerId,  // OLD
    publicKey = normalizedKey,
    nickname = canonicalContact.nickname
)

// To:
return TransportIdentityResolution(
    canonicalPeerId = libp2pPeerId,  // NEW: Always use libp2p peer ID
    publicKey = normalizedKey,
    nickname = canonicalContact.nickname
)
```

#### iOS

```swift
// In resolveTransportIdentity, change:
return TransportIdentityResolution(
    canonicalPeerId: canonicalContact?.peerId ?? libp2pPeerId,  // OLD
    publicKey: normalizedKey,
    nickname: canonicalContact?.nickname
)

// To:
return TransportIdentityResolution(
    canonicalPeerId: libp2pPeerId,  // NEW: Always use libp2p peer ID
    publicKey: normalizedKey,
    nickname: canonicalContact?.nickname
)
```

## Testing Plan

1. **Unit Tests:**
   - Test `resolveTransportIdentity` with existing contacts having different `peerId` formats
   - Test `resolveCanonicalPeerId` with multiple contacts sharing the same public key

2. **Integration Tests:**
   - Test message delivery with contacts having different `peerId` formats
   - Test contact merge functionality

3. **Manual Testing:**
   - Reproduce the issue with Christy's contact
   - Verify that after the fix, only one thread exists

## Documentation Updates Required

- [ ] Update `docs/ID_UNIFICATION_IMPLEMENTATION.md` with the fix details
- [ ] Update `docs/CURRENT_STATE.md` to reflect the ID strategy
- [ ] Update `REMAINING_WORK_TRACKING.md` with the fix status
- [ ] Add migration notes to release documentation

## References

- User report: Android contact addition issue with Christy
- Logs: `docs/roo_task_mar-16-2026_5-17-28-pm.md`
- Existing ID unification plan: `docs/ID_UNIFICATION_IMPLEMENTATION.md`
- PeerIdValidator: [`android/app/src/main/java/com/scmessenger/android/utils/PeerIdValidator.kt`](android/app/src/main/java/com/scmessenger/android/utils/PeerIdValidator.kt)
