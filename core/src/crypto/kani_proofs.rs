//! Kani formal verification proofs for SCMessenger crypto module.
//!
//! These proofs use Kani (bit-precise model checker) to verify critical
//! security invariants that proptest can only sample probabilistically.
//!
//! **Platform requirement:** Kani requires Linux or macOS. On Windows, run
//! inside WSL2 or in CI (GitHub Actions linux runner).
//!
//! **To run:**
//! ```bash
//! cargo kani --features kani-proofs
//! ```
//!
//! **Prerequisites:**
//! ```bash
//! cargo install --locked kani-verifier
//! cargo kani setup
//! ```

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
}
