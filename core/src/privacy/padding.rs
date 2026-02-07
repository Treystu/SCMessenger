// Message Padding — Traffic analysis resistance via constant message sizes
//
// Pads messages to standard sizes to prevent attackers from inferring
// content length or message frequency patterns.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Standard message sizes for padding
pub const STANDARD_SIZES: &[usize] = &[256, 512, 1024, 2048, 4096];

/// Padding scheme options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PaddingScheme {
    /// No padding applied
    None,
    /// Pad to fixed size (in bytes)
    Fixed(usize),
    /// Pad to next power of two
    PowerOfTwo,
    /// Pad with random amount between min and max
    Random(usize, usize),
}

#[derive(Debug, Error)]
pub enum PaddingError {
    #[error("Invalid padding configuration: {0}")]
    InvalidConfig(String),
    #[error("Message too large for fixed padding: {0}")]
    MessageTooLarge(usize),
    #[error("Invalid padding format")]
    InvalidPaddingFormat,
}

/// Add padding to reach a target size
///
/// Padding format: [original_data][0x80][0x00...0x00]
/// The 0x80 marker indicates start of padding.
///
/// # Arguments
/// * `message` - Original message bytes
/// * `target_size` - Target padded size (must be >= message length)
///
/// # Returns
/// * Padded message of exactly `target_size` bytes
pub fn pad_message(message: &[u8], target_size: usize) -> Result<Vec<u8>, PaddingError> {
    if message.len() > target_size {
        return Err(PaddingError::MessageTooLarge(target_size));
    }

    let mut padded = Vec::with_capacity(target_size);
    padded.extend_from_slice(message);

    // Add padding marker (0x80)
    padded.push(0x80);

    // Fill remaining with zeros
    while padded.len() < target_size {
        padded.push(0x00);
    }

    Ok(padded)
}

/// Remove padding from a message
///
/// Finds the 0x80 marker and removes padding.
///
/// # Arguments
/// * `padded_message` - Message with padding applied
///
/// # Returns
/// * Original message bytes (without padding)
pub fn unpad_message(padded_message: &[u8]) -> Result<Vec<u8>, PaddingError> {
    // Find the 0x80 marker (last occurrence to handle data that might contain 0x80)
    let marker_pos = padded_message
        .iter()
        .rposition(|&b| b == 0x80)
        .ok_or(PaddingError::InvalidPaddingFormat)?;

    // Verify everything after marker is 0x00
    for i in (marker_pos + 1)..padded_message.len() {
        if padded_message[i] != 0x00 {
            return Err(PaddingError::InvalidPaddingFormat);
        }
    }

    Ok(padded_message[..marker_pos].to_vec())
}

/// Pad message to the next standard size
///
/// # Arguments
/// * `message` - Original message bytes
///
/// # Returns
/// * Padded message, or error if message exceeds largest standard size
pub fn pad_to_next_standard_size(message: &[u8]) -> Result<Vec<u8>, PaddingError> {
    let target_size = STANDARD_SIZES
        .iter()
        .find(|&&size| size >= message.len())
        .copied()
        .ok_or_else(|| {
            PaddingError::InvalidConfig(format!(
                "Message exceeds max standard size of {}",
                STANDARD_SIZES[STANDARD_SIZES.len() - 1]
            ))
        })?;

    pad_message(message, target_size)
}

/// Apply a padding scheme to a message
///
/// # Arguments
/// * `message` - Original message bytes
/// * `scheme` - Padding scheme to apply
///
/// # Returns
/// * Padded message according to scheme
pub fn apply_padding_scheme(
    message: &[u8],
    scheme: PaddingScheme,
) -> Result<Vec<u8>, PaddingError> {
    match scheme {
        PaddingScheme::None => Ok(message.to_vec()),
        PaddingScheme::Fixed(size) => pad_message(message, size),
        PaddingScheme::PowerOfTwo => {
            let target_size = (message.len() as u64).next_power_of_two() as usize;
            pad_message(message, target_size)
        }
        PaddingScheme::Random(min, max) => {
            if min > max || min == 0 {
                return Err(PaddingError::InvalidConfig(
                    "Invalid random range".to_string(),
                ));
            }
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let target_size = rng.gen_range(min..=max);
            if target_size < message.len() {
                return Err(PaddingError::MessageTooLarge(target_size));
            }
            pad_message(message, target_size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_message_exact_size() {
        let msg = b"hello";
        let padded = pad_message(msg, 10).unwrap();
        assert_eq!(padded.len(), 10);
        assert_eq!(padded[0..5], b"hello"[..]);
        assert_eq!(padded[5], 0x80);
        for i in 6..10 {
            assert_eq!(padded[i], 0x00);
        }
    }

    #[test]
    fn test_pad_message_minimum_size() {
        let msg = b"test";
        let padded = pad_message(msg, 5).unwrap();
        assert_eq!(padded.len(), 5);
        assert_eq!(padded[0..4], b"test"[..]);
    }

    #[test]
    fn test_pad_message_too_large() {
        let msg = b"hello";
        let result = pad_message(msg, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_unpad_message_basic() {
        let msg = b"hello";
        let padded = pad_message(msg, 20).unwrap();
        let unpadded = unpad_message(&padded).unwrap();
        assert_eq!(unpadded, msg);
    }

    #[test]
    fn test_unpad_message_no_padding() {
        let padded = b"test\x80\x00\x00\x00";
        let unpadded = unpad_message(padded).unwrap();
        assert_eq!(unpadded, b"test");
    }

    #[test]
    fn test_unpad_message_invalid_padding() {
        // Missing 0x80 marker
        let invalid = b"hello world";
        let result = unpad_message(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_unpad_message_invalid_trailing_bytes() {
        // Non-zero bytes after marker
        let invalid = b"test\x80\x01\x00\x00";
        let result = unpad_message(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_pad_to_next_standard_size_exact() {
        let msg = vec![1; 255];
        let padded = pad_to_next_standard_size(&msg).unwrap();
        assert_eq!(padded.len(), 256);
    }

    #[test]
    fn test_pad_to_next_standard_size_round_up() {
        let msg = vec![1; 300];
        let padded = pad_to_next_standard_size(&msg).unwrap();
        assert_eq!(padded.len(), 512);
    }

    #[test]
    fn test_pad_to_next_standard_size_small() {
        let msg = b"hi";
        let padded = pad_to_next_standard_size(msg).unwrap();
        assert_eq!(padded.len(), 256);
    }

    #[test]
    fn test_pad_to_next_standard_size_too_large() {
        let msg = vec![1; 5000];
        let result = pad_to_next_standard_size(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_padding_none() {
        let msg = b"original";
        let result = apply_padding_scheme(msg, PaddingScheme::None).unwrap();
        assert_eq!(result, msg);
    }

    #[test]
    fn test_apply_padding_fixed() {
        let msg = b"test";
        let result = apply_padding_scheme(msg, PaddingScheme::Fixed(16)).unwrap();
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_apply_padding_power_of_two() {
        let msg = vec![1; 100];
        let result = apply_padding_scheme(&msg, PaddingScheme::PowerOfTwo).unwrap();
        assert_eq!(result.len(), 128);
    }

    #[test]
    fn test_apply_padding_random() {
        let msg = b"msg";
        let result = apply_padding_scheme(msg, PaddingScheme::Random(32, 64)).unwrap();
        assert!(result.len() >= 32 && result.len() <= 64);
    }

    #[test]
    fn test_apply_padding_random_invalid() {
        let msg = b"msg";
        let result = apply_padding_scheme(msg, PaddingScheme::Random(64, 32));
        assert!(result.is_err());
    }

    #[test]
    fn test_round_trip_padding() {
        let original = b"This is a test message with various characters: !@#$%";
        let padded = pad_to_next_standard_size(original).unwrap();
        let unpadded = unpad_message(&padded).unwrap();
        assert_eq!(unpadded, original);
    }

    #[test]
    fn test_padding_multiple_sizes() {
        for &size in STANDARD_SIZES {
            let msg = vec![42; size / 2];
            let padded = pad_message(&msg, size).unwrap();
            assert_eq!(padded.len(), size);

            let unpadded = unpad_message(&padded).unwrap();
            assert_eq!(unpadded, msg);
        }
    }

    #[test]
    fn test_padding_empty_message() {
        let msg = b"";
        let padded = pad_message(msg, 32).unwrap();
        assert_eq!(padded.len(), 32);

        let unpadded = unpad_message(&padded).unwrap();
        assert!(unpadded.is_empty());
    }

    #[test]
    fn test_padding_with_embedded_marker() {
        // Message containing 0x80 byte
        let msg = vec![1, 2, 3, 0x80, 5, 6];
        let padded = pad_message(&msg, 64).unwrap();
        let unpadded = unpad_message(&padded).unwrap();
        assert_eq!(unpadded, msg);
    }

    #[test]
    fn test_standard_sizes_constant() {
        assert!(STANDARD_SIZES.len() > 0);
        for i in 1..STANDARD_SIZES.len() {
            assert!(STANDARD_SIZES[i] > STANDARD_SIZES[i - 1]);
        }
    }
}
