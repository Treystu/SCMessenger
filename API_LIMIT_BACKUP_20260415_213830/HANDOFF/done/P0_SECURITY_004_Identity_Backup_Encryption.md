# P0_SECURITY_004: Identity Backup Encryption

**Priority:** P0 (Critical Security)
**Platform:** Core/Rust
**Status:** DONE (Core implementation complete)
**Completed:** 2026-04-15

## Findings

The passphrase-based encryption infrastructure is **already fully implemented** in the Rust core:

1. **`core/src/crypto/backup.rs`** — PBKDF2-HMAC-SHA256 (600k iterations) with Blake3-derived salt + XChaCha20-Poly1305 encryption. Includes roundtrip tests, wrong-passphrase rejection, and invalid-data handling.

2. **`core/src/lib.rs`** — `export_identity_backup(passphrase)` and `import_identity_backup(backup, passphrase)` both accept and use the passphrase parameter. Zeroization of key bytes after use.

3. **`core/src/api.udl`** — Both functions declare `passphrase` parameters.

## Remaining Gap (Platform-Specific, Not Core)

The mobile clients and WASM bindings still call the OLD function signatures without passphrases:
- Android `MeshRepository.kt`: `exportIdentityBackup()` / `importIdentityBackup(backup)` — no passphrase
- iOS `MeshRepository.swift`: same
- WASM `app.js`: same

This requires:
1. Regenerating UniFFI bindings with the passphrase parameter
2. Adding passphrase UI to Android, iOS, and WASM clients
3. This is platform UI work, not Rust core work

## Build Verification
- Rust `cargo check`: PASSED (encryption module already existed)