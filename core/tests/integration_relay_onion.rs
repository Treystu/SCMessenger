//! Integration test for multi-hop onion routing.
//!
//! Validates the full lifecycle:
//! 1. Alice prepares an onion-routed message through Relay1 and Relay2 to Bob.
//! 2. Relay1 receives the bytes, peels a layer, and identifies Relay2 as next hop.
//! 3. Relay2 receives the payload, peels a layer, and identifies Bob as next hop.
//! 4. Bob receives the remaining payload, peels the final layer, and decrypts the message.

use scmessenger_core::{IronCore, MessageType};

// ============================================================================
// Helpers
// ============================================================================

fn make_node() -> IronCore {
    let node = IronCore::new();
    node.grant_consent();
    node.initialize_identity()
        .expect("identity initialization must succeed");
    node
}

fn pubkey(node: &IronCore) -> String {
    node.get_identity_info()
        .public_key_hex
        .expect("node must be initialized")
}

// ============================================================================
// Test — 2-Hop Onion Relay
// ============================================================================

#[test]
fn test_onion_relay_lifecycle() {
    let alice = make_node();
    let relay1 = make_node();
    let relay2 = make_node();
    let bob = make_node();

    let plaintext = "This message is anonymous and secure.";

    // 1. Alice prepares the inner payload (standard envelope for Bob)
    let bob_envelope = alice
        .prepare_message(pubkey(&bob), plaintext.to_string(), None)
        .expect("Alice failed to prepare inner envelope");

    // 2. Alice prepares the onion layers: Alice -> Relay1 -> Relay2 -> Bob
    // Note: prepare_onion_message takes the full path of relays INCLUDING the destination
    let path = vec![pubkey(&relay1), pubkey(&relay2), pubkey(&bob)];
    let path_json = serde_json::to_string(&path).unwrap();
    
    let onion_bytes = alice
        .prepare_onion_message(bob_envelope, path_json)
        .expect("Alice failed to prepare onion message");

    // 3. Relay 1 receives onion_bytes from Alice
    let relay1_result = relay1
        .receive_message(onion_bytes)
        .expect("Relay1 failed to process onion packet");
    
    assert_eq!(relay1_result.message_type, MessageType::OnionRelay);
    assert_eq!(relay1_result.recipient_id, pubkey(&relay2), "Relay1 should forward to Relay2");

    // 4. Relay 2 receives payload from Relay 1
    let relay2_result = relay2
        .receive_message(relay1_result.payload)
        .expect("Relay2 failed to process onion packet");
        
    assert_eq!(relay2_result.message_type, MessageType::OnionRelay);
    assert_eq!(relay2_result.recipient_id, pubkey(&bob), "Relay2 should forward to Bob");

    // 5. Bob receives payload from Relay 2 (final hop)
    let bob_result = bob
        .receive_message(relay2_result.payload)
        .expect("Bob failed to process final onion layer");
        
    // Bob should see the decrypted text message
    assert_eq!(bob_result.message_type, MessageType::Text);
    assert_eq!(
        bob_result.text_content().unwrap(),
        plaintext,
        "Bob should recover original plaintext"
    );
    
    assert_eq!(bob_result.sender_id, alice.get_identity_info().identity_id.unwrap());
}
