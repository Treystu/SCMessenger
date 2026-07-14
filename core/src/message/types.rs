use serde::{Deserialize, Serialize};
use serde_cbor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub version: u8,
    pub sender_public_key: Vec<u8>,
    pub ephemeral_public_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ratchet_dh_public: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub ratchet_message_number: Option<u64>,
    pub pq_kem_ciphertext: Option<Vec<u8>>,
    pub pq_encaps_key: Option<Vec<u8>>,
    pub transcript_hash: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedEnvelope {
    pub envelope: Envelope,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireEnvelope {
    pub version: u8,
    pub sender_public_key: Vec<u8>,
    pub ephemeral_public_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ratchet_dh_public: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub ratchet_message_number: Option<u64>,
    pub pq_kem_ciphertext: Option<Vec<u8>>,
    pub pq_encaps_key: Option<Vec<u8>>,
    pub transcript_hash: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireSignedEnvelope {
    pub envelope: WireEnvelope,
    pub signature: Vec<u8>,
}

// core/src/message/codec.rs
use crate::message::types::{Envelope, SignedEnvelope, WireEnvelope, WireSignedEnvelope};
use serde_cbor;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_cbor::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

pub fn encode(envelope: &Envelope) -> Result<Vec<u8>, CodecError> {
    let wire = WireEnvelope {
        version: envelope.version,
        sender_public_key: envelope.sender_public_key.clone(),
        ephemeral_public_key: envelope.ephemeral_public_key.clone(),
        nonce: envelope.nonce.clone(),
        ratchet_dh_public: envelope.ratchet_dh_public.clone(),
        ciphertext: envelope.ciphertext.clone(),
        ratchet_message_number: envelope.ratchet_message_number,
        pq_kem_ciphertext: envelope.pq_kem_ciphertext.clone(),
        pq_encaps_key: envelope.pq_encaps_key.clone(),
        transcript_hash: envelope.transcript_hash.clone(),
    };
    let bytes = serde_cbor::to_vec(&wire)?;
    Ok(bytes)
}

pub fn decode(bytes: &[u8]) -> Result<Envelope, CodecError> {
    let wire: WireEnvelope = serde_cbor::from_slice(bytes)?;
    Ok(Envelope {
        version: wire.version,
        sender_public_key: wire.sender_public_key,
        ephemeral_public_key: wire.ephemeral_public_key,
        nonce: wire.nonce,
        ratchet_dh_public: wire.ratchet_dh_public,
        ciphertext: wire.ciphertext,
        ratchet_message_number: wire.ratchet_message_number,
        pq_kem_ciphertext: wire.pq_kem_ciphertext,
        pq_encaps_key: wire.pq_encaps_key,
        transcript_hash: wire.transcript_hash,
    })
}

pub fn encode_signed(envelope: &SignedEnvelope) -> Result<Vec<u8>, CodecError> {
    let wire = WireSignedEnvelope {
        envelope: WireEnvelope {
            version: envelope.envelope.version,
            sender_public_key: envelope.envelope.sender_public_key.clone(),
            ephemeral_public_key: envelope.envelope.ephemeral_public_key.clone(),
            nonce: envelope.envelope.nonce.clone(),
            ratchet_dh_public: envelope.envelope.ratchet_dh_public.clone(),
            ciphertext: envelope.envelope.ciphertext.clone(),
            ratchet_message_number: envelope.envelope.ratchet_message_number,
            pq_kem_ciphertext: envelope.envelope.pq_kem_ciphertext.clone(),
            pq_encaps_key: envelope.envelope.pq_encaps_key.clone(),
            transcript_hash: envelope.envelope.transcript_hash.clone(),
        },
        signature: envelope.signature.clone(),
    };
    let bytes = serde_cbor::to_vec(&wire)?;
    Ok(bytes)
}

pub fn decode_signed(bytes: &[u8]) -> Result<SignedEnvelope, CodecError> {
    let wire: WireSignedEnvelope = serde_cbor::from_slice(bytes)?;
    Ok(SignedEnvelope {
        envelope: Envelope {
            version: wire.envelope.version,
            sender_public_key: wire.envelope.sender_public_key,
            ephemeral_public_key: wire.envelope.ephemeral_public_key,
            nonce: wire.envelope.nonce,
            ratchet_dh_public: wire.envelope.ratchet_dh_public,
            ciphertext: wire.envelope.ciphertext,
            ratchet_message_number: wire.envelope.ratchet_message_number,
            pq_kem_ciphertext: wire.envelope.pq_kem_ciphertext,
            pq_encaps_key: wire.envelope.pq_encaps_key,
            transcript_hash: wire.envelope.transcript_hash,
        },
        signature: wire.signature,
    })
}