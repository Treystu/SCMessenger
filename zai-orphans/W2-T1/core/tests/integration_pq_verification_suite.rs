use scmessenger_core::crypto::{
    decrypt_with_ratchet_fallback, encrypt_with_ratchet_fallback, RatchetSessionManager,
};
use scmessenger_core::identity::{sign_bundle, verify_bundle, IdentityKeys};
use scmessenger_core::relay::invite::{
    InviteError, InviteToken, SeedLedgerEntry, INVITE_SIGNING_DOMAIN,
};

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

/// Build a dual-signed invite from a real identity: Ed25519 and ML-DSA-65 sign
/// the same domain-separated bytes, exactly as a platform client would.
fn dual_signed_invite(keys: &IdentityKeys, seed_ledger: Vec<SeedLedgerEntry>) -> InviteToken {
    let mut token = InviteToken::new(
        "alice".to_string(),
        keys.signing_key.verifying_key().to_bytes().to_vec(),
        "bob".to_string(),
    )
    .with_seed_ledger(seed_ledger);

    let pq = keys.mldsa_keypair.as_ref().expect("ML-DSA keypair");
    // The PQ public key is inside the signed bytes, so attach it first.
    token.pq_public_key = Some(pq.verifying_key().to_vec());

    let data = token.get_signable_data().expect("signable data");
    token.signature = keys.sign(&data).expect("ed25519 sign");
    token.pq_signature = Some(keys.sign_mldsa(&data).expect("mldsa sign"));
    token
}

#[test]
fn test_pqc_11_dual_signature_invites() {
    let alice = IdentityKeys::generate();
    let seed_ledger = vec![SeedLedgerEntry {
        multiaddr: "/ip4/198.51.100.7/tcp/9001".to_string(),
    }];

    let token = dual_signed_invite(&alice, seed_ledger.clone());

    // Both signatures verify over the same bytes.
    assert!(token.verify().is_ok(), "honest dual-signed invite");
    assert!(token.verify_with_policy(true).is_ok());
    assert!(token.is_valid(true));

    // Tampered ML-DSA signature.
    let mut tampered_pq = token.clone();
    tampered_pq.pq_signature.as_mut().expect("pq signature")[0] ^= 1;
    assert!(matches!(
        tampered_pq.verify(),
        Err(InviteError::PqVerificationFailed)
    ));
    assert!(!tampered_pq.is_valid(true));

    // Tampered Ed25519 signature.
    let mut tampered_ed = token.clone();
    tampered_ed.signature[0] ^= 1;
    assert!(matches!(
        tampered_ed.verify(),
        Err(InviteError::VerificationFailed)
    ));

    // The F1 forgery: junk in both signature fields.
    let mut forged = token.clone();
    forged.signature = vec![0x00];
    forged.pq_signature = Some(vec![0x01]);
    assert!(!forged.is_valid(true), "forged invite must not validate");
    assert!(!forged.is_valid(false), "forged invite must not validate");

    // Seed-ledger injection after signing.
    let mut injected = token.clone();
    injected.seed_ledger.push(SeedLedgerEntry {
        multiaddr: "/ip4/6.6.6.6/tcp/9001".to_string(),
    });
    assert!(
        injected.verify().is_err(),
        "injected seed entry must break both signatures"
    );

    // A different identity's key must not verify the token.
    let mallory = IdentityKeys::generate();
    let mut swapped = token.clone();
    swapped.inviter_public_key = mallory.signing_key.verifying_key().to_bytes().to_vec();
    assert!(swapped.verify().is_err());
}

#[test]
fn test_pqc_11b_invite_requires_pq_when_policy_demands_it() {
    // Ed25519-only invite: verifies, but not under a require_pq policy.
    let alice = IdentityKeys::generate();
    let mut token = InviteToken::new(
        "alice".to_string(),
        alice.signing_key.verifying_key().to_bytes().to_vec(),
        "bob".to_string(),
    );
    let data = token.get_signable_data().expect("signable data");
    token.signature = alice.sign(&data).expect("ed25519 sign");

    assert!(token.verify().is_ok());
    assert!(matches!(
        token.verify_with_policy(true),
        Err(InviteError::PqSignatureRequired)
    ));

    // Bolting a PQ signature onto a token that names no PQ key is rejected,
    // not silently skipped.
    let mut bolted = token.clone();
    bolted.pq_signature = Some(vec![0x01; 3309]);
    assert!(matches!(bolted.verify(), Err(InviteError::MalformedKey(_))));
    assert!(!bolted.is_valid(true));
}

#[test]
fn test_pqc_11c_invite_signature_is_domain_separated() {
    let alice = IdentityKeys::generate();
    let mut token = InviteToken::new(
        "alice".to_string(),
        alice.signing_key.verifying_key().to_bytes().to_vec(),
        "bob".to_string(),
    );

    let domained = token.get_signable_data().expect("signable data");
    assert!(domained.starts_with(INVITE_SIGNING_DOMAIN));

    // Sign the bincode body without the domain prefix.
    token.signature = alice
        .sign(&domained[INVITE_SIGNING_DOMAIN.len()..])
        .expect("ed25519 sign");
    assert!(matches!(
        token.verify(),
        Err(InviteError::VerificationFailed)
    ));
}
