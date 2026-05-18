// WS13.6 Migration and Compatibility Mode Integration Tests
//
// Validates that pre-WS13 stores and wire formats are correctly handled
// during the migration window (Phase A and Phase B compat mode).

use scmessenger_core::identity::{IdentityManager, IdentityStore};
use scmessenger_core::store::contacts::{Contact, ContactManager};
use scmessenger_core::store::relay_custody::{
    CustodyCompatMode, CustodyEnforcement, CustodyError, RelayCustodyStore,
};
use std::sync::Arc;
use uuid::Uuid;

// --- Identity hydration / backfill tests ---

#[test]
fn pre_ws13_identity_store_backfills_device_id_on_hydrate() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("pre_ws13_identity").to_str().unwrap().to_string();

    const MAX_REOPEN_ATTEMPTS: u64 = 10;
    const REOPEN_BACKOFF_BASE_MS: u64 = 25;

    // Phase 1: Simulate a pre-WS13 store: persist identity keys but NO device metadata.
    let original_identity_id = {
        let backend = Arc::new(
            scmessenger_core::store::backend::SledStorage::new(&path).unwrap(),
        ) as Arc<dyn scmessenger_core::store::backend::StorageBackend>;
        let mut manager = IdentityManager::with_backend(backend.clone()).unwrap();
        manager.initialize().unwrap();

        // Clear device metadata to simulate pre-WS13 state (use same backend to avoid sled lock)
        let store = IdentityStore::persistent(backend.clone());
        store.clear().unwrap();
        // Re-save only the keys and nickname (not device metadata)
        if let Some(keys) = manager.keys() {
            let keys_only_store = IdentityStore::persistent(backend.clone());
            keys_only_store.save_keys(keys).unwrap();
        }

        let id = manager.identity_id().unwrap();
        drop(manager);
        id
    };

    // Phase 2: Load the store on WS13 code. Device metadata should be auto-generated.
    let backend2: Arc<dyn scmessenger_core::store::backend::StorageBackend> =
        (0..MAX_REOPEN_ATTEMPTS)
            .find_map(|attempt| {
                match scmessenger_core::store::backend::SledStorage::new(&path) {
                    Ok(storage) => Some(Arc::new(storage) as Arc<dyn scmessenger_core::store::backend::StorageBackend>),
                    Err(_) if attempt + 1 < MAX_REOPEN_ATTEMPTS => {
                        std::thread::sleep(std::time::Duration::from_millis(
                            REOPEN_BACKOFF_BASE_MS * (attempt + 1),
                        ));
                        None
                    }
                    Err(err) => panic!("failed to reopen sled store: {err}"),
                }
            })
            .unwrap();
    let manager2 = IdentityManager::with_backend(backend2).unwrap();

    // Identity must survive migration
    assert_eq!(manager2.identity_id(), Some(original_identity_id));

    // Device metadata must be backfilled
    let device_id = manager2
        .device_id()
        .expect("device_id must be backfilled on WS13 hydration");
    assert!(
        Uuid::parse_str(&device_id).is_ok(),
        "backfilled device_id must be valid UUID"
    );

    let seniority = manager2
        .seniority_timestamp()
        .expect("seniority_timestamp must be backfilled on WS13 hydration");
    assert!(seniority > 0, "seniority_timestamp must be non-zero");
}

#[test]
fn identity_hydrate_does_not_rotate_key_material() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("no_key_rotation")
        .to_str()
        .unwrap()
        .to_string();

    let (original_identity_id, original_pub_key) = {
        let backend = Arc::new(
            scmessenger_core::store::backend::SledStorage::new(&path).unwrap(),
        );
        let mut manager = IdentityManager::with_backend(backend).unwrap();
        manager.initialize().unwrap();
        (
            manager.identity_id().unwrap(),
            manager.public_key_hex().unwrap(),
        )
    };

    // Re-open the store. The identity keys must NOT change.
    let backend2 = Arc::new(
        scmessenger_core::store::backend::SledStorage::new(&path).unwrap(),
    );
    let manager2 = IdentityManager::with_backend(backend2).unwrap();

    assert_eq!(manager2.identity_id(), Some(original_identity_id));
    assert_eq!(manager2.public_key_hex(), Some(original_pub_key));
}

#[test]
fn identity_device_metadata_does_not_leak_into_backup_payload() {
    // device_id is installation-local and must NOT appear in export_key_bytes.
    let mut manager = IdentityManager::new();
    manager.initialize().unwrap();

    let exported = manager.export_key_bytes().unwrap();
    assert!(exported.len() > 0, "export must contain key bytes");

    // Import into a fresh manager: device_id must differ (installation-local)
    let mut manager2 = IdentityManager::new();
    manager2.import_key_bytes(&exported).unwrap();

    assert_ne!(
        manager.device_id(),
        manager2.device_id(),
        "device_id must NOT be carried in backup payload"
    );
}

// --- Legacy contact deserialization tests ---

#[test]
fn legacy_contact_loads_with_none_default_for_last_known_device_id() {
    // Simulate a pre-WS13 contact record serialized without last_known_device_id.
    let pre_ws13_json = r#"{
        "peer_id": "12D3KooWLegacyPeer",
        "nickname": "Alice",
        "local_nickname": null,
        "public_key": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
        "added_at": 1700000000,
        "last_seen": 1700050000,
        "notes": null
    }"#;

    let contact: Contact = serde_json::from_str(pre_ws13_json)
        .expect("pre-WS13 contact JSON must deserialize");
    assert!(
        contact.last_known_device_id.is_none(),
        "legacy contacts must default last_known_device_id to None"
    );
    assert_eq!(contact.peer_id, "12D3KooWLegacyPeer");
    assert_eq!(contact.nickname, Some("Alice".to_string()));
}

#[test]
fn ws13_contact_roundtrips_with_last_known_device_id() {
    let mgr = ContactManager::new(Arc::new(
        scmessenger_core::store::backend::MemoryStorage::new(),
    ));

    let mut contact = Contact::new("peer-ws13".to_string(), "pk-hex-32bytes-xxxxxxxxx".to_string());
    contact.last_known_device_id = Some("550e8400-e29b-41d4-a716-446655440000".to_string());
    mgr.add(contact).unwrap();

    let loaded = mgr.get("peer-ws13".to_string()).unwrap().unwrap();
    assert_eq!(
        loaded.last_known_device_id.as_deref(),
        Some("550e8400-e29b-41d4-a716-446655440000")
    );
}

// --- Compat mode relay request tests ---

#[test]
fn legacy_relay_request_without_intended_device_id_accepted_in_compat_mode_phase_a() {
    let store = RelayCustodyStore::in_memory();

    // Accept custody with None for both identity_id and device_id (legacy v0.2.0 client)
    let result = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-legacy-1".to_string(),
        vec![1, 2, 3],
        None,
        None,
    );

    assert!(
        result.is_ok(),
        "legacy relay requests without device metadata must be accepted in compat mode"
    );
    let msg = result.unwrap();
    assert!(msg.recipient_identity_id.is_none());
    assert!(msg.intended_device_id.is_none());
}

#[test]
fn legacy_relay_request_with_identity_but_no_device_accepted_in_compat_mode() {
    let store = RelayCustodyStore::in_memory();
    let identity_id = "a".repeat(64);

    // Accept custody with identity_id but no intended_device_id
    let result = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-legacy-2".to_string(),
        vec![4, 5, 6],
        Some(identity_id),
        None,
    );

    assert!(
        result.is_ok(),
        "relay request with identity_id but no device_id must be accepted in compat mode"
    );
    let msg = result.unwrap();
    assert!(msg.recipient_identity_id.is_some());
    assert!(msg.intended_device_id.is_none());
}

#[test]
fn ws13_relay_request_with_device_metadata_is_enforced() {
    let store = RelayCustodyStore::in_memory();
    let identity_id = "f".repeat(64);
    let device_id = Uuid::new_v4().to_string();

    // Register the identity first
    store
        .register_identity(identity_id.clone(), device_id.clone(), 1700000000)
        .unwrap();

    // Accept custody with full WS13 metadata
    let result = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-ws13-1".to_string(),
        vec![7, 8, 9],
        Some(identity_id),
        Some(device_id),
    );

    assert!(
        result.is_ok(),
        "WS13 relay request with matching device_id must be accepted"
    );
}

// --- Custody enforcement state machine tests ---

#[test]
fn custody_enforcement_active_match_accepts() {
    let store = RelayCustodyStore::in_memory();
    let identity_id = "1".repeat(64);
    let device_id = Uuid::new_v4().to_string();

    store
        .register_identity(identity_id.clone(), device_id.clone(), 1700000000)
        .unwrap();

    let result = store.enforce_custody(&identity_id, &device_id);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        CustodyEnforcement::Active {
            identity_id,
            device_id
        }
    );
}

#[test]
fn custody_enforcement_device_mismatch_rejects() {
    let store = RelayCustodyStore::in_memory();
    let identity_id = "2".repeat(64);
    let device_id = Uuid::new_v4().to_string();
    let wrong_device = Uuid::new_v4().to_string();

    store
        .register_identity(identity_id.clone(), device_id, 1700000000)
        .unwrap();

    let result = store.enforce_custody(&identity_id, &wrong_device);
    assert_eq!(result.unwrap_err(), CustodyError::DeviceMismatch);
}

#[test]
fn custody_enforcement_no_registration_rejects() {
    let store = RelayCustodyStore::in_memory();
    let identity_id = "3".repeat(64);

    let result = store.enforce_custody(&identity_id, &Uuid::new_v4().to_string());
    assert_eq!(result.unwrap_err(), CustodyError::NoRegistration);
}

#[test]
fn custody_compat_mode_phase_a_is_default() {
    assert_eq!(CustodyCompatMode::default(), CustodyCompatMode::PhaseA);
}

#[test]
fn pre_ws13_store_migration_preserves_contact_data() {
    let backend = Arc::new(scmessenger_core::store::backend::MemoryStorage::new());
    let mgr = ContactManager::new(backend.clone());

    // Add a pre-WS13 contact (no device_id)
    let mut contact =
        Contact::new("peer-migration-test".to_string(), "migration-pk".to_string());
    contact.nickname = Some("MigrationBob".to_string());
    contact.added_at = 1700000000;
    mgr.add(contact).unwrap();

    // Reload (simulates store re-open after upgrade)
    let mgr2 = ContactManager::new(backend);
    let loaded = mgr2.get("peer-migration-test".to_string()).unwrap().unwrap();

    assert_eq!(loaded.nickname, Some("MigrationBob".to_string()));
    assert_eq!(loaded.added_at, 1700000000);
    assert!(
        loaded.last_known_device_id.is_none(),
        "pre-WS13 contacts must load with None device_id"
    );

    // Post-WS13: update with device_id
    mgr2.update_last_known_device_id(
        "peer-migration-test".to_string(),
        Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
    )
    .unwrap();

    let reloaded = mgr2.get("peer-migration-test".to_string()).unwrap().unwrap();
    assert_eq!(
        reloaded.last_known_device_id.as_deref(),
        Some("550e8400-e29b-41d4-a716-446655440000")
    );
    // Original fields must survive
    assert_eq!(reloaded.nickname, Some("MigrationBob".to_string()));
    assert_eq!(reloaded.added_at, 1700000000);
}

// --- Phase B compat mode tests ---

#[test]
fn phase_b_store_accepts_legacy_request_without_device_id() {
    let mut store = RelayCustodyStore::in_memory();
    store.set_compat_mode(CustodyCompatMode::PhaseB);
    let identity_id = "b".repeat(64);

    let result = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-phase-b-legacy".to_string(),
        vec![1, 2, 3],
        Some(identity_id.clone()),
        None,
    );

    assert!(
        result.is_ok(),
        "Phase B must accept legacy requests without device_id (with deprecation warning)"
    );
    let msg = result.unwrap();
    assert_eq!(msg.recipient_identity_id.as_deref(), Some(identity_id.as_str()));
    assert!(msg.intended_device_id.is_none());
}

#[test]
fn phase_b_enforces_ws13_device_mismatch_at_store_level() {
    let mut store = RelayCustodyStore::in_memory();
    store.set_compat_mode(CustodyCompatMode::PhaseB);
    let identity_id = "c".repeat(64);
    let registered_device = Uuid::new_v4().to_string();
    let wrong_device = Uuid::new_v4().to_string();

    store
        .register_identity(identity_id.clone(), registered_device, 1700000000)
        .unwrap();

    let result = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-phase-b-mismatch".to_string(),
        vec![4, 5, 6],
        Some(identity_id),
        Some(wrong_device),
    );

    assert!(
        result.is_err(),
        "Phase B must enforce device mismatch for WS13+ clients"
    );
}

#[test]
fn phase_b_accepts_ws13_active_device_match() {
    let mut store = RelayCustodyStore::in_memory();
    store.set_compat_mode(CustodyCompatMode::PhaseB);
    let identity_id = "d".repeat(64);
    let device_id = Uuid::new_v4().to_string();

    store
        .register_identity(identity_id.clone(), device_id.clone(), 1700000000)
        .unwrap();

    let result = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-phase-b-match".to_string(),
        vec![7, 8, 9],
        Some(identity_id),
        Some(device_id),
    );

    assert!(result.is_ok(), "Phase B must accept WS13+ active device match");
}

#[test]
fn compat_mode_transition_from_phase_a_to_phase_b() {
    let mut store = RelayCustodyStore::in_memory();
    assert_eq!(store.compat_mode(), CustodyCompatMode::PhaseA);

    // Accept a legacy request in Phase A
    let identity_id = "e".repeat(64);
    let result_a = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-transition-a".to_string(),
        vec![10, 11, 12],
        Some(identity_id.clone()),
        None,
    );
    assert!(result_a.is_ok());

    // Transition to Phase B
    store.set_compat_mode(CustodyCompatMode::PhaseB);
    assert_eq!(store.compat_mode(), CustodyCompatMode::PhaseB);

    // Legacy requests must still be accepted in Phase B
    let result_b = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-transition-b".to_string(),
        vec![13, 14, 15],
        Some(identity_id),
        None,
    );
    assert!(
        result_b.is_ok(),
        "Phase B must still accept legacy requests after transition"
    );
}

#[test]
fn phase_a_also_enforces_ws13_device_mismatch_at_store_level() {
    let store = RelayCustodyStore::in_memory();
    let identity_id = "f".repeat(64);
    let registered_device = Uuid::new_v4().to_string();
    let wrong_device = Uuid::new_v4().to_string();

    store
        .register_identity(identity_id.clone(), registered_device, 1700000000)
        .unwrap();

    let result = store.accept_custody(
        "source-peer".to_string(),
        "dest-peer".to_string(),
        "msg-phase-a-store-mismatch".to_string(),
        vec![16, 17, 18],
        Some(identity_id),
        Some(wrong_device),
    );

    assert!(
        result.is_err(),
        "Phase A must also enforce device mismatch for WS13+ clients at store level"
    );
}

#[test]
fn pre_ws13_identity_with_no_device_metadata_backfills_without_data_loss() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join("ws13_migration_no_data_loss")
        .to_str()
        .unwrap()
        .to_string();

    const MAX_REOPEN_ATTEMPTS: u64 = 10;
    const REOPEN_BACKOFF_BASE_MS: u64 = 25;

    // Simulate pre-WS13 store: identity keys but no device metadata
    let (original_identity_id, original_nickname) = {
        let backend = Arc::new(
            scmessenger_core::store::backend::SledStorage::new(&path).unwrap(),
        ) as Arc<dyn scmessenger_core::store::backend::StorageBackend>;
        let mut manager = IdentityManager::with_backend(backend.clone()).unwrap();
        manager.initialize().unwrap();
        manager.set_nickname("PreWS13User".to_string()).unwrap();
        let id = manager.identity_id().unwrap();
        let nick = manager.nickname().unwrap();
        // Clear device metadata to simulate pre-WS13 (use same backend to avoid sled lock)
        let store = IdentityStore::persistent(backend.clone());
        store.clear().unwrap();
        // Re-save keys and nickname only
        if let Some(keys) = manager.keys() {
            let keys_only_store = IdentityStore::persistent(backend.clone());
            keys_only_store.save_keys(keys).unwrap();
            keys_only_store.save_nickname(&nick).unwrap();
        }
        drop(manager);
        (id, nick)
    };

    // Load on WS13 code: all data preserved, device metadata backfilled
    let backend2: Arc<dyn scmessenger_core::store::backend::StorageBackend> =
        (0..MAX_REOPEN_ATTEMPTS)
            .find_map(|attempt| {
                match scmessenger_core::store::backend::SledStorage::new(&path) {
                    Ok(storage) => Some(Arc::new(storage) as Arc<dyn scmessenger_core::store::backend::StorageBackend>),
                    Err(_) if attempt + 1 < MAX_REOPEN_ATTEMPTS => {
                        std::thread::sleep(std::time::Duration::from_millis(
                            REOPEN_BACKOFF_BASE_MS * (attempt + 1),
                        ));
                        None
                    }
                    Err(err) => panic!("failed to reopen sled store: {err}"),
                }
            })
            .unwrap();
    let manager2 = IdentityManager::with_backend(backend2).unwrap();

    assert_eq!(
        manager2.identity_id(),
        Some(original_identity_id),
        "identity_id must survive migration"
    );
    assert_eq!(
        manager2.nickname(),
        Some(original_nickname),
        "nickname must survive migration"
    );
    assert!(
        manager2.device_id().is_some(),
        "device_id must be backfilled"
    );
    assert!(
        manager2.seniority_timestamp().is_some(),
        "seniority_timestamp must be backfilled"
    );
}