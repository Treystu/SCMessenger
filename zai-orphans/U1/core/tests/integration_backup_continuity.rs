use scmessenger_core::{IronCore, MessageType};

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
        .expect("node must be initialized before calling pubkey()")
}

#[test]
fn test_backup_continuity_and_corruption() {
    // 1. Setup Alice and Bob
    let alice = make_node();
    let bob = make_node();

    let plaintext = "Hello restored Alice, this is Bob.";

    // 2. Alice exports identity backup
    let passphrase = "my-secure-backup-passphrase";
    let backup_hex = alice
        .export_identity_backup(passphrase.to_string())
        .expect("export_identity_backup should succeed");

    assert!(!backup_hex.is_empty());

    // 3. Import Alice's backup into a fresh node (Alice2)
    let alice2 = IronCore::new();
    alice2.grant_consent();
    alice2
        .import_identity_backup(backup_hex.clone(), passphrase.to_string())
        .expect("import_identity_backup should succeed");

    // Verify public key is restored correctly
    assert_eq!(pubkey(&alice), pubkey(&alice2));

    // 4. Bob prepares a message for Alice2 (using her restored public key)
    let prepared = bob
        .prepare_message(
            pubkey(&alice2),
            plaintext.to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message should succeed");

    // Alice2 decrypts the message
    let received = alice2
        .receive_message(prepared.envelope_data)
        .expect("receive_message should succeed on restored identity");

    assert_eq!(
        received.text_content().expect("message must carry text"),
        plaintext
    );

    // 5. Test corruption validation
    // Tamper with the backup hex string (flip a character)
    let mut tampered_backup = backup_hex.clone();
    let last_char = tampered_backup.pop().unwrap();
    let tampered_char = if last_char == '0' { '1' } else { '0' };
    tampered_backup.push(tampered_char);

    let alice3 = IronCore::new();
    alice3.grant_consent();
    let import_result = alice3.import_identity_backup(tampered_backup, passphrase.to_string());
    assert!(
        import_result.is_err(),
        "Importing a tampered backup must fail"
    );

    // Test wrong passphrase
    let alice4 = IronCore::new();
    alice4.grant_consent();
    let wrong_passphrase_result =
        alice4.import_identity_backup(backup_hex, "wrong-passphrase".to_string());
    assert!(
        wrong_passphrase_result.is_err(),
        "Importing with a wrong passphrase must fail"
    );
}

#[test]
fn test_backup_with_salt_continuity() {
    let alice = make_node();
    let bob = make_node();

    let passphrase = "secure-salt-passphrase";
    let salt = vec![42u8; 16];

    let backup_hex = alice
        .export_identity_backup_with_salt(passphrase.to_string(), Some(salt))
        .expect("export_identity_backup_with_salt should succeed");

    assert!(!backup_hex.is_empty());

    let alice2 = IronCore::new();
    alice2.grant_consent();
    alice2
        .import_identity_backup(backup_hex, passphrase.to_string())
        .expect("import_identity_backup of salted backup should succeed");

    assert_eq!(pubkey(&alice), pubkey(&alice2));

    // Decrypt verification
    let plaintext = "Verifying message decrypt with custom salt backup";
    let prepared = bob
        .prepare_message(
            pubkey(&alice2),
            plaintext.to_string(),
            MessageType::Text,
            None,
        )
        .expect("prepare_message should succeed");

    let received = alice2
        .receive_message(prepared.envelope_data)
        .expect("receive_message should succeed");

    assert_eq!(
        received.text_content().expect("message must carry text"),
        plaintext
    );
}
