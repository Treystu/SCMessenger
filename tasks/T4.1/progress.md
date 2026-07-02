# T4.1 — Ratchet session persistence audit & restart safety

**Status:** completed
**Track:** 4 (Cryptographic Identity, Anti-Entropy & UI Hardening)
**Dependencies:** T5.4
**Blocks:** T4.5

## Technical Context
- `crypto/session_manager.rs` (ratchet session persistence), `crypto/ratchet.rs` (`MAX_SKIP_KEYS=64`, zeroizing keys), `identity/store.rs` (sled-backed)
- Risk class: app restart mid-conversation losing ratchet state -> permanent decrypt failure

## Implementation
1. Verification-first: integration test that (1) establishes ratchet A<->B, exchanges 10 messages, (2) serializes/persists A's session, drops A's process state, rehydrates, (3) continues conversation both directions including an out-of-order message from before the restart
2. Audit that persisted session material is encrypted-at-rest or at minimum that the threat model (device storage = trusted) is documented
3. Confirm `Zeroize` on the serialization buffers (a `Zeroizing<Vec<u8>>` wrapper on the encode path)

## Edge Cases
- Skipped-keys map across restart (out-of-order buffer must survive persistence or the test above fails)
- Concurrent session mutation during flush (parking_lot guards exist — verify no deadlock with sled flush reentrance, `identity/store.rs` reopens-on-drop pattern is suspicious here)

## Verification
- [x] New `core/tests/integration_ratchet_persistence.rs` green
- [x] Proptest: random persist/restore points in a 200-message exchange never produce decrypt failure beyond the documented MAX_SKIP_KEYS window
