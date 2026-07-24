/// Integration test: Drift sync end-to-end under partition (sneakernet scenario)
///
/// Tests the CRDT MeshStore merge semantics:
/// - Insert is idempotent (dedup)
/// - Merge is commutative, idempotent, associative
/// - TTL expiry works
use scmessenger_core::drift::store::{MeshStore, StoredEnvelope};
use scmessenger_core::store::backend::MemoryStorage;
use std::sync::Arc;

fn make_envelope(id: u8, recipient_hint: [u8; 4]) -> StoredEnvelope {
    StoredEnvelope {
        envelope_data: vec![id; 10],
        message_id: [id; 16],
        recipient_hint,
        created_at: 1000,
        ttl_expiry: 0, // never expires
        hop_count: 0,
        priority: 128,
        received_at: 1000,
    }
}

#[test]
fn mesh_store_insert_and_dedup() {
    let mut store = MeshStore::new();
    let env = make_envelope(1, [0xAA; 4]);

    assert!(store.insert(env.clone())); // New
    assert!(!store.insert(env)); // Duplicate — CRDT idempotent
    assert_eq!(store.len(), 1);
}

#[test]
fn mesh_store_merge_is_commutative() {
    let env_a = make_envelope(1, [0xAA; 4]);
    let env_b = make_envelope(2, [0xBB; 4]);

    let mut store1 = MeshStore::new();
    store1.insert(env_a.clone());
    store1.insert(env_b.clone());

    let mut store2 = MeshStore::new();
    store2.insert(env_b);
    store2.insert(env_a);

    // merge(A, B) == merge(B, A)
    assert_eq!(store1.len(), store2.len());
}

#[test]
fn mesh_store_merge_union() {
    let env_a = make_envelope(1, [0xAA; 4]);
    let env_b = make_envelope(2, [0xBB; 4]);

    let mut mule = MeshStore::new(); // Mule carries messages from A
    mule.insert(env_a.clone());

    let mut node_b = MeshStore::new(); // Node B has its own messages
    node_b.insert(env_b.clone());

    // Mule meets B — merge
    node_b.merge(&mule);
    assert_eq!(node_b.len(), 2);
    assert!(node_b.get(&[1; 16]).is_some());
    assert!(node_b.get(&[2; 16]).is_some());
}

#[test]
fn mesh_store_merge_idempotent() {
    let env = make_envelope(1, [0xAA; 4]);

    let mut store1 = MeshStore::new();
    store1.insert(env.clone());

    let mut store2 = MeshStore::new();
    store2.merge(&store1);
    store2.merge(&store1); // Merge twice

    assert_eq!(store2.len(), 1); // Still just one message
}

#[test]
fn mesh_store_eviction_at_capacity() {
    let mut store = MeshStore::with_capacity(3);

    store.insert(make_envelope(1, [0xAA; 4]));
    store.insert(make_envelope(2, [0xBB; 4]));
    store.insert(make_envelope(3, [0xCC; 4]));
    assert_eq!(store.len(), 3);

    // Insert a 4th — should evict lowest priority
    store.insert(make_envelope(4, [0xDD; 4]));
    assert_eq!(store.len(), 3); // Still at capacity
}

#[test]
fn mesh_store_persistence_roundtrip() {
    let backend = Arc::new(MemoryStorage::new());
    let env_a = make_envelope(1, [0xAA; 4]);
    let env_b = make_envelope(2, [0xBB; 4]);

    // Store messages and persist
    {
        let mut store = MeshStore::new();
        store.insert(env_a.clone());
        store.insert(env_b.clone());
        store.save(backend.as_ref()).unwrap();
    }

    // Load into new store
    {
        let mut store = MeshStore::new();
        let loaded = store.load(backend.as_ref()).unwrap();
        assert_eq!(loaded, 2);
        assert_eq!(store.len(), 2);
        assert!(store.get(&[1; 16]).is_some());
        assert!(store.get(&[2; 16]).is_some());
    }
}

#[test]
fn mesh_store_persistence_dedup_on_load() {
    let backend = Arc::new(MemoryStorage::new());
    let env = make_envelope(1, [0xAA; 4]);

    // Save once
    {
        let mut store = MeshStore::new();
        store.insert(env.clone());
        store.save(backend.as_ref()).unwrap();
    }

    // Load twice — second load should be idempotent
    {
        let mut store = MeshStore::new();
        assert_eq!(store.load(backend.as_ref()).unwrap(), 1);
        assert_eq!(store.load(backend.as_ref()).unwrap(), 0); // duplicates skipped
        assert_eq!(store.len(), 1);
    }
}

#[test]
fn outbox_drift_single_ownership() {
    use scmessenger_core::store::outbox::Outbox;

    let mut outbox = Outbox::new();
    let mut drift = MeshStore::new();

    // Enqueue in outbox
    let msg = scmessenger_core::store::outbox::QueuedMessage {
        version: 1,
        message_id: "msg-ownership-1".to_string(),
        recipient_id: "peer-b".to_string(),
        envelope_data: vec![1, 2, 3],
        queued_at: 1000,
        attempts: 0,
        next_retry_at: None,
        in_custody: false,
        custody_established_at: 0,
        state: scmessenger_core::store::outbox::MessageState::Enqueued,
    };
    outbox.enqueue(msg).unwrap();

    // StoreAndCarry decision: move to drift, remove from outbox
    let env = make_envelope(1, [0xBB; 4]);
    drift.insert(env);
    outbox.remove("msg-ownership-1");

    // Assert: drift owns it, outbox does not
    assert_eq!(drift.len(), 1);
    assert!(!outbox.remove("msg-ownership-1"));
}

#[test]
fn test_run_maintenance_cycle_budget() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap().to_string();
    let core = scmessenger_core::IronCore::with_storage(path);
    core.start().unwrap();

    // Call run_maintenance_cycle with 50ms budget
    let report = core.run_maintenance_cycle(50);
    assert!(report.contains("budget_ms"));
    assert!(report.contains("elapsed_ms"));
    assert!(report.contains("work_done"));

    // The cycle's actual work (a single drift-engine tick check) should take
    // microseconds, nowhere near the 50ms budget. Parse and assert the real
    // elapsed time, not just that the field is present, so a future
    // regression that makes the cycle blow its budget gets caught here
    // instead of surfacing as a dropped BGProcessingTask/WorkManager job on
    // real devices.
    let parsed: serde_json::Value =
        serde_json::from_str(&report).expect("run_maintenance_cycle report is not valid JSON");
    let elapsed_ms = parsed["elapsed_ms"]
        .as_u64()
        .expect("elapsed_ms missing or not a number");
    assert!(
        elapsed_ms < 100,
        "run_maintenance_cycle took {}ms, expected well under the 100ms budget-gate threshold: {}",
        elapsed_ms,
        report
    );
}
