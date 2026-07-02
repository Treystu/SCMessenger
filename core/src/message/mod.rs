// Message module — types and serialization for the messaging protocol

pub mod codec;
pub mod ephemeral;
pub mod types;

pub use codec::{decode_envelope, decode_message, encode_envelope, encode_message};
pub use ephemeral::*;
pub use types::{DeliveryStatus, Envelope, Message, MessageType, Receipt, SignedEnvelope};
