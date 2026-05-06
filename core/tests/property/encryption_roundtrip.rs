// Property-based tests for encryption round-trip
// Validates: Requirements 3.4, 13.4

use proptest::prelude::*;
use scmessenger_core::crypto::encrypt::{decrypt_message, encrypt_message};
use ed25519_dalek::SigningKey;
use rand::RngCore;
use zeroize::Zeroize;

/// Generate a random Ed25519 signing key for testing
fn generate_keypair() -> SigningKey {
    let mut secret = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut secret);
    let key = SigningKey::from_bytes(&secret);
    secret.zeroize();
    key
}

// Strategy for generating arbitrary plaintext (0-1024 bytes)
fn arb_plaintext() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..1024)
}

proptest! {
    /// Property 1: Encryption/decryption round-trip consistency
    /// Validates: Requirements 3.4, 13.4
    /// Property: decrypt(encrypt(plaintext, recipient_key), recipient_key) == plaintext
    #[test]
    fn test_encryption_roundtrip(plaintext in arb_plaintext()) {
        // Generate sender and recipient keypairs
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();
        
        // Encrypt
        let envelope = encrypt_message(&sender_key, &recipient_public, &plaintext)
            .expect("encryption should succeed");
        
        // Decrypt
        let decrypted = decrypt_message(&recipient_key, &envelope)
            .expect("decryption should succeed");
        
        // Property: Decrypted plaintext should match original
        prop_assert_eq!(plaintext, decrypted, "Round-trip encryption should preserve plaintext");
    }

    /// Property 2: Different plaintexts produce different ciphertexts
    /// Validates: Requirements 13.4 (encryption correctness)
    /// Property: encrypt(plaintext1) != encrypt(plaintext2) when plaintext1 != plaintext2
    #[test]
    fn test_different_plaintexts_different_ciphertexts(
        plaintext1 in arb_plaintext(),
        plaintext2 in arb_plaintext(),
    ) {
        // Skip if plaintexts are identical
        prop_assume!(plaintext1 != plaintext2);
        
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();
        
        let envelope1 = encrypt_message(&sender_key, &recipient_public, &plaintext1)
            .expect("encryption should succeed");
        let envelope2 = encrypt_message(&sender_key, &recipient_public, &plaintext2)
            .expect("encryption should succeed");
        
        // Property: Different plaintexts should produce different ciphertexts
        prop_assert_ne!(
            envelope1.ciphertext,
            envelope2.ciphertext,
            "Different plaintexts should produce different ciphertexts"
        );
    }

    /// Property 3: Same plaintext produces different ciphertexts (nonce randomness)
    /// Validates: Requirements 13.4 (nonce uniqueness)
    /// Property: encrypt(plaintext) != encrypt(plaintext) due to random nonces
    #[test]
    fn test_same_plaintext_different_ciphertexts(plaintext in arb_plaintext()) {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();
        
        // Encrypt same plaintext twice
        let envelope1 = encrypt_message(&sender_key, &recipient_public, &plaintext)
            .expect("encryption should succeed");
        let envelope2 = encrypt_message(&sender_key, &recipient_public, &plaintext)
            .expect("encryption should succeed");
        
        // Property: Nonces should be different (randomness)
        prop_assert_ne!(
            envelope1.nonce,
            envelope2.nonce,
            "Each encryption should use a unique random nonce"
        );
        
        // Property: Ephemeral keys should be different
        prop_assert_ne!(
            envelope1.ephemeral_public_key,
            envelope2.ephemeral_public_key,
            "Each encryption should use a unique ephemeral keypair"
        );
        
        // Property: Ciphertexts should be different
        prop_assert_ne!(
            envelope1.ciphertext,
            envelope2.ciphertext,
            "Same plaintext should produce different ciphertexts due to randomness"
        );
    }

    /// Property 4: Wrong recipient cannot decrypt
    /// Validates: Requirements 3.4 (encryption security)
    /// Property: decrypt(encrypt(plaintext, recipient1), recipient2) fails when recipient1 != recipient2
    #[test]
    fn test_wrong_recipient_cannot_decrypt(plaintext in arb_plaintext()) {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let wrong_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();
        
        // Encrypt for recipient_key
        let envelope = encrypt_message(&sender_key, &recipient_public, &plaintext)
            .expect("encryption should succeed");
        
        // Try to decrypt with wrong_key
        let result = decrypt_message(&wrong_key, &envelope);
        
        // Property: Decryption should fail
        prop_assert!(result.is_err(), "Wrong recipient should not be able to decrypt");
    }

    /// Property 5: Tampered ciphertext fails decryption
    /// Validates: Requirements 13.4 (authenticated encryption)
    /// Property: decrypt(tamper(encrypt(plaintext))) fails
    #[test]
    fn test_tampered_ciphertext_fails(plaintext in arb_plaintext()) {
        // Skip empty plaintexts (no ciphertext to tamper)
        prop_assume!(!plaintext.is_empty());
        
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();
        
        let mut envelope = encrypt_message(&sender_key, &recipient_public, &plaintext)
            .expect("encryption should succeed");
        
        // Tamper with the last byte of ciphertext
        if let Some(byte) = envelope.ciphertext.last_mut() {
            *byte ^= 0xFF;
        }
        
        // Try to decrypt tampered ciphertext
        let result = decrypt_message(&recipient_key, &envelope);
        
        // Property: Decryption should fail (authenticated encryption detects tampering)
        prop_assert!(result.is_err(), "Tampered ciphertext should fail decryption");
    }

    /// Property 6: Tampered nonce fails decryption
    /// Validates: Requirements 13.4 (authenticated encryption)
    /// Property: decrypt with wrong nonce fails
    #[test]
    fn test_tampered_nonce_fails(plaintext in arb_plaintext()) {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();
        
        let mut envelope = encrypt_message(&sender_key, &recipient_public, &plaintext)
            .expect("encryption should succeed");
        
        // Tamper with the nonce
        if let Some(byte) = envelope.nonce.last_mut() {
            *byte ^= 0xFF;
        }
        
        // Try to decrypt with tampered nonce
        let result = decrypt_message(&recipient_key, &envelope);
        
        // Property: Decryption should fail
        prop_assert!(result.is_err(), "Tampered nonce should fail decryption");
    }

    /// Property 7: Sender public key binding prevents spoofing
    /// Validates: Requirements 13.4 (sender authentication)
    /// Property: Changing sender_public_key in envelope fails decryption
    #[test]
    fn test_sender_spoofing_fails(plaintext in arb_plaintext()) {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let attacker_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();
        
        let mut envelope = encrypt_message(&sender_key, &recipient_public, &plaintext)
            .expect("encryption should succeed");
        
        // Attacker tries to replace sender public key
        envelope.sender_public_key = attacker_key.verifying_key().to_bytes().to_vec();
        
        // Try to decrypt with spoofed sender
        let result = decrypt_message(&recipient_key, &envelope);
        
        // Property: Decryption should fail (AAD binding prevents sender spoofing)
        prop_assert!(result.is_err(), "Sender spoofing should fail decryption");
    }
}

/// Property 8: Empty plaintext encryption
/// Validates: Requirements 13.8 (edge case: empty input)
#[test]
fn test_empty_plaintext_roundtrip() {
    let sender_key = generate_keypair();
    let recipient_key = generate_keypair();
    let recipient_public = recipient_key.verifying_key().to_bytes();
    
    let plaintext = b"";
    
    let envelope = encrypt_message(&sender_key, &recipient_public, plaintext)
        .expect("encryption should succeed");
    let decrypted = decrypt_message(&recipient_key, &envelope)
        .expect("decryption should succeed");
    
    assert_eq!(plaintext.to_vec(), decrypted, "Empty plaintext should round-trip");
}

/// Property 9: Maximum size plaintext encryption
/// Validates: Requirements 13.8 (edge case: maximum sizes)
#[test]
fn test_max_size_plaintext_roundtrip() {
    let sender_key = generate_keypair();
    let recipient_key = generate_keypair();
    let recipient_public = recipient_key.verifying_key().to_bytes();
    
    // Test with 64KB plaintext
    let plaintext = vec![0x42u8; 65536];
    
    let envelope = encrypt_message(&sender_key, &recipient_public, &plaintext)
        .expect("encryption should succeed");
    let decrypted = decrypt_message(&recipient_key, &envelope)
        .expect("decryption should succeed");
    
    assert_eq!(plaintext, decrypted, "Large plaintext should round-trip");
}

/// Property 10: Envelope structure validation
/// Validates: Requirements 13.1 (data structure correctness)
#[test]
fn test_envelope_structure() {
    let sender_key = generate_keypair();
    let recipient_key = generate_keypair();
    let recipient_public = recipient_key.verifying_key().to_bytes();
    
    let plaintext = b"test message";
    
    let envelope = encrypt_message(&sender_key, &recipient_public, plaintext)
        .expect("encryption should succeed");
    
    // Validate envelope structure
    assert_eq!(envelope.sender_public_key.len(), 32, "Sender public key should be 32 bytes");
    assert_eq!(envelope.ephemeral_public_key.len(), 32, "Ephemeral public key should be 32 bytes");
    assert_eq!(envelope.nonce.len(), 24, "Nonce should be 24 bytes (XChaCha20)");
    assert!(!envelope.ciphertext.is_empty(), "Ciphertext should not be empty");
    
    // Ciphertext should be plaintext + 16 bytes (Poly1305 tag)
    assert_eq!(
        envelope.ciphertext.len(),
        plaintext.len() + 16,
        "Ciphertext should be plaintext + 16-byte authentication tag"
    );
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_property_test_strategies_compile() {
        // Smoke test to ensure strategies compile
        let _ = arb_plaintext();
    }
}
