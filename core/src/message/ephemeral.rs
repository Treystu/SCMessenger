//! Ephemeral message functionality with TTL (Time-To-Live) support.

use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for TTL (Time-To-Live) based expiration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtlConfig {
    /// The duration in seconds after which a message expires.
    pub expires_in_seconds: u64,
}

/// Checks if a message has expired based on its creation timestamp and TTL configuration.
///
/// # Arguments
///
/// * `creation_timestamp` - Unix timestamp when the message was created
/// * `ttl` - TTL configuration specifying expiration duration
///
/// # Returns
///
/// True if the current time exceeds the creation time plus the TTL duration, false otherwise.
pub fn is_expired(creation_timestamp: u64, ttl: &TtlConfig) -> bool {
    // A clock set before the Unix epoch is a configuration fault, not a reason to
    // take the node down: the same `unwrap_or_default` house pattern is used for
    // the wall clock in `core/src/store/dedup.rs`. The expiry bound saturates so a
    // hostile or corrupt `creation_timestamp` cannot overflow the addition.
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    current_time > creation_timestamp.saturating_add(ttl.expires_in_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_expired() {
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let ttl = TtlConfig {
            expires_in_seconds: 10,
        };

        assert!(!is_expired(creation_time, &ttl));
    }

    #[test]
    fn test_expired() {
        let ttl = TtlConfig {
            expires_in_seconds: 0,
        };

        // Creation time is set to 1 second in the past
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
            - 1;

        assert!(is_expired(creation_time, &ttl));
    }

    #[test]
    fn test_future_creation_timestamp_is_not_expired() {
        let ttl = TtlConfig {
            expires_in_seconds: 60,
        };
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + 3600;

        assert!(!is_expired(creation_time, &ttl));
    }

    #[test]
    fn test_saturated_creation_timestamp_does_not_overflow() {
        // A corrupt timestamp at the top of the u64 range must not overflow the
        // expiry addition (which would panic in a debug build).
        let ttl = TtlConfig {
            expires_in_seconds: u64::MAX,
        };

        assert!(!is_expired(u64::MAX, &ttl));
    }
}
