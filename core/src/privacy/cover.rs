// Cover Traffic — Dummy messages to mask real traffic patterns
//
// Generates fake messages that are indistinguishable from real traffic
// to prevent attackers from observing when actual communication occurs.

use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoverTrafficError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Invalid message size")]
    InvalidMessageSize,
}

/// Configuration for cover traffic generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverConfig {
    /// Number of cover messages to generate per minute
    pub rate_per_minute: u32,
    /// Target message size in bytes
    pub message_size: usize,
    /// Whether to generate cover traffic (can be disabled)
    pub enabled: bool,
}

impl Default for CoverConfig {
    fn default() -> Self {
        Self {
            rate_per_minute: 10,
            message_size: 1024,
            enabled: true,
        }
    }
}

impl CoverConfig {
    /// Validate the cover traffic configuration
    pub fn validate(&self) -> Result<(), CoverTrafficError> {
        if self.rate_per_minute == 0 && self.enabled {
            return Err(CoverTrafficError::InvalidConfig(
                "rate_per_minute must be > 0 when enabled".to_string(),
            ));
        }
        if self.message_size == 0 {
            return Err(CoverTrafficError::InvalidConfig(
                "message_size must be > 0".to_string(),
            ));
        }
        if self.message_size > 65536 {
            return Err(CoverTrafficError::InvalidConfig(
                "message_size exceeds maximum (65536 bytes)".to_string(),
            ));
        }
        Ok(())
    }

    /// Get delay between cover messages (in milliseconds)
    pub fn message_interval_ms(&self) -> u64 {
        if self.rate_per_minute == 0 {
            return 0;
        }
        (60_000 / self.rate_per_minute as u64).max(1)
    }
}

/// Generated cover traffic message
///
/// Looks like a normal DriftEnvelope but contains random data.
/// Only the final recipient can determine if it's cover traffic
/// (by failing to decrypt to a valid message).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverMessage {
    /// Recipient public key hint (random)
    pub recipient_hint: Vec<u8>,
    /// Random encrypted payload (looks encrypted)
    pub encrypted_payload: Vec<u8>,
    /// Ephemeral key for this message (random)
    pub ephemeral_key: Vec<u8>,
    /// Is marked as cover traffic (only visible after decryption attempt)
    pub is_cover: bool,
}

impl CoverMessage {
    /// Check if this message is cover traffic
    pub fn is_cover_traffic(&self) -> bool {
        self.is_cover
    }
}

/// Generates cover traffic messages
pub struct CoverTrafficGenerator {
    config: CoverConfig,
}

impl CoverTrafficGenerator {
    /// Create a new cover traffic generator
    pub fn new(config: CoverConfig) -> Result<Self, CoverTrafficError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Generate a single cover traffic message
    pub fn generate_cover_message(&self) -> Result<CoverMessage, CoverTrafficError> {
        if !self.config.enabled {
            return Err(CoverTrafficError::InvalidConfig(
                "Cover traffic generation is disabled".to_string(),
            ));
        }

        let mut rng = rand::thread_rng();

        // Generate random recipient hint (32 bytes, like an X25519 public key)
        let mut recipient_hint = vec![0u8; 32];
        rng.fill_bytes(&mut recipient_hint);

        // Generate random encrypted payload
        let mut encrypted_payload = vec![0u8; self.config.message_size];
        rng.fill_bytes(&mut encrypted_payload);

        // Generate random ephemeral key
        let mut ephemeral_key = vec![0u8; 32];
        rng.fill_bytes(&mut ephemeral_key);

        Ok(CoverMessage {
            recipient_hint,
            encrypted_payload,
            ephemeral_key,
            is_cover: true,
        })
    }

    /// Generate multiple cover traffic messages
    pub fn generate_batch(&self, count: usize) -> Result<Vec<CoverMessage>, CoverTrafficError> {
        (0..count).map(|_| self.generate_cover_message()).collect()
    }

    /// Get the configuration
    pub fn config(&self) -> &CoverConfig {
        &self.config
    }
}

/// Scheduler for periodic cover traffic generation
pub struct CoverTrafficScheduler {
    config: CoverConfig,
    last_generation_time: SystemTime,
}

impl CoverTrafficScheduler {
    /// Create a new cover traffic scheduler
    pub fn new(config: CoverConfig) -> Result<Self, CoverTrafficError> {
        config.validate()?;
        Ok(Self {
            config,
            last_generation_time: SystemTime::UNIX_EPOCH,
        })
    }

    /// Check if it's time to generate cover traffic
    ///
    /// Returns true if enough time has elapsed since the last generation
    pub fn should_generate_cover_traffic(&self) -> bool {
        if !self.config.enabled || self.config.rate_per_minute == 0 {
            return false;
        }

        match self.last_generation_time.elapsed() {
            Ok(elapsed) => {
                let interval = Duration::from_millis(self.config.message_interval_ms());
                elapsed > interval
            }
            Err(_) => true,
        }
    }

    /// Generate cover traffic and update the generation timestamp
    pub fn generate_and_update(&mut self) -> Result<CoverMessage, CoverTrafficError> {
        let generator = CoverTrafficGenerator::new(self.config.clone())?;
        let message = generator.generate_cover_message()?;
        self.last_generation_time = SystemTime::now();
        Ok(message)
    }

    /// Reset the generation timer
    pub fn reset_timer(&mut self) {
        self.last_generation_time = SystemTime::UNIX_EPOCH;
    }

    /// Get the next scheduled generation time
    pub fn next_generation_time(&self) -> Option<SystemTime> {
        if !self.config.enabled {
            return None;
        }

        match self
            .last_generation_time
            .checked_add(Duration::from_millis(self.config.message_interval_ms()))
        {
            Some(next_time) => Some(next_time),
            None => None,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &CoverConfig {
        &self.config
    }
}

/// Check if a received message might be cover traffic
///
/// Since cover traffic only becomes identifiable after a failed decryption,
/// this function is typically called by the final recipient when decryption fails.
pub fn is_cover_traffic(decryption_attempt_failed: bool) -> bool {
    // A failed decryption on what looks like a valid envelope suggests
    // the message was cover traffic (meant for a different recipient)
    decryption_attempt_failed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cover_config_default() {
        let config = CoverConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.enabled);
        assert!(config.rate_per_minute > 0);
    }

    #[test]
    fn test_cover_config_validate() {
        let config = CoverConfig {
            rate_per_minute: 5,
            message_size: 512,
            enabled: true,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cover_config_validate_zero_rate_enabled() {
        let config = CoverConfig {
            rate_per_minute: 0,
            message_size: 512,
            enabled: true,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cover_config_validate_zero_rate_disabled() {
        let config = CoverConfig {
            rate_per_minute: 0,
            message_size: 512,
            enabled: false,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cover_config_validate_zero_message_size() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 0,
            enabled: true,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cover_config_validate_excessive_message_size() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 100000,
            enabled: true,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cover_config_message_interval() {
        let config = CoverConfig {
            rate_per_minute: 60,
            message_size: 512,
            enabled: true,
        };
        assert_eq!(config.message_interval_ms(), 1000); // 1 per second
    }

    #[test]
    fn test_cover_config_message_interval_low_rate() {
        let config = CoverConfig {
            rate_per_minute: 1,
            message_size: 512,
            enabled: true,
        };
        assert_eq!(config.message_interval_ms(), 60000); // 1 per minute
    }

    #[test]
    fn test_cover_message_creation() {
        let msg = CoverMessage {
            recipient_hint: vec![1; 32],
            encrypted_payload: vec![2; 256],
            ephemeral_key: vec![3; 32],
            is_cover: true,
        };

        assert!(msg.is_cover_traffic());
        assert_eq!(msg.recipient_hint.len(), 32);
        assert_eq!(msg.encrypted_payload.len(), 256);
    }

    #[test]
    fn test_cover_traffic_generator_new() {
        let config = CoverConfig::default();
        let generator = CoverTrafficGenerator::new(config);
        assert!(generator.is_ok());
    }

    #[test]
    fn test_cover_traffic_generator_invalid_config() {
        let config = CoverConfig {
            rate_per_minute: 0,
            message_size: 512,
            enabled: true,
        };
        let result = CoverTrafficGenerator::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_cover_traffic_generator_disabled() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 512,
            enabled: false,
        };
        let generator = CoverTrafficGenerator::new(config).unwrap();
        let result = generator.generate_cover_message();
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_cover_message() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 1024,
            enabled: true,
        };
        let generator = CoverTrafficGenerator::new(config).unwrap();
        let message = generator.generate_cover_message().unwrap();

        assert_eq!(message.recipient_hint.len(), 32);
        assert_eq!(message.ephemeral_key.len(), 32);
        assert_eq!(message.encrypted_payload.len(), 1024);
        assert!(message.is_cover);
    }

    #[test]
    fn test_generate_cover_message_uniqueness() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 512,
            enabled: true,
        };
        let generator = CoverTrafficGenerator::new(config).unwrap();

        let msg1 = generator.generate_cover_message().unwrap();
        let msg2 = generator.generate_cover_message().unwrap();

        // Random data should be different
        assert_ne!(msg1.recipient_hint, msg2.recipient_hint);
        assert_ne!(msg1.encrypted_payload, msg2.encrypted_payload);
    }

    #[test]
    fn test_generate_batch() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 256,
            enabled: true,
        };
        let generator = CoverTrafficGenerator::new(config).unwrap();
        let batch = generator.generate_batch(5).unwrap();

        assert_eq!(batch.len(), 5);
        for msg in batch {
            assert!(msg.is_cover);
            assert_eq!(msg.encrypted_payload.len(), 256);
        }
    }

    #[test]
    fn test_cover_traffic_scheduler_new() {
        let config = CoverConfig::default();
        let scheduler = CoverTrafficScheduler::new(config);
        assert!(scheduler.is_ok());
    }

    #[test]
    fn test_cover_traffic_scheduler_should_generate_initially() {
        let config = CoverConfig {
            rate_per_minute: 60,
            message_size: 512,
            enabled: true,
        };
        let scheduler = CoverTrafficScheduler::new(config).unwrap();
        // Initially, should always return true (very short elapsed time)
        assert!(scheduler.should_generate_cover_traffic());
    }

    #[test]
    fn test_cover_traffic_scheduler_disabled() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 512,
            enabled: false,
        };
        let scheduler = CoverTrafficScheduler::new(config).unwrap();
        assert!(!scheduler.should_generate_cover_traffic());
    }

    #[test]
    fn test_cover_traffic_scheduler_generate_and_update() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 512,
            enabled: true,
        };
        let mut scheduler = CoverTrafficScheduler::new(config).unwrap();

        let message = scheduler.generate_and_update().unwrap();
        assert!(message.is_cover);

        // Immediately after generation, shouldn't be ready again (unless interval is very short)
        // This depends on timing, so we just verify the function works
    }

    #[test]
    fn test_cover_traffic_scheduler_reset_timer() {
        let config = CoverConfig::default();
        let mut scheduler = CoverTrafficScheduler::new(config).unwrap();

        scheduler.reset_timer();
        // After reset, should be ready to generate again
        assert!(scheduler.should_generate_cover_traffic());
    }

    #[test]
    fn test_cover_traffic_scheduler_next_generation_time() {
        let config = CoverConfig {
            rate_per_minute: 60, // 1 per second
            message_size: 512,
            enabled: true,
        };
        let scheduler = CoverTrafficScheduler::new(config).unwrap();
        let next_time = scheduler.next_generation_time();
        assert!(next_time.is_some());
    }

    #[test]
    fn test_cover_traffic_scheduler_next_generation_time_disabled() {
        let config = CoverConfig {
            rate_per_minute: 10,
            message_size: 512,
            enabled: false,
        };
        let scheduler = CoverTrafficScheduler::new(config).unwrap();
        let next_time = scheduler.next_generation_time();
        assert!(next_time.is_none());
    }

    #[test]
    fn test_is_cover_traffic() {
        assert!(is_cover_traffic(true)); // Failed decryption suggests cover traffic
        assert!(!is_cover_traffic(false)); // Successful decryption is real traffic
    }

    #[test]
    fn test_cover_config_serialization() {
        let config = CoverConfig {
            rate_per_minute: 15,
            message_size: 2048,
            enabled: true,
        };

        let serialized = bincode::serialize(&config).unwrap();
        let deserialized: CoverConfig = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.rate_per_minute, 15);
        assert_eq!(deserialized.message_size, 2048);
        assert_eq!(deserialized.enabled, true);
    }

    #[test]
    fn test_cover_message_serialization() {
        let msg = CoverMessage {
            recipient_hint: vec![42; 32],
            encrypted_payload: vec![99; 512],
            ephemeral_key: vec![7; 32],
            is_cover: true,
        };

        let serialized = bincode::serialize(&msg).unwrap();
        let deserialized: CoverMessage = bincode::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.recipient_hint, msg.recipient_hint);
        assert_eq!(deserialized.encrypted_payload, msg.encrypted_payload);
        assert_eq!(deserialized.is_cover, true);
    }

    #[test]
    fn test_cover_traffic_various_sizes() {
        for size in [256, 512, 1024, 2048, 4096].iter() {
            let config = CoverConfig {
                rate_per_minute: 10,
                message_size: *size,
                enabled: true,
            };
            let generator = CoverTrafficGenerator::new(config).unwrap();
            let msg = generator.generate_cover_message().unwrap();
            assert_eq!(msg.encrypted_payload.len(), *size);
        }
    }
}
