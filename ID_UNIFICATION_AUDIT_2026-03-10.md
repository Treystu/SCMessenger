# ID Unification Audit - 2026-03-10

**Status:** IN PROGRESS  
**Priority:** CRITICAL  
**Blocking:** Message persistence, contact resolution, delivery tracking

## Problem Statement

Multiple ID formats are used across different transports and layers, causing confusion and failures in:
- Message delivery
- Contact lookups
- Conversation queries
- Peer discovery

## Current ID Formats in Use

### 1. Identity ID (Blake3 Hash)
**Format:** 64-character hex string  
**Example:** `f77690efd3e66f6b4551aa3c25cec073e787657e99af4ef5b451bb2eca9315a2`  
**Source:** Blake3 hash of Ed25519 public key  
**Used For:**
- Canonical user identity
- Contact storage primary key
- Message history peer ID (SHOULD BE)
- Display names

**Properties:**
- Case-insensitive (hex)
- Deterministic from public key
- Globally unique
- Not bound to transport

### 2. LibP2P Peer ID
**Format:** Multibase encoded (typically base58btc with "12D3" prefix)  
**Example:** `12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198`  
**Source:** Derived from LibP2P keypair  
**Used For:**
- Network layer routing
- Swarm connections
- Relay addressing
- Circuit relay paths

**Properties:**
- Case-SENSITIVE (base58)
- Bound to LibP2P transport
- Changes if LibP2P keys regenerated
- Not stable across app reinstalls if keys not persisted

### 3. Public Key (Ed25519 Hex)
**Format:** 64-character hex string  
**Example:** `357bed7e9c40b619af83bcef754ed06bb701155a20e79e8c3c5aeadd185e70a3`  
**Source:** Raw Ed25519 public key  
**Used For:**
- Message encryption/decryption
- Signature verification
- Deriving Identity ID (via Blake3)

**Properties:**
- Case-insensitive (hex)
- Core cryptographic identity
- Most fundamental ID
- Should be source of truth

### 4. BLE Peripheral Address
**Format:** MAC address  
**Example:** `6C:5E:E4:9E:6C:00`  
**Source:** Bluetooth hardware/OS  
**Used For:**
- BLE discovery
- Local transport routing
- Temporary peer tracking

**Properties:**
- Transport-specific
- May change (privacy features)
- Not globally unique
- Not cryptographically bound

### 5. Shortened Display IDs
**Format:** Various shortened formats  
**Examples:**
- `peer-YZxAX198` (from LibP2P)
- `peer-f77690ef` (from Identity ID)

**Used For:**
- UI display
- Default nicknames
- Log messages

**Properties:**
- Not unique (collisions possible)
- Display only
- Never for lookups

## ID Mismatch Problems Observed

### Issue 1: Message History Peer ID Confusion
**Observed:** 2026-03-10

**Sequence:**
1. Message sent to LibP2P peer ID: `12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198`
2. Peer ID normalized to lowercase: `12d3koowmdrhwp6civdhwswd2rnjnm9vdbtex8vktqn9yzxax198`
3. Message saved to history with normalized peer ID
4. UI queries with original case: `12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198`
5. History query fails (before case-insensitive fix)

**Root Cause:**
LibP2P peer IDs used as primary key instead of Identity ID.

**Partial Fix:**
Case-insensitive matching in history queries.

**Proper Fix Needed:**
Use Identity ID as primary key for all storage.

### Issue 2: Contact Lookup Failures
**Observed:** Ongoing

**Symptoms:**
- Android can't find iOS contact
- Multiple peer IDs for same device
- Contact auto-creation with wrong ID

**Example from logs:**
```
Peer identity read from 6C:5E:E4:9E:6C:00: a974b6f9... identity=f77690efd3e6 nickname='Christy'
Canonicalized sender f77690efd3e6... -> 12D3KooWMDrHwP6C... using public key match
```

**Problem:**
Contact stored with one ID type, looked up with another.

### Issue 3: Multiple IDs for Same Device
**Observed:** 2026-03-10

For iOS device "Christy":
- Identity ID: `f77690efd3e66f6b4551aa3c25cec073e787657e99af4ef5b451bb2eca9315a2`
- LibP2P Peer ID: `12D3KooWMDrHwP6CiVdHWSwD2RNJNM9VDBTeX8vKTqn9YZxAX198`
- BLE Address: `6C:5E:E4:9E:6C:00`
- Public Key: `357bed7e9c40b619af83bcef754ed06bb701155a20e79e8c3c5aeadd185e70a3`

**Problem:**
Android doesn't know these all refer to the same device.

## Current ID Resolution Flow

### Android `sendMessage(peerId)`
```kotlin
1. PeerIdValidator.normalize(peerId) → lowercase
2. ironCore?.resolveIdentity(normalizedPeerId) → public key
3. Fallback: contactManager?.get(normalizedPeerId)
4. Fallback: discoveredPeers[normalizedPeerId]
5. Fallback: extractPublicKeyFromPeerId(normalizedPeerId)
6. Save to history with normalizedPeerId
```

**Problem:** Using LibP2P peer ID throughout instead of Identity ID.

### Core `resolve_identity()`
```rust
// Should return Identity ID, but currently returns public key
pub fn resolve_identity(&self, any_id: String) -> Result<String, IronCoreError>
```

**Problem:** Method name says "identity" but returns public key.

## Proposed ID Unification Strategy

### Canonical ID Hierarchy

```
PUBLIC KEY (Ed25519 hex)
    ↓ (Blake3 hash)
IDENTITY ID (64-char hex)  ← PRIMARY KEY FOR EVERYTHING
    ↓ (derived)
LibP2P Peer ID (multibase)  ← Transport only
    ↓ (transport-specific)
BLE Address (MAC)  ← Discovery only
```

### Rule: One Source of Truth

**All storage operations use Identity ID:**
- Message history: `peer_id` = Identity ID
- Contacts: `peer_id` = Identity ID  
- Delivery tracking: keyed by Identity ID
- Conversation queries: by Identity ID

**Transport IDs stored as metadata:**
- Contact: `libp2p_peer_id` field (optional)
- Contact: `ble_address` field (optional)
- Contact: `last_known_addresses` field

### Conversion Functions Needed

```rust
// Core API
pub fn public_key_to_identity_id(public_key_hex: &str) -> String
pub fn libp2p_peer_id_to_identity_id(peer_id: &str) -> Result<String>
pub fn resolve_to_identity_id(any_id: &str) -> Result<String>
pub fn identity_id_to_libp2p_peer_id(identity_id: &str) -> Option<String>
```

```kotlin
// Android
fun resolveToIdentityId(peerId: String): String?
fun getLibp2pPeerIdForIdentity(identityId: String): String?
```

## Migration Plan

### Phase 1: Add Identity ID to All Structs (WS13.1 ✅)
- [x] Add `device_id` to IdentityManager
- [x] Add `seniority_timestamp` to IdentityManager
- [x] Persist device metadata

### Phase 2: Update Contact Schema (WS13.2 ✅)
- [x] Add `last_known_device_id` to Contact
- [x] Expose `get_device_id()` and `get_seniority_timestamp()`

### Phase 3: Unified ID Resolution (IN PROGRESS)
- [ ] Implement `resolve_to_identity_id()` in core
- [ ] Update all message storage to use Identity ID
- [ ] Add mapping table: Identity ID → LibP2P Peer IDs
- [ ] Update contact lookup to try all ID types
- [ ] Migrate existing data

### Phase 4: Update All Callsites
- [ ] Android `sendMessage()` → use Identity ID
- [ ] Android `getConversation()` → use Identity ID
- [ ] iOS message operations → use Identity ID
- [ ] Update UI to display Identity ID (shortened)

### Phase 5: Remove Ambiguity
- [ ] Deprecate storing LibP2P ID as primary key
- [ ] LibP2P ID becomes transport metadata only
- [ ] Clear documentation of ID hierarchy
- [ ] Add validation that rejects wrong ID type

## Testing Requirements

### Unit Tests
- [ ] `test_identity_id_from_public_key()`
- [ ] `test_libp2p_to_identity_id_conversion()`
- [ ] `test_resolve_any_id_format()`
- [ ] `test_case_insensitive_identity_id_matching()`

### Integration Tests
- [ ] Send message using LibP2P ID → stored with Identity ID
- [ ] Query conversation with LibP2P ID → finds messages by Identity ID
- [ ] Add contact with one ID → lookup with any ID type succeeds
- [ ] Device with multiple transports → single Identity ID

### Migration Tests
- [ ] Existing messages with LibP2P peer IDs migrate to Identity IDs
- [ ] Existing contacts updated with correct Identity ID
- [ ] Old and new clients can communicate

## Rollout Strategy

### Version 0.2.1 (WS13.3-13.6)
- Implement unified resolution
- Add Identity ID as primary key alongside LibP2P ID
- Both work during transition

### Version 0.3.0
- Migrate all data to Identity ID primary
- LibP2P ID becomes secondary/metadata
- Deprecate LibP2P-only lookups

### Version 0.4.0
- Remove LibP2P ID as primary key
- Pure Identity ID system
- LibP2P ID cached for active connections only

## References

**Related Issues:**
- `ANDROID_ID_MISMATCH_RCA.md` - Previous ID confusion
- `PEER_ID_RESOLUTION_FIX.md` - Earlier resolution attempts
- `ID_UNIFICATION_PLAN.md` - Original unification plan

**Code Locations:**
- `core/src/identity/mod.rs` - Identity ID generation
- `core/src/lib.rs` - `resolve_identity()` method
- `android/.../MeshRepository.kt` - ID resolution on Android
- `iOS/.../MeshRepository.swift` - ID resolution on iOS

**Specifications:**
- Blake3: https://github.com/BLAKE3-team/BLAKE3-specs
- LibP2P Peer IDs: https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md
- Multibase: https://github.com/multiformats/multibase
