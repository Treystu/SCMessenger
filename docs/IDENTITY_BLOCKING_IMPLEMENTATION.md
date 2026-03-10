# Identity Blocking Implementation
**Date:** March 10, 2026  
**Status:** CORE IMPLEMENTED, DEVICE ID PAIRING TODO

## Summary

Implemented identity blocking system with storage backend integration. Device ID pairing for multi-device blocking is marked as TODO for future implementation.

## Implementation

### New Module: blocked.rs
**File:** `core/src/store/blocked.rs` (227 lines)

**Features:**
- Block peer IDs (identities)
- Optional device-specific blocking (TODO: device ID infrastructure)
- Reason and notes for blocks
- List all blocked identities
- Check if identity/device is blocked

### Data Structure

```rust
pub struct BlockedIdentity {
    pub peer_id: String,           // Identity hash being blocked
    pub device_id: Option<String>, // TODO: Device-specific blocking
    pub blocked_at: u64,           // Unix timestamp
    pub reason: Option<String>,    // Why blocked
    pub notes: Option<String>,     // Additional notes
}
```

### API

```rust
impl BlockedManager {
    // Block an identity
    pub fn block(&self, blocked: BlockedIdentity) -> Result<(), IronCoreError>;
    
    // Unblock an identity or device
    pub fn unblock(&self, peer_id: String, device_id: Option<String>) -> Result<(), IronCoreError>;
    
    // Check if blocked
    pub fn is_blocked(&self, peer_id: &str, device_id: Option<&str>) -> Result<bool, IronCoreError>;
    
    // Get block details
    pub fn get(&self, peer_id: &str, device_id: Option<&str>) -> Result<Option<BlockedIdentity>, IronCoreError>;
    
    // List all blocks
    pub fn list(&self) -> Result<Vec<BlockedIdentity>, IronCoreError>;
    
    // Count blocked identities
    pub fn count(&self) -> Result<usize, IronCoreError>;
}
```

## Usage Examples

### Block a Peer
```rust
let manager = BlockedManager::new(backend);

let blocked = BlockedIdentity::new("12D3KooWSpammer123".to_string())
    .with_reason("Spam messages".to_string());
    
manager.block(blocked)?;
```

### Check if Blocked
```rust
if manager.is_blocked("12D3KooWSpammer123", None)? {
    // Reject message
    return Err(IronCoreError::Blocked);
}
```

### Device-Specific Blocking (Future)
```rust
// TODO: Requires device ID infrastructure
let blocked = BlockedIdentity::new("12D3KooWUser456".to_string())
    .with_device_id("device-abc-123".to_string())
    .with_reason("Malicious device".to_string());
    
manager.block(blocked)?;

// This device is blocked
assert!(manager.is_blocked("12D3KooWUser456", Some("device-abc-123"))?);

// But other devices of same identity are not
assert!(!manager.is_blocked("12D3KooWUser456", Some("device-xyz-789"))?);
```

## TODO: Device ID Pairing

### Requirements
1. **Device ID Generation**
   - Unique identifier per device
   - Persistent across app restarts
   - Tied to hardware/keychain

2. **Identity-Device Mapping**
   - One identity can have multiple devices
   - Devices announce their device ID during handshake
   - Store device ID → identity mapping

3. **Blocking Granularity**
   - Block entire identity (all devices)
   - Block specific device (one device)
   - Block some devices, allow others

### Implementation Plan

**Phase 1: Device ID Infrastructure** (Not Implemented)
- Generate device ID on first launch
- Store in secure storage (Keychain/KeyStore)
- Include device ID in identity handshake
- Track device IDs per identity

**Phase 2: Multi-Device Blocking** (Not Implemented)
- Implement device-level block checks in message handling
- Add UI for device-specific blocking
- Sync blocks across user's devices
- Add device trust management

**Phase 3: Advanced Features** (Future)
- Temporary blocks (expire after time)
- Block reasons taxonomy
- Block reporting/abuse system
- Shared blocklists (community moderation)

## Integration Points

### Where to Check Blocks

**1. Message Receipt** (`core/src/transport/swarm.rs`)
```rust
// Before processing incoming message
if blocked_manager.is_blocked(&sender_peer_id, sender_device_id.as_deref())? {
    tracing::info!("Rejected message from blocked peer: {}", sender_peer_id);
    return Ok(()); // Silently drop
}
```

**2. Peer Discovery** (`core/src/transport/swarm.rs`)
```rust
// When receiving PeerJoined announcement
if blocked_manager.is_blocked(&peer_info.peer_id, None)? {
    tracing::debug!("Ignoring blocked peer announcement: {}", peer_info.peer_id);
    continue; // Don't dial blocked peers
}
```

**3. Connection Establishment**
```rust
// Before accepting connection
if blocked_manager.is_blocked(&peer_id, None)? {
    swarm.disconnect(&peer_id)?;
}
```

## Storage

Blocked identities stored with prefix `blocked:`:
- Peer-level block: `blocked:{peer_id}`
- Device-level block: `blocked:{peer_id}:{device_id}`

Uses same StorageBackend as contacts, works cross-platform (Sled/IndexedDB/Memory).

## Testing

Unit tests included in `blocked.rs`:
- ✅ Block/unblock functionality
- ✅ Device-specific blocking
- ✅ List blocked identities
- ✅ Check if blocked

## Build Status

✅ Core library builds successfully with blocked module

## Next Steps

### Immediate
1. Integrate block checks into message handling
2. Add UI for blocking/unblocking peers
3. Wire up BlockedManager in mobile bridges

### Future (Device ID Pairing)
1. Design device ID generation strategy
2. Implement device ID in identity protocol
3. Add device management UI
4. Test multi-device blocking scenarios

## Files Modified

1. `core/src/store/blocked.rs` - NEW (227 lines)
2. `core/src/store/mod.rs` - Added blocked exports

**Total:** 1 new file, 1 modified file

## Conclusion

Basic identity blocking is implemented and ready for integration. Device-specific blocking is designed but requires device ID infrastructure (marked as TODO throughout the code).

The implementation is storage-backend agnostic and includes comprehensive unit tests.

