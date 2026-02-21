// Timing Jitter — Resistance to timing correlation attacks
//
// Adds random delays to relay forwarding to prevent attackers from
// linking incoming and outgoing traffic by timing patterns.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Distribution type for jitter delays
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JitterDistribution {
    /// Uniform random distribution
    Uniform,
    /// Exponential distribution (more likely to be small delays)
    Exponential,
}

/// Configuration for jitter applied to relay forwarding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitterConfig {
    /// Minimum delay in milliseconds
    pub min_delay_ms: u32,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u32,
    /// Distribution type for jitter
    pub distribution: JitterDistribution,
}

impl Default for JitterConfig {
    fn default() -> Self {
        Self {
            min_delay_ms: 50,
            max_delay_ms: 500,
            distribution: JitterDistribution::Uniform,
        }
    }
}

impl JitterConfig {
    /// Validate jitter configuration
    pub fn validate(&self) -> Result<(), JitterError> {
        if self.min_delay_ms > self.max_delay_ms {
            return Err(JitterError::InvalidConfig(
                "min_delay_ms must not exceed max_delay_ms".to_string(),
            ));
        }
        if self.max_delay_ms == 0 {
            return Err(JitterError::InvalidConfig(
                "max_delay_ms must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Message priority levels with associated jitter policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    /// High priority: 0-50ms jitter
    HighPriority,
    /// Normal priority: 50-500ms jitter
    Normal,
    /// Low priority: 100-2000ms jitter
    LowPriority,
}

impl MessagePriority {
    /// Get the jitter config for this priority level
    pub fn jitter_config(&self) -> JitterConfig {
        match self {
            MessagePriority::HighPriority => JitterConfig {
                min_delay_ms: 0,
                max_delay_ms: 50,
                distribution: JitterDistribution::Uniform,
            },
            MessagePriority::Normal => JitterConfig {
                min_delay_ms: 50,
                max_delay_ms: 500,
                distribution: JitterDistribution::Uniform,
            },
            MessagePriority::LowPriority => JitterConfig {
                min_delay_ms: 100,
                max_delay_ms: 2000,
                distribution: JitterDistribution::Exponential,
            },
        }
    }
}

/// Relay timing policy for different message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayTimingPolicy {
    /// Policy for high-priority messages
    pub high_priority_config: JitterConfig,
    /// Policy for normal messages
    pub normal_config: JitterConfig,
    /// Policy for low-priority messages
    pub low_priority_config: JitterConfig,
}

impl Default for RelayTimingPolicy {
    fn default() -> Self {
        Self {
            high_priority_config: MessagePriority::HighPriority.jitter_config(),
            normal_config: MessagePriority::Normal.jitter_config(),
            low_priority_config: MessagePriority::LowPriority.jitter_config(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JitterError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Timing jitter generator for relay operations
pub struct TimingJitter {
    config: JitterConfig,
}

impl TimingJitter {
    /// Create a new timing jitter with the given configuration
    pub fn new(config: JitterConfig) -> Result<Self, JitterError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Create jitter for a specific message priority
    pub fn with_priority(priority: MessagePriority) -> Result<Self, JitterError> {
        Self::new(priority.jitter_config())
    }

    /// Compute a random jitter delay based on the configuration
    pub fn compute_jitter(&self) -> Duration {
        compute_jitter(&self.config)
    }

    /// Get the configuration
    pub fn config(&self) -> &JitterConfig {
        &self.config
    }
}

/// Compute a random jitter delay
///
/// Generates a random delay between min and max according to the
/// configured distribution type.
///
/// # Arguments
/// * `config` - Jitter configuration with min/max delays and distribution type
///
/// # Returns
/// * Duration representing the delay to apply
pub fn compute_jitter(config: &JitterConfig) -> Duration {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let delay_ms = match config.distribution {
        JitterDistribution::Uniform => rng.gen_range(config.min_delay_ms..=config.max_delay_ms),
        JitterDistribution::Exponential => {
            // Exponential distribution: bias toward smaller delays
            let uniform = rng.gen::<f64>(); // 0.0 to 1.0
            let _range = (config.max_delay_ms - config.min_delay_ms) as f64;
            // Use exponential bias: values closer to 0 are more likely
            let exponential = -uniform.ln();
            let scaled = (exponential * 100.0) as u32;
            let clamped = scaled.min(config.max_delay_ms - config.min_delay_ms);
            config.min_delay_ms + clamped
        }
    };

    Duration::from_millis(delay_ms as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_config_default() {
        let config = JitterConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.min_delay_ms, 50);
        assert_eq!(config.max_delay_ms, 500);
    }

    #[test]
    fn test_jitter_config_validate_valid() {
        let config = JitterConfig {
            min_delay_ms: 10,
            max_delay_ms: 100,
            distribution: JitterDistribution::Uniform,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_jitter_config_validate_equal() {
        let config = JitterConfig {
            min_delay_ms: 50,
            max_delay_ms: 50,
            distribution: JitterDistribution::Uniform,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_jitter_config_validate_invalid_order() {
        let config = JitterConfig {
            min_delay_ms: 100,
            max_delay_ms: 10,
            distribution: JitterDistribution::Uniform,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_jitter_config_validate_zero_max() {
        let config = JitterConfig {
            min_delay_ms: 0,
            max_delay_ms: 0,
            distribution: JitterDistribution::Uniform,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_message_priority_jitter_config() {
        let high = MessagePriority::HighPriority.jitter_config();
        assert_eq!(high.min_delay_ms, 0);
        assert_eq!(high.max_delay_ms, 50);

        let normal = MessagePriority::Normal.jitter_config();
        assert_eq!(normal.min_delay_ms, 50);
        assert_eq!(normal.max_delay_ms, 500);

        let low = MessagePriority::LowPriority.jitter_config();
        assert_eq!(low.min_delay_ms, 100);
        assert_eq!(low.max_delay_ms, 2000);
    }

    #[test]
    fn test_timing_jitter_new() {
        let config = JitterConfig::default();
        let jitter = TimingJitter::new(config);
        assert!(jitter.is_ok());
    }

    #[test]
    fn test_timing_jitter_new_invalid_config() {
        let config = JitterConfig {
            min_delay_ms: 100,
            max_delay_ms: 10,
            distribution: JitterDistribution::Uniform,
        };
        let result = TimingJitter::new(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_timing_jitter_with_priority() {
        let jitter = TimingJitter::with_priority(MessagePriority::Normal).unwrap();
        assert_eq!(jitter.config().min_delay_ms, 50);
        assert_eq!(jitter.config().max_delay_ms, 500);
    }

    #[test]
    fn test_compute_jitter_uniform() {
        let config = JitterConfig {
            min_delay_ms: 100,
            max_delay_ms: 200,
            distribution: JitterDistribution::Uniform,
        };

        for _ in 0..100 {
            let delay = compute_jitter(&config);
            let millis = delay.as_millis() as u32;
            assert!(millis >= 100 && millis <= 200);
        }
    }

    #[test]
    fn test_compute_jitter_exponential() {
        let config = JitterConfig {
            min_delay_ms: 50,
            max_delay_ms: 500,
            distribution: JitterDistribution::Exponential,
        };

        for _ in 0..100 {
            let delay = compute_jitter(&config);
            let millis = delay.as_millis() as u32;
            assert!(millis >= 50 && millis <= 500);
        }
    }

    #[test]
    fn test_compute_jitter_equal_bounds() {
        let config = JitterConfig {
            min_delay_ms: 100,
            max_delay_ms: 100,
            distribution: JitterDistribution::Uniform,
        };

        let delay = compute_jitter(&config);
        assert_eq!(delay, Duration::from_millis(100));
    }

    #[test]
    fn test_compute_jitter_zero_min() {
        let config = JitterConfig {
            min_delay_ms: 0,
            max_delay_ms: 100,
            distribution: JitterDistribution::Uniform,
        };

        for _ in 0..100 {
            let delay = compute_jitter(&config);
            let millis = delay.as_millis() as u32;
            assert!(millis <= 100);
        }
    }

    #[test]
    fn test_relay_timing_policy_default() {
        let policy = RelayTimingPolicy::default();
        assert!(policy.high_priority_config.validate().is_ok());
        assert!(policy.normal_config.validate().is_ok());
        assert!(policy.low_priority_config.validate().is_ok());
    }

    #[test]
    fn test_timing_jitter_config_access() {
        let config = JitterConfig {
            min_delay_ms: 75,
            max_delay_ms: 250,
            distribution: JitterDistribution::Uniform,
        };
        let jitter = TimingJitter::new(config).unwrap();
        let retrieved_config = jitter.config();
        assert_eq!(retrieved_config.min_delay_ms, 75);
        assert_eq!(retrieved_config.max_delay_ms, 250);
    }

    #[test]
    fn test_jitter_distribution_serialization() {
        let uniform = JitterDistribution::Uniform;
        let serialized = bincode::serialize(&uniform).unwrap();
        let deserialized: JitterDistribution = bincode::deserialize(&serialized).unwrap();
        assert_eq!(uniform, deserialized);

        let exponential = JitterDistribution::Exponential;
        let serialized = bincode::serialize(&exponential).unwrap();
        let deserialized: JitterDistribution = bincode::deserialize(&serialized).unwrap();
        assert_eq!(exponential, deserialized);
    }

    #[test]
    fn test_jitter_config_serialization() {
        let config = JitterConfig {
            min_delay_ms: 10,
            max_delay_ms: 100,
            distribution: JitterDistribution::Exponential,
        };
        let serialized = bincode::serialize(&config).unwrap();
        let deserialized: JitterConfig = bincode::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.min_delay_ms, 10);
        assert_eq!(deserialized.max_delay_ms, 100);
    }

    #[test]
    fn test_exponential_distribution_bias() {
        let config = JitterConfig {
            min_delay_ms: 50,
            max_delay_ms: 500,
            distribution: JitterDistribution::Exponential,
        };

        // Exponential should have more small values than large values
        let mut small_count = 0;
        let mut large_count = 0;

        for _ in 0..1000 {
            let delay = compute_jitter(&config);
            let millis = delay.as_millis() as u32;
            if millis < 200 {
                small_count += 1;
            } else if millis > 300 {
                large_count += 1;
            }
        }

        // Exponential should have more in the small range
        assert!(small_count > large_count);
    }
}
