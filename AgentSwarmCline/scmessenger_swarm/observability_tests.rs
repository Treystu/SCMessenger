use super::*;
use serde_json;
use std::error::Error;

#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures for valid and invalid payloads
    fn valid_payload() -> RelayTracePayload {
        RelayTracePayload {
            message_id: "msg_123".to_string(),
            relay_node_hash: "a".repeat(64),
            latency_ms: 100,
        }
    }

    fn valid_payload_with_zero_latency() -> RelayTracePayload {
        RelayTracePayload {
            message_id: "msg_zero".to_string(),
            relay_node_hash: "1".repeat(64),
            latency_ms: 0,
        }
    }

    // ========================================================================
    // Happy Path Tests
    // ========================================================================
    #[test]
    fn test_valid_payload_passes_validation() {
        let payload = valid_payload();
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_valid_payload_with_zero_latency() {
        let payload = valid_payload_with_zero_latency();
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_display_format() {
        let payload = valid_payload();
        let display = payload.to_string();
        assert!(display.starts_with("[RelayTrace]"));
        assert!(display.contains("msg_id=msg_123"));
        assert!(display.contains("node="));
        assert!(display.contains("latency=100ms"));
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================
    #[test]
    fn test_minimum_valid_message_id() {
        let payload = RelayTracePayload {
            message_id: "a".to_string(),
            relay_node_hash: "b".repeat(64),
            latency_ms: 50,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_unicode_in_message_id() {
        let payload = RelayTracePayload {
            message_id: "hello_世界_🚀".to_string(),
            relay_node_hash: "c".repeat(64),
            latency_ms: 200,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_very_long_message_id() {
        let payload = RelayTracePayload {
            message_id: "x".repeat(1000),
            relay_node_hash: "d".repeat(64),
            latency_ms: 150,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_lowercase_hex_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_lower".to_string(),
            relay_node_hash: "abcdef0123456789".repeat(4)[..64].to_string(),
            latency_ms: 75,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_uppercase_hex_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_upper".to_string(),
            relay_node_hash: "ABCDEF0123456789".repeat(4)[..64].to_string(),
            latency_ms: 125,
        };
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_mixed_case_hex_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_mixed".to_string(),
            relay_node_hash: "aBcDeF0123456789".repeat(4)[..64].to_string(),
            latency_ms: 250,
        };
        assert!(payload.validate().is_ok());
    }

    // ========================================================================
    // Error Condition Tests
    // ========================================================================
    #[test]
    fn test_empty_message_id() {
        let payload = RelayTracePayload {
            message_id: "".to_string(),
            relay_node_hash: "a".repeat(64),
            latency_ms: 100,
        };
        match payload.validate() {
            Err(RelayTraceError::EmptyMessageId) => {},
            _ => panic!("Expected EmptyMessageId error"),
        }
    }

    #[test]
    fn test_empty_relay_node_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_empty_hash".to_string(),
            relay_node_hash: "".to_string(),
            latency_ms: 100,
        };
        match payload.validate() {
            Err(RelayTraceError::InvalidNodeHash) => {},
            _ => panic!("Expected InvalidNodeHash error"),
        }
    }

    #[test]
    fn test_short_relay_node_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_short".to_string(),
            relay_node_hash: "a".repeat(63),
            latency_ms: 100,
        };
        match payload.validate() {
            Err(RelayTraceError::InvalidNodeHash) => {},
            _ => panic!("Expected InvalidNodeHash error"),
        }
    }

    #[test]
    fn test_long_relay_node_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_long".to_string(),
            relay_node_hash: "a".repeat(65),
            latency_ms: 100,
        };
        match payload.validate() {
            Err(RelayTraceError::InvalidNodeHash) => {},
            _ => panic!("Expected InvalidNodeHash error"),
        }
    }

    #[test]
    fn test_non_hex_characters_in_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_non_hex".to_string(),
            relay_node_hash: "g".repeat(64), // 'g' is not a valid hex character
            latency_ms: 100,
        };
        match payload.validate() {
            Err(RelayTraceError::InvalidNodeHash) => {},
            _ => panic!("Expected InvalidNodeHash error"),
        }
    }

    #[test]
    fn test_special_characters_in_hash() {
        let payload = RelayTracePayload {
            message_id: "msg_special".to_string(),
            relay_node_hash: "a".repeat(62) + "@#", // Special characters
            latency_ms: 100,
        };
        match payload.validate() {
            Err(RelayTraceError::InvalidNodeHash) => {},
            _ => panic!("Expected InvalidNodeHash error"),
        }
    }

    // ========================================================================
    // Serde Round-Trip Tests
    // ========================================================================
    #[test]
    fn test_serialize_deserialize_round_trip() {
        let original = valid_payload();
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: RelayTracePayload = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_serde_snake_case_keys() {
        let payload = valid_payload();
        let serialized = serde_json::to_string(&payload).unwrap();
        assert!(serialized.contains("message_id"));
        assert!(serialized.contains("relay_node_hash"));
        assert!(serialized.contains("latency_ms"));
    }

    #[test]
    fn test_deserialize_valid_json() {
        let json = r#"
        {
            "message_id": "test_msg",
            "relay_node_hash": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
            "latency_ms": 42
        }"#;
        let payload: RelayTracePayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.message_id, "test_msg");
        assert_eq!(payload.latency_ms, 42);
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn test_deserialize_invalid_json() {
        let json = r#"
        {
            "message_id": "test_msg",
            "relay_node_hash": "invalid",
            "latency_ms": 42
        }"#;
        let payload: RelayTracePayload = serde_json::from_str(json).unwrap();
        assert!(payload.validate().is_err());
    }

    // ========================================================================
    // Boundary Value Tests
    // ========================================================================
    #[test]
    fn test_hash_length_boundaries() {
        // 63 characters - should fail
        let payload_63 = RelayTracePayload {
            message_id: "msg_63".to_string(),
            relay_node_hash: "a".repeat(63),
            latency_ms: 100,
        };
        assert!(payload_63.validate().is_err());

        // 64 characters - should pass
        let payload_64 = RelayTracePayload {
            message_id: "msg_64".to_string(),
            relay_node_hash: "a".repeat(64),
            latency_ms: 100,
        };
        assert!(payload_64.validate().is_ok());

        // 65 characters - should fail
        let payload_65 = RelayTracePayload {
            message_id: "msg_65".to_string(),
            relay_node_hash: "a".repeat(65),
            latency_ms: 100,
        };
        assert!(payload_65.validate().is_err());
    }

    #[test]
    fn test_latency_boundaries() {
        // Minimum latency (0)
        let payload_min = RelayTracePayload {
            message_id: "msg_min".to_string(),
            relay_node_hash: "a".repeat(64),
            latency_ms: 0,
        };
        assert!(payload_min.validate().is_ok());

        // Maximum latency (u64::MAX)
        let payload_max = RelayTracePayload {
            message_id: "msg_max".to_string(),
            relay_node_hash: "b".repeat(64),
            latency_ms: u64::MAX,
        };
        assert!(payload_max.validate().is_ok());
    }

    #[test]
    fn test_hex_digit_boundaries() {
        // Valid hex digits (0-9, a-f, A-F)
        let valid_digits = [
            "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
            "a", "b", "c", "d", "e", "f",
            "A", "B", "C", "D", "E", "F"
        ];
        
        for digit in valid_digits {
            let hash = digit.repeat(64);
            let payload = RelayTracePayload {
                message_id: "msg_valid".to_string(),
                relay_node_hash: hash,
                latency_ms: 100,
            };
            assert!(payload.validate().is_ok(), "Failed for digit: {}", digit);
        }

        // Invalid characters (g, z, special chars)
        let invalid_chars = ["g", "z", "@", "#", " ", "\t", "\n"];
        
        for invalid_char in invalid_chars {
            let mut hash = "a".repeat(63);
            hash.push_str(invalid_char);
            let payload = RelayTracePayload {
                message_id: "msg_invalid".to_string(),
                relay_node_hash: hash,
                latency_ms: 100,
            };
            assert!(payload.validate().is_err(), "Should have failed for char: {}", invalid_char);
        }
    }

    // ========================================================================
    // Property-Based Style Tests
    // ========================================================================
    #[test]
    fn test_any_valid_hex_hash_passes_validation() {
        let valid_hex_chars: Vec<char> = "0123456789abcdefABCDEF".chars().collect();
        
        for &c1 in &valid_hex_chars {
            for &c2 in &valid_hex_chars {
                let hash = format!("{}{}", c1.to_string().repeat(32), c2.to_string().repeat(32));
                let payload = RelayTracePayload {
                    message_id: "test".to_string(),
                    relay_node_hash: hash,
                    latency_ms: 100,
                };
                assert!(payload.validate().is_ok(), 
                    "Failed for hex chars: {} and {}", c1, c2);
            }
        }
    }

    #[test]
    fn test_display_always_starts_with_relay_trace() {
        let test_cases = vec![
            valid_payload(),
            valid_payload_with_zero_latency(),
            RelayTracePayload {
                message_id: "test".to_string(),
                relay_node_hash: "F".repeat(64),
                latency_ms: u64::MAX,
            }
        ];
        
        for payload in test_cases {
            let display = payload.to_string();
            assert!(display.starts_with("[RelayTrace]"));
            assert!(display.contains("msg_id="));
            assert!(display.contains("node="));
            assert!(display.contains("latency="));
            assert!(display.contains("ms"));
        }
    }

    #[test]
    fn test_display_contains_all_field_values() {
        let payload = RelayTracePayload {
            message_id: "unique_id_123".to_string(),
            relay_node_hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            latency_ms: 999,
        };
        let display = payload.to_string();
        assert!(display.contains("unique_id_123"));
        assert!(display.contains("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"));
        assert!(display.contains("999ms"));
    }

    // ========================================================================
    // Clone and Eq Behavior Tests
    // ========================================================================
    #[test]
    fn test_clone_creates_equal_payload() {
        let original = valid_payload();
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_partial_eq_works_correctly() {
        let payload1 = RelayTracePayload {
            message_id: "msg1".to_string(),
            relay_node_hash: "a".repeat(64),
            latency_ms: 100,
        };
        
        let payload2 = RelayTracePayload {
            message_id: "msg1".to_string(),
            relay_node_hash: "a".repeat(64),
            latency_ms: 100,
        };
        
        let payload3 = RelayTracePayload {
            message_id: "msg2".to_string(),
            relay_node_hash: "a".repeat(64),
            latency_ms: 100,
        };
        
        assert_eq!(payload1, payload2);
        assert_ne!(payload1, payload3);
    }

    // ========================================================================
    // Error Display Tests
    // ========================================================================
    #[test]
    fn test_error_display_messages() {
        let empty_id_error = RelayTraceError::EmptyMessageId;
        assert_eq!(empty_id_error.to_string(), "Message ID cannot be empty");
        
        let invalid_hash_error = RelayTraceError::InvalidNodeHash;
        assert_eq!(invalid_hash_error.to_string(), "Relay node hash must be a 64-character hexadecimal string");
    }

    // ========================================================================
    // std::error::Error Compliance Test
    // ========================================================================
    #[test]
    fn test_error_implements_error_trait() {
        let error = RelayTraceError::EmptyMessageId;
        
        // Can be used as a trait object
        let _: &dyn Error = &error;
        
        // Has source (even if None)
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_debug_display() {
        let empty_id_error = RelayTraceError::EmptyMessageId;
        let invalid_hash_error = RelayTraceError::InvalidNodeHash;
        
        // Debug should work
        assert!(format!("{:?}", empty_id_error).contains("EmptyMessageId"));
        assert!(format!("{:?}", invalid_hash_error).contains("InvalidNodeHash"));
        
        // Display should work
        assert_eq!(empty_id_error.to_string(), "Message ID cannot be empty");
        assert_eq!(invalid_hash_error.to_string(), "Relay node hash must be a 64-character hexadecimal string");
    }
}
