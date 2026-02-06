// Message module — types and serialization for the messaging protocol

pub mod types;
pub mod codec;

pub use types::{Message, MessageType, Receipt, DeliveryStatus, Envelope};
pub use codec::{encode_message, decode_message, encode_envelope, decode_envelope};
