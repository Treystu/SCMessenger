// Message module — types and serialization for the messaging protocol

pub mod codec;
pub mod ephemeral;
pub mod types;

pub use codec::{
    decode_envelope, decode_message, encode_envelope, encode_message, decode_wire_envelope,
    decode_wire_signed_envelope, encode_wire_envelope, encode_wire_signed_envelope,
};
pub use ephemeral::*;
pub use types::{
    DeliveryStatus, Envelope, EnvelopeV2, Message, MessageType, Receipt, SignedEnvelope,
    SignedEnvelopeV2, WireEnvelope, WireSignedEnvelope, WIRE_TAG_V2,
};
