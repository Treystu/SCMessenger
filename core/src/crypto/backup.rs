//! Backup encryption utilities using PBKDF2 and XChaCha20-Poly1305
//!
//! Encrypted identity backups use:
//! - PBKDF2-HMAC-SHA256 for key derivation (600,000 iterations)
//! - Blake3 hash of passphrase as salt (deterministic per passphrase)
//! - XChaCha20-Poly1305 for authenticated encryption (24-byte random nonce)
//! - Output format: hex(nonce || ciphertext_with_tag)

use crate::IronCoreError;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// PBKDF2 iteration count — 600,000 per OWASP 2023 recommendation.
const PBKDF2_ITERATIONS: u32 = 600_000;

/// Key length for XChaCha20-Poly1305 (256 bits = 32 bytes).
const KEY_LEN: usize = 32;

/// Nonce length for XChaCha20 (192 bits = 24 bytes).
const NONCE_LEN: usize = 24;

/// Tag length for Poly1305 (128 bits = 16 bytes).
const TAG_LEN: usize = 16;

/// Minimum encrypted data length: nonce (24) + tag (16) = 40 bytes.
const MIN_DATA_LEN: usize = NONCE_LEN + TAG_LEN;

/// Derive a 32-byte key from passphrase using PBKDF2-HMAC-SHA256.
fn derive_key(passphrase: &str) -> Result<[u8; KEY_LEN], IronCoreError> {
    // Use Blake3 hash of the passphrase as the salt (deterministic, 32 bytes)
    let salt = blake3::hash(passphrase.as_bytes());

    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        passphrase.as_bytes(),
        salt.as_bytes(),
        PBKDF2_ITERATIONS,
        &mut key,
    );

    Ok(key)
}

/// Encrypt a backup payload using XChaCha20-Poly1305 with a PBKDF2-derived key.
///
/// # Arguments
/// * `payload` - The plaintext string to encrypt
/// * `passphrase` - The passphrase used to derive the encryption key
///
/// # Returns
/// Hex-encoded string containing the nonce followed by the ciphertext and authentication tag.
///
/// # Errors
/// Returns `IronCoreError::CryptoError` if key derivation or encryption fails.
pub fn encrypt_backup(payload: &str, passphrase: &str) -> Result<String, IronCoreError> {
    use rand::RngCore;

    // Generate a cryptographically secure random 24-byte nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    // Derive key from passphrase
    let key = derive_key(passphrase)?;

    // Initialize cipher and encrypt
    let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|_| IronCoreError::CryptoError)?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload.as_bytes())
        .map_err(|_| IronCoreError::CryptoError)?;

    // Combine nonce and ciphertext (with tag) into a single buffer, then hex-encode
    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(hex::encode(combined))
}

/// Decrypt a backup payload that was encrypted with `encrypt_backup`.
///
/// # Arguments
/// * `encrypted_hex` - Hex-encoded string containing the nonce and ciphertext
/// * `passphrase` - The passphrase used to derive the decryption key
///
/// # Returns
/// The decrypted plaintext string.
///
/// # Errors
/// Returns `IronCoreError::CryptoError` if decryption fails (wrong passphrase, corrupted data, etc.).
/// Returns `IronCoreError::InvalidInput` if the hex string or data length is invalid.
pub fn decrypt_backup(encrypted_hex: &str, passphrase: &str) -> Result<String, IronCoreError> {
    // Decode the hex string
    let data = hex::decode(encrypted_hex).map_err(|_| IronCoreError::InvalidInput)?;

    // Validate minimum length: nonce (24) + tag (16)
    if data.len() < MIN_DATA_LEN {
        return Err(IronCoreError::InvalidInput);
    }

    // Split into nonce and ciphertext (which includes the authentication tag)
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);

    // Derive key from passphrase
    let key = derive_key(passphrase)?;

    // Initialize cipher and decrypt
    let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|_| IronCoreError::CryptoError)?;
    let nonce = XNonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| IronCoreError::CryptoError)?;

    String::from_utf8(plaintext).map_err(|_| IronCoreError::CryptoError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let payload = r#"{"version":1,"secret_key_hex":"aabbccdd","nickname":"test"}"#;
        let passphrase = "correct-horse-battery-staple";

        let encrypted = encrypt_backup(payload, passphrase).unwrap();
        let decrypted = decrypt_backup(&encrypted, passphrase).unwrap();

        assert_eq!(payload, decrypted);
    }

    #[test]
    fn test_decrypt_wrong_passphrase_fails() {
        let payload = "sensitive data";
        let passphrase = "correct-passphrase";

        let encrypted = encrypt_backup(payload, passphrase).unwrap();
        let result = decrypt_backup(&encrypted, "wrong-passphrase");

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_invalid_hex_fails() {
        let result = decrypt_backup("not-valid-hex!!", "passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_truncated_data_fails() {
        // Only 24 bytes of nonce, no ciphertext
        let short_data = hex::encode([0u8; 24]);
        let result = decrypt_backup(&short_data, "passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_different_passphrases_produce_different_ciphertexts() {
        let payload = "same payload";

        let encrypted_a = encrypt_backup(payload, "passphrase-a").unwrap();
        let encrypted_b = encrypt_backup(payload, "passphrase-b").unwrap();

        // Different passphrases → different keys → different ciphertexts
        assert_ne!(encrypted_a, encrypted_b);
    }
}
