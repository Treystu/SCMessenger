/// Drift Frame — transport layer framing with length and CRC32

use super::DriftError;
use crc32fast::Hasher;

/// Drift Frame wraps a payload for transport over unreliable networks
///
/// Format (total overhead: 7 bytes):
/// [2 bytes] length (LE u16) - includes type and payload but NOT length/CRC
/// [1 byte]  frame_type
/// [N bytes] payload
/// [4 bytes] CRC32 over length + type + payload
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftFrame {
    /// Frame type (data, sync, ping, etc.)
    pub frame_type: FrameType,
    /// Payload (typically a DriftEnvelope)
    pub payload: Vec<u8>,
}

/// Frame type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    /// Data frame containing a DriftEnvelope (0x01)
    Data = 0x01,
    /// Sync request (0x02)
    SyncReq = 0x02,
    /// Sync response (0x03)
    SyncResp = 0x03,
    /// Ping/heartbeat (0x04)
    Ping = 0x04,
    /// Peer information announcement (0x05)
    PeerInfo = 0x05,
}

impl FrameType {
    /// Convert from u8 to FrameType
    pub fn from_u8(value: u8) -> Result<Self, DriftError> {
        match value {
            0x01 => Ok(FrameType::Data),
            0x02 => Ok(FrameType::SyncReq),
            0x03 => Ok(FrameType::SyncResp),
            0x04 => Ok(FrameType::Ping),
            0x05 => Ok(FrameType::PeerInfo),
            other => Err(DriftError::InvalidFrameType(other)),
        }
    }

    /// Convert to u8
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl DriftFrame {
    /// Transport overhead: 2 bytes (length) + 1 byte (type) + 4 bytes (CRC32) = 7 bytes
    pub const TRANSPORT_OVERHEAD: usize = 7;

    /// Serialize frame to bytes
    ///
    /// Format: [2 LE length][1 type][N payload][4 LE CRC32]
    /// Where length = 1 + payload.len() (includes type byte but not length/CRC fields)
    pub fn to_bytes(&self) -> Result<Vec<u8>, DriftError> {
        let payload_len = 1 + self.payload.len(); // type byte + payload

        if payload_len > u16::MAX as usize {
            return Err(DriftError::BufferTooShort {
                need: payload_len,
                got: u16::MAX as usize,
            });
        }

        let mut buf = Vec::with_capacity(Self::TRANSPORT_OVERHEAD + self.payload.len());

        // Write length (2 bytes, LE) - length includes type and payload but NOT length field itself
        let length = payload_len as u16;
        buf.extend_from_slice(&length.to_le_bytes());

        // Write type (1 byte)
        buf.push(self.frame_type.as_u8());

        // Write payload
        buf.extend_from_slice(&self.payload);

        // Calculate CRC32 over length + type + payload (everything except CRC itself)
        let mut hasher = Hasher::new();
        hasher.update(&buf);
        let crc32 = hasher.finalize();

        // Write CRC32 (4 bytes, LE)
        buf.extend_from_slice(&crc32.to_le_bytes());

        Ok(buf)
    }

    /// Deserialize frame from bytes
    ///
    /// Returns error if:
    /// - Buffer too short
    /// - Invalid frame type
    /// - CRC32 mismatch
    pub fn from_bytes(data: &[u8]) -> Result<Self, DriftError> {
        if data.len() < Self::TRANSPORT_OVERHEAD {
            return Err(DriftError::BufferTooShort {
                need: Self::TRANSPORT_OVERHEAD,
                got: data.len(),
            });
        }

        // Read length (2 bytes, LE)
        let length = u16::from_le_bytes([data[0], data[1]]) as usize;

        // Calculate expected total size: 2 (length) + length + 4 (CRC32)
        let expected_total = 2 + length + 4;
        if data.len() != expected_total {
            return Err(DriftError::BufferTooShort {
                need: expected_total,
                got: data.len(),
            });
        }

        // Extract and verify CRC32
        let crc_offset = data.len() - 4;
        let data_to_check = &data[..crc_offset];
        let received_crc = u32::from_le_bytes([
            data[crc_offset],
            data[crc_offset + 1],
            data[crc_offset + 2],
            data[crc_offset + 3],
        ]);

        let mut hasher = Hasher::new();
        hasher.update(data_to_check);
        let computed_crc = hasher.finalize();

        if computed_crc != received_crc {
            return Err(DriftError::CrcMismatch);
        }

        // Read frame type (1 byte)
        let frame_type = FrameType::from_u8(data[2])?;

        // Extract payload
        let payload = data[3..crc_offset].to_vec();

        Ok(DriftFrame {
            frame_type,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_frame() -> DriftFrame {
        DriftFrame {
            frame_type: FrameType::Data,
            payload: b"test payload data".to_vec(),
        }
    }

    #[test]
    fn test_frame_type_conversion() {
        assert_eq!(FrameType::Data.as_u8(), 0x01);
        assert_eq!(FrameType::SyncReq.as_u8(), 0x02);
        assert_eq!(FrameType::SyncResp.as_u8(), 0x03);
        assert_eq!(FrameType::Ping.as_u8(), 0x04);
        assert_eq!(FrameType::PeerInfo.as_u8(), 0x05);

        assert_eq!(FrameType::from_u8(0x01).unwrap(), FrameType::Data);
        assert_eq!(FrameType::from_u8(0x02).unwrap(), FrameType::SyncReq);
        assert!(FrameType::from_u8(0x99).is_err());
    }

    #[test]
    fn test_frame_serialize_deserialize() {
        let original = make_test_frame();
        let bytes = original.to_bytes().unwrap();

        // Check structure: 2 (length) + 1 (type) + payload + 4 (CRC32)
        assert_eq!(
            bytes.len(),
            2 + 1 + original.payload.len() + 4
        );

        let restored = DriftFrame::from_bytes(&bytes).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn test_frame_empty_payload() {
        let mut frame = make_test_frame();
        frame.payload = vec![];

        let bytes = frame.to_bytes().unwrap();
        assert_eq!(bytes.len(), 2 + 1 + 4); // length + type + CRC32

        let restored = DriftFrame::from_bytes(&bytes).unwrap();
        assert_eq!(restored.payload.len(), 0);
    }

    #[test]
    fn test_frame_large_payload() {
        let mut frame = make_test_frame();
        frame.payload = vec![0xBB; 10000];

        let bytes = frame.to_bytes().unwrap();
        let restored = DriftFrame::from_bytes(&bytes).unwrap();

        assert_eq!(restored.payload.len(), 10000);
        assert!(restored.payload.iter().all(|&b| b == 0xBB));
    }

    #[test]
    fn test_frame_crc32_validation() {
        let original = make_test_frame();
        let mut bytes = original.to_bytes().unwrap();

        // Tamper with the payload (before CRC)
        bytes[5] ^= 0xFF;

        let result = DriftFrame::from_bytes(&bytes);
        assert!(matches!(result, Err(DriftError::CrcMismatch)));
    }

    #[test]
    fn test_frame_crc32_tamper_type() {
        let original = make_test_frame();
        let mut bytes = original.to_bytes().unwrap();

        // Tamper with the type byte
        bytes[2] ^= 0x01;

        let result = DriftFrame::from_bytes(&bytes);
        assert!(matches!(result, Err(DriftError::CrcMismatch)));
    }

    #[test]
    fn test_frame_crc32_tamper_crc() {
        let original = make_test_frame();
        let mut bytes = original.to_bytes().unwrap();

        // Tamper with the CRC itself - this won't be detected,
        // but we can verify the whole frame validates
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;

        let result = DriftFrame::from_bytes(&bytes);
        assert!(matches!(result, Err(DriftError::CrcMismatch)));
    }

    #[test]
    fn test_frame_buffer_too_short() {
        let data = [0u8; 5];
        let result = DriftFrame::from_bytes(&data);

        match result {
            Err(DriftError::BufferTooShort { need, got }) => {
                assert_eq!(need, DriftFrame::TRANSPORT_OVERHEAD);
                assert_eq!(got, 5);
            }
            other => panic!("Expected BufferTooShort, got {:?}", other),
        }
    }

    #[test]
    fn test_frame_length_mismatch() {
        let frame = make_test_frame();
        let bytes = frame.to_bytes().unwrap();

        // Try to deserialize with truncated buffer
        let result = DriftFrame::from_bytes(&bytes[..bytes.len() - 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_frame_invalid_type() {
        let mut frame = make_test_frame();
        frame.frame_type = FrameType::Data;

        let mut bytes = frame.to_bytes().unwrap();
        bytes[2] = 0x99; // Invalid type

        // Will fail on CRC check first
        let result = DriftFrame::from_bytes(&bytes);
        assert!(matches!(result, Err(DriftError::CrcMismatch)));
    }

    #[test]
    fn test_frame_all_types() {
        for frame_type in &[
            FrameType::Data,
            FrameType::SyncReq,
            FrameType::SyncResp,
            FrameType::Ping,
            FrameType::PeerInfo,
        ] {
            let frame = DriftFrame {
                frame_type: *frame_type,
                payload: b"test".to_vec(),
            };

            let bytes = frame.to_bytes().unwrap();
            let restored = DriftFrame::from_bytes(&bytes).unwrap();

            assert_eq!(restored.frame_type, *frame_type);
            assert_eq!(restored.payload, b"test".to_vec());
        }
    }

    #[test]
    fn test_frame_length_calculation() {
        let frame = DriftFrame {
            frame_type: FrameType::Data,
            payload: b"hello".to_vec(),
        };

        let bytes = frame.to_bytes().unwrap();

        // Extract length field
        let length = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;

        // Length should be: 1 (type) + 5 (payload)
        assert_eq!(length, 6);

        // Total frame size should be: 2 (length) + 1 (type) + 5 (payload) + 4 (CRC32)
        assert_eq!(bytes.len(), 2 + 6 + 4);
    }

    #[test]
    fn test_frame_crc32_deterministic() {
        let frame = make_test_frame();
        let bytes1 = frame.to_bytes().unwrap();
        let bytes2 = frame.to_bytes().unwrap();

        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn test_frame_multiple_roundtrips() {
        let original = make_test_frame();

        let bytes1 = original.to_bytes().unwrap();
        let frame1 = DriftFrame::from_bytes(&bytes1).unwrap();

        let bytes2 = frame1.to_bytes().unwrap();
        let frame2 = DriftFrame::from_bytes(&bytes2).unwrap();

        assert_eq!(original, frame2);
    }
}
