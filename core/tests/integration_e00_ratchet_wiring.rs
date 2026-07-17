//! Integration tests for the E-00 ratchet-aware send/receive wiring.
//!
//! These tests exercise `IronCore::prepare_message` and `IronCore::receive_message`
//! through the new ratchet-aware fallback wrappers without network or swarm
//! machinery.  They verify both the PQ-capable ratchet round-trip and the legacy
//! ECDH fallback for peers with no stored bundle.
//!
//! Run with:
//!   cargo test --test integration_e00_ratchet_wiring

use scmessenger_core::{IronCore, MessageType};

/// Stand up an initialised IronCore instance with a generated identity.
fn make_node() -> IronCore {
    let node = IronCore::new();
    node.grant_consent();
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

/// Exchange signed public-key bundles between two nodes so that each side's
/// ratchet path can locate the peer's bundle.
fn exchange_bundles(alice: &IronCore, bob: &IronCore) {
    let alice_keys = alice
        .get_identity_keys()
        .expect("alice must have identity keys");
    let bob_keys = bob
        .get_identity_keys()
        .expect("bob must have identity keys");

    let alice_bundle =
        scmessenger_core::identity::sign_bundle(&alice_keys).expect("alice bundle must sign");
    let bob_bundle =
        scmessenger_core::identity::sign_bundle(&bob_keys).expect("bob bundle must sign");

    // Store each other's bundle keyed by the other's hex public key.
    alice
        .contacts_store_manager()
        .save_contact_bundle(&bob_keys.public_key_hex(), &bob_bundle)
        .expect("alice must store bob's bundle");
    bob.contacts_store_manager()
        .save_contact_bundle(&alice_keys.public_key_hex(), &alice_bundle)
        .expect("bob must store alice's bundle");
}

/// Send a text message from `sender` to `recipient` and return the prepared envelope.
fn send_text(
    sender: &IronCore,
    recipient_pubkey: &str,
    text: &str,
) -> scmessenger_core::PreparedMessage {
    sender
        .prepare_message(
            recipient_pubkey.to_string(),
            text.to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message must succeed")
}

/// Receive an envelope and assert the text content matches.
fn receive_and_assert(recipient: &IronCore, envelope_data: Vec<u8>, expected_text: &str) {
    let received = recipient
        .receive_message(envelope_data)
        .expect("receive_message must succeed");
    assert_eq!(
        received.text_content().expect("message must carry text"),
        expected_text,
        "decrypted plaintext must match the original"
    );
}

// ============================================================================
// Test 1 — Ratchet round-trip with bundle exchange
// ============================================================================

/// Two nodes with exchanged bundles perform an end-to-end ratchet encrypt /
/// decrypt flow.  Sending a second message proves the ratchet advanced rather
/// than reusing a static key.
#[test]
fn test_ratchet_roundtrip_two_messages() {
    let alice = make_node();
    let bob = make_node();
    exchange_bundles(&alice, &bob);

    let bob_pubkey = pubkey(&bob);

    let first = send_text(&alice, &bob_pubkey, "hello ratchet");
    receive_and_assert(&bob, first.envelope_data, "hello ratchet");

    let second = send_text(&alice, &bob_pubkey, "hello again ratchet");
    receive_and_assert(&bob, second.envelope_data, "hello again ratchet");
}

// ============================================================================
// Test 2 — Legacy fallback when recipient has no stored bundle
// ============================================================================

/// A recipient without a stored bundle forces the sender to fall back to the
/// legacy static-ECDH path.  The message must still decrypt successfully.
#[test]
fn test_legacy_fallback_no_bundle() {
    let alice = make_node();
    let bob = make_node();

    // Deliberately do NOT exchange bundles -- the ratchet path should fall back
    // to the V1 / static-ECDH legacy path.
    let bob_pubkey = pubkey(&bob);
    let prepared = send_text(&alice, &bob_pubkey, "legacy fallback");
    receive_and_assert(&bob, prepared.envelope_data, "legacy fallback");
}
