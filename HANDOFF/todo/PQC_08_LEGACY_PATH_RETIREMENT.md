# TASK: PQC-08 — Retire legacy static-ECDH sending between v2 peers

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-07. Wave 4. Min tier: Haiku.

## Why

Audit finding F1: `encrypt_message` (`core/src/crypto/encrypt.rs`) does static-ephemeral ECDH against the recipient's identity-derived key — the single worst HNDL surface (no forward secrecy classically, full retro-decrypt quantumly). Once PQC-06/07 exist, there is no reason two capable peers ever use it.

## Rules to implement

In `encrypt_with_ratchet_fallback` (and any other caller of `encrypt_message` — inventory with `rg -n "encrypt_message\(" core/src --type rust`):

1. Peer has verified v2 bundle (0x02 negotiable) -> hybrid ratchet session MUST be used or established; falling through to `encrypt_message` for such a peer is now an ERROR path, not a fallback.
2. Peer is v1-only -> classical DH-ratchet session is REQUIRED where possible; bare `encrypt_message` remains ONLY for the first message to a peer with no session and no ratchet capability (current behavior), and each such send logs an audit event `legacy_static_ecdh_send`.
3. `require_pq = true` (PQC-04): case 2 becomes an error.
4. ALL decrypt paths stay untouched and tested (old stored messages must decrypt forever).

## Steps

1. Implement gating exactly as above; keep the decision logic in ONE function with unit tests per branch (v2 peer / v1 peer with ratchet / v1 peer bare / strict mode).
2. Audit events: reuse the existing audit log manager (locate via `rg -n "audit" core/src/iron_core.rs`).
3. Update any integration tests that asserted bare-ECDH sends between capable peers (expect ratcheted/hybrid instead).
4. Grep the CLI and wasm crates for direct `encrypt_message` calls and route them through the fallback function if any exist (list findings here).

## Definition of Done

- [ ] Standard gates PASS.
- [ ] `cargo test -p scmessenger-core --test integration_e2e --test integration_pq_session --test integration_ironcore_roundtrip` green.
- [ ] Branch-coverage unit tests for the gating function green.
- [ ] Call-site inventory written into this file.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Delete `encrypt_message`/`decrypt_message` or the Ed25519->X25519 conversion helpers.
- Change default `require_pq` to true.
