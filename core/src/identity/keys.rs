// Cryptographic key management

use anyhow::Result;
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use zeroize::Zeroize;

/// Key pair for signing and verification
#[derive(Clone)]
pub struct KeyPair {
    pub signing_key: SigningKey,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut secret_key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret_key_bytes);
        let signing_key = SigningKey::from_bytes(&secret_key_bytes);
        secret_key_bytes.zeroize();
        Self { signing_key }
    }

    /// Get verifying key
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

/// Identity keys (signing + optional encryption)
#[derive(Clone)]
pub struct IdentityKeys {
    pub signing_key: SigningKey,
}

impl IdentityKeys {
    /// Generate new identity keys
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut secret_key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret_key_bytes);
        let signing_key = SigningKey::from_bytes(&secret_key_bytes);
        secret_key_bytes.zeroize();
        Self { signing_key }
    }

    /// Get public key as hex
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }

    /// Get identity ID (Blake3 hash of public key)
    pub fn identity_id(&self) -> String {
        let public_key = self.signing_key.verifying_key().to_bytes();
        let hash = blake3::hash(&public_key);
        hex::encode(hash.as_bytes())
    }

    /// Sign data
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let signature = self.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verify signature
    pub fn verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> Result<bool> {
        let verifying_key = VerifyingKey::from_bytes(
            public_key
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid public key"))?,
        )?;

        let sig = Ed25519Signature::from_bytes(
            signature
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid signature"))?,
        );

        Ok(verifying_key.verify(data, &sig).is_ok())
    }

    /// Serialize keys to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }

    /// Deserialize keys from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid key bytes"))?,
        );
        Ok(Self { signing_key })
    }

    /// Convert to libp2p Keypair for network identity
    ///
    /// This allows using the same Ed25519 identity keys for both
    /// message encryption/signing AND libp2p network routing,
    /// eliminating the confusion of having two separate IDs.
    ///
    /// NOTE: ed25519-dalek's SigningKey::to_bytes() returns the 32-byte seed,
    /// but libp2p's SecretKey::try_from_bytes() expects the 64-byte expanded form
    /// (seed || public_key). We expand the key here to bridge the format gap.
    /// Derive a libp2p PeerId directly from the 32-byte Ed25519 public key.
    ///
    /// PeerId is a hash of the encoded public key — the secret key is never
    /// needed. This avoids the hacky 64-byte SecretKey expansion and works
    /// even when only the public half is available.
    pub fn to_libp2p_peer_id(&self) -> Result<String> {
        // Primary path: derive PeerId from raw public key bytes.
        // Falls back to full keypair conversion if the public-key-only path
        // fails (handles libp2p version API skew where try_from_bytes may move).
        let pub_bytes = self.signing_key.verifying_key().to_bytes();
        match libp2p::identity::ed25519::PublicKey::try_from_bytes(&pub_bytes) {
            Ok(libp2p_pub) => Ok(libp2p::identity::PublicKey::from(libp2p_pub)
                .to_peer_id()
                .to_string()),
            Err(_) => {
                // Fallback: derive via full keypair → public → PeerId.
                let kp = self.to_libp2p_keypair()?;
                Ok(kp.public().to_peer_id().to_string())
            }
        }
    }

    pub fn to_libp2p_keypair(&self) -> Result<libp2p::identity::Keypair> {
        // libp2p 0.53 SecretKey::try_from_bytes expects exactly 32 bytes (seed),
        // NOT the 64-byte expanded form. Pass the raw seed from ed25519-dalek.
        let mut seed = self.signing_key.to_bytes();
        let ed25519_secret = libp2p::identity::ed25519::SecretKey::try_from_bytes(&mut seed)
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to convert Ed25519 secret key to libp2p format: {}",
                    e
                )
            })?;
        let ed25519_keypair = libp2p::identity::ed25519::Keypair::from(ed25519_secret);
        Ok(libp2p::identity::Keypair::from(ed25519_keypair))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let keys = IdentityKeys::generate();
        let public_hex = keys.public_key_hex();
        let id = keys.identity_id();

        assert_eq!(public_hex.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(id.len(), 64); // Blake3 hash = 32 bytes = 64 hex chars
    }

    #[test]
    fn test_signing() {
        let keys = IdentityKeys::generate();
        let data = b"test message";

        let signature = keys.sign(data).unwrap();
        assert_eq!(signature.len(), 64); // Ed25519 signature = 64 bytes
    }

    #[test]
    fn test_verification() {
        let keys = IdentityKeys::generate();
        let data = b"test message";

        let signature = keys.sign(data).unwrap();
        let public_key = keys.signing_key.verifying_key().to_bytes();

        let valid = IdentityKeys::verify(data, &signature, &public_key).unwrap();
        assert!(valid);

        // Test with wrong data
        let invalid = IdentityKeys::verify(b"wrong data", &signature, &public_key).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_serialization() {
        let keys = IdentityKeys::generate();
        let bytes = keys.to_bytes();

        let restored = IdentityKeys::from_bytes(&bytes).unwrap();

        // Verify they produce the same public key
        assert_eq!(keys.public_key_hex(), restored.public_key_hex());
        assert_eq!(keys.identity_id(), restored.identity_id());
    }
}
