# Root Cause Analysis - Android Contact & Messaging Issues
**Date**: 2026-03-14  
**Audit Period**: 2026-03-13 12:00 HST to 20:40 HST  
**Device**: Android Pixel 6a (26261JEGR01896)

## Executive Summary

Android device is experiencing **critical message send failures** due to ID normalization mismatches. The root cause is a **systemic ID confusion problem** where different parts of the Android app use different ID types (peer_id hash vs. Ed25519 public_key) interchangeably, violating the core's strict validation requirements.

## Issue #1: Send Message Failure (CRITICAL)

### Symptom
```
uniffi.api.IronCoreException$InvalidInput: Invalid input
at uniffi.api.IronCore.prepareMessageWithId(api.kt:8215)
```

User cannot send messages to saved contact "Christy".

### Root Cause

The `prepareMessageWithId()` call in Rust core is receiving the wrong type of ID:

**What's being passed**: `df222906...` (peer_id / identity hash - 64 hex chars)  
**What's expected**: `a974b6f9...` (Ed25519 public key - 64 hex chars)

Both are 64 hex characters but they're **fundamentally different**:
- **Peer ID** = Blake3 hash of the public key + metadata
- **Public Key** = Raw Ed25519 public key bytes

The core's `validate_ed25519_public_key()` function (in `core/src/crypto/encrypt.rs`) performs strict validation:
1. Hex decode
2. Exactly 32 bytes (64 hex chars)
3. Valid compressed Ed25519 point decompression

The peer ID hash fails step #3 because it's not a valid Ed25519 point on the curve25519 - it's just a hash.

### Evidence Trail

1. **IdentityDiscovered event** (correct data):
   ```
   peerId=df222906d561a0bd28fe8a71a6c7949ad225238409d0c2a18b07305b0260cb31  
   publicKey=a974b6f989bde92863315c7a398631fb4da2a3f8b9d0b42a835544ed5af5a4f7
   ```

2. **Auto-create contact** (correct call):
   ```
   upsertFederatedContact(
     canonicalPeerId = "df222906d561...",
     publicKey = "a974b6f9...",
     nickname = "Christy"
   )
   ```

3. **Send message preparation** (WRONG - using peer ID prefix):
   ```
   Preparing message for df222906... with key: df222906...
   ```

The bug happens because:
- Contact is stored correctly with `public_key = "a974b6f9..."`
- But when retrieving, the code is using the truncated `peer_id` prefix instead
- This happens at the contact lookup or resolution stage

### Where the Bug Occurs

In `MeshRepository.kt` sendMessage():

```kotlin
// Line ~2415
val contact = contactManager?.get(routingPeerId)
if (publicKey == null && contact != null && !contact.publicKey.isNullOrEmpty()) {
    publicKey = contact.publicKey.trim()
    
    // CRITICAL: Validate public key length
    if (publicKey.length != 64) {  // <-- This triggers!
        // Bug detection code runs but recovery fails
    }
}
```

The issue is that `contact.publicKey` is returning the truncated peer_id (`df222906...`) instead of the full Ed25519 key (`a974b6f9...`).

**Hypothesis**: The contact was stored with `peer_id` in the `publicKey` field, likely due to:
1. A bug in earlier discovery/storage code (now fixed)
2. Database migration issue from pre-WS13 schema
3. Fresh install but with stale relay data cached somewhere

##Issue #2: Contact Found But Not Recognized in Chat (HIGH)

### Symptom
```
ContactsViewModel: Peer already saved as contact: df222906d561a0bd
ChatScreenKt: contactFound=false
```

### Root Cause

**ID Truncation Mismatch** between different parts of the app:

1. **ContactsViewModel** (line 202):
   ```kotlin
   Timber.d("Peer already saved as contact: ${event.peerId.take(16)}, skipping nearby")
   // Uses first 16 chars: df222906d561a0bd
   ```

2. **ChatScreen** (line 54):
   ```kotlin
   val normalizedPeerId = PeerIdValidator.normalize(conversationId)
   val contact = viewModel.getContactForPeer(normalizedPeerId)
   // Tries to lookup using full ID: df222906d561a0bd28fe8a71a6c7949ad225238409d0c2a18b07305b0260cb31
   ```

3. **Contact Lookup Logic**:
   ```kotlin
   // getContact() tries to find by peer_id
   // But ContactsViewModel is checking against `peerId.take(16)`  
   // vs ChatScreen using full normalized ID
   ```

The problem is **inconsistent ID representation** across the codebase:
- Some places use full 64-char peer_id
- Some places use truncated 16-char prefix  
- Some places use libp2p_peer_id (12D3Koo...)
- Some places use public_key

This violates the **WS13 unified ID requirement**.

### Impact
- User sees hash instead of contact name
- "Add Contact" button shows even though contact exists
- Confusing UX that looks broken

## Issue #3: Multiple Auto-Create Events (MEDIUM)

### Symptom
Same contact auto-created 3 times in 0.5 seconds:
```
20:34:06.804 Auto-created/updated contact for peer: df222906... (Christy)
20:34:06.990 Auto-created/updated contact for peer: df222906... (Christy)
20:34:07.313 Auto-created/updated contact for peer: df222906... (Christy)
```

### Root Cause

`onPeerIdentified` callback firing multiple times without deduplication:

1. Peer connects via multiple transports simultaneously (WiFi + relay-circuit + BLE)
2. Each transport triggers its own `PeerIdentified` event
3. No idempotent guard in `upsertFederatedContact()` to prevent redundant DB writes
4. Database updates happen in rapid succession

This is actually by **design** - the `upsert` logic is meant to handle multiple updates. But it's wasteful and can cause:
- Database write amplification
- Potential race conditions
- Unnecessary UI recompositions

### Fix Required
Add a recent-update cache/debounce mechanism:
- Track last upsert timestamp per peer
- Skip updates within 5-second window unless data actually changed

## Issue #4: Nearby Peer Shows After Being Saved (MEDIUM)

### Status
**Uncertain** - logs show correct filtering but user reports seeing it.

### Evidence
```
03-13 20:33:59.812 Peer already saved as contact: df222906d561a0bd, skipping nearby
03-13 20:33:59.813 Loaded 1 contacts, filtered nearby peers to 0
```

This suggests:
- Filtering logic IS working (nearby peers filtered to 0)
- But UI may not be refreshing properly
- OR user saw a transient state before the filter applied

### Needs Further Investigation
- Screen recording to confirm UI behavior
- Check UI state flow timing
- Verify StateFlow emission ordering

## Unified Fix Strategy

### Phase 1: ID Standardization (CRITICAL)

**Goal**: Establish single source of truth for IDs across the entire Android app.

**Standard to adopt**:
- **Canonical ID** = Blake3(public_key + metadata) = peer_id = 64-char hex hash
- **Public Key** = Ed25519 public key = 64-char hex, used for encryption only
- **LibP2P Peer ID** = Base58-encoded multihash = "12D3Koo..." = transport identifier only

**Required Changes**:
1. Add `PeerIdentifier` data class to wrap ID types explicitly:
   ```kotlin
   data class PeerIdentifier(
       val canonicalId: String,        // peer_id for lookups
       val publicKey: String,           // for encryption
       val libp2pId: String? = null    // for transport routing
   )
   ```

2. Update all contact lookup functions to use `canonicalId`
3. Update all encryption calls to use `publicKey` explicitly
4. Add compile-time type safety to prevent mixing

### Phase 2: Contact Lookup Fix (HIGH)

**In ChatScreen.kt**:
```kotlin
// Before:
val normalizedPeerId = PeerIdValidator.normalize(conversationId)
val contact = viewModel.getContactForPeer(normalizedPeerId)

// After:
val identifier = PeerIdResolver.resolve(conversationId, contactManager)
val contact = identifier?.let { viewModel.getContactForPeer(it.canonicalId) }
```

**In ContactsViewModel.kt**:
```kotlin
// Before:
Timber.d("Peer already saved as contact: ${event.peerId.take(16)}")

// After:
Timber.d("Peer already saved as contact: ${event.peerId}")  // Use full ID
```

### Phase 3: Send Message Fix (CRITICAL)

**In MeshRepository.kt sendMessage()**:
```kotlin
// After contact lookup:
val contact = contactManager?.get(routingPeerId)
if (contact != null) {
    // ALWAYS use contact.publicKey for encryption
    // NEVER use contact.peerId for encryption
    publicKey = contact.publicKey.trim()
    
    if (publicKey.length != 64) {
        // This should NEVER happen with proper ID discipline
        throw IllegalStateException(
            "Contact has invalid public key: peerId=${contact.peerId}, " +
            "publicKey=${contact.publicKey}, expected 64 chars got ${publicKey.length}"
        )
    }
}
```

### Phase 4: Deduplication Guard (MEDIUM)

**In MeshRepository.kt**:
```kotlin
private val recentContactUpserts = ConcurrentHashMap<String, Long>()

private fun upsertFederatedContact(...) {
    val now = System.currentTimeMillis()
    val lastUpdate = recentContactUpserts[normalizedPeerId]
    
    if (lastUpdate != null && (now - lastUpdate) < 5000) {
        // Skip if updated within last 5 seconds
        Timber.d("Skipping redundant contact upsert for $normalizedPeerId")
        return
    }
    
    // ... existing upsert logic ...
    
    recentContactUpserts[normalizedPeerId] = now
}
```

### Phase 5: Database Migration (if needed)

If existing contacts have corrupted public_key fields:

```kotlin
// In MeshRepository initialization:
suspend fun migrateCorruptedPublicKeys() {
    val contacts = contactManager?.list() ?: return
    
    contacts.forEach { contact ->
        // If publicKey looks like a peer_id (Blake3 hash), it's corrupted
        val publicKeyBytes = hex::decode(contact.publicKey).getOrNull()
        val isValidEd25519 = publicKeyBytes?.let { bytes ->
            bytes.size == 32 && 
            // Try decompression to validate
            CompressedEdwardsY.from_slice(bytes).decompress().is_some()
        } ?: false
        
        if (!isValidEd25519) {
            Timber.w("Found corrupted publicKey for ${contact.peerId}, attempting recovery...")
            
            // Try to recover from discovered peers or transport cache
            val recoveredKey = ironCore?.resolveIdentity(contact.peerId)
            if (recoveredKey != null && recoveredKey.length == 64) {
                contactManager?.update(contact.copy(publicKey = recoveredKey))
                Timber.i("Recovered publicKey for ${contact.peerId}")
            } else {
                // Can't recover - delete the contact, user will need to re-discover
                contactManager?.remove(contact.peerId)
                Timber.e("Could not recover publicKey for ${contact.peerId}, deleted contact")
            }
        }
    }
}
```

## Testing Plan

1. **Fresh Install Test**:
   - Uninstall app
   - Reinstall and create new identity
   - Discover iOS peer "Christy"
   - Verify contact is saved with correct `publicKey` (not peer_id)
   - Attempt to send message
   - Verify message sends successfully

2. **Existing Install Test**:
   - Run migration on current device
   - Verify corrupted contacts are fixed or removed
   - Attempt to send to "Christy"
   - Verify send succeeds

3. **Multiple Transport Test**:
   - Connect to peer via WiFi, BLE, and relay simultaneously
   - Verify only one contact upsert occurs (or quick succession is harmless)
   - Verify contact has all transport hints

4. **ID Consistency Test**:
   - Add contact via nearby discovery
   - Navigate to chat screen
   - Verify contact name is displayed (not hash)
   - Verify "contactFound=true" in logs
   - Verify peer does NOT appear in nearby list

## Estimated Implementation Time

- Phase 1 (ID Standardization): 4-6 hours
- Phase 2 (Contact Lookup Fix): 1-2 hours  
- Phase 3 (Send Message Fix): 1-2 hours
- Phase 4 (Deduplication): 1 hour
- Phase 5 (Migration): 2-3 hours
- Testing & Verification: 2-3 hours

**Total**: 11-17 hours

## Risk Assessment

**HIGH RISK** - This is a fundamental architecture issue affecting:
- All messaging functionality (can't send to contacts)
- Contact management (can't recognize saved contacts)
- User experience (broken UI, confusing state)

**BLOCKER** for v0.2.1 release.

**Dependency**: May need iOS audit to ensure it's not also affected.
