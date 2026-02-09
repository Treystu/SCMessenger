// Per-message encryption: X25519 ECDH + XChaCha20-Poly1305
//
// Flow:
// 1. Convert sender's Ed25519 signing key → X25519 static secret
// 2. Generate ephemeral X25519 keypair
// 3. ECDH: ephemeral_secret × recipient_x25519_public → shared_secret
// 4. KDF: Blake3::derive_key(shared_secret) → symmetric_key
// 5. Encrypt: XChaCha20-Poly1305(symmetric_key, random_nonce, plaintext)
// 6. Output: Envelope { sender_pub, ephemeral_pub, nonce, ciphertext }
//
// Recipient reverses:
// 1. Convert recipient's Ed25519 key → X25519 static secret
// 2. ECDH: recipient_secret × ephemeral_public → shared_secret
// 3. KDF: same derivation → symmetric_key
// 4. Decrypt: XChaCha20-Poly1305(symmetric_key, nonce, ciphertext)

use anyhow::{bail, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature as Ed25519Signature};
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroize;

/// KDF context string for deriving encryption keys from ECDH shared secrets.
/// Changing this breaks compatibility with all existing messages.
const KDF_CONTEXT: &str = "iron-core v2 message encryption 2026-02-05";

/// Convert an Ed25519 signing key to an X25519 static secret for ECDH.
///
/// Ed25519 and X25519 share the same underlying curve (Curve25519),
/// so we can derive X25519 keys from Ed25519 keys deterministically.
/// The conversion uses the clamped SHA-512 hash of the Ed25519 secret key,
/// which is how Ed25519 internally derives its scalar.
fn ed25519_to_x25519_secret(signing_key: &SigningKey) -> StaticSecret {
    // Ed25519 secret scalar is SHA-512(secret_key_bytes)[0..32], clamped.
    // x25519-dalek StaticSecret expects the raw 32-byte secret and does its own clamping.
    let mut hash = <sha2::Sha512 as sha2::Digest>::digest(signing_key.to_bytes());
    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&hash[..32]);

    let secret = StaticSecret::from(secret_bytes);

    // Zeroize intermediates
    secret_bytes.zeroize();
    hash.as_mut_slice().zeroize();

    secret
}

/// Convert an Ed25519 verifying (public) key to an X25519 public key.
///
/// Uses the birational map from Ed25519 (twisted Edwards) to X25519 (Montgomery).
/// This is the standard conversion: u = (1 + y) / (1 - y) mod p.
fn ed25519_public_to_x25519(public_key_bytes: &[u8; 32]) -> Result<X25519PublicKey> {
    use curve25519_dalek::edwards::CompressedEdwardsY;

    let compressed = CompressedEdwardsY::from_slice(public_key_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid Ed25519 public key"))?;

    let edwards_point = compressed
        .decompress()
        .ok_or_else(|| anyhow::anyhow!("Failed to decompress Ed25519 public key"))?;

    let montgomery = edwards_point.to_montgomery();
    Ok(X25519PublicKey::from(montgomery.to_bytes()))
}

/// Derive a symmetric encryption key from an ECDH shared secret using Blake3.
fn derive_key(shared_secret: &[u8]) -> [u8; 32] {
    blake3::derive_key(KDF_CONTEXT, shared_secret)
}

/// Encrypt a plaintext message for a recipient.
///
/// # Arguments
/// * `sender_signing_key` - Sender's Ed25519 signing key (for sender identification)
/// * `recipient_public_key` - Recipient's Ed25519 public key bytes (32 bytes)
/// * `plaintext` - The message bytes to encrypt
///
/// # Returns
/// An `Envelope` containing everything needed for decryption.
pub fn encrypt_message(
    sender_signing_key: &SigningKey,
    recipient_public_key: &[u8; 32],
    plaintext: &[u8],
) -> Result<crate::message::Envelope> {
    // Convert recipient's Ed25519 public key to X25519
    let recipient_x25519 = ed25519_public_to_x25519(recipient_public_key)?;

    // Generate ephemeral X25519 keypair for this message
    let ephemeral_secret = EphemeralSecret::random_from_rng(rand::rngs::OsRng);
    let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

    // ECDH: ephemeral_secret × recipient_public → shared_secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_x25519);

    // KDF: derive symmetric key
    let mut symmetric_key = derive_key(shared_secret.as_bytes());

    // Generate random nonce (24 bytes for XChaCha20)
    let mut nonce_bytes = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = XNonce::from_slice(&nonce_bytes);

    // Encrypt with AAD (Additional Authenticated Data)
    // Bind sender public key as AAD to prevent sender spoofing
    let sender_public_bytes = sender_signing_key.verifying_key().to_bytes();
    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let ciphertext = cipher
        .encrypt(nonce, Payload {
            msg: plaintext,
            aad: &sender_public_bytes,
        })
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Zeroize key material
    symmetric_key.zeroize();

    Ok(crate::message::Envelope {
        sender_public_key: sender_signing_key.verifying_key().to_bytes().to_vec(),
        ephemeral_public_key: ephemeral_public.to_bytes().to_vec(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Decrypt an envelope using the recipient's signing key.
///
/// # Arguments
/// * `recipient_signing_key` - Recipient's Ed25519 signing key
/// * `envelope` - The encrypted envelope
///
/// # Returns
/// The decrypted plaintext bytes.
pub fn decrypt_message(
    recipient_signing_key: &SigningKey,
    envelope: &crate::message::Envelope,
) -> Result<Vec<u8>> {
    // Validate envelope fields
    if envelope.ephemeral_public_key.len() != 32 {
        bail!("Invalid ephemeral public key length");
    }
    if envelope.nonce.len() != 24 {
        bail!("Invalid nonce length");
    }

    // Convert recipient's Ed25519 signing key to X25519 static secret
    let recipient_x25519_secret = ed25519_to_x25519_secret(recipient_signing_key);

    // Reconstruct ephemeral public key
    let mut ephemeral_bytes = [0u8; 32];
    ephemeral_bytes.copy_from_slice(&envelope.ephemeral_public_key);
    let ephemeral_public = X25519PublicKey::from(ephemeral_bytes);

    // ECDH: recipient_secret × ephemeral_public → shared_secret
    let shared_secret = recipient_x25519_secret.diffie_hellman(&ephemeral_public);

    // KDF: same derivation as encryption
    let mut symmetric_key = derive_key(shared_secret.as_bytes());

    // Reconstruct nonce
    let nonce = XNonce::from_slice(&envelope.nonce);

    // Decrypt with AAD (must match the sender public key used during encryption)
    // This prevents sender spoofing attacks
    if envelope.sender_public_key.len() != 32 {
        bail!("Invalid sender public key length");
    }
    let cipher = XChaCha20Poly1305::new_from_slice(&symmetric_key)
        .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, Payload {
            msg: envelope.ciphertext.as_ref(),
            aad: envelope.sender_public_key.as_ref(),
        })
        .map_err(|_| anyhow::anyhow!("Decryption failed: invalid ciphertext, wrong key, or tampered sender public key"))?;

    // Zeroize key material
    symmetric_key.zeroize();

    Ok(plaintext)
}

/// Sign an envelope with the sender's signing key.
///
/// Creates a canonical serialization of the envelope and signs it with Ed25519.
/// This allows relays to verify envelope authenticity without decryption.
///
/// # Arguments
/// * `envelope` - The encrypted envelope to sign
/// * `sender_signing_key` - Sender's Ed25519 signing key
///
/// # Returns
/// A `SignedEnvelope` containing the envelope and its signature.
pub fn sign_envelope(
    envelope: crate::message::Envelope,
    sender_signing_key: &SigningKey,
) -> Result<crate::message::SignedEnvelope> {
    // Create canonical representation for signing
    // We sign the serialized envelope to cover all fields
    let envelope_bytes = bincode::serialize(&envelope)
        .map_err(|e| anyhow::anyhow!("Failed to serialize envelope: {}", e))?;

    // Sign the envelope bytes
    let signature = sender_signing_key.sign(&envelope_bytes);

    Ok(crate::message::SignedEnvelope {
        envelope,
        signature: signature.to_bytes().to_vec(),
    })
}

/// Verify a signed envelope's signature.
///
/// Checks that the signature matches the envelope content and was created
/// by the sender whose public key is in the envelope. This allows relays
/// to reject forged envelopes without decrypting them.
///
/// # Arguments
/// * `signed_envelope` - The signed envelope to verify
///
/// # Returns
/// `Ok(())` if signature is valid, `Err` otherwise.
pub fn verify_envelope(signed_envelope: &crate::message::SignedEnvelope) -> Result<()> {
    // Extract sender's public key from envelope
    if signed_envelope.envelope.sender_public_key.len() != 32 {
        bail!("Invalid sender public key length");
    }

    let mut sender_public_bytes = [0u8; 32];
    sender_public_bytes.copy_from_slice(&signed_envelope.envelope.sender_public_key);

    let verifying_key = VerifyingKey::from_bytes(&sender_public_bytes)
        .map_err(|e| anyhow::anyhow!("Invalid sender public key: {}", e))?;

    // Verify signature length
    if signed_envelope.signature.len() != 64 {
        bail!("Invalid signature length");
    }

    let mut signature_bytes = [0u8; 64];
    signature_bytes.copy_from_slice(&signed_envelope.signature);

    let signature = Ed25519Signature::from_bytes(&signature_bytes);

    // Create canonical representation (same as signing)
    let envelope_bytes = bincode::serialize(&signed_envelope.envelope)
        .map_err(|e| anyhow::anyhow!("Failed to serialize envelope: {}", e))?;

    // Verify signature
    verifying_key
        .verify(&envelope_bytes, &signature)
        .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn generate_keypair() -> SigningKey {
        let mut secret = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret);
        let key = SigningKey::from_bytes(&secret);
        secret.zeroize();
        key
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Hello, this is a secret message!";

        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();
        let decrypted = decrypt_message(&recipient_key, &envelope).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_recipient_fails() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let wrong_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Wrong recipient should fail to decrypt
        let result = decrypt_message(&wrong_key, &envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message";
        let mut envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Tamper with ciphertext
        if let Some(byte) = envelope.ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt_message(&recipient_key, &envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_messages_different_ciphertext() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let env1 = encrypt_message(&sender_key, &recipient_public, b"message 1").unwrap();
        let env2 = encrypt_message(&sender_key, &recipient_public, b"message 1").unwrap();

        // Same plaintext should produce different ciphertext (different ephemeral keys + nonces)
        assert_ne!(env1.ciphertext, env2.ciphertext);
        assert_ne!(env1.ephemeral_public_key, env2.ephemeral_public_key);
        assert_ne!(env1.nonce, env2.nonce);
    }

    #[test]
    fn test_sender_public_key_in_envelope() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender_key, &recipient_public, b"hello").unwrap();

        assert_eq!(
            envelope.sender_public_key,
            sender_key.verifying_key().to_bytes().to_vec()
        );
    }

    #[test]
    fn test_empty_plaintext() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let envelope = encrypt_message(&sender_key, &recipient_public, b"").unwrap();
        let decrypted = decrypt_message(&recipient_key, &envelope).unwrap();

        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_large_plaintext() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = vec![0x42u8; 60_000]; // 60 KB
        let envelope = encrypt_message(&sender_key, &recipient_public, &plaintext).unwrap();
        let decrypted = decrypt_message(&recipient_key, &envelope).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_invalid_envelope_nonce() {
        let recipient_key = generate_keypair();
        let envelope = crate::message::Envelope {
            sender_public_key: vec![0u8; 32],
            ephemeral_public_key: vec![0u8; 32],
            nonce: vec![0u8; 12], // Wrong size (should be 24)
            ciphertext: vec![0u8; 32],
        };

        let result = decrypt_message(&recipient_key, &envelope);
        assert!(result.is_err());
    }

    #[test]
    fn test_aad_binding_prevents_sender_spoofing() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let attacker_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message";
        let mut envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Attacker tries to replace sender public key with their own
        envelope.sender_public_key = attacker_key.verifying_key().to_bytes().to_vec();

        // Decryption should fail due to AAD mismatch
        let result = decrypt_message(&recipient_key, &envelope);
        assert!(result.is_err(), "AAD binding should prevent sender spoofing");
    }

    #[test]
    fn test_sign_and_verify_envelope() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Test message for signing";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Sign the envelope
        let signed_envelope = sign_envelope(envelope, &sender_key).unwrap();

        // Verify the signature
        let result = verify_envelope(&signed_envelope);
        assert!(result.is_ok(), "Valid signature should verify successfully");
    }

    #[test]
    fn test_tampered_envelope_fails_verification() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Test message";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();
        let mut signed_envelope = sign_envelope(envelope, &sender_key).unwrap();

        // Tamper with the ciphertext
        if let Some(byte) = signed_envelope.envelope.ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        // Verification should fail
        let result = verify_envelope(&signed_envelope);
        assert!(result.is_err(), "Tampered envelope should fail verification");
    }

    #[test]
    fn test_forged_signature_fails_verification() {
        let sender_key = generate_keypair();
        let attacker_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Test message";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();

        // Attacker signs with their own key
        let mut signed_envelope = sign_envelope(envelope.clone(), &attacker_key).unwrap();

        // Replace sender public key with original sender (attempting forgery)
        signed_envelope.envelope.sender_public_key = sender_key.verifying_key().to_bytes().to_vec();

        // Verification should fail (signature doesn't match sender_public_key)
        let result = verify_envelope(&signed_envelope);
        assert!(result.is_err(), "Forged signature should fail verification");
    }

    #[test]
    fn test_relay_can_verify_without_decrypting() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();
        let recipient_public = recipient_key.verifying_key().to_bytes();

        let plaintext = b"Secret message that relay can't read";
        let envelope = encrypt_message(&sender_key, &recipient_public, plaintext).unwrap();
        let signed_envelope = sign_envelope(envelope, &sender_key).unwrap();

        // Relay can verify authenticity without knowing recipient's key
        let verification = verify_envelope(&signed_envelope);
        assert!(verification.is_ok(), "Relay should be able to verify envelope");

        // But relay still can't decrypt (would need recipient's key)
        // This demonstrates the purpose: relays can reject forged messages
        // without being able to read the content
    }
}
