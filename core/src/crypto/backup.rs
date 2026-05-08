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

/// Derive a 32-byte key from passphrase using PBKDF2-HMAC-SHA256 and a provided salt.
fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], IronCoreError> {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(
        passphrase.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
    );

    Ok(key)
}

/// Encrypt a backup payload using XChaCha20-Poly1305 with a PBKDF2-derived key.
/// Supports an optional custom 16-byte salt (e.g. from touch-screen entropy).
/// If `None`, a random 16-byte salt is generated internally.
///
/// # Arguments
/// * `payload` - The plaintext string to encrypt
/// * `passphrase` - The passphrase used to derive the encryption key
/// * `custom_salt` - An optional 16-byte salt
///
/// # Returns
/// Hex-encoded string containing the salt, the nonce, and the ciphertext (with tag).
///
/// # Errors
/// Returns `IronCoreError::CryptoError` if key derivation or encryption fails.
pub fn encrypt_backup(
    payload: &str,
    passphrase: &str,
    custom_salt: Option<&[u8; 16]>,
) -> Result<String, IronCoreError> {
    use rand::RngCore;

    // Determine or generate the 16-byte salt
    let mut salt_bytes = [0u8; 16];
    if let Some(s) = custom_salt {
        salt_bytes.copy_from_slice(s);
    } else {
        rand::thread_rng().fill_bytes(&mut salt_bytes);
    }

    // Generate a cryptographically secure random 24-byte nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    // Derive key using the salt
    let key = derive_key(passphrase, &salt_bytes)?;

    // Initialize cipher and encrypt
    let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|_| IronCoreError::CryptoError)?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload.as_bytes())
        .map_err(|_| IronCoreError::CryptoError)?;

    // Combine salt, nonce, and ciphertext (with tag) into a single buffer, then hex-encode
    let mut combined = Vec::with_capacity(16 + NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&salt_bytes);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(hex::encode(combined))
}

/// Decrypt a backup payload that was encrypted with `encrypt_backup` (supporting custom and legacy salts).
///
/// # Arguments
/// * `encrypted_hex` - Hex-encoded string containing the salt, nonce, and ciphertext
/// * `passphrase` - The passphrase used to derive the decryption key
///
/// # Returns
/// The decrypted plaintext string.
///
/// # Errors
/// Returns `IronCoreError::CryptoError` if decryption fails.
/// Returns `IronCoreError::InvalidInput` if the hex string or data length is invalid.
pub fn decrypt_backup(encrypted_hex: &str, passphrase: &str) -> Result<String, IronCoreError> {
    // Decode the hex string
    let data = hex::decode(encrypted_hex).map_err(|_| IronCoreError::InvalidInput)?;

    // Try new format first if length is sufficient: salt (16) + nonce (24) + tag (16) = 56 bytes
    if data.len() >= 16 + NONCE_LEN + TAG_LEN {
        let (salt_bytes, rest) = data.split_at(16);
        let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

        // Derive key from passphrase and extracted salt
        if let Ok(key) = derive_key(passphrase, salt_bytes) {
            if let Ok(cipher) = XChaCha20Poly1305::new_from_slice(&key) {
                let nonce = XNonce::from_slice(nonce_bytes);
                if let Ok(plaintext) = cipher.decrypt(nonce, ciphertext) {
                    if let Ok(plaintext_str) = String::from_utf8(plaintext) {
                        return Ok(plaintext_str);
                    }
                }
            }
        }
    }

    // Fallback to legacy format: nonce (24) + tag (16) = 40 bytes minimum
    if data.len() >= MIN_DATA_LEN {
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);

        // Derive key from passphrase using legacy deterministic Blake3 hash of the passphrase as salt
        let salt = blake3::hash(passphrase.as_bytes());
        let key = derive_key(passphrase, salt.as_bytes())?;

        let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|_| IronCoreError::CryptoError)?;
        let nonce = XNonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| IronCoreError::CryptoError)?;

        return String::from_utf8(plaintext).map_err(|_| IronCoreError::CryptoError);
    }

    Err(IronCoreError::InvalidInput)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let payload = r#"{"version":1,"secret_key_hex":"aabbccdd","nickname":"test"}"#;
        let passphrase = "correct-horse-battery-staple";

        let encrypted = encrypt_backup(payload, passphrase, None).unwrap();
        let decrypted = decrypt_backup(&encrypted, passphrase).unwrap();

        assert_eq!(payload, decrypted);
    }

    #[test]
    fn test_custom_salt_encrypt_decrypt() {
        let payload = "my custom salt data";
        let passphrase = "my-secret-passphrase";
        let salt = [42u8; 16];

        let encrypted = encrypt_backup(payload, passphrase, Some(&salt)).unwrap();
        let decrypted = decrypt_backup(&encrypted, passphrase).unwrap();

        assert_eq!(payload, decrypted);
    }

    #[test]
    fn test_decrypt_wrong_passphrase_fails() {
        let payload = "sensitive data";
        let passphrase = "correct-passphrase";

        let encrypted = encrypt_backup(payload, passphrase, None).unwrap();
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

        let encrypted_a = encrypt_backup(payload, "passphrase-a", None).unwrap();
        let encrypted_b = encrypt_backup(payload, "passphrase-b", None).unwrap();

        // Different passphrases → different keys → different ciphertexts
        assert_ne!(encrypted_a, encrypted_b);
    }
}
