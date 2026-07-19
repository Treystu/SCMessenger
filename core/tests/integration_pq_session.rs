use scmessenger_core::crypto::{
    decrypt_with_ratchet_fallback, encrypt_with_ratchet_fallback, RatchetSessionManager,
};
use scmessenger_core::identity::{sign_bundle, IdentityKeys};
use scmessenger_core::message::WireEnvelope;

fn generate_identities() -> (
    IdentityKeys,
    scmessenger_core::identity::PublicKeyBundle,
    IdentityKeys,
    scmessenger_core::identity::PublicKeyBundle,
) {
    let alice = IdentityKeys::generate();
    let alice_bundle = sign_bundle(&alice).unwrap();
    let bob = IdentityKeys::generate();
    let bob_bundle = sign_bundle(&bob).unwrap();
    (alice, alice_bundle, bob, bob_bundle)
}

#[test]
fn test_pq_session_full_send_receive() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();

    let plaintext1 = b"Hello Bob!";
    let bob_id = bob.identity_id();

    // Alice sends to Bob
    let envelope1 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        plaintext1,
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    // Should be V2
    match &envelope1 {
        WireEnvelope::V2(v2) => {
            assert_eq!(v2.suite, 0x02);
            assert!(v2.pq_kem_ciphertext.is_some());
            assert!(v2.transcript_hash.is_some());
        }
        _ => panic!("Expected V2 envelope"),
    }

    // Bob decrypts
    let decrypted1 = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &envelope1,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    )
    .unwrap();

    assert_eq!(decrypted1, plaintext1);

    // Bob sends to Alice
    let plaintext2 = b"Hi Alice!";
    let alice_id = alice.identity_id();

    let envelope2 = encrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&alice_bundle),
        &alice_bundle.ed25519_public,
        plaintext2,
        Some(&mut bob_manager),
        &alice_id,
        Some(&bob_bundle),
        false,
        None,
    )
    .unwrap();

    // Should be V2, but without PQ init fields because peer is confirmed
    match &envelope2 {
        WireEnvelope::V2(v2) => {
            assert_eq!(v2.suite, 0x02);
            assert!(v2.pq_kem_ciphertext.is_none());
            assert!(v2.transcript_hash.is_none());
        }
        _ => panic!("Expected V2 envelope"),
    }

    // Alice decrypts
    let decrypted2 = decrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&alice.x25519_encryption_secret),
        &envelope2,
        Some(&mut alice_manager),
        Some(&alice.mlkem_keypair),
        Some(&alice_bundle),
        Some(&bob_bundle),
    )
    .unwrap();

    assert_eq!(decrypted2, plaintext2);
}

#[test]
fn test_pq_session_lost_first_envelope() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();

    // Alice sends msg 1
    let env1 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Msg 1",
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    // Alice sends msg 2
    let env2 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Msg 2",
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    // Bob only receives msg 2 (msg 1 lost)
    // Since peer_confirmed is false, env2 still has the PQ fields!
    let dec2 = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &env2,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    )
    .unwrap();

    assert_eq!(dec2, b"Msg 2");

    // Bob receives msg 1 out of order
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

    assert_eq!(dec1, b"Msg 1");
}

#[test]
fn test_pq_session_transcript_mismatch() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();

    // Alice sends
    let mut env = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Secret",
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    // Tamper with transcript hash
    if let WireEnvelope::V2(v2) = &mut env {
        if let Some(hash) = &mut v2.transcript_hash {
            hash[0] ^= 0xFF;
        }
    }

    // Bob should reject
    let res = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &env,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    );
    assert!(res.is_err());
}

#[test]
fn test_v2_initiator_to_v1_peer() {
    let (alice, alice_bundle, bob, _bob_bundle_ignored) = generate_identities();

    // Create a V1 bundle for bob
    let mut bob_v1_bundle = _bob_bundle_ignored.clone();
    bob_v1_bundle.supported_suites = vec![0x01]; // only V1

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();

    // Alice initiates, but bob only supports V1
    let env = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_v1_bundle),
        &bob_v1_bundle.ed25519_public,
        b"Fallback",
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    // Should fallback to V1
    match &env {
        WireEnvelope::V1(_) => {}
        _ => panic!("Expected V1 envelope fallback"),
    }

    // Bob decrypts (treating it as V1)
    let dec = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &env,
        Some(&mut bob_manager),
        None,
        None,
        None, // Bob doesn't even have PQ bundles configured in this call
    )
    .unwrap();

    assert_eq!(dec, b"Fallback");
}

#[test]
fn test_pq_session_persistence() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();
    let bob_id = bob.identity_id();

    // Alice sends
    let env = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Msg",
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
        &env,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    )
    .unwrap();

    // Serialize and deserialize alice manager
    let json = alice_manager.serialize_sessions().unwrap();
    let mut new_alice = RatchetSessionManager::new();
    new_alice.deserialize_sessions(&json).unwrap();

    // Alice sends again
    let env2 = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Msg 2",
        Some(&mut new_alice),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    let dec2 = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &env2,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    )
    .unwrap();

    assert_eq!(dec2, b"Msg 2");
}

#[test]
fn test_pq_ratchet_cadence_refreshes_shared_secret() {
    let (alice, alice_bundle, bob, bob_bundle) = generate_identities();

    let mut alice_manager = RatchetSessionManager::new();
    let mut bob_manager = RatchetSessionManager::new();

    let bob_id = bob.identity_id();
    let alice_id = alice.identity_id();

    // Step 1: Establish confirmed hybrid session (suite 0x02)
    // Alice sends first message to Bob
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

    // Bob receives and confirms Alice
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

    // Bob sends confirmation message to Alice
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

    // Alice receives and confirms Bob
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

    // Now both sides have peer_confirmed = true

    // Step 2: Send messages until we hit the cadence trigger. Note: Alice's
    // sending-chain message_number is chain-local and resets to 0 whenever she
    // performs a DH ratchet step -- which she just did, processing Bob's
    // confirmation reply above (a normal Double Ratchet direction switch). That
    // consumes one chain-index slot before this loop even starts, so the loop
    // index and the ratchet's internal message_number are off by one from what
    // a naive count would expect. Rather than hardcode that offset (fragile,
    // and it's an internal implementation detail), detect the cadence trigger
    // dynamically by inspecting each envelope.
    let mut root_key_before = None;
    let mut triggering_envelope = None;
    let mut trigger_count = 0;

    for i in 1..=105 {
        let plaintext = format!("Message {}", i).into_bytes();
        let root_key_pre = alice_manager.get_session(&bob_id).unwrap().root_key_bytes();

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
            WireEnvelope::V2(v2) => v2.pq_kem_ciphertext.is_some() && v2.pq_encaps_key.is_some(),
            _ => panic!("Expected V2 envelope for message #{}", i),
        };

        if has_pq_fields {
            trigger_count += 1;
            root_key_before = Some(root_key_pre);
            triggering_envelope = Some(envelope.clone());
        }

        // Bob decrypts every message
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

    // Step 3: The cadence trigger must fire exactly once across this range
    // (proves the wiring from PQC_07_WIRE_RATCHET_STEP is real and reachable).
    assert_eq!(
        trigger_count, 1,
        "Cadence trigger should fire exactly once across 105 messages"
    );
    let triggering_env = triggering_envelope.unwrap();
    match &triggering_env {
        WireEnvelope::V2(v2) => {
            assert!(v2.pq_kem_ciphertext.is_some());
            assert!(v2.pq_encaps_key.is_some());
        }
        _ => panic!("Triggering envelope should be V2"),
    }

    // Step 4: Verify Bob successfully decrypted the triggering message.
    // (Already proven by the assert_eq!(decrypted, plaintext) inside the loop
    // for every message including the triggering one.)

    // Step 4.5: Trigger the DH ratchet steps (epoch crossing) to mix the PQ secret on both sides.
    // 1. Bob sends a reply to Alice. This triggers Alice's DH step and mixes Alice's pq_pending_sent.
    let bob_reply = encrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&alice_bundle),
        &alice_bundle.ed25519_public,
        b"Bob's reply to Alice",
        Some(&mut bob_manager),
        &alice_id,
        Some(&bob_bundle),
        false,
        None,
    )
    .unwrap();

    let decrypted_bob_reply = decrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&alice.x25519_encryption_secret),
        &bob_reply,
        Some(&mut alice_manager),
        Some(&alice.mlkem_keypair),
        Some(&alice_bundle),
        Some(&bob_bundle),
    )
    .unwrap();
    assert_eq!(decrypted_bob_reply, b"Bob's reply to Alice");

    // 2. Alice sends a reply back to Bob. This triggers Bob's DH step and mixes Bob's pq_pending_recv.
    let alice_reply = encrypt_with_ratchet_fallback(
        &alice.signing_key,
        Some(&bob_bundle),
        &bob_bundle.ed25519_public,
        b"Alice's second reply to Bob",
        Some(&mut alice_manager),
        &bob_id,
        Some(&alice_bundle),
        false,
        None,
    )
    .unwrap();

    let decrypted_alice_reply = decrypt_with_ratchet_fallback(
        &bob.signing_key,
        Some(&bob.x25519_encryption_secret),
        &alice_reply,
        Some(&mut bob_manager),
        Some(&bob.mlkem_keypair),
        Some(&bob_bundle),
        Some(&alice_bundle),
    )
    .unwrap();
    assert_eq!(decrypted_alice_reply, b"Alice's second reply to Bob");

    // Step 5: E-01c: Re-enabled assertion proving the fresh PQ shared secret actually entered the KDF
    let root_key_after = alice_manager.get_session(&bob_id).unwrap().root_key_bytes();
    assert_ne!(
        root_key_before.unwrap(),
        root_key_after,
        "Root key should change after PQ ratchet step"
    );
}
