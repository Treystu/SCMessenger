//! Integration tests: Contact Block / Unblock / Delete state machine.
//!
//! Verifies the three evidentiary-retention scenarios required by v0.2.1:
//!
//! **Scenario 1** — Blocked-only: message from a blocked peer is persisted in
//! the DB (evidentiary retention) but is hidden from every UI-facing history
//! query (`conversation`, `recent`).
//!
//! **Scenario 2** — Unblock restores visibility: after the peer is unblocked
//! all previously hidden messages immediately become visible in normal queries.
//!
//! **Scenario 3** — Blocked + Deleted (cascade purge): all existing stored
//! messages for the peer are purged and subsequent network payloads are dropped
//! at the ingress layer without being persisted.
//!
//! Run with:
//!   cargo test --test integration_contact_block

use scmessenger_core::IronCore;
use std::option::Option::{Some, None};

// ============================================================================
// Helpers
// ============================================================================

/// Stand up an initialised `IronCore` instance with a generated identity.
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

/// Return the Blake3 identity id (64 hex chars) for a node.
fn identity_id(node: &IronCore) -> String {
    node.get_identity_info()
        .identity_id
        .expect("node must be initialized before calling identity_id()")
}

// ============================================================================
// Scenario 1 — Blocked peer: message persisted but hidden from UI queries
// ============================================================================

/// Alice sends a message to Bob.  Bob blocks Alice, then Alice's message is
/// received (simulated by decrypting on Bob's core).
///
/// Expected:
/// - The message IS in the raw history store (evidentiary retention).
/// - Normal `conversation()` query returns **no** messages for Alice's peer.
#[test]
fn test_blocked_message_persisted_but_hidden() {
    let alice = make_node();
    let bob = make_node();

    let alice_id = identity_id(&alice);
    let plaintext = "Hello Bob — I am Alice.";

    // Alice prepares an envelope for Bob.
    let envelope = alice
        .prepare_message(pubkey(&bob), plaintext.to_string(), None)
        .expect("prepare_message must succeed");

    // Bob blocks Alice BEFORE receiving the message.
    bob.block_peer(alice_id.clone(), None, Some("test block".to_string()))
        .expect("block_peer must succeed");

    assert!(
        bob.is_peer_blocked(alice_id.clone(), None)
            .expect("is_peer_blocked must succeed"),
        "Alice must be marked as blocked on Bob's node"
    );

    // Bob receives Alice's message.  The Core must still store it (evidentiary
    // retention) but mark it as hidden.
    bob.receive_message(envelope)
        .expect("receive_message must succeed even from blocked peer");

    let history = bob.history_store_manager();

    // 1a. The raw history (including hidden records) MUST contain the message.
    let raw_records = history
        .recent_including_hidden(Some(alice_id.clone()), 100)
        .expect("recent_including_hidden must succeed");

    assert_eq!(
        raw_records.len(),
        1,
        "evidentiary retention: the message from the blocked peer must be in the DB"
    );
    assert!(
        raw_records[0].hidden,
        "the stored message must be marked hidden"
    );
    assert_eq!(
        raw_records[0].content, plaintext,
        "stored plaintext must match original"
    );

    // 1b. The normal UI-facing `conversation()` query must return ZERO messages.
    let ui_records = history
        .conversation(alice_id.clone(), 100)
        .expect("conversation must succeed");

    assert_eq!(
        ui_records.len(),
        0,
        "UI query must return no messages from a blocked peer (evidentiary retention)"
    );

    // 1c. The recent() query across all peers must also exclude hidden messages.
    let all_recent = history.recent(None, 100).expect("recent must succeed");

    let alice_visible: Vec<_> = all_recent
        .iter()
        .filter(|r| r.peer_id.eq_ignore_ascii_case(&alice_id))
        .collect();

    assert_eq!(
        alice_visible.len(),
        0,
        "recent() must not surface hidden messages from blocked peers"
    );
}

// ============================================================================
// Scenario 2 — Unblock restores visibility of retained messages
// ============================================================================

/// Continuing from Scenario 1: after Bob unblocks Alice, all previously hidden
/// messages must immediately become visible in normal history queries.
#[test]
fn test_unblock_restores_hidden_message_visibility() {
    let alice = make_node();
    let bob = make_node();

    let alice_id = identity_id(&alice);
    let plaintext = "Visible after unblock.";

    // Alice sends; Bob blocks Alice; Bob receives the hidden message.
    let envelope = alice
        .prepare_message(pubkey(&bob), plaintext.to_string(), None)
        .expect("prepare_message must succeed");

    bob.block_peer(alice_id.clone(), None, None)
        .expect("block_peer must succeed");

    bob.receive_message(envelope)
        .expect("receive_message must succeed");

    let history = bob.history_store_manager();

    // Confirm the message is hidden while Alice is blocked.
    let hidden_records = history
        .recent_including_hidden(Some(alice_id.clone()), 100)
        .expect("recent_including_hidden must succeed");
    assert_eq!(hidden_records.len(), 1, "one hidden record must exist");
    assert!(hidden_records[0].hidden, "record must be marked hidden");

    // Unblock Alice — this must unhide all her retained messages.
    bob.unblock_peer(alice_id.clone(), None)
        .expect("unblock_peer must succeed");

    assert!(
        !bob.is_peer_blocked(alice_id.clone(), None)
            .expect("is_peer_blocked must succeed"),
        "Alice must be unblocked"
    );

    // After unblocking, the normal conversation query must surface the message.
    let visible_records = history
        .conversation(alice_id.clone(), 100)
        .expect("conversation must succeed");

    assert_eq!(
        visible_records.len(),
        1,
        "previously hidden message must become visible after unblock"
    );
    assert!(
        !visible_records[0].hidden,
        "record must no longer be marked hidden after unblock"
    );
    assert_eq!(
        visible_records[0].content, plaintext,
        "recovered plaintext must match original"
    );
}

// ============================================================================
// Scenario 3 — Blocked + Deleted: cascade purge and ingress drop
// ============================================================================

/// Bob previously had a conversation with Alice (one message stored).  Bob then
/// invokes `block_and_delete_peer`, which must:
/// a. Purge all existing stored messages for Alice.
/// b. Mark Alice as blocked + deleted so future ingress payloads are dropped.
#[test]
fn test_block_and_delete_purges_messages_and_drops_future_payloads() {
    let alice = make_node();
    let bob = make_node();

    let alice_id = identity_id(&alice);

    // --- Setup: establish an existing conversation ---
    // Alice sends a first message; Bob receives it normally (no block yet).
    let envelope1 = alice
        .prepare_message(pubkey(&bob), "First message".to_string(), None)
        .expect("prepare_message must succeed");

    bob.receive_message(envelope1)
        .expect("receive_message must succeed");

    let history = bob.history_store_manager();
    let pre_purge = history
        .conversation(alice_id.clone(), 100)
        .expect("conversation must succeed");

    assert_eq!(
        pre_purge.len(),
        1,
        "one message must be stored before block+delete"
    );

    // --- Action: block AND delete Alice ---
    bob.block_and_delete_peer(alice_id.clone(), None, Some("permanent block".to_string()))
        .expect("block_and_delete_peer must succeed");

    // 3a. Existing messages must be purged.
    let post_purge = history
        .recent_including_hidden(Some(alice_id.clone()), 100)
        .expect("recent_including_hidden must succeed");

    assert_eq!(
        post_purge.len(),
        0,
        "cascade purge: all existing messages for the blocked+deleted peer must be removed"
    );

    // 3b. Future payloads from Alice must be rejected at ingress.
    //     Alice sends a second message — Bob's core must reject and NOT persist it.
    let envelope2 = alice
        .prepare_message(pubkey(&bob), "Should be dropped".to_string(), None)
        .expect("prepare_message must succeed");

    // receive_message must return Err(Blocked) so callers cannot surface the
    // decrypted content — this is the correct ingress-reject semantic.
    let result = bob.receive_message(envelope2);
    assert!(
        result.is_err(),
        "receive_message must return Err for blocked+deleted peer"
    );

    let after_drop = history
        .recent_including_hidden(Some(alice_id.clone()), 100)
        .expect("recent_including_hidden must succeed");

    assert_eq!(
        after_drop.len(),
        0,
        "ingress drop: payload from blocked+deleted peer must NOT be stored"
    );

    // 3c. Bob's inbox counter must reflect that the dropped payload was NOT
    //     registered in the dedup store.
    assert_eq!(
        bob.inbox_count(),
        1, // Only the first (pre-purge) message was registered in the dedup inbox.
        "inbox must hold only the pre-purge message; dropped payloads must not be counted"
    );
}
