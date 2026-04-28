use scmessenger_core::store::backend::SledStorage;
use scmessenger_core::store::{Outbox, QueuedMessage};
use std::sync::Arc;

fn make_msg(id: &str, recipient: &str, attempts: u32) -> QueuedMessage {
    QueuedMessage {
        message_id: id.to_string(),
        recipient_id: recipient.to_string(),
        envelope_data: vec![1, 2, 3, 4],
        queued_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        attempts,
    }
}

#[test]
fn retry_attempt_state_persists_across_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("retry_outbox")
        .to_str()
        .unwrap()
        .to_string();

    {
        let backend = Arc::new(SledStorage::new(&path).unwrap());
        let mut outbox = Outbox::persistent(backend);
        outbox.enqueue(make_msg("msg-retry", "peer-a", 0)).unwrap();
        outbox.record_attempt("msg-retry");
        outbox.record_attempt("msg-retry");
        outbox.record_attempt("msg-retry");
    }

    {
        let backend = Arc::new(SledStorage::new(&path).unwrap());
        let outbox = Outbox::persistent(backend);
        let queued = outbox.peek_for_peer("peer-a");
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].message_id, "msg-retry");
        assert_eq!(queued[0].attempts, 3);
    }
}

#[test]
fn undelivered_message_does_not_transition_to_terminal_drop() {
    let mut outbox = Outbox::new();
    outbox
        .enqueue(make_msg("msg-undelivered", "peer-b", u32::MAX - 2))
        .unwrap();

    // Simulate repeated retry ticks over a long-lived undelivered flow.
    for _ in 0..50 {
        outbox.record_attempt("msg-undelivered");
    }

    let queued = outbox.peek_for_peer("peer-b");
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].message_id, "msg-undelivered");
    assert_eq!(queued[0].attempts, u32::MAX);
}
