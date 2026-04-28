// Cryptography module — message encryption and key exchange

pub mod backup;
pub mod encrypt;
pub mod ratchet;
pub mod session_manager;

#[cfg(test)]
mod proptest_harness;

#[cfg(feature = "kani-proofs")]
mod kani_proofs;

pub use encrypt::{
    decrypt_message, decrypt_message_ratcheted, decrypt_with_ratchet_fallback,
    ed25519_public_to_x25519, ed25519_to_x25519_secret, encrypt_message,
    encrypt_message_ratcheted, encrypt_with_ratchet_fallback, is_ratcheted_envelope,
    sign_envelope, validate_ed25519_public_key, verify_envelope,
};
pub use ratchet::{RatchetEncryptResult, RatchetKey, RatchetSession};
pub use session_manager::{RatchetSessionManager, SerializableRatchetSession};
