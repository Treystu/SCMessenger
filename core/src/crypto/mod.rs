// Cryptography module — message encryption and key exchange

pub mod backup;
pub mod encrypt;

pub use encrypt::{decrypt_message, encrypt_message, validate_ed25519_public_key};
