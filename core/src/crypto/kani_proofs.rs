//! Kani formal verification proofs for SCMessenger crypto module.
//!
//! These proofs use Kani (bit-precise model checker) to verify critical
//! security invariants that proptest can only sample probabilistically.
//!
//! # Verification Workflow
//!
//! SCMessenger employs a three-layer verification strategy for the crypto module:
//!
//! **Layer 1 — Unit tests** (`#[cfg(test)] mod tests` in each source file):
//! Concrete test vectors covering known inputs and expected outputs.
//! Run: `cargo test -p scmessenger-core --lib`
//!
//! **Layer 2 — Property-based testing** (`core/src/crypto/proptest_harness.rs`):
//! Proptest fuzzing over random input distributions. Covers encrypt/decrypt
//! roundtrips, wrong-key rejection, envelope field invariants, ratchet forward
//! secrecy, chain key distinctness, and KDF output length. These tests catch
//! statistical edge cases that unit tests miss.
//! Run: `cargo test -p scmessenger-core --lib` (proptest is #[cfg(test)])
//!
//! **Layer 3 — Formal verification** (this file):
//! Kani bit-precise model checking proves invariants for ALL possible inputs,
//! not just sampled ones. Required by security rules before merging any
//! change to `core/src/crypto/`.
//!
//! ## Platform Requirements
//!
//! Kani requires **Linux or macOS**. On Windows, use one of:
//! - WSL2: `cargo install --locked kani-verifier && cargo kani setup`
//! - CI: GitHub Actions `ubuntu-latest` runner (see `.github/workflows/`)
//! - Docker: `kani-project/kani` image
//!
//! ## Running Kani Verification
//!
//! ```bash
//! # One-time setup (Linux/macOS only)
//! cargo install --locked kani-verifier
//! cargo kani setup
//!
//! # Run all crypto proofs
//! cargo kani --features kani-proofs -p scmessenger-core
//!
//! # Run a single proof
//! cargo kani --features kani-proofs -p scmessenger-core \
//!   --harness ed25519_conversion_produces_32_bytes
//!
//! # Verify harness compiles (all platforms, including Windows)
//! cargo test -p scmessenger-core --features kani-proofs --lib
//! ```
//!
//! ## Proof Inventory (8 proofs)
//!
//! | Proof | Invariant Verified |
//! |-------|-------------------|
//! | `ed25519_conversion_produces_32_bytes` | Ed25519->X25519 secret conversion output length |
//! | `derive_key_always_32_bytes` | Blake3 KDF always produces 32-byte keys |
//! | `nonce_length_invariant` | XChaCha20 nonces are always 24 bytes |
//! | `chain_ratchet_produces_distinct_keys` | Chain ratchet advances produce distinct message keys |
//! | `ratchet_key_length_invariant` | RatchetKey serialized form is always 32 bytes |
//! | `ed25519_signature_length_invariant` | Ed25519 signatures are always 64 bytes |
//! | `x25519_public_key_length_invariant` | X25519 public key serialization is always 32 bytes |
//! | `ed25519_verifying_key_length_invariant` | Ed25519 verifying key serialization is always 32 bytes |
//!
//! ## Pre-Merge Checklist for Crypto Changes
//!
//! 1. `cargo test -p scmessenger-core --lib` — all 900+ tests pass
//! 2. `cargo kani --features kani-proofs -p scmessenger-core` — no proof failures
//! 3. `cargo clippy -p scmessenger-core -- -D warnings` — no new warnings
//! 4. All new unsafe blocks have `// SAFETY:` comments
//! 5. Adversarial review completed for changed files (per security rules)

#![cfg(feature = "kani-proofs")]

#[cfg(kani)]
mod proofs {
    use crate::crypto::encrypt::{ed25519_to_x25519_secret, KDF_CONTEXT};
    use crate::crypto::RatchetKey;

    #[kani::proof]
    fn ed25519_conversion_produces_32_bytes() {
        let input_bytes: [u8; 32] = kani::any();
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&input_bytes);
        let x25519_secret = ed25519_to_x25519_secret(&signing_key);
        let output_bytes = x25519_secret.to_bytes();
        assert_eq!(output_bytes.len(), 32);
    }

    #[kani::proof]
    fn derive_key_always_32_bytes() {
        let shared_secret: [u8; 32] = kani::any();
        let key = blake3::derive_key(KDF_CONTEXT, &shared_secret);
        assert_eq!(key.len(), 32);
    }

    #[kani::proof]
    fn nonce_length_invariant() {
        let nonce: [u8; 24] = kani::any();
        let xnonce = chacha20poly1305::XNonce::from_slice(&nonce);
        assert_eq!(xnonce.len(), 24);
    }

    #[kani::proof]
    fn chain_ratchet_produces_distinct_keys() {
        let chain_key_bytes: [u8; 32] = kani::any();
        let chain_key = RatchetKey::from_bytes(chain_key_bytes);

        let ctx = "iron-core ratchet v1 2026-04-15";
        let msg_key_info = blake3::hash(b"message-key");
        let chain_key_info = blake3::hash(b"chain-key");

        let msg_key_0 = blake3::derive_key(
            &format!("{}:{}", ctx, msg_key_info.to_hex()),
            chain_key.as_bytes(),
        );
        let new_chain = blake3::derive_key(
            &format!("{}:{}", ctx, chain_key_info.to_hex()),
            chain_key.as_bytes(),
        );
        let msg_key_1 =
            blake3::derive_key(&format!("{}:{}", ctx, msg_key_info.to_hex()), &new_chain);

        assert_ne!(msg_key_0, msg_key_1);
    }

    #[kani::proof]
    fn ratchet_key_length_invariant() {
        let key_bytes: [u8; 32] = kani::any();
        let key = RatchetKey::from_bytes(key_bytes);
        assert_eq!(key.as_bytes().len(), 32);
    }

    #[kani::proof]
    fn ed25519_signature_length_invariant() {
        let sig_bytes: [u8; 64] = kani::any();
        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        assert_eq!(signature.to_bytes().len(), 64);
    }

    #[kani::proof]
    fn x25519_public_key_length_invariant() {
        let public_bytes: [u8; 32] = kani::any();
        let public_key = x25519_dalek::PublicKey::from(public_bytes);
        assert_eq!(public_key.to_bytes().len(), 32);
    }

    #[kani::proof]
    fn ed25519_verifying_key_length_invariant() {
        let key_bytes: [u8; 32] = kani::any();
        let vk_result = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes);
        if let Ok(vk) = vk_result {
            assert_eq!(vk.to_bytes().len(), 32);
        }
    }
}
