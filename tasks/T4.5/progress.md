# T4.5 — Key backup/recovery flow verification

**Status:** partial
**Track:** 4 (Cryptographic Identity, Anti-Entropy & UI Hardening)
**Dependencies:** T4.1
**Blocks:** none

## Technical Context
- `crypto/backup.rs` exists (key backup/recovery); AuditEventType has BackupExported/BackupImported
- No evidence of end-to-end test or mobile UX wiring

## Implementation
1. Verification-first: integration test exporting identity+ratchet sessions+contacts to an encrypted backup blob (passphrase-derived key — audit `backup.rs` KDF: must be Argon2id or scrypt, NOT bare blake3 of passphrase; add if missing, this is the one likely real crypto gap)
2. Importing on a fresh IronCore, asserting full conversational continuity (can decrypt next ratchet message)
3. Wire export/import through FFI if not present

## Edge Cases
- Backup of a *registered* device must handle seniority (imported device re-registers, doesn't clone seniority — interaction with `RegistrationState`)
- Partial import (corrupt blob) must be atomic — all-or-nothing with explicit `CorruptionDetected` error (enum variant exists)
- Passphrase KDF parameters must be embedded in blob header for forward-compat

## Verification
- [x] Roundtrip integration test
- [x] Tampered-blob test -> CorruptionDetected, no partial state
- [x] KDF asserted memory-hard (test that derivation takes >100 ms or checks algorithm tag)
- [x] Audit events emitted both directions

## Update (2026-07-01)

**KDF decision: switched to Argon2id.** No compatibility constraint against it
was found anywhere in the codebase (no WASM/embedded-specific concern
documented, `argon2` is a pure-Rust crate that builds fine for all of this
workspace's targets including wasm32). Parameters: 19 MiB memory, 2
iterations, 1 lane (OWASP's password-storage minimums). `backup.rs` now
writes a format-tag byte (`0x02`) ahead of `salt || nonce || ciphertext` so
future KDF/format changes don't need to guess from blob shape, and
`decrypt_backup` still decrypts backups from both older PBKDF2-based formats
(with and without a stored salt) for backward compatibility - every *new*
backup is Argon2id.

**Scope grew beyond `backup.rs` itself.** `export_identity_backup`/
`export_identity_backup_with_salt` previously only backed up the raw identity
key bytes - no ratchet sessions, no contacts, and no `BackupExported` audit
event at all (only import logged). Extended the payload to a versioned JSON
envelope (identity key + `RatchetSessionManager::serialize_sessions()` +
`ContactManager::list()`), with `import_identity_backup` validating the
entire payload (parses ratchet-session JSON and identity key bytes) before
writing anything, so a tampered/malformed backup can't leave a mix of
old/new state. Added the missing export-side audit event. Old bare-hex-key
backups still import correctly (payload parsing falls back when the JSON
envelope doesn't parse).

**Real bug found and fixed along the way:** `store::ContactManager::list()`
(and `count()`/`verify_integrity()`) scanned the *entire* shared backend with
an empty key prefix and tried to parse every value as a `Contact` -
`IronCore` hands identity/history/logs/blocked-list/contacts the same
`Arc<dyn StorageBackend>`, so as soon as identity data (always written on
`initialize_identity()`) shared that backend, `list()` hit non-Contact JSON
and returned `Internal` for the *entire* contact list. This is exercised by
the CLI (`cli/src/main.rs`/`api.rs`/`api_axum.rs` all call
`contacts_store_manager()`), so real desktop usage was affected; Android/iOS
are unaffected since they go through the separately-backed
`contacts_bridge::ContactManager` (own sled file). Fixed by namespacing
contact keys under a `contact:` prefix. **Caveat (resolved 2026-07-02, see
S4 below):** this was a key-format change with no migration path - any
contacts added via the CLI before this fix were orphaned (unprefixed keys).

New tests: `core/src/crypto/backup.rs` (KDF/format unit tests),
`core/tests/integration_backup.rs` (three new IronCore-level tests:
continuity-through-restore via a real two-party ratchet exchange, tampered
backup leaves zero partial state, audit events fire on both export and
import - the existing tests in that file already covered the lower-level
`backup.rs`/`RatchetSessionManager` primitives directly, just never through
`IronCore`'s actual public API).

## Update (2026-07-02, S8 reconciliation)
The unprefixed-contacts caveat above is now fixed: `ContactManager::new`
(`core/src/store/contacts.rs`) runs a one-time migration on open that
rewrites any bare-`peer_id`-keyed contact under its `contact:`-prefixed key
(S4), covered by `test_unprefixed_contacts_migrate_on_open`. Backup
export/import also now carries the mobile UniFFI-bridge contacts store
(`bridge_contacts_json`, T1) and validates ratchet-session entries strictly
instead of silently dropping corrupt ones on import (T3), both landing
after this task's original pass.

`cargo test --workspace --all-features` and
`cargo clippy --workspace --all-features -- -D warnings` re-run locally
(verified 2026-07-02, local run) - both green, including the backup/contacts
suites this task and its follow-ups (T1, T3, S4) added.
