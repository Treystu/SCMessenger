# Cryptography Rules

## Non-Negotiable Algorithms
- Identity: Ed25519
- Identity hash: Blake3
- Key exchange: X25519 ECDH
- KDF: Blake3 derive_key
- Encryption: XChaCha20-Poly1305
- Auth: Ed25519 signatures + AAD

## NEVER substitute algorithms without explicit owner approval.
## Reference types: IdentityKeys, Envelope, Message in core/src/crypto/
