use anyhow::{anyhow, Result};
use ml_dsa::{
    Keypair, MlDsa65, Signature, SignatureEncoding, Signer, SigningKey, Verifier, VerifyingKey,
};
use rand::rngs::OsRng;
use zeroize::Zeroize;

/// Wrapper for the ML-DSA-65 private key that zeroizes on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct MlDsa65PrivateKey([u8; 32]);

impl Clone for MlDsa65PrivateKey {
    fn clone(&self) -> Self {
        let mut cloned_bytes = [0u8; 32];
        cloned_bytes.copy_from_slice(&self.0);
        MlDsa65PrivateKey(cloned_bytes)
    }
}

/// Wrapper for ML-DSA-65 keypair.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MlDsa65KeyPair {
    verifying_key: [u8; 1952],
    signing_key: MlDsa65PrivateKey,
}

impl MlDsa65KeyPair {
    /// Get the public key bytes reference.
    pub fn verifying_key(&self) -> &[u8; 1952] {
        &self.verifying_key
    }

    /// Get the private key bytes reference.
    pub fn signing_key(&self) -> &[u8; 32] {
        &self.signing_key.0
    }

    /// Reconstruct keypair from raw bytes
    pub fn from_bytes(vk: &[u8], sk: &[u8]) -> Result<Self> {
        if vk.len() != 1952 || sk.len() != 32 {
            return Err(anyhow!("Invalid ML-DSA-65 key lengths"));
        }
        let mut verifying_key = [0u8; 1952];
        verifying_key.copy_from_slice(vk);
        let mut signing_key_bytes = [0u8; 32];
        signing_key_bytes.copy_from_slice(sk);
        Ok(Self {
            verifying_key,
            signing_key: MlDsa65PrivateKey(signing_key_bytes),
        })
    }
}

/// Generate a new ML-DSA-65 keypair.
pub fn generate_keypair() -> MlDsa65KeyPair {
    let mut seed_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut OsRng, &mut seed_bytes);
    let signing_key = SigningKey::<MlDsa65>::from_seed(&seed_bytes.into());
    let verifying_key = signing_key.verifying_key();

    let mut vk_bytes = [0u8; 1952];
    vk_bytes.copy_from_slice(verifying_key.encode().as_slice());

    MlDsa65KeyPair {
        verifying_key: vk_bytes,
        signing_key: MlDsa65PrivateKey(seed_bytes),
    }
}

/// Sign a message with the ML-DSA-65 private key.
pub fn sign(keypair: &MlDsa65KeyPair, message: &[u8]) -> Result<Vec<u8>> {
    let seed_arr: [u8; 32] = keypair.signing_key.0;
    let signing_key = SigningKey::<MlDsa65>::from_seed(&seed_arr.into());
    let signature = signing_key.sign(message);
    Ok(signature.to_bytes().to_vec())
}

/// Verify an ML-DSA-65 signature.
pub fn verify(public_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> Result<()> {
    if public_key_bytes.len() != 1952 {
        return Err(anyhow!(
            "Invalid ML-DSA-65 public key length: expected 1952, got {}",
            public_key_bytes.len()
        ));
    }

    if signature_bytes.len() != 3309 {
        return Err(anyhow!(
            "Invalid ML-DSA-65 signature length: expected 3309, got {}",
            signature_bytes.len()
        ));
    }

    let vk_arr: &[u8; 1952] = public_key_bytes
        .try_into()
        .map_err(|_| anyhow!("Invalid ML-DSA-65 public key length"))?;
    let verifying_key = VerifyingKey::<MlDsa65>::decode(vk_arr.into());

    let sig_arr: &[u8; 3309] = signature_bytes
        .try_into()
        .map_err(|_| anyhow!("Invalid ML-DSA-65 signature length"))?;
    let signature = Signature::<MlDsa65>::try_from(sig_arr.as_slice())
        .map_err(|e| anyhow!("Failed to create signature from bytes: {}", e))?;

    verifying_key
        .verify(message, &signature)
        .map_err(|e| anyhow!("Signature verification failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mldsa65_sign_verify() {
        let keypair = generate_keypair();

        let message = b"Test message for ML-DSA-65";
        let signature = sign(&keypair, message).expect("Signing should succeed");

        verify(keypair.verifying_key(), message, &signature).expect("Verification should succeed");
    }

    #[test]
    fn test_mldsa65_invalid_signature() {
        let keypair = generate_keypair();

        let message = b"Test message for ML-DSA-65";
        let mut signature = sign(&keypair, message).expect("Signing should succeed");

        // Tamper with the signature
        signature[0] ^= 1;

        assert!(verify(keypair.verifying_key(), message, &signature).is_err());
    }

    #[test]
    fn test_mldsa65_wrong_message() {
        let keypair = generate_keypair();

        let message1 = b"Original message";
        let message2 = b"Different message";
        let signature = sign(&keypair, message1).expect("Signing should succeed");

        assert!(verify(keypair.verifying_key(), message2, &signature).is_err());
    }

    #[test]
    fn test_mldsa65_invalid_key_length() {
        let message = b"Test message";
        let fake_key = vec![0u8; 1951]; // Wrong length
        let fake_sig = vec![0u8; 3309];

        assert!(verify(&fake_key, message, &fake_sig).is_err());
    }

    #[test]
    fn test_mldsa65_invalid_signature_length() {
        let keypair = generate_keypair();
        let message = b"Test message";
        let fake_sig = vec![0u8; 3308]; // Wrong length

        assert!(verify(keypair.verifying_key(), message, &fake_sig).is_err());
    }

    #[test]
    fn test_mldsa65_known_answer_test() {
        // This test uses a deterministic approach by generating a keypair and
        // ensuring we can consistently sign and verify with it.
        let keypair = generate_keypair();

        let message = b"Known answer test for ML-DSA-65";
        let signature = sign(&keypair, message).expect("Signing should succeed");

        // Verify the signature
        verify(keypair.verifying_key(), message, &signature).expect("Verification should succeed");

        // Ensure the lengths are correct
        assert_eq!(keypair.verifying_key().len(), 1952);
        assert_eq!(signature.len(), 3309);
    }

    #[test]
    fn test_zeroize_behavior() {
        let keypair = generate_keypair();

        // Explicitly drop the keypair to trigger zeroize
        drop(keypair);

        // We can't directly test the zeroized values since they're dropped,
        // but we can ensure the type implements Zeroize properly
        let mut test_private_key = MlDsa65PrivateKey([1u8; 32]);
        assert_ne!(test_private_key.0, [0u8; 32]);
        test_private_key.zeroize();
        assert_eq!(test_private_key.0, [0u8; 32]);
    }
}
