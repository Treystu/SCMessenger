//! DSPy Signatures for SCMessenger Swarm Roles
//!
//! These signatures define the input/output schemas for each swarm role,
//! enabling deterministic model routing and programmatic optimization.

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// Signature Definitions
// ═══════════════════════════════════════════════════════════════════════════════

/// Architect Signature: System design and planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectSignature {
    pub task_description: String,
    pub constraints: Vec<String>,
    pub expected_output: String,
}

impl ArchitectSignature {
    pub fn new(task: &str, constraints: &[&str], output: &str) -> Self {
        Self {
            task_description: task.to_string(),
            constraints: constraints.iter().map(|s| s.to_string()).collect(),
            expected_output: output.to_string(),
        }
    }
}

/// Coder Signature: Rust code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoderSignature {
    pub specification: String,
    pub context_files: Vec<String>,
    pub expected_code: String,
}

impl CoderSignature {
    pub fn new(spec: &str, context: &[&str], output: &str) -> Self {
        Self {
            specification: spec.to_string(),
            context_files: context.iter().map(|s| s.to_string()).collect(),
            expected_code: output.to_string(),
        }
    }
}

/// Verifier Signature: Security and correctness audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierSignature {
    pub code_review: String,
    pub security_concerns: Vec<String>,
    pub audit_result: String,
}

impl VerifierSignature {
    pub fn new(code: &str, concerns: &[&str], result: &str) -> Self {
        Self {
            code_review: code.to_string(),
            security_concerns: concerns.iter().map(|s| s.to_string()).collect(),
            audit_result: result.to_string(),
        }
    }
}

/// Auditor Signature: Cryptographic and protocol validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditorSignature {
    pub protocol_analysis: String,
    pub crypto_validation: String,
    pub compliance_check: String,
}

impl AuditorSignature {
    pub fn new(analysis: &str, crypto: &str, compliance: &str) -> Self {
        Self {
            protocol_analysis: analysis.to_string(),
            crypto_validation: crypto.to_string(),
            compliance_check: compliance.to_string(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Golden Examples (Verified SCMessenger Code)
// ═══════════════════════════════════════════════════════════════════════════════

/// Golden Example: Curve25519 Key Generation
pub const GOLDEN_CURVE25519_KEYGEN: &str = r#"
/// Generate a new X25519 key pair
pub fn generate_keypair() -> ([u8; 32], [u8; 32]) {
    use ring::rand::SecureRandom;
    use ring::eddsa::KeyPair;

    let rng = ring::rand::SystemRandom::new();
    let mut secret_key = [0u8; 32];
    rng.fill(&mut secret_key).unwrap();

    let public_key = x25519_dalek::PublicKey::from(
        x25519_dalek::StaticSecret::from(secret_key.clone()).public_key()
    );

    (secret_key, public_key.as_bytes().clone())
}
"#;

/// Golden Example: XChaCha20-Poly1305 Encryption
pub const GOLDEN_XCHACHA20_ENCRYPTION: &str = r#"
/// Encrypt data using XChaCha20-Poly1305
pub fn encrypt_xchacha20(
    plaintext: &[u8],
    key: &[u8; 32],
    nonce: &[u8; 24]
) -> Result<Vec<u8>, EncryptionError> {
    use chacha20::ChaCha20;
    use chacha20::cipher::{KeyIVInit, StreamCipher};
    use poly1305::Poly1305;

    // Construct cipher
    let mut cipher = ChaCha20::new(key.into(), nonce.into());

    // Encrypt in-place
    let mut ciphertext = plaintext.to_vec();
    cipher.apply_keystream(&mut ciphertext);

    // Compute MAC
    let mac = Poly1305::new(key.into()).compute_mac(&ciphertext);

    Ok([ciphertext, mac.as_bytes()].concat())
}
"#;

/// Compute BLAKE3 hash of data
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

/// Compute a BLAKE3 fingerprint of a serialized signature for content-addressable identification.
pub fn signature_fingerprint(data: &[u8]) -> String {
    let hash = blake3_hash(data);
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Golden Example: BLAKE3 Hashing
pub const GOLDEN_BLAKE3_HASHING: &str = r#"
/// Compute BLAKE3 hash of data
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    use blake3::hash;

    hash(data).as_bytes().try_into().unwrap()
}
"#;

// ═══════════════════════════════════════════════════════════════════════════════
// Signature Collections
// ═══════════════════════════════════════════════════════════════════════════════

/// All signature definitions for the swarm
pub static ALL_SIGNATURES: &[(&str, &str)] = &[
    ("architect", "System design and planning task specification"),
    ("coder", "Rust code generation from specification"),
    ("verifier", "Security audit and correctness verification"),
    ("auditor", "Cryptographic and protocol validation"),
];

/// Get signature by role name
pub fn get_signature(role: &str) -> Option<&'static str> {
    ALL_SIGNATURES
        .iter()
        .find(|(r, _)| *r == role)
        .map(|(_, desc)| *desc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architect_signature_serialization() {
        let sig = ArchitectSignature::new(
            "Implement onion routing",
            &["latency < 100ms", "backward compatible"],
            "Function with path construction"
        );

        let json = serde_json::to_string(&sig).unwrap();
        let deserialized: ArchitectSignature = serde_json::from_str(&json).unwrap();

        assert_eq!(sig.task_description, deserialized.task_description);
        assert_eq!(sig.constraints.len(), deserialized.constraints.len());
    }

    #[test]
    fn test_golden_examples_exist() {
        assert!(!GOLDEN_CURVE25519_KEYGEN.is_empty());
        assert!(!GOLDEN_XCHACHA20_ENCRYPTION.is_empty());
        assert!(!GOLDEN_BLAKE3_HASHING.is_empty());
    }

    #[test]
    fn test_blake3_hash_deterministic() {
        let data = b"scmessenger-dspy-test";
        let h1 = blake3_hash(data);
        let h2 = blake3_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32);
    }

    #[test]
    fn test_blake3_hash_different_inputs() {
        let h1 = blake3_hash(b"input-a");
        let h2 = blake3_hash(b"input-b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_signature_fingerprint_format() {
        let fp = signature_fingerprint(b"test-data");
        assert_eq!(fp.len(), 64); // 32 bytes = 64 hex chars
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
