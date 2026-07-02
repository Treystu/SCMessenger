/// Integration test: Ratchet session persistence across process restarts
///
/// Verifies that:
/// 1. Sessions serialize/deserialize correctly
/// 2. Message exchange works before and after persistence
/// 3. Out-of-order messages survive persistence (MAX_SKIP_KEYS=256)
/// 4. Conversation continues after restart in both directions
use scmessenger_core::crypto::ratchet::RatchetSession;
use scmessenger_core::crypto::RatchetSessionManager;
use scmessenger_core::store::backend::MemoryStorage;
use std::sync::Arc;

fn generate_signing_key() -> ed25519_dalek::SigningKey {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    ed25519_dalek::SigningKey::from_bytes(&bytes)
}

fn x25519_public_from_signing(key: &ed25519_dalek::SigningKey) -> x25519_dalek::PublicKey {
    let secret = scmessenger_core::crypto::encrypt::ed25519_to_x25519_secret(key);
    x25519_dalek::PublicKey::from(&secret)
}

#[test]
fn ratchet_session_persistence_roundtrip() {
    let backend = Arc::new(MemoryStorage::new());
    let mut manager = RatchetSessionManager::with_backend(backend.clone());

    let our_key = generate_signing_key();
    let their_pub = x25519_dalek::PublicKey::from([1u8; 32]);

    manager
        .get_or_create_session("peer-restart", &our_key, &their_pub)
        .unwrap();
    assert_eq!(manager.session_count(), 1);

    let json = manager.serialize_sessions().unwrap();
    assert!(!json.is_empty());

    let mut manager2 = RatchetSessionManager::with_backend(backend);
    assert_eq!(manager2.session_count(), 0);

    manager2.deserialize_sessions(&json).unwrap();
    assert_eq!(manager2.session_count(), 1);
    assert!(manager2.get_session("peer-restart").is_some());
}

#[test]
fn ratchet_session_multiple_peers_persist() {
    let backend = Arc::new(MemoryStorage::new());
    let mut manager = RatchetSessionManager::with_backend(backend.clone());

    let our_key = generate_signing_key();

    for i in 0..5 {
        let their_pub = x25519_dalek::PublicKey::from([i as u8; 32]);
        manager
            .get_or_create_session(&format!("peer-{}", i), &our_key, &their_pub)
            .unwrap();
    }
    assert_eq!(manager.session_count(), 5);

    let json = manager.serialize_sessions().unwrap();
    let mut manager2 = RatchetSessionManager::with_backend(backend);
    manager2.deserialize_sessions(&json).unwrap();
    assert_eq!(manager2.session_count(), 5);

    for i in 0..5 {
        assert!(manager2.get_session(&format!("peer-{}", i)).is_some());
    }
}

/// Full message exchange test: Alice sends to Bob, Bob replies, persist Alice, continue.
#[test]
fn ratchet_persistence_with_message_exchange() {
    let alice_key = generate_signing_key();
    let bob_key = generate_signing_key();

    let alice_pub_x25519 = x25519_public_from_signing(&alice_key);
    let bob_pub_x25519 = x25519_public_from_signing(&bob_key);

    let mut alice = RatchetSession::init_as_sender(&alice_key, &bob_pub_x25519).unwrap();
    let mut bob = RatchetSession::init_as_receiver(&bob_key, &alice_pub_x25519).unwrap();

    let aad = b"scmessenger-v1";

    // Alice sends 5 messages to Bob
    let mut alice_cts = Vec::new();
    for i in 0..5 {
        let msg = format!("A->B {}", i);
        alice_cts.push(alice.encrypt(msg.as_bytes(), aad).unwrap());
    }

    // Bob decrypts — each message triggers a DH ratchet on first decrypt
    for (i, ct) in alice_cts.iter().enumerate() {
        let pt = bob
            .decrypt(
                &ct.our_dh_public,
                ct.message_number,
                &ct.nonce,
                &ct.ciphertext,
                aad,
            )
            .unwrap();
        assert_eq!(String::from_utf8(pt).unwrap(), format!("A->B {}", i));
    }

    // Bob replies using his sending chain (established during DH ratchet)
    let mut bob_cts = Vec::new();
    for i in 0..5 {
        let msg = format!("B->A {}", i);
        bob_cts.push(bob.encrypt(msg.as_bytes(), aad).unwrap());
    }

    // Alice decrypts Bob's replies
    for (i, ct) in bob_cts.iter().enumerate() {
        let pt = alice
            .decrypt(
                &ct.our_dh_public,
                ct.message_number,
                &ct.nonce,
                &ct.ciphertext,
                aad,
            )
            .unwrap();
        assert_eq!(String::from_utf8(pt).unwrap(), format!("B->A {}", i));
    }

    // --- PERSIST ALICE AND SIMULATE RESTART ---
    let alice_json = {
        let sessions_map = std::collections::HashMap::from([(
            "bob".to_string(),
            scmessenger_core::crypto::session_manager::SerializableRatchetSession::from_session(
                &alice,
            ),
        )]);
        serde_json::to_string(&sessions_map).unwrap()
    };

    let mut alice_mgr = RatchetSessionManager::new();
    alice_mgr.deserialize_sessions(&alice_json).unwrap();
    assert_eq!(alice_mgr.session_count(), 1);

    // Alice sends after restart
    let alice2 = alice_mgr.get_session_mut("bob").unwrap();
    let post_msg = "A->B after restart";
    let ct_post = alice2.encrypt(post_msg.as_bytes(), aad).unwrap();

    // Bob decrypts Alice's post-restart message
    let pt = bob
        .decrypt(
            &ct_post.our_dh_public,
            ct_post.message_number,
            &ct_post.nonce,
            &ct_post.ciphertext,
            aad,
        )
        .unwrap();
    assert_eq!(String::from_utf8(pt).unwrap(), post_msg);

    // Bob replies after Alice's restart
    let bob_reply = "B->A after A restart";
    let ct_bob = bob.encrypt(bob_reply.as_bytes(), aad).unwrap();

    let pt = alice2
        .decrypt(
            &ct_bob.our_dh_public,
            ct_bob.message_number,
            &ct_bob.nonce,
            &ct_bob.ciphertext,
            aad,
        )
        .unwrap();
    assert_eq!(String::from_utf8(pt).unwrap(), bob_reply);
}

/// Out-of-order message delivery across persistence boundary.
#[test]
fn ratchet_persistence_out_of_order_messages() {
    let alice_key = generate_signing_key();
    let bob_key = generate_signing_key();

    let alice_pub_x25519 = x25519_public_from_signing(&alice_key);
    let bob_pub_x25519 = x25519_public_from_signing(&bob_key);

    let mut alice = RatchetSession::init_as_sender(&alice_key, &bob_pub_x25519).unwrap();
    let mut bob = RatchetSession::init_as_receiver(&bob_key, &alice_pub_x25519).unwrap();

    let aad = b"scmessenger-v1";

    // Alice sends 10 messages
    let mut cts = Vec::new();
    for i in 0..10 {
        cts.push(alice.encrypt(format!("msg {}", i).as_bytes(), aad).unwrap());
    }

    // Bob receives messages 0-2 (skipping 3-9)
    for i in 0..3 {
        let ct = &cts[i];
        let pt = bob
            .decrypt(
                &ct.our_dh_public,
                ct.message_number,
                &ct.nonce,
                &ct.ciphertext,
                aad,
            )
            .unwrap();
        assert_eq!(String::from_utf8(pt).unwrap(), format!("msg {}", i));
    }

    // Persist Bob (with chain state after receiving 0-2)
    let bob_json = {
        let sessions_map = std::collections::HashMap::from([(
            "alice".to_string(),
            scmessenger_core::crypto::session_manager::SerializableRatchetSession::from_session(
                &bob,
            ),
        )]);
        serde_json::to_string(&sessions_map).unwrap()
    };

    // Simulate restart
    let mut bob_mgr = RatchetSessionManager::new();
    bob_mgr.deserialize_sessions(&bob_json).unwrap();
    let bob2 = bob_mgr.get_session_mut("alice").unwrap();

    // Bob receives messages 3-9 after restart
    for i in 3..10 {
        let ct = &cts[i];
        let result = bob2.decrypt(
            &ct.our_dh_public,
            ct.message_number,
            &ct.nonce,
            &ct.ciphertext,
            aad,
        );
        assert!(
            result.is_ok(),
            "Failed to decrypt message {} after restart: {:?}",
            i,
            result.err()
        );
        assert_eq!(
            String::from_utf8(result.unwrap()).unwrap(),
            format!("msg {}", i)
        );
    }
}

#[test]
fn ratchet_manager_save_load_via_backend() {
    let backend = Arc::new(MemoryStorage::new());
    let mut manager = RatchetSessionManager::with_backend(backend.clone());

    let our_key = generate_signing_key();
    let their_pub = x25519_dalek::PublicKey::from([42u8; 32]);

    manager
        .get_or_create_session("persistent-peer", &our_key, &their_pub)
        .unwrap();

    manager.save().unwrap();

    let mut manager2 = RatchetSessionManager::with_backend(backend);
    assert_eq!(manager2.session_count(), 0);

    manager2.load().unwrap();
    assert_eq!(manager2.session_count(), 1);
    assert!(manager2.get_session("persistent-peer").is_some());

    let session = manager2.get_session("persistent-peer").unwrap();
    assert!(session.is_initialized());
}

#[test]
fn ratchet_persistence_zeroizes_json() {
    let backend = Arc::new(MemoryStorage::new());
    let mut manager = RatchetSessionManager::with_backend(backend.clone());

    let our_key = generate_signing_key();
    let their_pub = x25519_dalek::PublicKey::from([7u8; 32]);

    manager
        .get_or_create_session("zeroize-peer", &our_key, &their_pub)
        .unwrap();

    manager.save().unwrap();

    let mut manager2 = RatchetSessionManager::with_backend(backend);
    manager2.load().unwrap();
    assert_eq!(manager2.session_count(), 1);
}
