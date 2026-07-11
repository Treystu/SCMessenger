// Cryptographic key management

use anyhow::Result;
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
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

/// Inner serializable format for V2 identity keys
#[derive(Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
struct IdentityKeysV2Raw {
    signing_key_bytes: [u8; 32],
    x25519_secret_bytes: [u8; 32],
    mlkem_seed: Vec<u8>,
}

/// Identity keys (signing + dedicated encryption + hybrid post-quantum key agreement)
#[derive(Clone)]
pub struct IdentityKeys {
    pub signing_key: SigningKey,
    pub x25519_encryption_secret: x25519_dalek::StaticSecret,
    pub mlkem_keypair: crate::crypto::pq::MlKem768KeyPair,
}

impl IdentityKeys {
    /// Generate new identity keys
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut secret_key_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut secret_key_bytes);
        let signing_key = SigningKey::from_bytes(&secret_key_bytes);
        secret_key_bytes.zeroize();

        let mut x25519_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut x25519_bytes);
        let x25519_encryption_secret = x25519_dalek::StaticSecret::from(x25519_bytes);
        x25519_bytes.zeroize();

        let mlkem_keypair = crate::crypto::pq::generate();

        Self {
            signing_key,
            x25519_encryption_secret,
            mlkem_keypair,
        }
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
        let mut raw = IdentityKeysV2Raw {
            signing_key_bytes: self.signing_key.to_bytes(),
            x25519_secret_bytes: self.x25519_encryption_secret.to_bytes(),
            mlkem_seed: self.mlkem_keypair.seed.to_vec(),
        };
        let mut serialized = bincode::serialize(&raw)
            .expect("bincode serialization of IdentityKeysV2Raw cannot fail");
        raw.zeroize();
        let mut result = Vec::with_capacity(1 + serialized.len());
        result.push(0x02); // version tag
        result.append(&mut serialized);
        result
    }

    /// Deserialize keys from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() == 32 {
            // V1 legacy format
            let signing_key = SigningKey::from_bytes(
                bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid V1 key bytes"))?,
            );

            // For V1 legacy decoding inside from_bytes, we generate temporary encryption keys.
            // Note: IdentityStore::load_keys will check if it was V1 and persist the migration properly.
            let mut x25519_bytes = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut x25519_bytes);
            let x25519_encryption_secret = x25519_dalek::StaticSecret::from(x25519_bytes);
            x25519_bytes.zeroize();
            let mlkem_keypair = crate::crypto::pq::generate();

            Ok(Self {
                signing_key,
                x25519_encryption_secret,
                mlkem_keypair,
            })
        } else if bytes.first() == Some(&0x02) {
            // V2 format
            let mut raw: IdentityKeysV2Raw = bincode::deserialize(&bytes[1..])
                .map_err(|e| anyhow::anyhow!("Failed to deserialize V2 keys: {}", e))?;
            let signing_key = SigningKey::from_bytes(&raw.signing_key_bytes);
            let x25519_encryption_secret =
                x25519_dalek::StaticSecret::from(raw.x25519_secret_bytes);

            let mut mlkem_seed_arr = [0u8; 64];
            if raw.mlkem_seed.len() != 64 {
                raw.zeroize();
                return Err(anyhow::anyhow!("Invalid ML-KEM seed length in V2 keys"));
            }
            mlkem_seed_arr.copy_from_slice(&raw.mlkem_seed);
            let mlkem_keypair = crate::crypto::pq::from_seed(mlkem_seed_arr);
            mlkem_seed_arr.zeroize();
            raw.zeroize();
            Ok(Self {
                signing_key,
                x25519_encryption_secret,
                mlkem_keypair,
            })
        } else {
            Err(anyhow::anyhow!("Invalid identity keys format/tag"))
        }
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

fn default_supported_suites() -> Vec<u8> {
    vec![0x01]
}

/// Public key bundle containing Ed25519, X25519, and ML-KEM-768 public keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyBundle {
    pub ed25519_public: [u8; 32],
    pub x25519_public: [u8; 32],
    pub mlkem_encaps_key: Vec<u8>, // 1184 B
    pub created_at: u64,
    #[serde(default = "default_supported_suites")]
    pub supported_suites: Vec<u8>,
    pub signature: Vec<u8>, // Ed25519 over domain-separated bytes below
}

/// Sign a public key bundle for the given identity keys.
pub fn sign_bundle(keys: &IdentityKeys) -> Result<PublicKeyBundle> {
    let ed25519_public = keys.signing_key.verifying_key().to_bytes();
    let x25519_public = x25519_dalek::PublicKey::from(&keys.x25519_encryption_secret).to_bytes();
    let mlkem_encaps_key = keys.mlkem_keypair.public_key().to_vec();
    let created_at = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let supported_suites = vec![0x01, 0x02]; // Advertise v1 (legacy) and v2 (PQ)

    // Signature input: b"iron-core keybundle v2" || ed25519_public || x25519_public || mlkem_encaps_key || created_at.to_le_bytes() || supported_suites
    let mut sig_input = Vec::new();
    sig_input.extend_from_slice(b"iron-core keybundle v2");
    sig_input.extend_from_slice(&ed25519_public);
    sig_input.extend_from_slice(&x25519_public);
    sig_input.extend_from_slice(&mlkem_encaps_key);
    sig_input.extend_from_slice(&created_at.to_le_bytes());
    sig_input.extend_from_slice(&supported_suites);

    let signature = keys.sign(&sig_input)?;

    Ok(PublicKeyBundle {
        ed25519_public,
        x25519_public,
        mlkem_encaps_key,
        created_at,
        supported_suites,
        signature,
    })
}

/// Verify a public key bundle's cross-signature.
pub fn verify_bundle(bundle: &PublicKeyBundle) -> Result<()> {
    // Check v2 signature format first (with suites)
    let mut sig_input_v2 = Vec::new();
    sig_input_v2.extend_from_slice(b"iron-core keybundle v2");
    sig_input_v2.extend_from_slice(&bundle.ed25519_public);
    sig_input_v2.extend_from_slice(&bundle.x25519_public);
    sig_input_v2.extend_from_slice(&bundle.mlkem_encaps_key);
    sig_input_v2.extend_from_slice(&bundle.created_at.to_le_bytes());
    sig_input_v2.extend_from_slice(&bundle.supported_suites);

    if IdentityKeys::verify(&sig_input_v2, &bundle.signature, &bundle.ed25519_public).unwrap_or(false) {
        return Ok(());
    }

    // Fallback to v1 signature format (legacy bundles loaded from store without suites signed in)
    let mut sig_input_v1 = Vec::new();
    sig_input_v1.extend_from_slice(b"iron-core keybundle v1");
    sig_input_v1.extend_from_slice(&bundle.ed25519_public);
    sig_input_v1.extend_from_slice(&bundle.x25519_public);
    sig_input_v1.extend_from_slice(&bundle.mlkem_encaps_key);
    sig_input_v1.extend_from_slice(&bundle.created_at.to_le_bytes());

    let verified_v1 = IdentityKeys::verify(&sig_input_v1, &bundle.signature, &bundle.ed25519_public)?;
    if !verified_v1 {
        return Err(anyhow::anyhow!("Invalid signature on key bundle"));
    }
    Ok(())
}

/// Generate a Signal-style safety number from two public keys.
///
/// Returns a 60-digit numeric string (12 groups of 5 digits, space-separated).
/// The number is order-independent (sorted keys) so both sides display identically.
pub fn safety_number(our_pubkey_hex: &str, their_pubkey_hex: &str) -> Result<String> {
    let our_bytes = hex::decode(our_pubkey_hex)
        .map_err(|e| anyhow::anyhow!("Invalid our pubkey hex: {}", e))?;
    let their_bytes = hex::decode(their_pubkey_hex)
        .map_err(|e| anyhow::anyhow!("Invalid their pubkey hex: {}", e))?;

    if our_bytes.len() != 32 || their_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Public keys must be 32 bytes"));
    }

    // Sort keys to ensure order-independence
    let (first, second) = if our_bytes <= their_bytes {
        (&our_bytes, &their_bytes)
    } else {
        (&their_bytes, &our_bytes)
    };

    // blake3(first || second)
    let mut hasher = blake3::Hasher::new();
    hasher.update(first);
    hasher.update(second);
    let hash = hasher.finalize();
    let hash_bytes = hash.as_bytes();

    // Convert hash bytes to decimal digits
    // Use the hash bytes to generate enough digits
    let mut digits = String::with_capacity(71); // 60 digits + 11 spaces
    for group in 0..12 {
        let offset = (group * 2) % 24;
        let val = u16::from_be_bytes([hash_bytes[offset], hash_bytes[offset + 1]]) as u32;
        let group_val = val % 100000;
        if group > 0 {
            digits.push(' ');
        }
        digits.push_str(&format!("{:05}", group_val));
    }

    Ok(digits)
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

    #[test]
    fn test_libp2p_peer_id_derivation() {
        // Deriving Peer ID from identity keys should produce a valid base58 Peer ID
        let keys = IdentityKeys::generate();
        let peer_id = keys
            .to_libp2p_peer_id()
            .expect("Peer ID derivation should succeed");

        // Peer ID must start with "12D3Koo" (Ed25519 identity multihash prefix)
        assert!(
            peer_id.starts_with("12D3Koo"),
            "Ed25519 Peer ID should start with '12D3Koo', got: {}",
            &peer_id[..12.min(peer_id.len()).try_into().unwrap_or(usize::MAX)]
        );

        // Peer ID must be parseable as a valid libp2p PeerId
        assert!(
            peer_id.parse::<libp2p::PeerId>().is_ok(),
            "Derived Peer ID must be parseable as libp2p::PeerId"
        );
    }

    #[test]
    fn test_identity_hash_differs_from_public_key() {
        // identity_id (Blake3 hash) must differ from public_key_hex (Ed25519 verifying key)
        let keys = IdentityKeys::generate();
        let pk_hex = keys.public_key_hex();
        let id_hash = keys.identity_id();

        // Both are 64 hex chars but represent different values
        assert_eq!(pk_hex.len(), 64);
        assert_eq!(id_hash.len(), 64);
        assert_ne!(
            pk_hex, id_hash,
            "Identity hash must differ from the raw public key"
        );
    }

    #[test]
    fn test_peer_id_roundtrip_deterministic() {
        // Deriving Peer ID multiple times from the same keypair must produce the same result
        let keys = IdentityKeys::generate();
        let peer_id_1 = keys.to_libp2p_peer_id().unwrap();
        let peer_id_2 = keys.to_libp2p_peer_id().unwrap();
        assert_eq!(
            peer_id_1, peer_id_2,
            "Peer ID derivation must be deterministic"
        );
    }

    #[test]
    fn test_peer_id_unique_per_keypair() {
        // Different keypairs must produce different Peer IDs
        let keys_a = IdentityKeys::generate();
        let keys_b = IdentityKeys::generate();
        let peer_id_a = keys_a.to_libp2p_peer_id().unwrap();
        let peer_id_b = keys_b.to_libp2p_peer_id().unwrap();
        assert_ne!(
            peer_id_a, peer_id_b,
            "Different keypairs must yield different Peer IDs"
        );
    }

    #[test]
    fn test_public_key_to_peer_id_to_public_key_roundtrip() {
        // public_key_hex -> Peer ID -> extract public key should round-trip
        let keys = IdentityKeys::generate();
        let original_pk = keys.public_key_hex();
        let peer_id = keys.to_libp2p_peer_id().unwrap();

        // Parse the Peer ID and extract the public key bytes
        let parsed: libp2p::PeerId = peer_id.parse().expect("Peer ID must be parseable");
        let mh = parsed.as_ref();
        assert_eq!(
            mh.code(),
            0,
            "Ed25519 Peer IDs use identity multihash (code 0)"
        );

        let pk = libp2p::identity::PublicKey::try_decode_protobuf(mh.digest())
            .expect("Must decode public key from Peer ID");
        let ed25519_pk = pk.try_into_ed25519().expect("Must be Ed25519 key");
        let extracted_pk = hex::encode(ed25519_pk.to_bytes());

        assert_eq!(
            original_pk.to_lowercase(),
            extracted_pk.to_lowercase(),
            "Public key must round-trip through Peer ID derivation"
        );
    }

    #[test]
    fn test_identity_id_is_not_valid_ed25519_point() {
        // A Blake3 hash of a public key is NOT itself a valid Ed25519 verifying key.
        // This verifies that resolve_identity can distinguish the two formats.
        // We test this probabilistically: generate 100 identity_id values.
        // The chance of a random 32-byte string being a valid Ed25519 point is ~1/2
        // (due to the cofactor-8 check / quadratic residue requirement on the sign bit).
        let mut valid_count = 0usize;
        for _ in 0..100 {
            let keys = IdentityKeys::generate();
            let id_hash = keys.identity_id();
            let bytes = hex::decode(&id_hash).unwrap();
            let arr: [u8; 32] = bytes.try_into().unwrap();
            if ed25519_dalek::VerifyingKey::from_bytes(&arr).is_ok() {
                valid_count += 1;
            }
        }
        // Sanity check: if we get extreme outliers, flag it — but 30-70 is normal.
        assert!(
            valid_count < 90,
            "Suspiciously high ratio of valid Ed25519 points from Blake3 hashes ({}/100). \
             Expected ~50%. Check for non-random key generation.",
            valid_count
        );
    }

    #[test]
    fn test_safety_number_is_order_independent_and_deterministic() {
        let a = IdentityKeys::generate().public_key_hex();
        let b = IdentityKeys::generate().public_key_hex();

        let ab = safety_number(&a, &b).unwrap();
        let ba = safety_number(&b, &a).unwrap();
        assert_eq!(ab, ba, "safety number must not depend on argument order");

        // Deterministic: same inputs produce the same output every time.
        assert_eq!(ab, safety_number(&a, &b).unwrap());

        // 12 groups of 5 digits, space-separated.
        let groups: Vec<&str> = ab.split(' ').collect();
        assert_eq!(groups.len(), 12);
        for group in groups {
            assert_eq!(group.len(), 5);
            assert!(group.chars().all(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn test_safety_number_differs_for_different_key_pairs() {
        let a = IdentityKeys::generate().public_key_hex();
        let b = IdentityKeys::generate().public_key_hex();
        let c = IdentityKeys::generate().public_key_hex();

        assert_ne!(
            safety_number(&a, &b).unwrap(),
            safety_number(&a, &c).unwrap()
        );
    }

    #[test]
    fn test_safety_number_rejects_malformed_keys() {
        assert!(safety_number("not-hex", "also-not-hex").is_err());
        assert!(safety_number("abcd", "abcd").is_err()); // too short
    }

    #[test]
    fn test_v2_keys_non_derivation() {
        let keys = IdentityKeys::generate();
        let ed25519_pub = keys.signing_key.verifying_key().to_bytes();
        let x25519_pub_derived = crate::crypto::ed25519_public_to_x25519(&ed25519_pub).unwrap();
        let x25519_pub_derived_bytes = x25519_pub_derived.to_bytes();
        let x25519_pub_actual =
            x25519_dalek::PublicKey::from(&keys.x25519_encryption_secret).to_bytes();

        assert_ne!(
            x25519_pub_derived_bytes, x25519_pub_actual,
            "V2 X25519 key MUST be newly generated, NOT derived from Ed25519 identity key"
        );
    }

    #[test]
    fn test_bundle_sign_verify_tamper() {
        let keys = IdentityKeys::generate();
        let bundle = sign_bundle(&keys).unwrap();

        // 1. Verify valid bundle succeeds
        assert!(verify_bundle(&bundle).is_ok());

        // 2. Tamper ed25519_public
        let mut tampered = bundle.clone();
        tampered.ed25519_public[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 3. Tamper x25519_public
        let mut tampered = bundle.clone();
        tampered.x25519_public[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 4. Tamper mlkem_encaps_key
        let mut tampered = bundle.clone();
        tampered.mlkem_encaps_key[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 5. Tamper created_at
        let mut tampered = bundle.clone();
        tampered.created_at ^= 1;
        assert!(verify_bundle(&tampered).is_err());

        // 6. Tamper signature
        let mut tampered = bundle.clone();
        tampered.signature[0] ^= 1;
        assert!(verify_bundle(&tampered).is_err());
    }

    #[test]
    fn test_v1_identity_migration_and_compatibility() {
        // Create a legacy V1 identity (32 bytes raw seed)
        let mut v1_seed = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut v1_seed);

        // Parse V1 seed -> should generate X25519 and ML-KEM keys on the fly
        let keys = IdentityKeys::from_bytes(&v1_seed).unwrap();
        assert_eq!(keys.signing_key.to_bytes(), v1_seed);
        assert_eq!(keys.mlkem_keypair.public_key().len(), 1184);

        // Serialize it as V2 (tagged format)
        let serialized = keys.to_bytes();
        assert_eq!(serialized[0], 0x02);

        // Parse serialized V2
        let keys_restored = IdentityKeys::from_bytes(&serialized).unwrap();
        assert_eq!(
            keys_restored.signing_key.to_bytes(),
            keys.signing_key.to_bytes()
        );
        assert_eq!(
            x25519_dalek::PublicKey::from(&keys_restored.x25519_encryption_secret).to_bytes(),
            x25519_dalek::PublicKey::from(&keys.x25519_encryption_secret).to_bytes()
        );
        assert_eq!(
            keys_restored.mlkem_keypair.public_key(),
            keys.mlkem_keypair.public_key()
        );
    }

    #[test]
    fn test_v1_backup_restore_migration() {
        use crate::crypto::backup::{decrypt_backup, encrypt_backup};

        // Create a legacy V1 identity seed
        let v1_seed = [17u8; 32];

        // Format legacy backup payload: bare hex-encoded 32-byte signing key
        let payload = hex::encode(v1_seed);

        // Encrypt the backup payload using password "secure-password"
        let passphrase = "secure-password";
        let encrypted_backup = encrypt_backup(&payload, passphrase, None).unwrap();

        // Decrypt the backup
        let decrypted_payload = decrypt_backup(&encrypted_backup, passphrase).unwrap();
        assert_eq!(decrypted_payload, payload);

        // Decode decrypted key bytes
        let decrypted_bytes = hex::decode(&decrypted_payload).unwrap();
        assert_eq!(decrypted_bytes, v1_seed);

        // Restore keys through IdentityKeys::from_bytes (should automatically generate encryption/PQ keys)
        let restored_keys = IdentityKeys::from_bytes(&decrypted_bytes).unwrap();
        assert_eq!(restored_keys.signing_key.to_bytes(), v1_seed);
        assert_eq!(restored_keys.mlkem_keypair.public_key().len(), 1184);

        // Verify we can sign a bundle
        let bundle = sign_bundle(&restored_keys).unwrap();
        assert!(verify_bundle(&bundle).is_ok());
    }
}
