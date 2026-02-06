# V1 TypeScript Crypto Reference

These files are from the SC V1 TypeScript implementation. They serve as **porting guides** for implementing equivalent algorithms in Rust. They are not running code in this project.

## Files

| File | Algorithm | Status in V2 |
|---|---|---|
| `x3dh.ts` | Extended Triple Diffie-Hellman key agreement | Future (Double Ratchet prerequisite) |
| `double-ratchet.ts` | Signal Double Ratchet protocol | Future (session-based forward secrecy) |
| `primitives.ts` | Low-level crypto primitives (ECDH, AEAD, KDF) | Ported to `core/src/crypto/encrypt.rs` |
| `envelope.ts` | Message envelope encryption/decryption | Ported to `core/src/message/types.rs` + `core/src/crypto/encrypt.rs` |
| `shamir.ts` | Shamir's Secret Sharing | Future (social key recovery) |
| `storage.ts` | Encrypted key storage | Ported to `core/src/identity/store.rs` (using sled) |

## Porting Notes

The V2 Rust implementation uses the same cryptographic primitives (Ed25519, X25519, XChaCha20-Poly1305, Blake3) but with different library implementations:

- V1: `tweetnacl`, `@stablelib/*` (JavaScript)
- V2: `ed25519-dalek`, `x25519-dalek`, `chacha20poly1305`, `blake3` (Rust)

The core encryption flow (ephemeral ECDH + AEAD) is architecturally identical between V1 and V2.
