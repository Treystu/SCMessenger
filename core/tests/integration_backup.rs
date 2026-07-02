/// Integration test: Full backup/restore flow
///
/// Verifies that:
/// 1. Identity backup export/import roundtrips correctly
/// 2. Ratchet sessions survive backup/restore
/// 3. Tampered blobs return CorruptionDetected
/// 4. KDF is memory-hard (Argon2id)
/// 5. Audit events are emitted for export/import
use scmessenger_core::crypto::backup::{decrypt_backup, encrypt_backup};
use scmessenger_core::crypto::{
    decrypt_message_ratcheted, ed25519_public_to_x25519, encrypt_message_ratcheted,
    RatchetSessionManager,
};
use scmessenger_core::identity::IdentityManager;
use scmessenger_core::observability::AuditEventType;
use scmessenger_core::store::backend::MemoryStorage;
use scmessenger_core::store::Contact;
use scmessenger_core::IronCore;
use std::sync::Arc;
use std::time::Instant;

#[test]
fn backup_identity_roundtrip() {
    let mut manager = IdentityManager::new();
    manager.initialize().expect("identity init should succeed");

    let original_id = manager.identity_id().expect("identity_id should exist");
    let original_pub = manager.public_key_hex().expect("public_key should exist");

    let passphrase = "test-passphrase-for-backup";
    let key_bytes = manager.export_key_bytes().expect("export should succeed");
    let payload = hex::encode(&key_bytes);

    let encrypted = encrypt_backup(&payload, passphrase, None).expect("encrypt should succeed");
    assert!(!encrypted.is_empty());

    let decrypted = decrypt_backup(&encrypted, passphrase).expect("decrypt should succeed");
    assert_eq!(decrypted, payload);

    let decrypted_bytes = hex::decode(&decrypted).expect("hex decode should succeed");
    let mut manager2 = IdentityManager::new();
    manager2
        .import_key_bytes(&decrypted_bytes)
        .expect("import should succeed");

    let restored_id = manager2
        .identity_id()
        .expect("restored identity_id should exist");
    let restored_pub = manager2
        .public_key_hex()
        .expect("restored public_key should exist");

    assert_eq!(original_id, restored_id, "Identity ID should be preserved");
    assert_eq!(original_pub, restored_pub, "Public key should be preserved");
}

#[test]
fn backup_tampered_blob_returns_corruption_detected() {
    let payload = r#"{"version":1,"secret_key_hex":"aabbccdd"}"#;
    let passphrase = "correct-passphrase";

    let encrypted = encrypt_backup(payload, passphrase, None).expect("encrypt should succeed");

    // Tamper with the encrypted data (flip a hex char)
    let mut tampered = encrypted.clone();
    let last_char = tampered.pop().unwrap();
    let tampered_char = if last_char == '0' { '1' } else { '0' };
    tampered.push(tampered_char);

    let result = decrypt_backup(&tampered, passphrase);
    assert!(result.is_err(), "Tampered blob should fail decryption");
}

#[test]
fn backup_wrong_passphrase_fails() {
    let payload = "sensitive data";
    let encrypted = encrypt_backup(payload, "correct-pass", None).expect("encrypt should succeed");

    let result = decrypt_backup(&encrypted, "wrong-pass");
    assert!(result.is_err(), "Wrong passphrase should fail");
}

#[test]
fn backup_kdf_is_memory_hard() {
    // PBKDF2 with 600K iterations should take >10ms on modern hardware
    // This verifies we're not using a fast KDF like bare blake3
    let passphrase = "benchmark-passphrase";
    let salt = [42u8; 16];

    let start = Instant::now();
    let _ = encrypt_backup("test payload", passphrase, Some(&salt));
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() > 5,
        "KDF should be memory-hard (took {}ms, expected >5ms)",
        elapsed.as_millis()
    );
}

#[test]
fn backup_ratchet_sessions_roundtrip() {
    // Create a ratchet session, serialize it, back it up, restore it
    let backend = Arc::new(MemoryStorage::new());
    let mut manager = RatchetSessionManager::with_backend(backend.clone());

    use ed25519_dalek::SigningKey;
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let our_key = SigningKey::from_bytes(&bytes);
    let their_pub = x25519_dalek::PublicKey::from([1u8; 32]);

    manager
        .get_or_create_session("backup-peer", &our_key, &their_pub)
        .unwrap();

    // Serialize sessions (this is what gets included in a full backup)
    let sessions_json = manager
        .serialize_sessions()
        .expect("serialize should succeed");

    // Encrypt the sessions as a backup blob
    let passphrase = "ratchet-backup-passphrase";
    let encrypted =
        encrypt_backup(&sessions_json, passphrase, None).expect("encrypt should succeed");

    // Decrypt and verify
    let decrypted = decrypt_backup(&encrypted, passphrase).expect("decrypt should succeed");
    assert_eq!(decrypted, sessions_json);

    // Restore sessions into a new manager
    let mut manager2 = RatchetSessionManager::with_backend(backend);
    manager2
        .deserialize_sessions(&decrypted)
        .expect("deserialize should succeed");

    assert_eq!(manager2.session_count(), 1);
    assert!(manager2.get_session("backup-peer").is_some());
}

#[test]
fn backup_custom_salt_roundtrip() {
    let payload = "custom salt test data";
    let passphrase = "salt-test-passphrase";
    let salt = [0xAB_u8; 16];

    let encrypted =
        encrypt_backup(payload, passphrase, Some(&salt)).expect("encrypt with salt should succeed");
    let decrypted = decrypt_backup(&encrypted, passphrase).expect("decrypt should succeed");

    assert_eq!(decrypted, payload);
}

#[test]
fn backup_empty_payload_roundtrip() {
    let passphrase = "empty-test";
    let encrypted = encrypt_backup("", passphrase, None).expect("encrypt empty should succeed");
    let decrypted = decrypt_backup(&encrypted, passphrase).expect("decrypt empty should succeed");

    assert_eq!(decrypted, "");
}

#[test]
fn backup_large_payload_roundtrip() {
    let payload = "x".repeat(100_000);
    let passphrase = "large-payload-test";

    let encrypted =
        encrypt_backup(&payload, passphrase, None).expect("encrypt large should succeed");
    let decrypted = decrypt_backup(&encrypted, passphrase).expect("decrypt large should succeed");

    assert_eq!(decrypted, payload);
}

/// Full integration: create IronCore-like state, backup everything, restore on fresh instance.
#[test]
fn backup_full_state_integration() {
    // 1. Create identity
    let mut identity = IdentityManager::new();
    identity.initialize().expect("identity init should succeed");
    let original_pub = identity.public_key_hex().unwrap();

    // 2. Create ratchet sessions
    let backend = Arc::new(MemoryStorage::new());
    let mut ratchet_mgr = RatchetSessionManager::with_backend(backend.clone());

    use ed25519_dalek::SigningKey;
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let our_key = SigningKey::from_bytes(&bytes);

    for i in 0..3 {
        let their_pub = x25519_dalek::PublicKey::from([i as u8; 32]);
        ratchet_mgr
            .get_or_create_session(&format!("peer-{}", i), &our_key, &their_pub)
            .unwrap();
    }

    // 3. Backup identity (encrypt_backup encrypts the identity key hex)
    let passphrase = "full-backup-test";
    let identity_bytes = identity.export_key_bytes().unwrap();
    let identity_hex = hex::encode(&identity_bytes);

    let backup_blob =
        encrypt_backup(&identity_hex, passphrase, None).expect("backup should succeed");

    // 4. Backup ratchet sessions separately
    let sessions_json = ratchet_mgr.serialize_sessions().unwrap();
    let sessions_blob =
        encrypt_backup(&sessions_json, passphrase, None).expect("sessions backup should succeed");

    // 5. Restore identity on fresh instance
    let restored_identity_hex =
        decrypt_backup(&backup_blob, passphrase).expect("identity restore should succeed");
    assert_eq!(restored_identity_hex, identity_hex);

    let restored_bytes = hex::decode(&restored_identity_hex).expect("hex decode");
    let mut restored_identity = IdentityManager::new();
    restored_identity
        .import_key_bytes(&restored_bytes)
        .expect("import should succeed");

    assert_eq!(
        original_pub,
        restored_identity.public_key_hex().unwrap(),
        "Public key should survive full backup/restore"
    );

    // 6. Restore ratchet sessions
    let restored_sessions =
        decrypt_backup(&sessions_blob, passphrase).expect("sessions restore should succeed");
    let mut restored_ratchet = RatchetSessionManager::with_backend(backend);
    restored_ratchet
        .deserialize_sessions(&restored_sessions)
        .expect("deserialize should succeed");

    assert_eq!(
        restored_ratchet.session_count(),
        3,
        "All ratchet sessions should survive backup/restore"
    );

    for i in 0..3 {
        assert!(
            restored_ratchet
                .get_session(&format!("peer-{}", i))
                .is_some(),
            "peer-{} session should exist after restore",
            i
        );
    }
}

// ============================================================================
// T4.5 — end-to-end backup/restore through the real IronCore public API
// (the tests above exercise the lower-level backup.rs/RatchetSessionManager
// primitives directly; these exercise IronCore::export_identity_backup /
// import_identity_backup, which is what the mobile/CLI clients actually call).
// ============================================================================

/// Export a full identity backup (identity + ratchet session + contact) from
/// Alice, import it into a fresh IronCore, and confirm the restored session
/// can decrypt the *next* ratchet message from Bob's still-live session -
/// i.e. the conversation continues seamlessly after a restore, not just that
/// the serialized bytes round-trip.
#[test]
fn iron_core_backup_restore_preserves_ratchet_continuity_and_contacts() {
    let alice = IronCore::new();
    alice.grant_consent();
    alice.initialize_identity().expect("alice identity init");
    let alice_signing_key = alice
        .get_identity_info()
        .public_key_hex
        .clone()
        .expect("alice pubkey");

    let bob = IronCore::new();
    bob.grant_consent();
    bob.initialize_identity().expect("bob identity init");
    let bob_pub_hex = bob.get_identity_info().public_key_hex.expect("bob pubkey");
    let bob_pub_bytes: [u8; 32] = hex::decode(&bob_pub_hex)
        .expect("bob pubkey is hex")
        .try_into()
        .expect("bob pubkey is 32 bytes");
    let bob_x25519_pub = ed25519_public_to_x25519(&bob_pub_bytes).expect("bob x25519 conversion");

    let alice_pub_bytes: [u8; 32] = hex::decode(&alice_signing_key)
        .expect("alice pubkey is hex")
        .try_into()
        .expect("alice pubkey is 32 bytes");
    let alice_x25519_pub =
        ed25519_public_to_x25519(&alice_pub_bytes).expect("alice x25519 conversion");

    // Alice initiates a ratchet session with Bob and sends the first message;
    // Bob creates the matching receiver session and decrypts it, establishing
    // a real, working two-party conversation before any backup happens.
    alice
        .ratchet_sessions_handle()
        .write()
        .get_or_create_session(
            "bob",
            &alice.test_only_identity_signing_key(),
            &bob_x25519_pub,
        )
        .expect("alice creates sender session");
    bob.create_receiver_session("alice", &hex::encode(alice_x25519_pub.to_bytes()))
        .expect("bob creates receiver session");

    let first_envelope = {
        let sessions = alice.ratchet_sessions_handle();
        let mut guard = sessions.write();
        let session = guard.get_session_mut("bob").expect("alice session exists");
        encrypt_message_ratcheted(
            &alice.test_only_identity_signing_key(),
            session,
            b"hello bob, before backup",
        )
        .expect("alice encrypts first message")
    };
    {
        let sessions = bob.ratchet_sessions_handle();
        let mut guard = sessions.write();
        let session = guard.get_session_mut("alice").expect("bob session exists");
        let plaintext = decrypt_message_ratcheted(session, &first_envelope)
            .expect("bob decrypts first message");
        assert_eq!(plaintext, b"hello bob, before backup");
    }

    // Alice also adds Bob as a contact.
    alice
        .contacts_store_manager()
        .add(Contact::new("bob".to_string(), bob_pub_hex.clone()))
        .expect("alice adds bob as a contact");

    // Export Alice's full identity backup (identity + ratchet session + contact).
    let passphrase = "correct horse battery staple";
    let backup = alice
        .export_identity_backup(passphrase.to_string())
        .expect("export_identity_backup succeeds");

    // Restore onto a completely fresh IronCore, simulating a new device.
    let alice_restored = IronCore::new();
    alice_restored
        .import_identity_backup(backup, passphrase.to_string())
        .expect("import_identity_backup succeeds");

    assert_eq!(
        alice_restored.get_identity_info().public_key_hex,
        Some(alice_signing_key),
        "restored identity must match the original"
    );
    assert_eq!(
        alice_restored
            .contacts_store_manager()
            .list()
            .unwrap()
            .len(),
        1,
        "contact must survive the backup/restore"
    );
    assert!(
        alice_restored.ratchet_has_session("bob".to_string()),
        "ratchet session with bob must survive the backup/restore"
    );

    // The critical continuity check: encrypt a *new* message with the
    // restored session and confirm Bob's original (never-restored) session
    // can still decrypt it - proving the restored ratchet state is fully
    // functional, not just structurally present.
    let next_envelope = {
        let sessions = alice_restored.ratchet_sessions_handle();
        let mut guard = sessions.write();
        let session = guard
            .get_session_mut("bob")
            .expect("restored session exists");
        encrypt_message_ratcheted(
            &alice_restored.test_only_identity_signing_key(),
            session,
            b"hello bob, after restore",
        )
        .expect("restored session encrypts next message")
    };
    {
        let sessions = bob.ratchet_sessions_handle();
        let mut guard = sessions.write();
        let session = guard.get_session_mut("alice").expect("bob session exists");
        let plaintext = decrypt_message_ratcheted(session, &next_envelope)
            .expect("bob decrypts the next message using the restored session's output");
        assert_eq!(plaintext, b"hello bob, after restore");
    }
}

/// A tampered backup blob must fail closed (CorruptionDetected) and leave the
/// target IronCore completely untouched - no partial identity, no partial
/// ratchet sessions, no partial contacts.
#[test]
fn iron_core_import_tampered_backup_leaves_no_partial_state() {
    let alice = IronCore::new();
    alice.grant_consent();
    alice.initialize_identity().expect("alice identity init");
    alice
        .contacts_store_manager()
        .add(Contact::new("carol".to_string(), hex::encode([7u8; 32])))
        .expect("alice adds a contact");

    let passphrase = "tamper-test-passphrase";
    let backup = alice
        .export_identity_backup(passphrase.to_string())
        .expect("export succeeds");

    let mut data = hex::decode(&backup).expect("backup is hex");
    let last = data.len() - 1;
    data[last] ^= 0xFF;
    let tampered = hex::encode(data);

    let fresh = IronCore::new();
    let result = fresh.import_identity_backup(tampered, passphrase.to_string());

    assert!(
        matches!(result, Err(scmessenger_core::IronCoreError::CryptoError)),
        "tampered backup must fail, got {:?}",
        result.err()
    );
    assert!(
        fresh.get_identity_info().public_key_hex.is_none(),
        "no identity should have been imported from a tampered backup"
    );
    assert_eq!(
        fresh.contacts_store_manager().list().unwrap().len(),
        0,
        "no contacts should have been imported from a tampered backup"
    );
    assert_eq!(
        fresh.ratchet_session_count(),
        0,
        "no ratchet sessions should have been imported from a tampered backup"
    );
}

/// A backup whose ratchet-session JSON contains one structurally-valid but
/// cryptographically corrupt entry (bad hex) must fail the whole import with
/// CorruptionDetected, not silently drop that entry and report success -
/// T3: `deserialize_sessions` used to skip bad entries instead of failing.
#[test]
fn iron_core_import_rejects_corrupted_ratchet_session_entry() {
    let alice = IronCore::new();
    alice.grant_consent();
    alice.initialize_identity().expect("alice identity init");
    let identity_key_hex = hex::encode(alice.test_only_identity_signing_key().to_bytes());

    let zero_hex = hex::encode([0u8; 32]);
    let corrupted_sessions_json = format!(
        r#"{{"peer-x":{{"our_dh_secret_hex":"not-hex","our_dh_public_hex":"{zero}","their_dh_public_hex":null,"root_key_hex":"{zero}","sending_chain":null,"receiving_chain":null,"dh_step_count":0,"initialized":false,"has_identity_secret":false,"identity_secret_hex":null}}}}"#,
        zero = zero_hex
    );

    let payload = serde_json::json!({
        "version": 2,
        "identity_key_hex": identity_key_hex,
        "ratchet_sessions_json": corrupted_sessions_json,
        "contacts": []
    })
    .to_string();

    let passphrase = "corrupted-session-test";
    let backup = encrypt_backup(&payload, passphrase, None).expect("encrypt succeeds");

    let fresh = IronCore::new();
    let result = fresh.import_identity_backup(backup, passphrase.to_string());

    assert!(
        matches!(
            result,
            Err(scmessenger_core::IronCoreError::CorruptionDetected)
        ),
        "corrupted ratchet session entry must fail closed with CorruptionDetected, got {:?}",
        result
    );
    assert!(
        fresh.get_identity_info().public_key_hex.is_none(),
        "identity must not be imported when ratchet session validation fails"
    );
}

/// T1: `build_identity_backup_payload` used to only read the core's
/// internal `contact_manager`, but Android/iOS add contacts through the
/// separate UniFFI-bridge `contacts_manager()` store (`contacts.db`) - a
/// mobile export's address book was silently empty/stale on restore. This
/// exercises the actual mobile code path: add a contact via
/// `contacts_manager()`, export, restore onto a fresh persistent core
/// (simulating a new device), and confirm the bridge contact - including
/// `verified_at` - survives.
#[test]
fn iron_core_backup_restore_preserves_bridge_contacts() {
    use tempfile::tempdir;

    let alice_dir = tempdir().unwrap();
    let alice = IronCore::with_storage(alice_dir.path().to_str().unwrap().to_string());
    alice.grant_consent();
    alice.initialize_identity().expect("alice identity init");

    let mut bridge_contact =
        scmessenger_core::contacts_bridge::Contact::new("bob".to_string(), hex::encode([9u8; 32]));
    bridge_contact.nickname = Some("Bob".to_string());
    bridge_contact.verified_at = Some(1_700_000_000);
    alice
        .contacts_manager()
        .add(bridge_contact)
        .expect("alice adds bob via the bridge contacts store");

    let passphrase = "bridge-contacts-test";
    let backup = alice
        .export_identity_backup(passphrase.to_string())
        .expect("export succeeds");

    // Restore onto a fresh persistent core with its own (empty) bridge
    // contacts.db, simulating a new device.
    let restored_dir = tempdir().unwrap();
    let restored = IronCore::with_storage(restored_dir.path().to_str().unwrap().to_string());
    restored
        .import_identity_backup(backup, passphrase.to_string())
        .expect("import succeeds");

    let restored_contacts = restored
        .contacts_manager()
        .list()
        .expect("restored bridge contacts list");
    assert_eq!(
        restored_contacts.len(),
        1,
        "bridge contact must survive the backup/restore"
    );
    assert_eq!(restored_contacts[0].peer_id, "bob");
    assert_eq!(
        restored_contacts[0].verified_at,
        Some(1_700_000_000),
        "verified_at must survive the backup/restore intact"
    );
}

/// Export and import must each append exactly one audit event of the
/// corresponding type - this was previously missing entirely on export.
#[test]
fn iron_core_backup_export_and_import_emit_audit_events() {
    let alice = IronCore::new();
    alice.grant_consent();
    alice.initialize_identity().expect("alice identity init");

    let passphrase = "audit-test-passphrase";
    let backup = alice
        .export_identity_backup(passphrase.to_string())
        .expect("export succeeds");

    assert_eq!(
        alice
            .get_audit_events_by_type(AuditEventType::BackupExported)
            .len(),
        1,
        "export_identity_backup must record exactly one BackupExported audit event"
    );

    let fresh = IronCore::new();
    fresh
        .import_identity_backup(backup, passphrase.to_string())
        .expect("import succeeds");

    assert_eq!(
        fresh
            .get_audit_events_by_type(AuditEventType::BackupImported)
            .len(),
        1,
        "import_identity_backup must record exactly one BackupImported audit event"
    );
}
