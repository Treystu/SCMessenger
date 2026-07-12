# TASK [CRITICAL]: PQ ratchet cadence never mixes the shared secret into root_key

Status: TODO. Found 2026-07-11 while adding test coverage for the PQC-07
cadence trigger (`PQC_07_CADENCE_TEST_COVERAGE.md`, done/). This is a
PRE-EXISTING bug in the original PQC-07 implementation, not introduced this
session. Mandatory adversarial review before merge (`core/src/crypto/`).

## Finding

The "PQ ratchet cadence" feature (every 100 messages, refresh PQ material for
ongoing forward secrecy) correctly transmits real ML-KEM-768 ciphertexts and
encapsulation keys over the wire, and both sides correctly encapsulate/
decapsulate them - but the resulting shared secret is **never actually mixed
into the session's root key, on either side, ever**:

1. `core/src/crypto/ratchet.rs::perform_pq_ratchet_step` (~line 680): the
   sender's own encapsulation produces `_ss_pq` (underscore-prefixed,
   deliberately unused) and only the ciphertext `ct` is kept/sent. This
   might be fine BY DESIGN if the sender is meant to derive its "fresh"
   secret later from the receiver's reply (see point 3) - but point 2 below
   means that path is also broken.
2. `core/src/crypto/encrypt.rs::decrypt_message_ratcheted_v2` (~line 313):
   `session.handle_incoming_pq_fields(pq_kem_ciphertext, pq_encaps_key)?;` -
   the `Result<Vec<u8>>` return value (the decapsulated shared secret) is
   NOT captured (no `let ss = ...`). It's computed and immediately dropped.
3. `core/src/crypto/ratchet.rs::handle_dh_ratchet` (~line 550-591), called
   from `decrypt()` (~line 527) whenever the peer's DH position changes -
   this is THE function that actually calls `root_key_ratchet_v2(..., pq_ss)`
   to mix PQ material into the root key. But `pq_ss` here is:
   ```rust
   let pq_ss = if self.negotiated_suite == Some(0x02) {
       // Check if we have PQ fields available (should be provided externally)
       // For now, we'll assume the PQ handling is done in the calling code
       // This function just handles the DH part, PQ is handled separately
       None
   } else {
       None
   };
   ```
   BOTH branches return `None` unconditionally - the "should be provided
   externally" never happens anywhere in the current call graph. So every
   single DH ratchet step for suite 0x02 sessions calls
   `root_key_ratchet_v2(&root_key, dh_output, None)` - the PQ component is
   always absent.

**Net effect: the entire ongoing PQ ratchet cadence is cryptographically
inert.** It costs bandwidth and computation (real KEM operations, real wire
transmission) but contributes ZERO security benefit - a quantum-capable
attacker who breaks the classical X25519 DH fully compromises the session
regardless of the PQ material ever exchanged post-bootstrap. Only the
INITIAL bootstrap hybrid handshake (a separate code path, PQC-06) actually
achieves hybrid security; nothing after that does.

## Verification test (already added, correctly demonstrates this)

`core/tests/integration_pq_session.rs::test_pq_ratchet_cadence_refreshes_shared_secret`
was written to assert the root key changes after the cadence trigger. That
assertion is currently commented out / disabled (see that test's own
comments) specifically because it correctly fails against current code -
re-enable it as part of this fix's Definition of Done.

## What a real fix needs

1. Thread the decapsulated shared secret from `handle_incoming_pq_fields`'s
   return value through to wherever the NEXT `handle_dh_ratchet` call
   happens for that session, so `pq_ss` in `handle_dh_ratchet` is real
   material, not always `None`. This likely means:
   - `decrypt_message_ratcheted_v2` captures `let ss_pq = session.handle_incoming_pq_fields(...)?;`
     and stores it on the session (e.g. a new `pending_pq_ss: Option<Vec<u8>>`
     field) rather than discarding it.
   - `decrypt()`/`handle_dh_ratchet()` reads and clears that stored value
     when performing the next DH ratchet step, instead of hardcoding `None`.
2. Consider whether `perform_pq_ratchet_step`'s own `_ss_pq` (sender side)
   should also be used somehow, or whether the ping-pong design (sender
   only sends ciphertext + new encaps key, receiver decapsulates and stores
   the secret for ITS next DH step) is the intended shape - clarify design
   intent before implementing, this is a real architecture question, not
   just a wiring bug.
3. Update/re-enable the disabled assertion in the coverage test once fixed.
4. Add a Kani proof or additional unit test specifically asserting no DH
   ratchet step for a suite-0x02 session with pending PQ material ever
   produces the SAME root key transition as one without it (i.e. prove the
   PQ material is load-bearing, not just present).

## Do NOT

- Do not change the wire format (`pq_kem_ciphertext`/`pq_encaps_key` fields)
  - only the internal handling of the already-decapsulated secret.
- Do not weaken the cadence trigger condition (message-100 logic) - that
  part works correctly.

## Gate

Mandatory `crypto-security-auditor` (or equivalent) adversarial review
before merge - this is a live-path change to the double-ratchet root key
derivation. Standard compile gate + `cargo test -p scmessenger-core --test
integration_pq_session --features test-utils -j 2` green with the
previously-disabled assertion re-enabled and passing.
