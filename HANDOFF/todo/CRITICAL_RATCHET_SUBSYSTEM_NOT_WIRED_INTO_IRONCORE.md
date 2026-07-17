# CRITICAL: Ratchet + PQ subsystem is unreachable from IronCore's production message path

Status: OPEN -- CRITICAL -- OPERATOR ESCALATION REQUIRED (architecture decision)
Filed: 2026-07-17 (native audit session, code-truth verification pass)
Tier: MAX design review, then CODER implementation waves
Review: crypto-security-auditor MANDATORY (this is the E-01 defect family's true root)

## Finding

The entire Double Ratchet + PQ-hybrid subsystem, although implemented and
tested in isolation, is dead code with respect to real message traffic.

- `IronCore::prepare_message_internal` (core/src/iron_core.rs:636-668), reached
  by every CLI/FFI send path (`prepare_message_with_id`/`prepare_message`, called
  from cli/src/api_axum.rs:52, cli/src/api.rs:503, cli/src/main.rs:2040/2231/2853/2936),
  calls the bare legacy `crypto::encrypt_message` (core/src/crypto/encrypt.rs:123)
  directly.
- `IronCore::receive_message` (core/src/iron_core.rs:2714-2726), called from
  cli/src/main.rs:1866/2740, cli/src/ble_mesh.rs:75, and
  core/src/mobile_bridge.rs:786/1277 (Android/iOS), calls the bare legacy
  `crypto::decrypt_message` directly.
- The ratchet-aware paths `encrypt_with_ratchet_fallback` /
  `decrypt_with_ratchet_fallback` (encrypt.rs:503-566, 581-660) -- the only
  functions that consult `RatchetSessionManager` -- are never called outside
  encrypt.rs itself and core/tests/integration_pq_session.rs.

Consequence: every real message sent by the app today has ZERO forward secrecy
and ZERO PQ protection, regardless of the correctness of the ratchet code.

## Contradiction with prior audit

`HANDOFF/done/PQC_08_LEGACY_PATH_RETIREMENT.md` (2026-07-11, "verified")
asserts "No direct encrypt_message( calls exist anywhere in cli/, wasm/,
mobile/, or desktop_bridge/". That inventory missed core/src/iron_core.rs
itself; the bare calls at iron_core.rs:667 and :2723 date to commit 5ae934aa2
(2026-07-02), nine days before that audit.

## Relationship to open tickets

This supersedes/reframes the E-01 family scope:
- PQC_07_PQ_SECRET_NEVER_MIXED_INTO_ROOT_KEY (E-01): still real
  (ratchet.rs:580-588 hardcodes pq_ss=None in both branches), but fixing it
  changes nothing for users until THIS ticket lands.
- PQC_07_WIRE_RATCHET_STEP (E-04): its narrow claim (cadence call inside
  encrypt_message_ratcheted) is TRUE and landed 2026-07-11; the wiring gap is
  one level higher, at IronCore.
- PQC_07_FORCE_RATCHET_SAME_DEFECT (E-02): still real (ratchet.rs:676).

## Required operator decisions before dispatch

1. Wire `prepare_message_internal`/`receive_message` through the session
   manager (encrypt_with_ratchet_fallback / decrypt_with_ratchet_fallback)?
   This is the architecture-direction change the standing rules say must be
   operator-approved.
2. Migration/compat: fallback behavior for peers without established ratchet
   sessions (the _fallback functions already model this -- verify semantics).
3. Sequencing vs E-01b (PQ mixing design): recommend wiring first
   (E-00), then E-01c lands inside an actually-live path.

## Acceptance (once approved)

- prepare_message_internal and receive_message route through the ratchet
  session manager; bare legacy encrypt_message/decrypt_message calls remain
  only inside the fallback implementation.
- integration_e2e + integration_pq_session green; new integration test proves
  a CLI-to-CLI message round-trip advances the ratchet state.
- Adversarial review PASS (crypto-security-auditor tier).
