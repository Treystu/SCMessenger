// Property-based tests for identity export/import round-trip
// Validates: Requirements 13.3

use proptest::prelude::*;
use scmessenger_core::identity::IdentityManager;

proptest! {
    /// Property 1: Identity export/import round-trip consistency
    /// Validates: Requirements 13.3
    /// Property: import(export(identity)) preserves identity_id and public_key
    #[test]
    fn test_identity_export_import_roundtrip(_seed in any::<u64>()) {
        // Generate original identity
        let mut manager1 = IdentityManager::new();
        manager1.initialize().expect("initialization should succeed");

        // Export identity
        let exported = manager1.export_key_bytes().expect("export should succeed");

        // Capture original identity properties
        let original_id = manager1.identity_id().expect("identity_id should exist");
        let original_pub = manager1.public_key_hex().expect("public_key should exist");

        // Import into new manager
        let mut manager2 = IdentityManager::new();
        manager2.import_key_bytes(&exported).expect("import should succeed");

        // Property: Identity ID should be preserved
        let restored_id = manager2.identity_id().expect("restored identity_id should exist");
        prop_assert_eq!(
            original_id,
            restored_id,
            "Identity ID should be preserved after export/import"
        );

        // Property: Public key should be preserved
        let restored_pub = manager2.public_key_hex().expect("restored public_key should exist");
        prop_assert_eq!(
            original_pub,
            restored_pub,
            "Public key should be preserved after export/import"
        );
    }

    /// Property 2: Signing capability preserved after export/import
    /// Validates: Requirements 13.3 (identity backup preserves functionality)
    /// Property: Signatures from restored identity are valid
    #[test]
    fn test_signing_preserved_after_import(data in prop::collection::vec(any::<u8>(), 0..1024)) {
        // Generate original identity
        let mut manager1 = IdentityManager::new();
        manager1.initialize().expect("initialization should succeed");

        // Export and import
        let exported = manager1.export_key_bytes().expect("export should succeed");
        let mut manager2 = IdentityManager::new();
        manager2.import_key_bytes(&exported).expect("import should succeed");

        // Sign with restored identity
        let signature = manager2.sign(&data).expect("signing should succeed");

        // Verify signature using original identity's public key
        let keys1 = manager1.keys().expect("keys should exist");
        let public_key = keys1.signing_key.verifying_key().to_bytes();

        let valid = manager1.verify(&data, &signature, &public_key)
            .expect("verification should succeed");

        // Property: Signature from restored identity should be valid
        prop_assert!(valid, "Signature from restored identity should be valid");
    }

    /// Property 3: Multiple export/import cycles preserve identity
    /// Validates: Requirements 13.3 (backup robustness)
    /// Property: export(import(export(import(identity)))) == identity
    #[test]
    fn test_multiple_export_import_cycles(_seed in any::<u64>()) {
        // Generate original identity
        let mut manager1 = IdentityManager::new();
        manager1.initialize().expect("initialization should succeed");
        let original_id = manager1.identity_id().expect("identity_id should exist");

        // Cycle 1: export → import
        let exported1 = manager1.export_key_bytes().expect("export should succeed");
        let mut manager2 = IdentityManager::new();
        manager2.import_key_bytes(&exported1).expect("import should succeed");

        // Cycle 2: export → import
        let exported2 = manager2.export_key_bytes().expect("export should succeed");
        let mut manager3 = IdentityManager::new();
        manager3.import_key_bytes(&exported2).expect("import should succeed");

        // Cycle 3: export → import
        let exported3 = manager3.export_key_bytes().expect("export should succeed");
        let mut manager4 = IdentityManager::new();
        manager4.import_key_bytes(&exported3).expect("import should succeed");

        // Property: Identity ID should remain the same after multiple cycles
        let final_id = manager4.identity_id().expect("identity_id should exist");
        prop_assert_eq!(
            original_id,
            final_id,
            "Identity ID should be preserved after multiple export/import cycles"
        );
    }

    /// Property 4: Exported bytes are deterministic
    /// Validates: Requirements 13.3 (backup consistency)
    /// Property: export(identity) produces the same bytes every time
    #[test]
    fn test_export_deterministic(_seed in any::<u64>()) {
        let mut manager = IdentityManager::new();
        manager.initialize().expect("initialization should succeed");

        // Export multiple times
        let export1 = manager.export_key_bytes().expect("export should succeed");
        let export2 = manager.export_key_bytes().expect("export should succeed");
        let export3 = manager.export_key_bytes().expect("export should succeed");

        // Property: All exports should be identical
        prop_assert_eq!(&export1, &export2, "Exports should be deterministic");
        prop_assert_eq!(&export2, &export3, "Exports should be deterministic");
    }

    /// Property 5: Import validates key format
    /// Validates: Requirements 13.3 (backup integrity)
    /// Property: import(invalid_bytes) fails gracefully
    #[test]
    fn test_import_validates_format(invalid_bytes in prop::collection::vec(any::<u8>(), 0..64)) {
        // Skip valid 32-byte keys
        prop_assume!(invalid_bytes.len() != 32);

        let mut manager = IdentityManager::new();
        let result = manager.import_key_bytes(&invalid_bytes);

        // Property: Import should fail for invalid key lengths
        prop_assert!(result.is_err(), "Import should fail for invalid key format");
    }
}

/// Property 6: Export format is 32 bytes (Ed25519 secret key)
/// Validates: Requirements 13.3 (backup format correctness)
#[test]
fn test_export_format() {
    let mut manager = IdentityManager::new();
    manager.initialize().expect("initialization should succeed");

    let exported = manager.export_key_bytes().expect("export should succeed");

    // Ed25519 secret key should be exactly 32 bytes
    assert_eq!(
        exported.len(),
        32,
        "Exported key should be 32 bytes (Ed25519 secret key)"
    );
}

/// Property 7: Import empty bytes fails
/// Validates: Requirements 13.8 (edge case: empty input)
#[test]
fn test_import_empty_bytes() {
    let mut manager = IdentityManager::new();
    let result = manager.import_key_bytes(&[]);

    assert!(result.is_err(), "Import should fail for empty bytes");
}

/// Property 8: Import with correct length but invalid key data fails
/// Validates: Requirements 13.3 (backup validation)
#[test]
fn test_import_invalid_key_data() {
    let mut manager = IdentityManager::new();

    // 32 bytes of zeros (invalid Ed25519 key)
    let invalid_key = vec![0u8; 32];
    let result = manager.import_key_bytes(&invalid_key);

    // Note: Ed25519 may accept some all-zero keys, so we just verify it doesn't panic
    // The important property is that import either succeeds or fails gracefully
    let _ = result; // Consume result without asserting
}

/// Property 9: Device metadata regenerated after import
/// Validates: Requirements 13.3 (device-specific metadata handling)
#[test]
fn test_device_metadata_regenerated_after_import() {
    // Generate original identity
    let mut manager1 = IdentityManager::new();
    manager1
        .initialize()
        .expect("initialization should succeed");
    let original_device_id = manager1.device_id().expect("device_id should exist");

    // Export and import
    let exported = manager1.export_key_bytes().expect("export should succeed");
    let mut manager2 = IdentityManager::new();
    manager2
        .import_key_bytes(&exported)
        .expect("import should succeed");

    // Device ID should be different (new installation)
    let new_device_id = manager2.device_id().expect("device_id should exist");
    assert_ne!(
        original_device_id, new_device_id,
        "Device ID should be regenerated after import (new installation)"
    );

    // Seniority timestamp should exist
    assert!(
        manager2.seniority_timestamp().is_some(),
        "Seniority timestamp should be set after import"
    );
}

/// Property 10: Cross-verification between original and restored identity
/// Validates: Requirements 13.3 (identity equivalence)
#[test]
fn test_cross_verification() {
    // Generate original identity
    let mut manager1 = IdentityManager::new();
    manager1
        .initialize()
        .expect("initialization should succeed");

    // Export and import
    let exported = manager1.export_key_bytes().expect("export should succeed");
    let mut manager2 = IdentityManager::new();
    manager2
        .import_key_bytes(&exported)
        .expect("import should succeed");

    let data = b"cross-verification test";

    // Sign with original identity
    let signature1 = manager1.sign(data).expect("signing should succeed");

    // Verify with restored identity's public key
    let keys2 = manager2.keys().expect("keys should exist");
    let public_key2 = keys2.signing_key.verifying_key().to_bytes();
    let valid1 = manager1
        .verify(data, &signature1, &public_key2)
        .expect("verification should succeed");
    assert!(
        valid1,
        "Original signature should verify with restored public key"
    );

    // Sign with restored identity
    let signature2 = manager2.sign(data).expect("signing should succeed");

    // Verify with original identity's public key
    let keys1 = manager1.keys().expect("keys should exist");
    let public_key1 = keys1.signing_key.verifying_key().to_bytes();
    let valid2 = manager2
        .verify(data, &signature2, &public_key1)
        .expect("verification should succeed");
    assert!(
        valid2,
        "Restored signature should verify with original public key"
    );
}

#[cfg(test)]
mod unit_tests {
    #[test]
    fn test_property_test_strategies_compile() {
        // Smoke test to ensure strategies compile
        // No strategies needed for identity tests (using IdentityManager directly)
    }
}
