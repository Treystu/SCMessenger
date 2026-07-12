# PQC-05/06/07 Adversarial Review — Consolidated Verdict

Date: 2026-07-11
Reviewers: Qwen3-VL-235B-thinking (Part A: pq/mod.rs, session_manager.rs,
negotiation.rs), NVIDIA Nemotron-3-Ultra-550B via OpenRouter (Part B:
ratchet.rs, encrypt.rs, integration_pq_session.rs). Orchestrator
(this session) independently verified the CRITICAL findings against
current source before recording them below.

Process note: Qwen3-VL-235B-thinking failed Part B twice (once combined,
once ratchet.rs-alone) by hallucinating a plausible-but-wrong
reconstruction of the file instead of reviewing it (e.g. silently
inventing a version where the PQ shared secret is always discarded) —
neither hallucinated response was applied (no `--apply` used). OpenRouter's
Nemotron-3-Ultra produced a real prose-only verdict on the first retry with
a reinforced anti-code-dump prompt.

## VERDICT: FAIL (one CRITICAL integration gap, confirmed) -- WIRING FIX LANDED 2026-07-11

`PQC_07_WIRE_RATCHET_STEP.md` is implemented and committed: sender side now
triggers `perform_pq_ratchet_step()` every 100 messages post-confirmation;
receiver side calls `validate_pq_fields_present()` +
`handle_incoming_pq_fields()` for post-confirmation messages only (the
bootstrap ciphertext on the first message is deliberately NOT reprocessed
here, since `init_as_receiver_hybrid` already consumes it at session
setup). Full workspace compile gate green, all existing tests pass.
REMAINING GAP: no test exercises the 100-message cadence path itself (see
`HANDOFF/todo/PQC_07_CADENCE_TEST_COVERAGE.md`) -- treat this finding as
"fix landed, not yet proven by a dedicated test" until that lands. A
second adversarial pass should re-review the actual wiring code once that
test exists, since the fix was hand-applied by the orchestrator after two
Qwen dispatch failures (truncation, then a diff that didn't apply) rather
than a clean model-generated-and-verified round trip.

## CONFIRMED CRITICAL FINDING

**PQC-07's PQ ratchet is implemented but never wired into the live
encrypt/decrypt path.**

- `encrypt_message_ratcheted` (`core/src/crypto/encrypt.rs:330-378`) never
  calls `RatchetSession::perform_pq_ratchet_step()`. It only ever sends the
  ORIGINAL bootstrap ML-KEM ciphertext (`session.bootstrap_hct`) on the
  first message before `peer_confirmed`; every subsequent message sends
  `(None, None)` for `pq_kem_ciphertext`/`pq_encaps_key` (line 341, 346).
- `decrypt_message_ratcheted_v2` (`core/src/crypto/encrypt.rs:276-313`)
  never reads `envelope.pq_kem_ciphertext`/`pq_encaps_key` and never calls
  `session.handle_incoming_pq_fields(...)`. It calls `session.decrypt(...)`
  directly with no PQ-field handling at all.
- Consequence: `self.pq_ss` (the ML-KEM shared secret mixed into
  `root_key_ratchet_v2` on every DH ratchet step, `ratchet.rs`
  `handle_dh_ratchet`) is set ONCE at session bootstrap
  (`init_as_sender_hybrid`/`init_as_receiver_hybrid`) and never refreshed.
  Every later DH ratchet step reuses the SAME initial ML-KEM shared secret
  via `self.pq_ss.clone()` — there is no periodic PQ re-encapsulation in
  the live path, despite `perform_pq_ratchet_step`/
  `handle_incoming_pq_fields` existing, compiling, and passing their OWN
  isolated unit tests.
- Practical impact: the hybrid session gets ONE dose of post-quantum
  protection at bootstrap. It does not get the PQ-forward-secrecy property
  PQC-07 was meant to add (an attacker who later breaks ML-KEM-768 and also
  learns the bootstrap ciphertext can compute the same `pq_ss` that EVERY
  subsequent ratchet step in the session's lifetime still depends on).
  Classical (X25519) forward secrecy is unaffected — this is specifically
  a PQ-forward-secrecy gap, not a break of the whole session.
- Fix: wire `perform_pq_ratchet_step` into `encrypt_message_ratcheted` (call
  it on a cadence -- e.g. every DH ratchet step or every N messages -- and
  place the returned ciphertext/encaps-key into the envelope every time,
  not just pre-confirmation) and wire `handle_incoming_pq_fields` into
  `decrypt_message_ratcheted_v2` (extract `pq_kem_ciphertext`/
  `pq_encaps_key` from the envelope, call it BEFORE `session.decrypt`, and
  thread the result so `handle_dh_ratchet` sees the fresh secret, not the
  stale bootstrap one).
- Follow-up implementation task: `HANDOFF/todo/PQC_07_WIRE_RATCHET_STEP.md`.

## OTHER FINDINGS (MEDIUM/LOW, from Qwen Part A — spot-checked, mostly hold)

- MEDIUM (session_manager.rs `load()`, ~line 60-62, NOT line 120 as
  originally cited): the `json` String read from the sled backend and
  passed to `deserialize_sessions(&json)` is not zeroized after use, even
  though it contains hex-encoded session secrets. Minor hygiene gap.
- MEDIUM (session_manager.rs, `bootstrap_hct` persistence): serialized
  sessions retain `bootstrap_hct` (containing the ML-KEM ciphertext) even
  after handshake completion / `peer_confirmed`. Not independently
  verified against a specific line this session — re-check before acting.
- LOW (pq/mod.rs:65, encapsulation randomness zeroization): not
  independently re-verified this session.
- FALSE POSITIVE (originally cited as LOW, "identity_secret bytes buffer
  not zeroized before copy"): verified FALSE. `session_manager.rs:410-421`
  already calls `bytes.zeroize()` (line 418) and `arr.zeroize()` (line 420)
  correctly. No action needed.

## OTHER FINDINGS (from OpenRouter Part B — not yet independently verified
against current source; treat as PLAUSIBLE pending spot-check before acting)

- HIGH: `handle_incoming_pq_fields` lacks upfront ciphertext-length
  validation before attempting decapsulation in some call order, and
  decapsulation may not be constant-time (implicit rejection). Needs
  re-check: production code path currently under discussion (this
  function is itself never called live per the CRITICAL finding above, so
  severity is somewhat theoretical until the wiring fix lands — re-audit
  the wired version once PQC_07_WIRE_RATCHET_STEP is implemented).
- HIGH: `RatchetSession::reconstruct()`/serialization does not persist
  `skipped_keys` — after a process restart, previously-skipped
  out-of-order message keys are lost, which could cause legitimate
  messages to become undecryptable (availability issue, not
  confidentiality).
- HIGH: downgrade risk when `require_pq=false` (the default) — a
  first-message-suppression MITM could potentially force a v1-only
  session between two v2-capable peers. Needs verification against the
  actual `should_use_ratcheted_encryption`/negotiation code (PQC-08 already
  fixed one bug in this area today; re-verify this specific claim before
  treating as confirmed).
- MEDIUM: `get_message_key`'s skipped-key eviction uses `keys().min()`
  (lexicographic on the tuple key), not true LRU/insertion-order eviction.
- LOW: `init_as_receiver_hybrid` clones `our_mlkem_keypair` into the
  session rather than moving it, unnecessarily duplicating potentially
  sensitive key material in memory.

## MISSING TESTS (combined, both reviewers agree on the shape of this list)

- Downgrade attack: two suite-0x02-capable peers forced to suite 0x01.
- PQ shared secret actually changes derived root keys across a ratchet
  step (currently only tested at session-bootstrap time).
- Tampered/bit-flipped `pq_kem_ciphertext` is rejected, not silently
  accepted or causing wrong-but-successful decryption.
- Session restore after restart does not lose skipped keys.
- Both directions of a full `perform_pq_ratchet_step` /
  `handle_incoming_pq_fields` round trip END TO END through the wired
  encrypt/decrypt functions (not just the isolated ratchet.rs unit tests).
- `validate_pq_fields_present` is actually enforced somewhere in the live
  decrypt path (currently appears to be unused/orphaned, same as the two
  PQ ratchet functions above — verify during the wiring fix).

## Gating decision

PQC-09/10 (hybrid onion, ML-DSA identity) do not depend on the ratchet
wiring gap and MAY proceed in parallel. PQC-11 (relay invite hybrid auth)
and PQC-13 (verification suite) MUST wait for
`PQC_07_WIRE_RATCHET_STEP.md` to land and pass its own gate + a follow-up
review pass, since they build on/verify the ratchet's actual security
properties. PQC-08 (legacy retirement) is otherwise unaffected by this
finding (it gates classical fallback, not the PQ ratchet cadence) and its
own bugs (found and fixed this session) are separate from this gap.
