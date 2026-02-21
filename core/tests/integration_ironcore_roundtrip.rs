//! Integration tests: Two-node in-process encrypt → send → receive → decrypt flow.
//!
//! These tests exercise the public `IronCore` API end-to-end without touching
//! any network or swarm machinery.  They are deliberately minimal: no tokio
//! runtime, no tempfiles, no libp2p — just pure crypto/message flow.
//!
//! Run with:
//!   cargo test --test integration_ironcore_roundtrip

use scmessenger_core::IronCore;

// ============================================================================
// Helpers
// ============================================================================

/// Stand up an initialised IronCore instance with a generated identity.
fn make_node() -> IronCore {
    let node = IronCore::new();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

/// Return the hex-encoded Ed25519 public key for a node.
fn pubkey(node: &IronCore) -> String {
    node.get_identity_info()
        .public_key_hex
        .expect("node must be initialized before calling pubkey()")
}

// ============================================================================
// Test 1 — Happy-path roundtrip
// ============================================================================

/// Alice encrypts a message addressed to Bob; Bob decrypts it and recovers the
/// original plaintext.  The sender identity embedded in the decrypted `Message`
/// must match Alice's identity id.
#[test]
fn test_two_node_message_roundtrip() {
    let alice = make_node();
    let bob = make_node();

    let plaintext = "Hello Bob, this message is for your eyes only.";

    // Alice prepares (encrypts) the envelope.
    let envelope_bytes = alice
        .prepare_message(pubkey(&bob), plaintext.to_string())
        .expect("prepare_message must succeed");

    assert!(
        !envelope_bytes.is_empty(),
        "envelope_bytes must not be empty"
    );

    // Bob decrypts the envelope.
    let received = bob
        .receive_message(envelope_bytes)
        .expect("receive_message must succeed");

    // Plaintext content must be recovered verbatim.
    assert_eq!(
        received.text_content().expect("message must carry text"),
        plaintext,
        "decrypted plaintext must match the original"
    );

    // The sender field must identify Alice, not Bob or anyone else.
    let alice_identity_id = alice
        .get_identity_info()
        .identity_id
        .expect("alice must have an identity id");

    assert_eq!(
        received.sender_id, alice_identity_id,
        "decrypted message sender_id must equal Alice's identity id"
    );
}

// ============================================================================
// Test 2 — Wrong recipient cannot decrypt
// ============================================================================

/// Eve intercepts an envelope that was encrypted for Bob.  Eve's attempt to
/// decrypt it must fail because she does not possess Bob's private key.
#[test]
fn test_wrong_recipient_cannot_decrypt() {
    let alice = make_node();
    let bob = make_node();
    let eve = make_node();

    let envelope_bytes = alice
        .prepare_message(pubkey(&bob), "Secret for Bob only".to_string())
        .expect("prepare_message must succeed");

    let result = eve.receive_message(envelope_bytes);
    assert!(
        result.is_err(),
        "Eve must not be able to decrypt a message encrypted for Bob"
    );
}

// ============================================================================
// Test 3 — Tampered ciphertext is rejected
// ============================================================================

/// Alice creates a valid envelope for Bob.  An adversary flips a byte in the
/// middle of the ciphertext.  Bob's decryption attempt must return an error
/// because the AEAD authentication tag will not match the modified ciphertext.
#[test]
fn test_envelope_signature_verification() {
    let alice = make_node();
    let bob = make_node();

    let mut envelope_bytes = alice
        .prepare_message(pubkey(&bob), "Tamper me if you dare".to_string())
        .expect("prepare_message must succeed");

    // Flip a byte well into the payload (past any headers / nonce material).
    // The envelope is bincode-encoded; the ciphertext lives toward the end.
    // Flipping any byte inside the AEAD ciphertext will invalidate the tag.
    let tamper_index = envelope_bytes.len() / 2;
    envelope_bytes[tamper_index] ^= 0xFF;

    let result = bob.receive_message(envelope_bytes);
    assert!(
        result.is_err(),
        "Bob must reject a tampered envelope (AEAD authentication failure)"
    );
}

// ============================================================================
// Test 4 — Deduplication: replaying the same envelope is rejected
// ============================================================================

/// Bob receives the same envelope twice.  The second delivery must be rejected
/// by the inbox deduplication layer, not silently accepted.
#[test]
fn test_duplicate_delivery_rejected() {
    let alice = make_node();
    let bob = make_node();

    let envelope_bytes = alice
        .prepare_message(pubkey(&bob), "Once is enough".to_string())
        .expect("prepare_message must succeed");

    // First delivery succeeds.
    bob.receive_message(envelope_bytes.clone())
        .expect("first delivery must succeed");

    // Second delivery of the identical envelope must be rejected.
    let result = bob.receive_message(envelope_bytes);
    assert!(
        result.is_err(),
        "duplicate envelope delivery must be rejected by the inbox"
    );

    // Bob's inbox must contain exactly one copy of the message.
    assert_eq!(
        bob.inbox_count(),
        1,
        "inbox must hold exactly one message despite the replay attempt"
    );
}

// ============================================================================
// Test 5 — Multiple independent messages flow correctly
// ============================================================================

/// Alice sends three distinct messages to Bob.  All three must be decryptable
/// and must arrive with the correct content in order.
#[test]
fn test_multiple_messages_roundtrip() {
    let alice = make_node();
    let bob = make_node();
    let bob_pubkey = pubkey(&bob);

    let messages = ["First message", "Second message", "Third message"];

    for expected_text in &messages {
        let envelope_bytes = alice
            .prepare_message(bob_pubkey.clone(), expected_text.to_string())
            .expect("prepare_message must succeed");

        let received = bob
            .receive_message(envelope_bytes)
            .expect("receive_message must succeed");

        assert_eq!(
            received.text_content().expect("message must carry text"),
            *expected_text,
            "decrypted text must match the sent text"
        );
    }

    assert_eq!(
        bob.inbox_count(),
        messages.len() as u32,
        "bob's inbox must hold all received messages"
    );
}

// ============================================================================
// Test 6 — Self-message: a node can send to itself
// ============================================================================

/// A node encrypts a message addressed to its own public key and then decrypts
/// it.  This validates that the ECDH key-derivation path works when sender and
/// recipient share the same Ed25519 signing key.
#[test]
fn test_self_message_roundtrip() {
    let node = make_node();

    let plaintext = "Note to self";

    let envelope_bytes = node
        .prepare_message(pubkey(&node), plaintext.to_string())
        .expect("prepare_message to self must succeed");

    let received = node
        .receive_message(envelope_bytes)
        .expect("receive_message from self must succeed");

    assert_eq!(
        received
            .text_content()
            .expect("self-message must carry text"),
        plaintext,
        "self-message plaintext must be recovered verbatim"
    );
}

// ============================================================================
// Test 7 — Empty-string body is handled gracefully
// ============================================================================

/// An empty payload must survive the full encrypt / decrypt round-trip without
/// panicking or returning an error.
#[test]
fn test_empty_payload_roundtrip() {
    let alice = make_node();
    let bob = make_node();

    let envelope_bytes = alice
        .prepare_message(pubkey(&bob), String::new())
        .expect("prepare_message with empty body must succeed");

    let received = bob
        .receive_message(envelope_bytes)
        .expect("receive_message of empty body must succeed");

    assert_eq!(
        received.text_content().unwrap_or_default(),
        "",
        "empty payload must round-trip as empty string"
    );
}
