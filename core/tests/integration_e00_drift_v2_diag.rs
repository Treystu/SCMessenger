//! Diagnostic test for E-00 ratchet round-trip failure through the Drift layer.
//!
//! This test replicates the exact encode/decode path that IronCore adds on top
//! of the raw ratchet crypto:
//!
//!   WireEnvelope::V2
//!     -> DriftEnvelope::from_v2_envelope
//!     -> DriftEnvelope::to_bytes
//!     -> DriftEnvelope::from_bytes
//!     -> DriftEnvelope::to_wire_envelope
//!     -> decrypt_with_ratchet_fallback
//!
//! It also decrypts the ORIGINAL WireEnvelope::V2 (no Drift round-trip) as a
//! sanity check.  All failures are reported with descriptive diagnostics so the
//! orchestrator can see exactly which field breaks or where decrypt fails.
//!
//! Run with:
//!   cargo test --test integration_e00_drift_v2_diag

use scmessenger_core::crypto::{
    decrypt_with_ratchet_fallback, encrypt_with_ratchet_fallback, RatchetSessionManager,
};
use scmessenger_core::drift::DriftEnvelope;
use scmessenger_core::identity::{sign_bundle, IdentityKeys};
use scmessenger_core::message::{EnvelopeV2, WireEnvelope};
use scmessenger_core::IronCore;

/// Stand up an initialised IronCore instance with a generated identity.
fn make_node() -> IronCore {
    let node = IronCore::new();
    node.grant_consent();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

/// Pull the IdentityKeys out of an initialised node.
fn node_keys(node: &IronCore) -> IdentityKeys {
    node.get_identity_keys()
        .expect("node must have identity keys")
}

/// Hex-encode the first byte of a slice, or "(empty)" / "(none)".
fn first_byte_hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        "(empty)".to_string()
    } else {
        format!("0x{:02x}", bytes[0])
    }
}

/// Pretty field description for an optional byte vector.
fn opt_field_info(name: &str, field: &Option<Vec<u8>>) -> String {
    match field {
        Some(v) => format!(
            "{}: Some(len={}, first={})",
            name,
            v.len(),
            first_byte_hex(v)
        ),
        None => format!("{}: None", name),
    }
}

/// Compare two EnvelopeV2 instances field-by-field and panic with diagnostics
/// on the first mismatch.
fn assert_v2_roundtrip_equal(original: &EnvelopeV2, restored: &EnvelopeV2) {
    assert_eq!(
        original.suite, restored.suite,
        "suite differs: original=0x{:02x}, restored=0x{:02x}",
        original.suite, restored.suite
    );

    assert_eq!(
        original.sender_public_key,
        restored.sender_public_key,
        "sender_public_key differs: original len={}, first={}; restored len={}, first={}",
        original.sender_public_key.len(),
        first_byte_hex(&original.sender_public_key),
        restored.sender_public_key.len(),
        first_byte_hex(&restored.sender_public_key)
    );

    assert_eq!(
        original.ephemeral_public_key,
        restored.ephemeral_public_key,
        "ephemeral_public_key differs: original len={}, first={}; restored len={}, first={}",
        original.ephemeral_public_key.len(),
        first_byte_hex(&original.ephemeral_public_key),
        restored.ephemeral_public_key.len(),
        first_byte_hex(&restored.ephemeral_public_key)
    );

    assert_eq!(
        original.nonce,
        restored.nonce,
        "nonce differs: original len={}, first={}; restored len={}, first={}",
        original.nonce.len(),
        first_byte_hex(&original.nonce),
        restored.nonce.len(),
        first_byte_hex(&restored.nonce)
    );

    assert_eq!(
        original.ciphertext,
        restored.ciphertext,
        "ciphertext differs: original len={}, first={}; restored len={}, first={}",
        original.ciphertext.len(),
        first_byte_hex(&original.ciphertext),
        restored.ciphertext.len(),
        first_byte_hex(&restored.ciphertext)
    );

    assert_eq!(
        original.ratchet_dh_public,
        restored.ratchet_dh_public,
        "ratchet_dh_public differs: original {}, restored {}",
        opt_field_info("ratchet_dh_public", &original.ratchet_dh_public),
        opt_field_info("ratchet_dh_public", &restored.ratchet_dh_public)
    );

    assert_eq!(
        original.ratchet_message_number, restored.ratchet_message_number,
        "ratchet_message_number differs: original={:?}, restored={:?}",
        original.ratchet_message_number, restored.ratchet_message_number
    );

    assert_eq!(
        original.pq_kem_ciphertext,
        restored.pq_kem_ciphertext,
        "pq_kem_ciphertext differs: original {}, restored {}",
        opt_field_info("pq_kem_ciphertext", &original.pq_kem_ciphertext),
        opt_field_info("pq_kem_ciphertext", &restored.pq_kem_ciphertext)
    );

    assert_eq!(
        original.pq_encaps_key,
        restored.pq_encaps_key,
        "pq_encaps_key differs: original {}, restored {}",
        opt_field_info("pq_encaps_key", &original.pq_encaps_key),
        opt_field_info("pq_encaps_key", &restored.pq_encaps_key)
    );

    assert_eq!(
        original.transcript_hash,
        restored.transcript_hash,
        "transcript_hash differs: original {}, restored {}",
        opt_field_info("transcript_hash", &original.transcript_hash),
        opt_field_info("transcript_hash", &restored.transcript_hash)
    );
}

#[test]
fn test_drift_v2_ratchet_roundtrip_diagnostic() {
    // -------------------------------------------------------------------------
    // 1. Create two identities using the public IronCore API (no test-utils).
    // -------------------------------------------------------------------------
    let alice_node = make_node();
    let bob_node = make_node();

    let alice = node_keys(&alice_node);
    let bob = node_keys(&bob_node);

    // -------------------------------------------------------------------------
    // 2. Sign and exchange bundles.
    // -------------------------------------------------------------------------
    let alice_bundle = sign_bundle(&alice).expect("alice bundle must sign");
    let bob_bundle = sign_bundle(&bob).expect("bob bundle must sign");

    // -------------------------------------------------------------------------
    // 3. Create ratchet session managers.
    // -------------------------------------------------------------------------
    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();

    // -------------------------------------------------------------------------
    // 4. Encrypt the first message directly through the ratchet fallback path.
    // -------------------------------------------------------------------------
    let plaintext = b"E-00 Drift V2 diagnostic plaintext";
    let bob_id = bob.identity_id();

    let wire1 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        plaintext,
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .expect("encrypt_with_ratchet_fallback must succeed");

    // The first message to a new peer must be V2 with PQ bootstrap fields.
    let v2_original = match &wire1 {
        WireEnvelope::V2(v2) => {
            eprintln!("[INFO] wire1 is WireEnvelope::V2");
            eprintln!("       suite=0x{:02x}", v2.suite);
            eprintln!(
                "       sender_public_key len={}",
                v2.sender_public_key.len()
            );
            eprintln!(
                "       ephemeral_public_key len={}",
                v2.ephemeral_public_key.len()
            );
            eprintln!("       nonce len={}", v2.nonce.len());
            eprintln!("       ciphertext len={}", v2.ciphertext.len());
            eprintln!(
                "       ratchet_dh_public={}",
                opt_field_info("ratchet_dh_public", &v2.ratchet_dh_public)
            );
            eprintln!(
                "       ratchet_message_number={:?}",
                v2.ratchet_message_number
            );
            eprintln!(
                "       pq_kem_ciphertext={}",
                opt_field_info("pq_kem_ciphertext", &v2.pq_kem_ciphertext)
            );
            eprintln!(
                "       pq_encaps_key={}",
                opt_field_info("pq_encaps_key", &v2.pq_encaps_key)
            );
            eprintln!(
                "       transcript_hash={}",
                opt_field_info("transcript_hash", &v2.transcript_hash)
            );
            v2.clone()
        }
        WireEnvelope::V1(_) => {
            panic!("Expected V2 envelope for a PQ-capable peer, got V1");
        }
    };

    assert_eq!(
        v2_original.suite, 0x02,
        "V2 envelope must advertise suite 0x02"
    );
    assert!(
        v2_original.pq_kem_ciphertext.is_some(),
        "first V2 message must carry pq_kem_ciphertext"
    );
    assert!(
        v2_original.transcript_hash.is_some(),
        "first V2 message must carry transcript_hash"
    );

    // -------------------------------------------------------------------------
    // 5. Run the Drift encode/decode layer: WireEnvelope::V2 -> Drift -> bytes -> Drift -> WireEnvelope::V2.
    // -------------------------------------------------------------------------
    let message_id = uuid::Uuid::new_v4().to_string();
    let recipient_pk = bob.signing_key.verifying_key().to_bytes();

    let drift_env = DriftEnvelope::from_v2_envelope(
        v2_original.clone(),
        message_id,
        recipient_pk,
        &alice.signing_key,
    )
    .expect("DriftEnvelope::from_v2_envelope must succeed");

    eprintln!(
        "[INFO] Drift envelope created: compressed={}, pq_flag present in DriftEnvelope if any PQ field set",
        drift_env.compressed
    );

    let drift_bytes = drift_env
        .to_bytes()
        .expect("DriftEnvelope::to_bytes must succeed");
    eprintln!("[INFO] Drift serialized bytes len={}", drift_bytes.len());

    // Byte-level offset inspection for ratchet/PQ extension layout.
    let ratchet_offset = 18 + 14 + 152 + 2 + drift_env.ciphertext.len();
    eprintln!(
        "[INFO] layout offsets: ratchet_offset={}, total_len={}",
        ratchet_offset,
        drift_bytes.len()
    );
    eprintln!(
        "[INFO] ratchet_flag at ratchet_offset: 0x{:02x}",
        drift_bytes[ratchet_offset]
    );
    let dh_start = ratchet_offset + 1;
    eprintln!(
        "[INFO] first 4 bytes of dh_public: {:?}",
        &drift_bytes[dh_start..dh_start + 4]
    );
    let pq_offset = ratchet_offset + 1 + 32 + 4;
    eprintln!(
        "[INFO] pq_offset bytes: pq_flag=0x{:02x}, suite=0x{:02x}",
        drift_bytes[pq_offset],
        drift_bytes[pq_offset + 1]
    );
    let hex_dump: String = drift_bytes[ratchet_offset..ratchet_offset + 10]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!(
        "[INFO] hex dump drift_bytes[ratchet_offset..ratchet_offset+10]: {}",
        hex_dump
    );
    eprintln!(
        "[INFO] DriftEnvelope BEFORE serialization: suite={:?}, pq_kem_ciphertext={}",
        drift_env.suite,
        opt_field_info("pq_kem_ciphertext", &drift_env.pq_kem_ciphertext)
    );

    let drift_restored =
        DriftEnvelope::from_bytes(&drift_bytes).expect("DriftEnvelope::from_bytes must succeed");

    eprintln!(
        "[INFO] DriftEnvelope AFTER from_bytes: suite={:?}, pq_kem_ciphertext={}",
        drift_restored.suite,
        opt_field_info("pq_kem_ciphertext", &drift_restored.pq_kem_ciphertext)
    );

    let wire2 = drift_restored.to_wire_envelope();

    // -------------------------------------------------------------------------
    // 6. Field-by-field comparison of the original V2 and the Drift-round-tripped V2.
    // -------------------------------------------------------------------------
    let v2_restored = match &wire2 {
        WireEnvelope::V2(v2) => {
            eprintln!("[INFO] wire2 (restored) is WireEnvelope::V2");
            v2.clone()
        }
        WireEnvelope::V1(_) => {
            panic!(
                "Drift round-trip changed V2 into V1; restored DriftEnvelope suite={:?}, pq_kem_ciphertext={}",
                drift_restored.suite,
                opt_field_info("pq_kem_ciphertext", &drift_restored.pq_kem_ciphertext)
            );
        }
    };

    assert_v2_roundtrip_equal(&v2_original, &v2_restored);
    eprintln!("[OK] V2 envelope fields are identical after Drift round-trip");

    // -------------------------------------------------------------------------
    // 7. Decrypt the Drift-round-tripped envelope.
    // -------------------------------------------------------------------------
    let decrypted2 = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &wire2,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    );

    match decrypted2 {
        Ok(ref pt) => {
            assert_eq!(
                pt, plaintext,
                "Drift-round-tripped plaintext must match the original"
            );
            eprintln!("[OK] Drift-round-tripped envelope decrypted successfully");
        }
        Err(ref e) => {
            panic!(
                "Decrypt of Drift-round-tripped envelope failed: {}\noriginal plaintext: {:?}",
                e,
                std::str::from_utf8(plaintext)
            );
        }
    }

    // -------------------------------------------------------------------------
    // 8. Sanity check: decrypt the ORIGINAL envelope with a fresh Bob manager.
    // -------------------------------------------------------------------------
    let mut fresh_bob_manager = RatchetSessionManager::new();
    let decrypted1 = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &wire1,
        Some(&mut fresh_bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    );

    match decrypted1 {
        Ok(ref pt) => {
            assert_eq!(
                pt, plaintext,
                "Original (non-Drift) plaintext must match the original"
            );
            eprintln!("[OK] Original envelope decrypted successfully with fresh manager");
        }
        Err(ref e) => {
            panic!(
                "Decrypt of original envelope (no Drift round-trip) failed: {}\nThis indicates the crypto path itself is broken, independent of Drift.",
                e
            );
        }
    }
}

#[test]
fn test_from_bytes_offset_trace() {
    // -------------------------------------------------------------------------
    // 1. Create two identities using the public IronCore API (no test-utils).
    // -------------------------------------------------------------------------
    let alice_node = make_node();
    let bob_node = make_node();

    let alice = node_keys(&alice_node);
    let bob = node_keys(&bob_node);

    // -------------------------------------------------------------------------
    // 2. Sign and exchange bundles.
    // -------------------------------------------------------------------------
    let alice_bundle = sign_bundle(&alice).expect("alice bundle must sign");
    let bob_bundle = sign_bundle(&bob).expect("bob bundle must sign");

    // -------------------------------------------------------------------------
    // 3. Create ratchet session managers.
    // -------------------------------------------------------------------------
    let mut alice_manager = RatchetSessionManager::new();

    // -------------------------------------------------------------------------
    // 4. Encrypt the first message directly through the ratchet fallback path.
    // -------------------------------------------------------------------------
    let plaintext = b"E-00 from_bytes offset trace plaintext";
    let bob_id = bob.identity_id();

    let wire1 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        plaintext,
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .expect("encrypt_with_ratchet_fallback must succeed");

    let v2_original = match &wire1 {
        WireEnvelope::V2(v2) => v2.clone(),
        WireEnvelope::V1(_) => {
            panic!("Expected V2 envelope for a PQ-capable peer, got V1");
        }
    };

    let message_id = uuid::Uuid::new_v4().to_string();
    let recipient_pk = bob.signing_key.verifying_key().to_bytes();

    let drift_env =
        DriftEnvelope::from_v2_envelope(v2_original, message_id, recipient_pk, &alice.signing_key)
            .expect("DriftEnvelope::from_v2_envelope must succeed");

    let drift_bytes = drift_env
        .to_bytes()
        .expect("DriftEnvelope::to_bytes must succeed");

    // Byte-level offset inspection for ratchet/PQ extension layout.
    let ratchet_offset = 18 + 14 + 152 + 2 + drift_env.ciphertext.len();
    let pq_offset = ratchet_offset + 1 + 32 + 4;
    eprintln!(
        "[INFO] from_bytes_offset_trace: drift_bytes.len()={}",
        drift_bytes.len()
    );
    eprintln!(
        "[INFO] from_bytes_offset_trace: ratchet_offset={}, ratchet_flag=0x{:02x}",
        ratchet_offset, drift_bytes[ratchet_offset]
    );
    eprintln!(
        "[INFO] from_bytes_offset_trace: pq_offset={}, pq_flag=0x{:02x}, suite=0x{:02x}",
        pq_offset,
        drift_bytes[pq_offset],
        drift_bytes[pq_offset + 1]
    );
    let hex_dump: String = drift_bytes[ratchet_offset..ratchet_offset + 10]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!(
        "[INFO] from_bytes_offset_trace: hex dump ratchet..+10: {}",
        hex_dump
    );
    eprintln!(
        "[INFO] from_bytes_offset_trace: DriftEnvelope BEFORE to_bytes: suite={:?}, pq_kem_ciphertext={}",
        drift_env.suite,
        opt_field_info("pq_kem_ciphertext", &drift_env.pq_kem_ciphertext)
    );

    // -------------------------------------------------------------------------
    // 5. Isolate from_bytes: report success and restored PQ fields.
    // -------------------------------------------------------------------------
    match DriftEnvelope::from_bytes(&drift_bytes) {
        Ok(drift_restored) => {
            eprintln!(
                "[INFO] from_bytes_offset_trace: from_bytes Ok; suite={:?}, pq_kem_ciphertext={}",
                drift_restored.suite,
                opt_field_info("pq_kem_ciphertext", &drift_restored.pq_kem_ciphertext)
            );
        }
        Err(e) => {
            eprintln!("[ERROR] from_bytes_offset_trace: from_bytes failed: {}", e);
        }
    }
}
