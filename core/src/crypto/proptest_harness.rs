//! Property-based verification harness for SCMessenger crypto module.
//!
//! Tests cryptographic invariants via `proptest` fuzzing:
//! - Encrypt/decrypt roundtrips recover original plaintext
//! - Different inputs produce different ciphertext (no key reuse)
//! - Wrong keys always fail decryption
//! - Envelope field invariants (lengths, format)
//! - Ratchet forward secrecy (unique keys per message)
//!
//! Backup module (PBKDF2) is excluded from proptest due to 600K-iteration
//! cost per case; covered by unit tests in backup.rs instead.

use crate::crypto::{
    decrypt_message, encrypt_message, sign_envelope, verify_envelope,
    RatchetSession, ed25519_to_x25519_secret,
};
use ed25519_dalek::SigningKey;
use proptest::prelude::*;
use rand::RngCore;
use x25519_dalek::PublicKey as X25519PublicKey;
use zeroize::Zeroize;

fn generate_keypair() -> SigningKey {
    let mut secret = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut secret);
    let key = SigningKey::from_bytes(&secret);
    secret.zeroize();
    key
}

fn arb_plaintext() -> impl Strategy<Value = Vec<u8>> {
    any::<Vec<u8>>().prop_filter("limit 4KB", |v| v.len() <= 4096)
}

fn arb_nonempty_plaintext() -> impl Strategy<Value = Vec<u8>> {
    any::<Vec<u8>>().prop_filter("non-empty", |v| !v.is_empty() && v.len() <= 4096)
}

proptest! {
    #[test]
    fn proptest_encrypt_decrypt_roundtrip(plaintext in arb_plaintext()) {
        let sender = generate_keypair();
        let recipient = generate_keypair();
        let recipient_pub = recipient.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender, &recipient_pub, &plaintext)
            .expect("encryption should not fail");
        let decrypted = decrypt_message(&recipient, &envelope)
            .expect("decrypt with correct key must succeed");

        prop_assert_eq!(decrypted, plaintext);
    }
}

proptest! {
    #[test]
    fn proptest_different_ciphertexts_same_plaintext(
        plaintext in arb_nonempty_plaintext()
    ) {
        let sender = generate_keypair();
        let recipient = generate_keypair();
        let recipient_pub = recipient.verifying_key().to_bytes();

        let env1 = encrypt_message(&sender, &recipient_pub, &plaintext).unwrap();
        let env2 = encrypt_message(&sender, &recipient_pub, &plaintext).unwrap();

        prop_assert_ne!(env1.ciphertext, env2.ciphertext);
        prop_assert_ne!(env1.ephemeral_public_key, env2.ephemeral_public_key);
        prop_assert_ne!(env1.nonce, env2.nonce);
    }
}

proptest! {
    #[test]
    fn proptest_wrong_key_fails(plaintext in arb_nonempty_plaintext()) {
        let sender = generate_keypair();
        let recipient = generate_keypair();
        let wrong = generate_keypair();
        let recipient_pub = recipient.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender, &recipient_pub, &plaintext).unwrap();
        let result = decrypt_message(&wrong, &envelope);

        prop_assert!(result.is_err());
    }
}

proptest! {
    #[test]
    fn proptest_envelope_field_lengths(plaintext in arb_plaintext()) {
        let sender = generate_keypair();
        let recipient = generate_keypair();
        let recipient_pub = recipient.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender, &recipient_pub, &plaintext).unwrap();

        prop_assert_eq!(envelope.ephemeral_public_key.len(), 32);
        prop_assert_eq!(envelope.nonce.len(), 24);
    }
}

proptest! {
    #[test]
    fn proptest_sign_verify_roundtrip(plaintext in arb_plaintext()) {
        let sender = generate_keypair();
        let recipient = generate_keypair();
        let recipient_pub = recipient.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender, &recipient_pub, &plaintext).unwrap();
        let signed = sign_envelope(envelope, &sender).unwrap();

        prop_assert!(verify_envelope(&signed).is_ok());
    }
}

proptest! {
    #[test]
    fn proptest_ratchet_roundtrip(plaintext in arb_nonempty_plaintext()) {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let alice_x25519 = {
            let secret = ed25519_to_x25519_secret(&alice_key);
            X25519PublicKey::from(&secret)
        };
        let bob_x25519 = {
            let secret = ed25519_to_x25519_secret(&bob_key);
            X25519PublicKey::from(&secret)
        };

        let mut alice_session = RatchetSession::init_as_sender(
            &alice_key, &bob_x25519
        ).unwrap();
        let mut bob_session = RatchetSession::init_as_receiver(
            &bob_key, &alice_x25519
        ).unwrap();

        let encrypted = alice_session.encrypt(&plaintext, b"aad").unwrap();
        let decrypted = bob_session.decrypt(
            &encrypted.our_dh_public,
            encrypted.message_number,
            &encrypted.nonce,
            &encrypted.ciphertext,
            b"aad",
        ).unwrap();

        prop_assert_eq!(decrypted, plaintext);
    }
}

proptest! {
    #[test]
    fn proptest_ratchet_forward_secrecy(plaintext in arb_nonempty_plaintext()) {
        let alice_key = generate_keypair();
        let bob_key = generate_keypair();
        let bob_x25519 = {
            let secret = ed25519_to_x25519_secret(&bob_key);
            X25519PublicKey::from(&secret)
        };

        let mut alice_session = RatchetSession::init_as_sender(
            &alice_key, &bob_x25519
        ).unwrap();

        let env1 = alice_session.encrypt(&plaintext, b"aad").unwrap();
        let env2 = alice_session.encrypt(&plaintext, b"aad").unwrap();

        prop_assert_ne!(env1.ciphertext, env2.ciphertext);
    }
}