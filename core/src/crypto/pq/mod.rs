use anyhow::{anyhow, Result};
use libcrux_ml_kem::{MlKemCiphertext, MlKemPrivateKey, MlKemPublicKey};
use libcrux_ml_kem::mlkem768;
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

/// Wrapper for the 2400-byte ML-KEM-768 private key that zeroizes on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MlKem768PrivateKey(pub [u8; 2400]);

/// Wrapper for ML-KEM-768 keypair.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct MlKem768KeyPair {
    pub_key: [u8; 1184],
    priv_key: MlKem768PrivateKey,
}

impl MlKem768KeyPair {
    /// Get the public key bytes reference.
    pub fn public_key(&self) -> &[u8; 1184] {
        &self.pub_key
    }

    /// Get the private key bytes reference.
    pub fn private_key(&self) -> &[u8; 2400] {
        &self.priv_key.0
    }
}

/// Generates a new ML-KEM-768 keypair using OS randomness.
pub fn generate() -> MlKem768KeyPair {
    let mut seed = [0u8; 64];
    OsRng.fill_bytes(&mut seed);
    let keypair = mlkem768::generate_key_pair(seed);
    
    let mut pub_key = [0u8; 1184];
    pub_key.copy_from_slice(keypair.public_key().as_ref());
    
    let mut priv_key_bytes = [0u8; 2400];
    priv_key_bytes.copy_from_slice(keypair.private_key().as_ref());
    
    MlKem768KeyPair {
        pub_key,
        priv_key: MlKem768PrivateKey(priv_key_bytes),
    }
}

/// Encapsulates a shared secret for the given public key.
pub fn encapsulate(encaps_key: &[u8]) -> Result<(Vec<u8> /*ct*/, [u8; 32] /*ss*/)> {
    if encaps_key.len() != 1184 {
        return Err(anyhow!(
            "Invalid ML-KEM-768 public key length: expected 1184, got {}",
            encaps_key.len()
        ));
    }
    
    let mut pub_key_bytes = [0u8; 1184];
    pub_key_bytes.copy_from_slice(encaps_key);
    let pub_key_obj = MlKemPublicKey::from(pub_key_bytes);
    
    let mut encap_rand = [0u8; 32];
    OsRng.fill_bytes(&mut encap_rand);
    
    let (ct, ss) = mlkem768::encapsulate(&pub_key_obj, encap_rand);
    Ok((ct.as_ref().to_vec(), ss))
}

/// Decapsulates the shared secret from the ciphertext using the keypair's private key.
pub fn decapsulate(keypair: &MlKem768KeyPair, ct: &[u8]) -> Result<[u8; 32]> {
    if ct.len() != 1088 {
        return Err(anyhow!(
            "Invalid ML-KEM-768 ciphertext length: expected 1088, got {}",
            ct.len()
        ));
    }
    
    let mut ct_bytes = [0u8; 1088];
    ct_bytes.copy_from_slice(ct);
    let ct_obj = MlKemCiphertext::from(ct_bytes);
    
    let priv_key_obj = MlKemPrivateKey::from(keypair.priv_key.0);
    let ss = mlkem768::decapsulate(&priv_key_obj, &ct_obj);
    
    Ok(ss)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let keypair = generate();
        assert_eq!(keypair.public_key().len(), 1184);
        assert_eq!(keypair.private_key().len(), 2400);

        let (ct, ss_enc) = encapsulate(keypair.public_key()).unwrap();
        assert_eq!(ct.len(), 1088);
        assert_eq!(ss_enc.len(), 32);

        let ss_dec = decapsulate(&keypair, &ct).unwrap();
        assert_eq!(ss_dec, ss_enc);
    }

    #[test]
    fn test_wrong_lengths() {
        let keypair = generate();

        // Encapsulate with invalid pubkey length
        let bad_pubkey = vec![0u8; 1183];
        assert!(encapsulate(&bad_pubkey).is_err());

        // Decapsulate with invalid ct length
        let bad_ct = vec![0u8; 1087];
        assert!(decapsulate(&keypair, &bad_ct).is_err());
    }

    #[test]
    fn test_tampered_ciphertext() {
        let keypair = generate();
        let (mut ct, ss_enc) = encapsulate(keypair.public_key()).unwrap();

        // Flip one byte of the ciphertext
        ct[0] ^= 1;

        let ss_dec = decapsulate(&keypair, &ct).unwrap();
        assert_ne!(ss_dec, ss_enc);
    }
}

