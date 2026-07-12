//! ML-DSA-65 post-quantum signature implementation (FIPS 204 final, RustCrypto `ml-dsa` 0.1.1).
//!
//! Verified against the real crate API (docs.rs summary + local registry source,
//! not guessed): key generation/signing/verification go through the standard
//! RustCrypto `crypto-common`/`signature` traits (`Generate`, `KeyInit`,
//! `KeyExport`, `Signer`, `Verifier`, `SignatureEncoding`), all re-exported at
//! `ml_dsa::` crate root. `SigningKey<MlDsa65>` is seed-derived (32-byte
//! `Seed`), so we persist the seed rather than attempting to serialize the
//! expanded key directly (the crate does not implement `serde::Serialize`).

use anyhow::{anyhow, Result};
use ml_dsa::{Generate, KeyExport, KeyInit, Keypair, MlDsa65, Signature, SignatureEncoding, Signer,
             SigningKey, VerifyingKey};

/// Wrapper for an ML-DSA-65 keypair.
#[derive(Clone)]
pub struct MlDsa65KeyPair {
    inner: SigningKey<MlDsa65>,
}

impl MlDsa65KeyPair {
    /// Generate a new random ML-DSA-65 keypair.
    pub fn generate() -> Self {
        // Uses the crate's own ambient-RNG convenience method (`getrandom`
        // feature), which internally wires up a rand_core-0.10-compatible
        // RNG -- the workspace's `rand` crate's OsRng implements the older
        // rand_core 0.6 and is not directly usable here.
        let inner = SigningKey::<MlDsa65>::generate();
        Self { inner }
    }

    /// Reconstruct a keypair deterministically from its 32-byte seed
    /// (this is what `seed_bytes()` returns and what gets persisted).
    pub fn from_seed(seed_bytes: &[u8]) -> Result<Self> {
        if seed_bytes.len() != 32 {
            return Err(anyhow!(
                "Invalid ML-DSA-65 seed length: expected 32, got {}",
                seed_bytes.len()
            ));
        }
        let seed = ml_dsa::Seed::try_from(seed_bytes)
            .map_err(|_| anyhow!("Invalid ML-DSA-65 seed"))?;
        Ok(Self {
            inner: SigningKey::<MlDsa65>::new(&seed),
        })
    }

    /// The 32-byte seed this keypair was derived from -- persist this, not
    /// the expanded signing key, for storage/backup.
    pub fn seed_bytes(&self) -> Vec<u8> {
        let seed: ml_dsa::Seed = self.inner.to_bytes();
        seed.as_slice().to_vec()
    }

    /// Sign data with ML-DSA-65. Returns the 3309-byte encoded signature.
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let sig: Signature<MlDsa65> = self
            .inner
            .try_sign(data)
            .map_err(|e| anyhow!("ML-DSA-65 signing failed: {}", e))?;
        Ok(sig.to_vec())
    }

    /// Get the 1952-byte encoded public key.
    pub fn public_key(&self) -> Vec<u8> {
        let key: ml_dsa::common::Key<VerifyingKey<MlDsa65>> = self.inner.verifying_key().to_bytes();
        key.as_slice().to_vec()
    }
}

/// Verify an ML-DSA-65 signature. `Ok(false)` means the signature is
/// cryptographically invalid (rejected, not an error); `Err` means the
/// input bytes themselves are malformed (wrong length / undecodable).
pub fn verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
    let verifying_key = VerifyingKey::<MlDsa65>::new_from_slice(public_key)
        .map_err(|_| anyhow!("Invalid ML-DSA-65 public key length"))?;

    let sig = match Signature::<MlDsa65>::try_from(signature) {
        Ok(s) => s,
        Err(_) => return Err(anyhow!("Invalid ML-DSA-65 signature encoding")),
    };

    Ok(ml_dsa::Verifier::verify(&verifying_key, data, &sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mldsa_sign_verify() {
        let keypair = MlDsa65KeyPair::generate();
        let data = b"test message";
        
        let signature = keypair.sign(data).unwrap();
        let public_key = keypair.public_key();
        
        let valid = verify(data, &signature, &public_key).unwrap();
        assert!(valid);
        
        // Test with wrong data
        let invalid = verify(b"wrong data", &signature, &public_key).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_signature_size() {
        let keypair = MlDsa65KeyPair::generate();
        let data = b"test";
        let signature = keypair.sign(data).unwrap();
        // ML-DSA-65 signatures are 3309 bytes
        assert_eq!(signature.len(), 3309);
    }

    #[test]
    fn test_public_key_size() {
        let keypair = MlDsa65KeyPair::generate();
        let public_key = keypair.public_key();
        // ML-DSA-65 public keys are 1952 bytes
        assert_eq!(public_key.len(), 1952);
    }

    #[test]
    fn test_seed_roundtrip_reconstructs_same_key() {
        let keypair = MlDsa65KeyPair::generate();
        let seed = keypair.seed_bytes();
        assert_eq!(seed.len(), 32);

        let restored = MlDsa65KeyPair::from_seed(&seed).unwrap();
        assert_eq!(keypair.public_key(), restored.public_key());

        // A signature made by the restored key must verify against the
        // original key's public key (same key material).
        let data = b"seed roundtrip";
        let sig = restored.sign(data).unwrap();
        assert!(verify(data, &sig, &keypair.public_key()).unwrap());
    }

    #[test]
    fn test_invalid_seed_length_rejected() {
        assert!(MlDsa65KeyPair::from_seed(&[0u8; 16]).is_err());
    }

    #[test]
    fn test_tampered_signature_rejected() {
        let keypair = MlDsa65KeyPair::generate();
        let data = b"test message";
        let mut signature = keypair.sign(data).unwrap();
        signature[0] ^= 1;
        let valid = verify(data, &signature, &keypair.public_key()).unwrap();
        assert!(!valid);
    }
}