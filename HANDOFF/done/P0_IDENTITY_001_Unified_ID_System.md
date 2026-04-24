# Unified Identity ID System Across All App Variants

**Date:** 2026-04-23
**Priority:** P0
**Status:** todo

## Problem Statement

The current identity system uses multiple incompatible ID formats across app variants, causing:
- CLI (`scm contact add`) rejects identity hashes (expects libp2p Peer ID `12D3Koo...`)
- Android app displays identity hash (`caccf865...`) as "Peer ID" instead of libp2p network Peer ID
- WebSocket/API endpoints cannot route messages between variants due to ID mismatch
- Ledger stores addresses but lookup by identity hash fails

## Affected Variants

| Variant | Identity Format | Network ID Format | Display Issue |
|---------|----------------|-------------------|---------------|
| CLI (Rust) | `identity.id` | `12D3Koo...` (libp2p) | Correct |
| Android | `identity.id` | `12D3Koo...` (libp2p) | Shows identity hash as Peer ID |
| WASM | TBD | TBD | Unknown |
| iOS | TBD | TBD | Unknown |

## Root Cause Analysis

1. **Android UI Layer:** `SettingsViewModel.getIdentityInfo()` returns `IdentityInfo` object; UI displays `info.id` (identity hash) instead of deriving/displaying the libp2p Peer ID
2. **QR Code / Sharing:** Exporting identity uses identity hash, but other user cannot add via `scm contact add` because it expects libp2p Peer ID
3. **Missing Cross-Reference:** No `identity.id` → `libp2p_peer_id` mapping exposed to users

## Required Fixes

### 1. Android Display Fix
- **File:** `android/app/src/main/java/com/scmessenger/android/ui/screens/SettingsScreen.kt` (or equivalent)
- **Change:** Display the actual libp2p Peer ID (derived from identity public key) as "Peer ID"
- **Add:** Display identity hash as "Identity Hash" or "Device ID" (secondary)

### 2. QR Code / Sharing Fix
- **File:** `android/app/src/main/java/com/scmessenger/android/ui/viewmodels/SettingsViewModel.kt`
- **Change:** QR code and share intent must encode the libp2p Peer ID + public key, not identity hash
- **Validation:** Ensure scanned QR code from Android can be added via `scm contact add <peer_id> <pubkey>`

### 3. CLI Validation Fix
- **File:** `cli/src/main.rs` — `cmd_init()`, `cmd_identity()`
- **Change:** After `initialize_identity()`, derive and display both:
  - Identity hash (stable, for backup/recovery)
  - libp2p Peer ID (for network routing)

### 4. Cross-Reference API
- **File:** `core/src/api.udl` and `core/src/lib.rs`
- **Add:** `get_libp2p_peer_id()` method to `IronCore` that derives Peer ID from identity public key
- **Expose:** To Kotlin (Android) and Swift (iOS) bindings

### 5. Ledger / Contact Store Unification
- **File:** `core/src/store/ContactManager`, `core/src/transport/escalation.rs`
- **Change:** Ensure contact lookup works by either identity hash OR libp2p Peer ID
- **Migration:** Add `peer_id` column to contact store if missing

## Acceptance Criteria

- [ ] Android Settings screen shows correct libp2p Peer ID as "Peer ID"
- [ ] Android QR code shares libp2p Peer ID (testable via CLI `scm contact add`)
- [ ] CLI `scm identity` shows both identity hash and libp2p Peer ID
- [ ] Contact add works with Android-displayed Peer ID across all variants
- [ ] Unit tests verify identity hash ↔ Peer ID round-trip
- [ ] Documentation updated with unified ID glossary

## Related Issues

- `P0_CLI_002_LAN_Message_Test.md` — Blocked by Peer ID mismatch
- `P0_ANDROID_017_Settings_ANR_Debug.md` — Settings screen is where ID is displayed incorrectly

## Notes

- Identity hash = blake3(identity_public_key) — stable, for backup/recovery
- libp2p Peer ID = multihash(ed25519_public_key) — network routing
- Both derive from same keypair but are different formats
- Users need network Peer ID to connect; identity hash is for recovery only
