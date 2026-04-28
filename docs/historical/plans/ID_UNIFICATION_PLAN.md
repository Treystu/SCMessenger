# Identity ID Unification Plan
**Date:** 2026-03-10
**Status:** Critical Issue - Multiple ID Systems Causing Confusion

## Problem Statement

We currently have **4 different identity/peer ID systems** in use:

1. **`public_key_hex`** (64 hex chars) - Ed25519 public key
2. **`identity_id`** (64 hex chars) - Blake3 hash of public key (legacy)
3. **`libp2p_peer_id`** (base58, starts with "12D3Koo") - libp2p PeerId
4. **`device_id`** (planned, not implemented) - Multi-device support

This causes:
- ❌ ID mismatch errors ("peer not found")
- ❌ Confusion in contact lookup
- ❌ Complex resolution logic scattered across codebase
- ❌ Difficulty debugging delivery issues

## Current ID Usage

### Core (Rust)

```rust
pub struct IdentityInfo {
    pub public_key_hex: String,         // PRIMARY KEY (64 hex)
    pub identity_id: Option<String>,    // Blake3 hash (legacy)
    pub libp2p_peer_id: Option<String>, // Transport routing
    pub nickname: Option<String>,
}
```

**Files:**
- `core/src/lib.rs` - IdentityInfo struct
- `core/src/identity/` - Identity management
- `core/src/store/contacts.rs` - Contact.peer_id
- `core/src/store/blocked.rs` - BlockedIdentity.peer_id
- `core/src/store/history.rs` - MessageRecord.peer_id

### Android

**MeshRepository.kt uses:**
- `peerId` parameter (accepts ANY of the 4 ID types!)
- Complex resolution in `sendMessage()` (lines 2236-2327)
- Discovered peers keyed by libp2p PeerId
- Contacts keyed by identity_id

### iOS

**Similar mixed usage** - needs audit

## Proposed Unification Strategy

### Phase 1: Standardize on `public_key_hex` (IMMEDIATE)

**Rationale:**
- Public key is the SOURCE OF TRUTH for identity
- All other IDs are derived from it
- Already used in encryption/signing
- Immutable (doesn't change with libp2p upgrades)

**Changes:**

1. **Rename all `peer_id` fields to `identity_public_key`**
   - `Contact.peer_id` → `Contact.identity_public_key`
   - `BlockedIdentity.peer_id` → `BlockedIdentity.identity_public_key`
   - `MessageRecord.peer_id` → `MessageRecord.identity_public_key`

2. **Add indexed lookup tables for ID resolution**
   ```rust
   // core/src/store/identity_index.rs (NEW)
   pub struct IdentityIndex {
       // Maps any ID type → canonical public_key_hex
       libp2p_to_public_key: HashMap<String, String>,
       identity_id_to_public_key: HashMap<String, String>,
       device_id_to_public_key: HashMap<String, Vec<String>>, // 1-to-many
   }
   ```

3. **Single resolution function**
   ```rust
   impl IronCore {
       pub fn resolve_identity(&self, any_id: String) -> Result<String, IronCoreError> {
           // Try each ID type, return canonical public_key_hex
       }
   }
   ```

### Phase 2: Device ID Infrastructure (FUTURE)

**Design:**
```rust
pub struct DeviceIdentity {
    pub device_id: String,              // UUID per device
    pub identity_public_key: String,    // Links to main identity
    pub device_name: Option<String>,    // "Luke's iPhone"
    pub created_at: u64,
}
```

**Use Cases:**
- Block specific devices while allowing others
- Track message sources across devices
- Enable "trusted devices" feature

### Phase 3: Deprecate `identity_id` (AFTER MIGRATION)

**Steps:**
1. Add migration code to update all databases
2. Mark `identity_id` as `@deprecated` in API
3. Remove after 2-3 releases

## Implementation Order

### Step 1: Core ID Resolution (2-3 hours)

- [ ] Create `core/src/store/identity_index.rs`
- [ ] Add `resolve_identity()` to IronCore
- [ ] Update UniFFI API to expose resolver

### Step 2: Android Simplification (1-2 hours)

- [ ] Replace complex sendMessage() resolution with single `resolve_identity()` call
- [ ] Update ContactsManager to index by public_key_hex
- [ ] Migrate existing contacts database

### Step 3: iOS Parity (2-3 hours)

- [ ] Audit iOS ID usage
- [ ] Apply same resolution pattern
- [ ] Test cross-platform messaging

### Step 4: Blocking UI (1 hour)

- [ ] Add block button to ChatScreen (Android)
- [ ] Add block button to MessageViewController (iOS)
- [ ] Show blocked indicator in UI

### Step 5: Device ID Foundation (FUTURE - 5-6 hours)

- [ ] Device ID generation
- [ ] Device pairing flow
- [ ] Multi-device blocking

## Migration Strategy

### Database Schema Changes

**Before:**
```sql
CREATE TABLE contacts (
    peer_id TEXT PRIMARY KEY,  -- Could be ANY ID type!
    public_key TEXT NOT NULL,
    ...
);
```

**After:**
```sql
CREATE TABLE contacts (
    identity_public_key TEXT PRIMARY KEY,  -- ALWAYS 64-char hex
    ...
);

CREATE TABLE identity_index (
    lookup_id TEXT PRIMARY KEY,      -- libp2p/identity_id/device_id
    identity_public_key TEXT NOT NULL,  -- Canonical ID
    id_type TEXT NOT NULL             -- 'libp2p'/'identity_id'/'device_id'
);
```

### Backward Compatibility

**Option A: Auto-migration on startup**
```rust
impl IronCore {
    pub fn migrate_legacy_ids(&self) -> Result<(), IronCoreError> {
        // Convert old peer_id → identity_public_key
        // Populate identity_index
    }
}
```

**Option B: Runtime fallback**
- Keep `identity_id` for existing contacts
- New contacts use `public_key_hex`
- Gradually migrate as contacts are updated

**Recommendation:** Option A (clean break, one-time pain)

## Testing Plan

1. **Unit Tests**
   - Test ID resolution for all formats
   - Test migration logic
   - Test backward compatibility

2. **Integration Tests**
   - Android → iOS messaging with ID resolution
   - Contact lookup with mixed ID types
   - Blocking with various ID formats

3. **Manual Tests**
   - Fresh install (new schema)
   - Upgrade from existing install (migration)
   - Cross-platform send/receive

## Rollout Plan

### Version 0.2.1

- ✅ Add `resolve_identity()` API
- ✅ Update Android to use resolver
- ✅ Add blocking UI
- ⚠️ Keep `identity_id` for compatibility

### Version 0.2.2

- ✅ iOS uses resolver
- ✅ Auto-migration on startup
- ⚠️ `identity_id` marked deprecated

### Version 0.3.0

- ✅ Device ID infrastructure
- ✅ Remove `identity_id` field
- ✅ Database schema v2

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Migration breaks existing contacts | Backup before migration; rollback support |
| Cross-version incompatibility | Maintain resolver in all versions |
| Performance (extra lookups) | Index all ID types; cache resolved IDs |
| User data loss | Export/import backup before upgrade |

## Documentation Updates

- [ ] Update DOCUMENTATION.md with ID unification
- [ ] Update API.md with resolve_identity()
- [ ] Add MIGRATION_GUIDE.md for developers
- [ ] Update mobile integration guides

## Conclusion

**ID unification is CRITICAL for v0.2.1 stability.**

Current complexity causes:
- Delivery failures
- Contact lookup errors
- Developer confusion

Unified system provides:
- ✅ Single source of truth
- ✅ Clear resolution path
- ✅ Foundation for device IDs
- ✅ Easier debugging

**Estimated Total Effort:** 8-12 hours
**Priority:** HIGH (blocking v0.2.1 release)
