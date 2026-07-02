//! Backup encryption utilities using Argon2id and XChaCha20-Poly1305
//!
//! Encrypted identity backups use:
//! - Argon2id for key derivation (memory-hard: resists GPU/ASIC brute force
//!   far better than PBKDF2, which is iteration-hard but cheap to
//!   parallelize on custom hardware)
//! - XChaCha20-Poly1305 for authenticated encryption (24-byte random nonce)
//! - Output format: hex(format_tag || salt || nonce || ciphertext_with_tag)
//!
//! Backups created before the switch to Argon2id (PBKDF2-HMAC-SHA256, with
//! or without a stored salt) are still decryptable: `decrypt_backup` tries
//! the current Argon2id format first, then falls back through the older
//! formats. Every new backup is encrypted with Argon2id.

use crate::IronCoreError;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

/// PBKDF2 iteration count. Retained only to decrypt backups created before
/// the switch to Argon2id — no longer used for new backups.
const PBKDF2_ITERATIONS: u32 = 600_000;

/// Format tag identifying an Argon2id-encrypted blob: `tag || salt || nonce
/// || ciphertext`. Older (untagged) blobs can't be confused with this format
/// in a way that would silently decrypt to the wrong plaintext — the AEAD
/// tag still has to authenticate correctly, so a coincidental first-byte
/// match just fails and `decrypt_backup` falls through to the older formats.
const FORMAT_TAG_ARGON2ID: u8 = 0x02;

/// Argon2id parameters, chosen per OWASP's password-storage minimums for
/// interactive login-class operations: 19 MiB memory, 2 iterations, 1
/// degree of parallelism. Embedding a format tag (rather than raw params)
/// means these can be tightened in a future format without breaking
/// decryption of blobs written today — a change would just get a new tag.
const ARGON2_MEMORY_KIB: u32 = 19 * 1024;
const ARGON2_TIME_COST: u32 = 2;
const ARGON2_PARALLELISM: u32 = 1;

/// Key length for XChaCha20-Poly1305 (256 bits = 32 bytes).
const KEY_LEN: usize = 32;

/// Nonce length for XChaCha20 (192 bits = 24 bytes).
const NONCE_LEN: usize = 24;

/// Tag length for Poly1305 (128 bits = 16 bytes).
const TAG_LEN: usize = 16;

/// Minimum encrypted data length (oldest, untagged format): nonce (24) + tag (16) = 40 bytes.
const MIN_DATA_LEN: usize = NONCE_LEN + TAG_LEN;

/// Derive a 32-byte key from a passphrase using Argon2id (memory-hard).
fn derive_key_argon2id(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], IronCoreError> {
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        Some(KEY_LEN),
    )
    .map_err(|_| IronCoreError::CryptoError)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|_| IronCoreError::CryptoError)?;
    Ok(key)
}

/// Derive a 32-byte key from a passphrase using PBKDF2-HMAC-SHA256. Used
/// only to decrypt backups created before the switch to Argon2id.
fn derive_key_pbkdf2(passphrase: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Try decrypting `ciphertext` with a derived `key`; `None` means the AEAD
/// tag didn't authenticate (wrong key/format/tampered data), not an error to
/// propagate — callers use this to probe candidate formats in order.
fn try_decrypt(key: &[u8; KEY_LEN], nonce_bytes: &[u8], ciphertext: &[u8]) -> Option<Vec<u8>> {
    let cipher = XChaCha20Poly1305::new_from_slice(key).ok()?;
    let nonce = XNonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).ok()
}

/// Encrypt a backup payload using XChaCha20-Poly1305 with an Argon2id-derived key.
/// Supports an optional custom 16-byte salt (e.g. from touch-screen entropy).
/// If `None`, a random 16-byte salt is generated internally.
///
/// # Arguments
/// * `payload` - The plaintext string to encrypt
/// * `passphrase` - The passphrase used to derive the encryption key
/// * `custom_salt` - An optional 16-byte salt
///
/// # Returns
/// Hex-encoded string containing the format tag, salt, nonce, and ciphertext (with tag).
///
/// # Errors
/// Returns `IronCoreError::CryptoError` if key derivation or encryption fails.
pub fn encrypt_backup(
    payload: &str,
    passphrase: &str,
    custom_salt: Option<&[u8; 16]>,
) -> Result<String, IronCoreError> {
    use rand::rngs::OsRng;
    use rand::RngCore;
    use zeroize::Zeroize;

    // Determine or generate the 16-byte salt
    let mut salt_bytes = [0u8; 16];
    if let Some(s) = custom_salt {
        salt_bytes.copy_from_slice(s);
    } else {
        OsRng.fill_bytes(&mut salt_bytes);
    }

    // Generate a cryptographically secure random 24-byte nonce
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Derive key using the salt
    let mut key = derive_key_argon2id(passphrase, &salt_bytes)?;

    // Initialize cipher and encrypt
    let cipher = XChaCha20Poly1305::new_from_slice(&key).map_err(|_| IronCoreError::CryptoError)?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, payload.as_bytes())
        .map_err(|_| IronCoreError::CryptoError)?;

    // Zeroize key material from stack
    key.zeroize();

    // Combine tag, salt, nonce, and ciphertext (with tag) into a single buffer, then hex-encode
    let mut combined = Vec::with_capacity(1 + 16 + NONCE_LEN + ciphertext.len());
    combined.push(FORMAT_TAG_ARGON2ID);
    combined.extend_from_slice(&salt_bytes);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(hex::encode(combined))
}

/// Decrypt a backup payload that was encrypted with `encrypt_backup` (supporting
/// the current Argon2id format and older PBKDF2-based formats for backward compat).
///
/// # Arguments
/// * `encrypted_hex` - Hex-encoded string containing the format tag, salt, nonce, and ciphertext
/// * `passphrase` - The passphrase used to derive the decryption key
///
/// # Returns
/// The decrypted plaintext string.
///
/// # Errors
/// Returns `IronCoreError::CorruptionDetected` if the data is tampered, the passphrase is
/// wrong, or the auth tag otherwise fails to verify against every known format.
/// Returns `IronCoreError::InvalidInput` if the hex string or data length is invalid.
pub fn decrypt_backup(encrypted_hex: &str, passphrase: &str) -> Result<String, IronCoreError> {
    use zeroize::Zeroize;

    let data = hex::decode(encrypted_hex).map_err(|_| IronCoreError::InvalidInput)?;

    if data.len() < MIN_DATA_LEN {
        return Err(IronCoreError::InvalidInput);
    }

    // Current format: tag(1) || salt(16) || nonce(24) || ciphertext+tag, Argon2id-derived key.
    if data.first() == Some(&FORMAT_TAG_ARGON2ID) && data.len() >= 1 + 16 + NONCE_LEN + TAG_LEN {
        let rest = &data[1..];
        let (salt_bytes, rest) = rest.split_at(16);
        let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

        if let Ok(mut key) = derive_key_argon2id(passphrase, salt_bytes) {
            let plaintext = try_decrypt(&key, nonce_bytes, ciphertext);
            key.zeroize();
            if let Some(plaintext) = plaintext {
                return String::from_utf8(plaintext).map_err(|_| IronCoreError::CorruptionDetected);
            }
        }
        // Falls through: a 1-in-256 chance an older-format blob's first byte
        // coincidentally matches the tag. The AEAD tag just won't authenticate
        // above, so trying the older formats below is still safe.
    }

    // Older format: salt(16) || nonce(24) || ciphertext+tag, PBKDF2-derived key.
    if data.len() >= 16 + NONCE_LEN + TAG_LEN {
        let (salt_bytes, rest) = data.split_at(16);
        let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

        let mut key = derive_key_pbkdf2(passphrase, salt_bytes);
        let plaintext = try_decrypt(&key, nonce_bytes, ciphertext);
        key.zeroize();
        if let Some(plaintext) = plaintext {
            return String::from_utf8(plaintext).map_err(|_| IronCoreError::CorruptionDetected);
        }
    }

    // Oldest format: nonce(24) || ciphertext+tag, PBKDF2 key from blake3(passphrase).
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let salt = blake3::hash(passphrase.as_bytes());
    let mut key = derive_key_pbkdf2(passphrase, salt.as_bytes());
    let plaintext = try_decrypt(&key, nonce_bytes, ciphertext);
    key.zeroize();
    if let Some(plaintext) = plaintext {
        return String::from_utf8(plaintext).map_err(|_| IronCoreError::CorruptionDetected);
    }

    Err(IronCoreError::CorruptionDetected)
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

    /// New backups must use the Argon2id format tag, not the legacy PBKDF2 shape.
    #[test]
    fn test_new_backups_use_argon2id_format_tag() {
        let encrypted = encrypt_backup("payload", "passphrase", None).unwrap();
        let data = hex::decode(&encrypted).unwrap();
        assert_eq!(data.first(), Some(&FORMAT_TAG_ARGON2ID));
    }

    /// T7: a wall-clock ">= 5ms" assertion is flaky by construction (will
    /// eventually miss on a fast/loaded machine) and can pass vacuously if
    /// the KDF ever silently regressed to something weak but still slow.
    /// Argon2id with fixed params is deterministic, so assert the exact
    /// derived key bytes instead - this both fails fast if the params or
    /// algorithm ever change unintentionally and is not timing-sensitive.
    /// Known-answer computed once with these exact inputs and params
    /// (Argon2id, 19 MiB, t=2, p=1) and re-verified by running this test
    /// twice in a row with identical results.
    #[test]
    fn test_kdf_is_memory_hard() {
        let key = derive_key_argon2id("some-passphrase", b"0123456789abcdef").unwrap();
        assert_eq!(
            hex::encode(key),
            "b15d39bb30bbb22dce599bce9286bbe137a89c28440f72b302b35fd791a8cce6",
            "Argon2id(19 MiB, t=2, p=1) derived key for these fixed inputs \
             changed - either the KDF params/algorithm regressed, or this \
             known-answer needs updating alongside an intentional change"
        );
    }

    /// A backup encrypted with the legacy PBKDF2-with-salt format (no format
    /// tag byte) must still decrypt correctly.
    #[test]
    fn test_legacy_pbkdf2_with_salt_format_still_decrypts() {
        use rand::rngs::OsRng;
        use rand::RngCore;

        let payload = "legacy payload";
        let passphrase = "legacy-passphrase";

        let mut salt_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut salt_bytes);
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);

        let key = derive_key_pbkdf2(passphrase, &salt_bytes);
        let cipher = XChaCha20Poly1305::new_from_slice(&key).unwrap();
        let nonce = XNonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, payload.as_bytes()).unwrap();

        let mut combined = Vec::new();
        combined.extend_from_slice(&salt_bytes);
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        let legacy_blob = hex::encode(combined);

        let decrypted = decrypt_backup(&legacy_blob, passphrase).unwrap();
        assert_eq!(decrypted, payload);
    }

    /// A backup encrypted with the oldest format (deterministic blake3(passphrase)
    /// salt, no stored salt at all) must still decrypt correctly.
    #[test]
    fn test_oldest_blake3_salt_format_still_decrypts() {
        use rand::rngs::OsRng;
        use rand::RngCore;

        let payload = "ancient payload";
        let passphrase = "ancient-passphrase";

        let salt = blake3::hash(passphrase.as_bytes());
        let key = derive_key_pbkdf2(passphrase, salt.as_bytes());
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let cipher = XChaCha20Poly1305::new_from_slice(&key).unwrap();
        let nonce = XNonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, payload.as_bytes()).unwrap();

        let mut combined = Vec::new();
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        let ancient_blob = hex::encode(combined);

        let decrypted = decrypt_backup(&ancient_blob, passphrase).unwrap();
        assert_eq!(decrypted, payload);
    }

    /// Tampering with any byte of a valid backup must fail closed with
    /// CorruptionDetected, not silently succeed or panic.
    #[test]
    fn test_tampered_blob_fails_with_corruption_detected() {
        let payload = "important data";
        let passphrase = "passphrase";

        let encrypted = encrypt_backup(payload, passphrase, None).unwrap();
        let mut data = hex::decode(&encrypted).unwrap();
        // Flip a bit well inside the ciphertext (past tag+salt+nonce).
        let last = data.len() - 1;
        data[last] ^= 0xFF;
        let tampered = hex::encode(data);

        let result = decrypt_backup(&tampered, passphrase);
        assert!(matches!(result, Err(IronCoreError::CorruptionDetected)));
    }
}
