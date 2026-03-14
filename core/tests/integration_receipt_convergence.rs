use libp2p::PeerId;
use scmessenger_core::store::relay_custody::{CustodyState, RelayCustodyStore};
use scmessenger_core::transport::mesh_routing::MultiPathDelivery;

#[test]
fn multi_forwarder_convergence_stops_duplicate_retry_and_purges_pending() {
    let destination = PeerId::random();
    let destination_id = destination.to_string();
    let relay_message_id = "relay-msg-ws4-converge".to_string();

    let forwarder_a_store = RelayCustodyStore::in_memory();
    let forwarder_b_store = RelayCustodyStore::in_memory();
    let mut forwarder_a_delivery = MultiPathDelivery::new();
    let mut forwarder_b_delivery = MultiPathDelivery::new();

    forwarder_a_delivery.start_delivery(relay_message_id.clone(), destination);
    forwarder_b_delivery.start_delivery(relay_message_id.clone(), destination);

    let custody_a = forwarder_a_store
        .accept_custody(
            "source-peer".to_string(),
            destination_id.clone(),
            relay_message_id.clone(),
            vec![1, 2, 3],
            None,
            None,
        )
        .expect("forwarder A should accept custody");
    let _custody_b = forwarder_b_store
        .accept_custody(
            "source-peer".to_string(),
            destination_id.clone(),
            relay_message_id.clone(),
            vec![1, 2, 3],
            None,
            None,
        )
        .expect("forwarder B should accept custody");

    // Forwarder A wins race: recipient accepted delivery.
    forwarder_a_store
        .mark_delivered(&destination_id, &custody_a.custody_id, "recipient_ack")
        .expect("forwarder A should mark custody delivered");
    assert!(
        forwarder_a_delivery.converge_delivery(&relay_message_id),
        "forwarder A should clear its own retry tracking after delivery"
    );

    // Convergence marker reaches forwarder B.
    let converged_count = forwarder_b_store
        .converge_delivered_for_message(
            &destination_id,
            &relay_message_id,
            "delivery_convergence_marker",
        )
        .expect("forwarder B should converge duplicate custody");
    assert_eq!(
        converged_count, 1,
        "forwarder B should purge one pending duplicate custody record"
    );
    assert!(
        forwarder_b_delivery.converge_delivery(&relay_message_id),
        "forwarder B should stop duplicate retry tracking"
    );

    assert!(
        forwarder_b_store
            .pending_for_destination(&destination_id, 100)
            .is_empty(),
        "forwarder B should have no pending duplicate attempts after convergence"
    );
    assert!(
        forwarder_b_delivery
            .delivery_attempt(&relay_message_id)
            .is_none(),
        "forwarder B retry state should be cleared after convergence"
    );
}

#[test]
fn convergence_purges_dispatching_duplicate_attempts() {
    let destination = PeerId::random().to_string();
    let relay_message_id = "relay-msg-dispatching-converge".to_string();
    let store = RelayCustodyStore::in_memory();

    let custody = store
        .accept_custody(
            "source-peer".to_string(),
            destination.clone(),
            relay_message_id.clone(),
            vec![9, 9, 9],
            None,
            None,
        )
        .expect("custody accepted");
    store
        .mark_dispatching(&destination, &custody.custody_id, "retry_tick")
        .expect("dispatching transition should succeed");

    let converged = store
        .converge_delivered_for_message(
            &destination,
            &relay_message_id,
            "delivery_convergence_marker",
        )
        .expect("convergence should succeed");
    assert_eq!(converged, 1);
    assert!(store.pending_for_destination(&destination, 100).is_empty());
    let transitions = store.transitions_for_custody(&custody.custody_id);
    assert_eq!(
        transitions.last().map(|t| t.to_state),
        Some(CustodyState::Delivered)
    );
}
