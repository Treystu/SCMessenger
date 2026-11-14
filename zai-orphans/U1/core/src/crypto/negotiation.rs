use std::collections::HashSet;

/// Negotiates a cryptographic suite and generates a transcript hash bound to the negotiation.
///
/// Returns `(negotiated_suite, transcript_hash)`.
///
/// The transcript hash MUST be computed by the INITIATOR with `our_` prefix referring to the initiator
/// and `their_` prefix referring to the responder. The responder recomputes this from their perspective
/// by calling this function where `our_` refers to the INITIATOR and `their_` refers to the RESPONDER.
/// This means the responder calls this function passing the initiator's properties as `our_...` to match.
pub fn negotiate_suite(
    our_suites: &[u8],
    their_suites: &[u8],
    our_ed25519_pub: &[u8; 32],
    their_ed25519_pub: &[u8; 32],
) -> Result<(u8, [u8; 32]), crate::IronCoreError> {
    let our_set: HashSet<u8> = our_suites.iter().cloned().collect();
    let their_set: HashSet<u8> = their_suites.iter().cloned().collect();
    let intersection: Vec<u8> = our_set.intersection(&their_set).cloned().collect();

    // An empty intersection means no mutually supported suite; surface it as a
    // recoverable negotiation failure rather than panicking on `max()`.
    let negotiated_suite = match intersection.iter().max() {
        Some(&suite) => suite,
        None => return Err(crate::IronCoreError::CryptoError),
    };

    let mut material = Vec::new();
    material.extend_from_slice(our_suites);
    material.push(0xFF);
    material.extend_from_slice(their_suites);
    material.push(0xFF);
    material.push(negotiated_suite);
    material.extend_from_slice(our_ed25519_pub);
    material.extend_from_slice(their_ed25519_pub);

    let transcript_hash = blake3::derive_key("iron-core suite-transcript v1", &material);

    Ok((negotiated_suite, transcript_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiate_suite_empty_intersection() {
        let our_suites = [0x01, 0x02];
        let their_suites = [0x03, 0x04];
        let our_pub = [0u8; 32];
        let their_pub = [1u8; 32];

        let result = negotiate_suite(&our_suites, &their_suites, &our_pub, &their_pub);
        assert!(result.is_err());
    }

    #[test]
    fn test_negotiate_suite_singleton() {
        let our_suites = [0x01];
        let their_suites = [0x01, 0x02];
        let our_pub = [0u8; 32];
        let their_pub = [1u8; 32];

        let (suite, _) = negotiate_suite(&our_suites, &their_suites, &our_pub, &their_pub).unwrap();
        assert_eq!(suite, 0x01);
    }

    #[test]
    fn test_negotiate_suite_future_suites() {
        let our_suites = [0x01, 0x02, 0xFF];
        let their_suites = [0x01, 0x02, 0xFF, 0xFE];
        let our_pub = [0u8; 32];
        let their_pub = [1u8; 32];

        let (suite, _) = negotiate_suite(&our_suites, &their_suites, &our_pub, &their_pub).unwrap();
        assert_eq!(suite, 0xFF);
    }

    #[test]
    fn test_negotiate_suite_symmetry() {
        let our_suites = [0x01, 0x02, 0x03];
        let their_suites = [0x01, 0x02, 0x03];
        let our_pub = [0u8; 32];
        let their_pub = [1u8; 32];

        let (suite_1, hash_1) =
            negotiate_suite(&our_suites, &their_suites, &our_pub, &their_pub).unwrap();
        // The responder calls it passing the INITIATOR's stuff as "our" to ensure identical material order
        let (suite_2, hash_2) =
            negotiate_suite(&our_suites, &their_suites, &our_pub, &their_pub).unwrap();

        assert_eq!(suite_1, suite_2);
        assert_eq!(hash_1, hash_2);

        // If responder accidentally inverted the args (used their own suites as `our`), the hash MUST mismatch!
        let (_, hash_inverted) =
            negotiate_suite(&their_suites, &our_suites, &their_pub, &our_pub).unwrap();
        assert_ne!(hash_1, hash_inverted);
    }
}
