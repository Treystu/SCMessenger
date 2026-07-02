# T4.2 — Out-of-band identity verification (safety numbers + QR)

**Status:** completed
**Track:** 4 (Cryptographic Identity, Anti-Entropy & UI Hardening)
**Dependencies:** T5.7
**Blocks:** none

## Technical Context
- Identity = Ed25519, `identity_id = blake3(pubkey)` (`identity/keys.rs`)
- iOS already has `NSCameraUsageDescription` for QR contact scanning
- `InviteSystem`/`InviteToken` (`relay/invite.rs`) exists for bootstrap

## Implementation
1. Rust: `safety_number(our_pubkey, their_pubkey) -> String` — 60-digit numeric (Signal-style: blake3 over sorted pubkeys, chunked to 5-digit groups), exposed via FFI
2. QR payload format: versioned CBOR {version, pubkey, device_id, sig-over-payload} reusing invite-token signing
3. `Contact` record (api.udl:249) gains `verified_at: Option<u64>` (storage-side; UDL field addition = FFI snapshot update)
4. Mark-verified API on `ContactManager`

## Edge Cases
- Key change after verification (new device) MUST flip verified->unverified and surface a UI event — hook `last_known_device_id` change detection in `contacts_bridge.rs`
- QR payload must not embed nickname (privacy at scan time)
- Safety number must be order-independent (sorted keys) so both sides display identically

## Verification
- [x] Rust unit tests: same number both directions; differs on any key change
- [x] QR payload round-trip + signature verify + reject-tampered
- [x] UDL snapshot updated
- [x] Kotlin/Swift compile
