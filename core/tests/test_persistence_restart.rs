use scmessenger_core::contacts_bridge::ContactManager;

#[test]
fn test_contact_persistence_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // First instance: add a contact
    {
        let manager = ContactManager::new(path.to_string()).unwrap();
        let contact = scmessenger_core::contacts_bridge::Contact {
            peer_id: "test-peer-001".to_string(),
            nickname: Some("Alice".to_string()),
            public_key: "a".repeat(64),
            added_at: 1000,
            last_seen: None,
            notes: None,
        };
        manager.add(contact).unwrap();
        assert_eq!(manager.count(), 1);
    }
    // manager dropped here — sled should flush

    // Second instance: verify data survived
    {
        let manager2 = ContactManager::new(path.to_string()).unwrap();
        assert_eq!(manager2.count(), 1);
        let retrieved = manager2.get("test-peer-001".to_string()).unwrap().unwrap();
        assert_eq!(retrieved.nickname, Some("Alice".to_string()));
    }
}
