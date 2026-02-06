// sc-mobile — Native mobile bindings for iOS and Android
// This crate exports the Iron Core API via UniFFI

pub use scmessenger_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_bindings_lifecycle() {
        let core = IronCore::new();
        assert!(!core.is_running());

        core.start().expect("Failed to start");
        assert!(core.is_running());

        core.stop();
        assert!(!core.is_running());
    }

    #[test]
    fn test_mobile_identity() {
        let core = IronCore::new();

        let info_before = core.get_identity_info();
        assert!(!info_before.initialized);

        core.initialize_identity().expect("Failed to initialize identity");

        let info_after = core.get_identity_info();
        assert!(info_after.initialized);
        assert!(info_after.identity_id.is_some());
        assert!(info_after.public_key_hex.is_some());
    }

    #[test]
    fn test_mobile_messaging() {
        let alice = IronCore::new();
        let bob = IronCore::new();

        alice.initialize_identity().unwrap();
        bob.initialize_identity().unwrap();

        let bob_pubkey = bob.get_identity_info().public_key_hex.unwrap();

        let envelope = alice
            .prepare_message(bob_pubkey, "Hello from mobile!".to_string())
            .unwrap();

        let msg = bob.receive_message(envelope).unwrap();
        assert_eq!(msg.text_content().unwrap(), "Hello from mobile!");
    }
}
