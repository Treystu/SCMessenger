use scmessenger_core::contacts_bridge::ContactManager;

#[test]
fn test_contact_persistence_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    // First instance: add a contact
    {
        let manager = ContactManager::new(path.to_string()).unwrap();
        let contact = scmessenger_core::contacts_bridge::Contact::new(
            "test-peer-001".to_string(),
            "a".repeat(64),
        )
        .with_nickname("Alice".to_string());
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
