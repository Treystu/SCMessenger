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
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    
    current_time > creation_timestamp + ttl.expires_in_seconds
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
        
        let ttl = TtlConfig { expires_in_seconds: 10 };
        
        assert!(!is_expired(creation_time, &ttl));
    }

    #[test]
    fn test_expired() {
        let ttl = TtlConfig { expires_in_seconds: 0 };
        
        // Creation time is set to 1 second in the past
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() - 1;
        
        assert!(is_expired(creation_time, &ttl));
    }
}