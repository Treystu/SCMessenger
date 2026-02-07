/// L2CAP channel abstraction for BLE logical link control
///
/// This module provides a channel abstraction over BLE L2CAP with support for
/// fragmentation and reassembly of messages that exceed the MTU.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Protocol Service Multiplexer (PSM) identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolServiceMultiplexer {
    /// SCMessenger primary PSM (0x0025)
    SCMessenger = 0x0025,
}

impl ProtocolServiceMultiplexer {
    /// Get the numeric PSM value
    pub fn value(&self) -> u16 {
        *self as u16
    }
}

/// Channel state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelState {
    /// Connection in progress
    Connecting,
    /// Channel established and ready
    Connected,
    /// Connection closing
    Closing,
    /// Channel closed
    Closed,
}

/// L2CAP channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2capConfig {
    /// Protocol Service Multiplexer
    pub psm: ProtocolServiceMultiplexer,
    /// Maximum Transmission Unit in bytes (default 672)
    pub mtu: u16,
    /// Channel timeout in seconds (default 30)
    pub timeout_secs: u64,
}

impl Default for L2capConfig {
    fn default() -> Self {
        Self {
            psm: ProtocolServiceMultiplexer::SCMessenger,
            mtu: 672,
            timeout_secs: 30,
        }
    }
}

impl L2capConfig {
    /// Create new L2CAP configuration
    pub fn new(psm: ProtocolServiceMultiplexer) -> Self {
        Self {
            psm,
            mtu: 672,
            timeout_secs: 30,
        }
    }

    /// Set the MTU size
    pub fn with_mtu(mut self, mtu: u16) -> Self {
        self.mtu = mtu;
        self
    }

    /// Set the timeout in seconds
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), L2capError> {
        if self.mtu < 23 {
            return Err(L2capError::InvalidMtu(self.mtu));
        }
        if self.timeout_secs == 0 {
            return Err(L2capError::InvalidTimeout);
        }
        Ok(())
    }
}

/// Errors for L2CAP operations
#[derive(Error, Debug, Clone)]
pub enum L2capError {
    #[error("Channel not connected")]
    NotConnected,
    #[error("Channel already connected")]
    AlreadyConnected,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Write failed: {0}")]
    WriteFailed(String),
    #[error("Read failed: {0}")]
    ReadFailed(String),
    #[error("Invalid MTU: {0}")]
    InvalidMtu(u16),
    #[error("Invalid timeout")]
    InvalidTimeout,
    #[error("Fragmentation error: {0}")]
    FragmentationError(String),
    #[error("Reassembly error: {0}")]
    ReassemblyError(String),
}

/// Fragment header structure: [total_fragments: u16 | fragment_index: u16 | data...]
#[derive(Debug, Clone)]
pub struct FragmentHeader {
    /// Total number of fragments
    pub total_fragments: u16,
    /// Index of this fragment (0-based)
    pub fragment_index: u16,
}

impl FragmentHeader {
    /// Size of fragment header in bytes
    const HEADER_SIZE: usize = 4;

    /// Create a new fragment header
    pub fn new(total_fragments: u16, fragment_index: u16) -> Result<Self, L2capError> {
        if fragment_index >= total_fragments {
            return Err(L2capError::FragmentationError(
                "Fragment index out of range".to_string(),
            ));
        }
        Ok(Self {
            total_fragments,
            fragment_index,
        })
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> [u8; 4] {
        let mut bytes = [0u8; 4];
        bytes[0..2].copy_from_slice(&self.total_fragments.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.fragment_index.to_le_bytes());
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, L2capError> {
        if bytes.len() < Self::HEADER_SIZE {
            return Err(L2capError::ReassemblyError(
                "Fragment header too short".to_string(),
            ));
        }
        let total_fragments = u16::from_le_bytes([bytes[0], bytes[1]]);
        let fragment_index = u16::from_le_bytes([bytes[2], bytes[3]]);
        Self::new(total_fragments, fragment_index)
    }
}

/// L2CAP Channel abstraction
pub struct L2capChannel {
    state: ChannelState,
    config: L2capConfig,
}

impl L2capChannel {
    /// Create a new L2CAP channel
    pub fn new(config: L2capConfig) -> Result<Self, L2capError> {
        config.validate()?;
        Ok(Self {
            state: ChannelState::Closed,
            config,
        })
    }

    /// Get current channel state
    pub fn state(&self) -> ChannelState {
        self.state
    }

    /// Get channel configuration
    pub fn config(&self) -> &L2capConfig {
        &self.config
    }

    /// Check if channel is connected
    pub fn is_connected(&self) -> bool {
        self.state == ChannelState::Connected
    }

    /// Transition to Connecting state
    pub fn initiate_connection(&mut self) -> Result<(), L2capError> {
        match self.state {
            ChannelState::Closed => {
                self.state = ChannelState::Connecting;
                Ok(())
            }
            ChannelState::Connecting | ChannelState::Connected => {
                Err(L2capError::AlreadyConnected)
            }
            ChannelState::Closing => Err(L2capError::ConnectionFailed(
                "Channel is closing".to_string(),
            )),
        }
    }

    /// Transition to Connected state
    pub fn confirm_connection(&mut self) -> Result<(), L2capError> {
        match self.state {
            ChannelState::Connecting => {
                self.state = ChannelState::Connected;
                Ok(())
            }
            _ => Err(L2capError::ConnectionFailed(
                "Not in connecting state".to_string(),
            )),
        }
    }

    /// Transition to Closing state
    pub fn initiate_close(&mut self) -> Result<(), L2capError> {
        match self.state {
            ChannelState::Connected | ChannelState::Connecting => {
                self.state = ChannelState::Closing;
                Ok(())
            }
            ChannelState::Closed => Err(L2capError::NotConnected),
            ChannelState::Closing => Ok(()), // Already closing
        }
    }

    /// Transition to Closed state
    pub fn confirm_close(&mut self) -> Result<(), L2capError> {
        match self.state {
            ChannelState::Closing | ChannelState::Connected | ChannelState::Connecting => {
                self.state = ChannelState::Closed;
                Ok(())
            }
            ChannelState::Closed => Ok(()), // Already closed
        }
    }
}

/// Fragmentation logic for splitting messages
pub struct L2capFragmenter {
    config: L2capConfig,
}

impl L2capFragmenter {
    /// Create a new fragmenter
    pub fn new(config: L2capConfig) -> Result<Self, L2capError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Calculate maximum payload size per fragment
    fn max_payload_per_fragment(&self) -> usize {
        // MTU - fragment header
        self.config.mtu as usize - FragmentHeader::HEADER_SIZE
    }

    /// Split a message into fragments
    pub fn fragment(&self, data: &[u8]) -> Result<Vec<Vec<u8>>, L2capError> {
        let max_payload = self.max_payload_per_fragment();

        if data.is_empty() {
            return Ok(vec![Vec::new()]);
        }

        let total_fragments = (data.len() + max_payload - 1) / max_payload;

        if total_fragments > u16::MAX as usize {
            return Err(L2capError::FragmentationError(
                "Message too large for fragmentation".to_string(),
            ));
        }

        let mut fragments = Vec::new();

        for (index, chunk) in data.chunks(max_payload).enumerate() {
            let header = FragmentHeader::new(total_fragments as u16, index as u16)?;
            let mut fragment = header.to_bytes().to_vec();
            fragment.extend_from_slice(chunk);
            fragments.push(fragment);
        }

        Ok(fragments)
    }
}

/// Reassembly logic for collecting fragments
#[allow(dead_code)]
pub struct L2capReassembler {
    config: L2capConfig,
}

impl L2capReassembler {
    /// Create a new reassembler
    pub fn new(config: L2capConfig) -> Result<Self, L2capError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Reassemble fragments into a complete message
    pub fn reassemble(&self, fragments: &[Vec<u8>]) -> Result<Vec<u8>, L2capError> {
        if fragments.is_empty() {
            return Ok(Vec::new());
        }

        // Parse first fragment to get total count
        let first_header = FragmentHeader::from_bytes(&fragments[0])?;
        let expected_total = first_header.total_fragments as usize;

        if fragments.len() != expected_total {
            return Err(L2capError::ReassemblyError(
                format!(
                    "Expected {} fragments, got {}",
                    expected_total,
                    fragments.len()
                ),
            ));
        }

        // Verify all fragments are present and in order
        for (i, fragment) in fragments.iter().enumerate() {
            let header = FragmentHeader::from_bytes(fragment)?;
            if header.fragment_index as usize != i {
                return Err(L2capError::ReassemblyError(format!(
                    "Fragment out of order: expected index {}, got {}",
                    i, header.fragment_index
                )));
            }
            if header.total_fragments != first_header.total_fragments {
                return Err(L2capError::ReassemblyError(
                    "Inconsistent fragment count".to_string(),
                ));
            }
        }

        // Reassemble payload
        let mut result = Vec::new();
        for fragment in fragments {
            if fragment.len() > FragmentHeader::HEADER_SIZE {
                result.extend_from_slice(&fragment[FragmentHeader::HEADER_SIZE..]);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psm_value() {
        let psm = ProtocolServiceMultiplexer::SCMessenger;
        assert_eq!(psm.value(), 0x0025);
    }

    #[test]
    fn test_l2cap_config_default() {
        let config = L2capConfig::default();
        assert_eq!(config.psm, ProtocolServiceMultiplexer::SCMessenger);
        assert_eq!(config.mtu, 672);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_l2cap_config_builder() {
        let config = L2capConfig::new(ProtocolServiceMultiplexer::SCMessenger)
            .with_mtu(512)
            .with_timeout(60);

        assert_eq!(config.mtu, 512);
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_l2cap_config_validation_valid() {
        let config = L2capConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_l2cap_config_validation_invalid_mtu() {
        let config = L2capConfig::default().with_mtu(10);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_l2cap_config_validation_invalid_timeout() {
        let config = L2capConfig::default().with_timeout(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_fragment_header_roundtrip() {
        let header = FragmentHeader::new(5, 2).expect("Valid header");
        let bytes = header.to_bytes();
        let recovered = FragmentHeader::from_bytes(&bytes).expect("Should deserialize");

        assert_eq!(recovered.total_fragments, 5);
        assert_eq!(recovered.fragment_index, 2);
    }

    #[test]
    fn test_fragment_header_invalid_index() {
        let result = FragmentHeader::new(5, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_l2cap_channel_state_machine() {
        let config = L2capConfig::default();
        let mut channel = L2capChannel::new(config).expect("Channel creation");

        assert_eq!(channel.state(), ChannelState::Closed);
        assert!(!channel.is_connected());

        channel.initiate_connection().expect("Initiate connection");
        assert_eq!(channel.state(), ChannelState::Connecting);

        channel.confirm_connection().expect("Confirm connection");
        assert_eq!(channel.state(), ChannelState::Connected);
        assert!(channel.is_connected());

        channel.initiate_close().expect("Initiate close");
        assert_eq!(channel.state(), ChannelState::Closing);

        channel.confirm_close().expect("Confirm close");
        assert_eq!(channel.state(), ChannelState::Closed);
    }

    #[test]
    fn test_l2cap_channel_invalid_double_connect() {
        let config = L2capConfig::default();
        let mut channel = L2capChannel::new(config).expect("Channel creation");

        channel.initiate_connection().expect("First connection");
        let result = channel.initiate_connection();
        assert!(result.is_err());
    }

    #[test]
    fn test_l2cap_fragmenter_small_message() {
        let config = L2capConfig::default();
        let fragmenter = L2capFragmenter::new(config).expect("Fragmenter creation");

        let data = vec![0u8; 100];
        let fragments = fragmenter.fragment(&data).expect("Fragmentation");

        assert_eq!(fragments.len(), 1);
        assert!(fragments[0].len() >= 100);
    }

    #[test]
    fn test_l2cap_fragmenter_large_message() {
        let config = L2capConfig::default().with_mtu(100);
        let fragmenter = L2capFragmenter::new(config).expect("Fragmenter creation");

        let data = vec![0u8; 500];
        let fragments = fragmenter.fragment(&data).expect("Fragmentation");

        assert!(fragments.len() > 1);

        // Each fragment should not exceed MTU
        for fragment in &fragments {
            assert!(fragment.len() <= 100);
        }
    }

    #[test]
    fn test_l2cap_fragmenter_empty_message() {
        let config = L2capConfig::default();
        let fragmenter = L2capFragmenter::new(config).expect("Fragmenter creation");

        let data = vec![];
        let fragments = fragmenter.fragment(&data).expect("Fragmentation");

        assert_eq!(fragments.len(), 1);
    }

    #[test]
    fn test_l2cap_reassembler_single_fragment() {
        let config = L2capConfig::default();
        let reassembler = L2capReassembler::new(config).expect("Reassembler creation");

        let header = FragmentHeader::new(1, 0).expect("Valid header");
        let mut fragment = header.to_bytes().to_vec();
        fragment.extend_from_slice(b"Hello");

        let fragments = vec![fragment];
        let result = reassembler.reassemble(&fragments).expect("Reassembly");

        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_l2cap_fragmenter_reassembler_roundtrip() {
        let config = L2capConfig::default().with_mtu(50);
        let fragmenter = L2capFragmenter::new(config.clone()).expect("Fragmenter");
        let reassembler = L2capReassembler::new(config).expect("Reassembler");

        let original = vec![0xAAu8; 500];
        let fragments = fragmenter.fragment(&original).expect("Fragmentation");
        let reassembled = reassembler.reassemble(&fragments).expect("Reassembly");

        assert_eq!(reassembled, original);
    }

    #[test]
    fn test_l2cap_reassembler_wrong_fragment_count() {
        let config = L2capConfig::default();
        let reassembler = L2capReassembler::new(config).expect("Reassembler creation");

        let header = FragmentHeader::new(3, 0).expect("Valid header");
        let fragment = vec![header.to_bytes()[..].to_vec()].concat();

        let fragments = vec![fragment];
        let result = reassembler.reassemble(&fragments);

        assert!(result.is_err());
    }

    #[test]
    fn test_l2cap_reassembler_out_of_order() {
        let config = L2capConfig::default();
        let reassembler = L2capReassembler::new(config).expect("Reassembler creation");

        let header1 = FragmentHeader::new(2, 1).expect("Valid header");
        let header2 = FragmentHeader::new(2, 0).expect("Valid header");

        let mut fragment1 = header1.to_bytes().to_vec();
        fragment1.extend_from_slice(b"World");

        let mut fragment2 = header2.to_bytes().to_vec();
        fragment2.extend_from_slice(b"Hello");

        let fragments = vec![fragment1, fragment2];
        let result = reassembler.reassemble(&fragments);

        assert!(result.is_err());
    }

    #[test]
    fn test_channel_state_transitions() {
        let config = L2capConfig::default();
        let mut channel = L2capChannel::new(config).expect("Channel creation");

        // Test multiple closes
        channel.initiate_connection().expect("Connect");
        channel.confirm_connection().expect("Confirm");
        channel.initiate_close().expect("Initiate close");
        channel.initiate_close().expect("Close is idempotent");
        channel.confirm_close().expect("Confirm close");
        channel.confirm_close().expect("Close confirmation is idempotent");
    }

    #[test]
    fn test_l2cap_channel_new_validates_config() {
        let config = L2capConfig::default().with_mtu(10);
        let result = L2capChannel::new(config);
        assert!(result.is_err());
    }
}
