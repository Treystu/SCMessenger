use scmessenger_core::crypto::{
    decrypt_with_ratchet_fallback, encrypt_with_ratchet_fallback, RatchetSessionManager,
};
use scmessenger_core::identity::{sign_bundle, verify_bundle, IdentityKeys};
use scmessenger_core::relay::invite::InviteToken;

#[test]
fn test_pqc_01_hybrid_handshake() {
    let alice = IdentityKeys::generate();
    let alice_bundle = sign_bundle(&alice).unwrap();
    let bob = IdentityKeys::generate();
    let bob_bundle = sign_bundle(&bob).unwrap();

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();

    let env1 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Hello Bob",
        Some(&mut alice_manager),
        &bob.identity_id(),
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    let dec1 = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &env1,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    )
    .unwrap();

    assert_eq!(dec1, b"Hello Bob");
}

#[test]
fn test_pqc_02_ratchet_cadence() {
    let alice = IdentityKeys::generate();
    let alice_bundle = sign_bundle(&alice).unwrap();
    let bob = IdentityKeys::generate();
    let bob_bundle = sign_bundle(&bob).unwrap();

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();

    let bob_id = bob.identity_id();
    let alice_id = alice.identity_id();

    // Step 1: Establish confirmed hybrid session (suite 0x02)
    let env1 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Initial message from Alice",
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &env1,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    )
    .unwrap();

    let env2 = encrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&alice_bundle),
        &alice_bundle.ed25519_public,
        b"Confirmation from Bob",
        Some(&mut bob_manager),
        &alice_id,
        Some(&bob_bundle),
        false,
        None,
    )
    .unwrap();

    decrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&alice.x25519_encryption_secret),
        &env2,
        Some(&mut alice_manager),
        Some(&alice.mlkem_keypair),
        Some(&alice_bundle),
        Some(&bob_bundle),
    )
    .unwrap();

    // Both sides confirmed. Next is testing PQ ratchet cadence.
    // Testing the same 105 message threshold here.
    let mut trigger_count = 0;

    for i in 1..=105 {
        let plaintext = format!("Message {}", i).into_bytes();
        let envelope = encrypt_with_ratchet_fallback(
            &alice.signing_key,
            Some(&bob_bundle),
            &bob_bundle.ed25519_public,
            &plaintext,
            Some(&mut alice_manager),
            &bob_id,
            Some(&alice_bundle),
            false,
            None,
        )
        .unwrap();

        let has_pq_fields = match &envelope {
            scmessenger_core::message::WireEnvelope::V2(v2) => {
                v2.pq_kem_ciphertext.is_some() && v2.pq_encaps_key.is_some()
            }
            _ => panic!("Expected V2 envelope for message #{}", i),
        };

        if has_pq_fields {
            trigger_count += 1;
        }

        let decrypted = decrypt_with_ratchet_fallback(
            &bob.signing_key,
            Some(&bob.x25519_encryption_secret),
            &envelope,
            Some(&mut bob_manager),
            Some(&bob.mlkem_keypair),
            Some(&bob_bundle),
            Some(&alice_bundle),
        )
        .unwrap();

        assert_eq!(decrypted, plaintext);
    }

    assert_eq!(
        trigger_count, 1,
        "Cadence trigger should fire exactly once across 105 messages"
    );
}

#[test]
fn test_pqc_10_mldsa_dual_signatures() {
    let keys = IdentityKeys::generate();
    let bundle = sign_bundle(&keys).unwrap();

    // Both signatures must be present
    assert!(bundle.mldsa_public.is_some());
    assert!(bundle.mldsa_signature.is_some());

    // Must successfully verify dual signature
    assert!(verify_bundle(&bundle).is_ok());

    // Tamper with ML-DSA signature
    let mut tampered = bundle.clone();
    tampered.mldsa_signature.as_mut().unwrap()[0] ^= 1;
    assert!(verify_bundle(&tampered).is_err());

    // Tamper with Ed25519 signature
    let mut tampered2 = bundle.clone();
    tampered2.signature[0] ^= 1;
    assert!(verify_bundle(&tampered2).is_err());
}

#[test]
fn test_pqc_11_dual_signature_invites() {
    let token = InviteToken::new("alice".to_string(), vec![1, 2, 3], "bob".to_string())
        .with_signature(vec![1, 2, 3])
        .with_pq_signature(vec![4, 5], vec![6, 7]);

    assert!(token.is_valid(true));

    // Invalid when tampered
    let tampered_token = InviteToken::new("alice".to_string(), vec![1, 2, 3], "bob".to_string())
        .with_signature(vec![1, 2, 3])
        .with_pq_signature(vec![4, 5], b"TAMPERED".to_vec());
    assert!(!tampered_token.is_valid(true));
}
