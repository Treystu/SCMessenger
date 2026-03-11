# ID Unification Implementation Plan

**Status:** Active Implementation
**Date:** 2026-03-10
**Tracking:** WS13+ Core Work

## Executive Summary

SCMessenger currently uses multiple identifier types across different layers:
- **Public Key** (canonical identity - Ed25519 64-char hex)
- **Identity ID** (Blake3 hash of public key - derived display ID)
- **PeerId** (libp2p transport routing ID - derived from Ed25519)
- **Device ID** (planned - not yet implemented)

This document standardizes ID usage, eliminates case sensitivity issues, and adds Device ID support.

---

## Current ID Types and Usage

### 1. Public Key (Canonical Identity)
- **Format:** 64-character hex string (Ed25519 public key)
- **Purpose:** Primary identity - persistent across all platforms
- **Usage:** Contacts, encryption, signatures
- **Case:** Lowercase hex (enforced)

### 2. Identity ID (Display/Derived)
- **Format:** 64-character hex string (Blake3 hash of public key)
- **Purpose:** Display identifier, legacy compatibility
- **Usage:** UI display (deprecated in favor of nickname)
- **Case:** Lowercase hex (enforced)
- **Note:** NOT canonical - for display only

### 3. libp2p PeerId
- **Format:** Base58-encoded multihash (starts with "12D3Koo...")
- **Purpose:** Transport-layer routing
- **Usage:** Network discovery, connection management
- **Case:** Case-sensitive (Base58)
- **Note:** NOT canonical - derived from Ed25519 keypair

### 4. Device ID (Planned)
- **Format:** UUID v4
- **Purpose:** Device-specific identifier for multi-device support
- **Usage:** Device pairing, blocking granularity
- **Case:** Lowercase (enforced)
- **Note:** TODO - not yet implemented

---

## Unification Rules

### Primary Identifier: Public Key
**ALL identity operations MUST use `public_key_hex` as the canonical identifier.**

1. **Contacts:** Store and query by `public_key`
2. **Messages:** Route by `public_key`, resolve transport via PeerId lookup
3. **Blocking:** Block by `public_key` (identity-level)
4. **Encryption:** Use `public_key` for recipient lookup

### Transport Resolution
**PeerId is ONLY used for transport routing, never for identity.**

1. Maintain `public_key -> PeerId` mapping in memory
2. Resolve PeerId to public_key on message receipt
3. Use PeerId only for libp2p dial operations

### Display Names
**Use nickname for all UI display, fallback to truncated public_key.**

1. Never display raw PeerId to users
2. Never display Identity ID in UI (deprecated)
3. Format: `nickname` or `pub_key[0:8]...pub_key[-8:]` if no nickname

---

## Case Sensitivity Standardization

### Storage
- **Public Keys:** Always lowercase hex in database
- **PeerIds:** Case-preserved (Base58 is case-sensitive)
- **Device IDs:** Always lowercase UUID

### Comparison
```kotlin
// Public key comparison - case-insensitive
fun publicKeysEqual(a: String, b: String): Boolean {
    return a.lowercase() == b.lowercase()
}

// PeerId comparison - case-sensitive
fun peerIdsEqual(a: String, b: String): Boolean {
    return a == b
}
```

### Normalization
```kotlin
// Apply on storage
fun normalizePublicKey(key: String): String {
    return key.lowercase().trim()
}

fun normalizeDeviceId(id: String): String {
    return id.lowercase().trim()
}

// PeerId - no normalization
fun normalizePeerId(id: String): String {
    return id.trim()
}
```

---

## Device ID Implementation

### Purpose
- Support multi-device per identity
- Enable device-level blocking
- Track device-specific settings

### Schema
```rust
pub struct DeviceInfo {
    pub device_id: String,        // UUID v4
    pub public_key: String,        // Identity public key
    pub device_name: Option<String>,
    pub added_at: u64,
    pub last_active: Option<u64>,
}
```

### TODO Items
1. [ ] Add `device_id` generation on first launch
2. [ ] Store `device_id` in identity store
3. [ ] Include `device_id` in identity beacons
4. [ ] Add device-level blocking table
5. [ ] Sync device list across paired devices

**Estimated:** ~200 LoC (core) + ~150 LoC (mobile) + ~100 LoC (wasm)

---

## Implementation Checklist

### Core (Rust)
- [x] Public key as canonical ID (already implemented)
- [x] Case-insensitive public key comparison (already implemented)
- [ ] Add `device_id` to IdentityInfo
- [ ] Add `device_id` to identity beacon
- [ ] Device ID persistence
- [ ] Device blocking API

### Android
- [ ] Normalize all public_key storage to lowercase
- [ ] Replace identity_id usage with public_key
- [ ] Add device_id field to data models
- [ ] Update contact queries to use public_key
- [ ] Update message routing to resolve PeerId -> public_key
- [ ] Add nickname display fallback logic
- [ ] Implement device blocking UI

### iOS
- [ ] Normalize all publicKey storage to lowercase
- [ ] Replace identityID usage with publicKey
- [ ] Add deviceID field to data models
- [ ] Update contact queries to use publicKey
- [ ] Update message routing to resolve PeerId -> publicKey
- [ ] Add nickname display fallback logic
- [ ] Implement device blocking UI

### WASM
- [ ] Normalize all public_key storage to lowercase
- [ ] Replace identity_id usage with public_key
- [ ] Add device_id field to data models
- [ ] Update contact queries to use public_key
- [ ] Add nickname display fallback logic

---

## Migration Strategy

### Database Migration
1. Add `device_id` column to identity table (nullable initially)
2. Normalize existing `public_key` values to lowercase
3. Remove unused `identity_id` indexes
4. Create `device_id` unique index

### Runtime Migration
1. On app start, check if `device_id` exists
2. If not, generate UUID v4 and store
3. Normalize all in-memory public keys to lowercase
4. Update PeerId -> public_key resolution map

---

## Validation Tests

### ID Resolution
- [ ] Public key lookup is case-insensitive
- [ ] PeerId correctly resolves to public_key
- [ ] Device ID generates on first launch
- [ ] Device ID persists across restarts

### Display
- [ ] Contacts show nickname (not ID)
- [ ] Conversations show nickname (not ID)
- [ ] Fallback to truncated public_key works

### Blocking
- [ ] Identity-level blocking works
- [ ] Device-level blocking works (after implementation)
- [ ] Blocked users still relay messages

---

## Related Documents
- `docs/IDENTITY_BLOCKING_IMPLEMENTATION.md`
- `ANDROID_ID_MISMATCH_RCA.md`
- `CASE_SENSITIVITY_AUDIT_2026-03-09.md`
- `ID_UNIFICATION_PLAN.md`

---

## Lines of Code Estimates

| Component | LoC Estimate |
|-----------|--------------|
| Core (device_id + normalization) | ~200 |
| Android (migration + UI) | ~300 |
| iOS (migration + UI) | ~300 |
| WASM (migration + UI) | ~150 |
| Tests | ~200 |
| **Total** | **~1,150 LoC** |

---

**Next Steps:**
1. Implement device_id generation in core
2. Add normalization enforcement in storage layers
3. Update Android/iOS contact display to use nicknames
4. Add device blocking UI
5. Run full integration tests
