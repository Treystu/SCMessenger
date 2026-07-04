# TASK: PQC-07 — Post-quantum ratchet steps (hybrid forward secrecy + post-compromise security)

Read `PQC_00_MASTER_PLAN.md` first. Depends on: PQC-06. Wave 3. Min tier: Sonnet ONLY — do not assign to Haiku. ADVERSARIAL REVIEW MANDATORY + release-gatekeeper before merge. This is the highest-risk task in the workstream.

## Why

After PQC-06, only the session ROOT is hybrid; ongoing forward secrecy still comes solely from X25519 DH ratchet steps (audit F2: a recorded transcript falls to Shor per-step). This task injects fresh ML-KEM secrets into the root chain during the conversation, so compromise windows are bounded by BOTH assumptions. Model: Signal's Triple Ratchet/SPQR, deliberately simplified (see "Deviation" below).

## Design (fixed)

State added to `RatchetSession` (suite 0x02 only):

```
pq_our_keypair:      MlKem768KeyPair          // rotates every PQ step we initiate
pq_their_encaps_key: Option<Vec<u8>>          // latest advertised by peer (starts: their bundle key)
pq_pending_ct:       Option<Vec<u8>>          // ct we must keep sending until peer acks (lossy mesh)
```

Piggyback on the EXISTING DH ratchet step (the point in `ratchet.rs` where a received new `ratchet_dh_public` triggers `root_key` advance — locate with `rg -n "dh_ratchet|root" core/src/crypto/ratchet.rs`):

- **When we perform a sending DH step:** also (a) `(ct, ss_pq) = encapsulate(pq_their_encaps_key)` [plain ML-KEM from PQC-01 — the classical half of this step is the DH output itself], (b) generate fresh `pq_our_keypair`, (c) root update becomes `root' = derive_key(ROOT_KDF_CONTEXT_V2, root || dh_output || ss_pq)` instead of `root || dh_output`, (d) outgoing envelopes carry `pq_kem_ciphertext = ct` and `pq_encaps_key = new encaps key` until acked.
- **When we receive a DH step envelope carrying pq fields:** decapsulate with OUR current `pq_our_keypair` — CAREFUL: the ct was made for the keypair we advertised LAST step; retain exactly one previous keypair (`pq_prev_keypair`) to absorb one-step-behind arrivals, mirroring how the DH ratchet tolerates one outstanding step; update `pq_their_encaps_key` from the envelope; apply the same root formula.
- **Ack semantics:** receiving any envelope from the peer whose root reflects our last PQ step (i.e., it decrypts under the new chains) clears `pq_pending_ct`.
- If a suite-0x02 envelope performs a DH step WITHOUT pq fields while `pq_their_encaps_key` is Some: reject the step (protocol violation — a PQ-stripping attempt), fail decryption, log audit event.
- New context string `ROOT_KDF_CONTEXT_V2 = "iron-core root-chain v2 hybrid 2026-07"`. Suite 0x01 sessions keep the v1 context and old formula untouched.

## Deviation from SPQR (documented, approved in audit)

SPQR chunk/erasure-codes encapsulation keys across many messages to keep per-message overhead tiny. We instead attach full PQ material (1184+1088 B) ONLY on DH-step envelopes (a small fraction of traffic). Acceptable because drift framing already fragments large payloads for BLE (verify: `rg -n "fragment|mtu|chunk" core/src/drift --type rust -i`). If review finds BLE-path envelopes with PQ fields failing to deliver, STOP and escalate — do not invent chunking ad hoc.

## Tests (all required)

1. Extend `integration_pq_session.rs`: long interleaved conversation (200+ messages, alternating direction every 1-30 messages, forcing many DH+PQ steps) — all decrypt.
2. Out-of-order and dropped envelopes across a PQ step boundary (skipped-key logic must still work; one-step-behind ct absorbed by `pq_prev_keypair`).
3. PQ-strip rejection test: hand-craft a DH-step envelope without pq fields on a 0x02 session -> decryption fails + audit event.
4. Transcript decryption resistance sanity: serialize a whole session transcript; assert plaintext recovery requires the session state (i.e., a fresh session from the same identity bundles CANNOT decrypt mid-conversation envelopes — this pins forward secrecy behavior).
5. Persistence: save/load mid-conversation across a pending (un-acked) PQ step; conversation continues.
6. Proptest: random step/drop/reorder schedules never panic and never desync (both sides end able to exchange).

## Definition of Done

- [ ] Standard gates PASS.
- [ ] All tests above green; `integration_e2e`, `integration_retry_lifecycle`, `integration_offline_partition_matrix` still green (ratchet interacts with retry/offline paths).
- [ ] Adversarial review verdict + release-gatekeeper verdict recorded here.
- [ ] File moved to HANDOFF/done/ + committed.

## Do NOT

- Change v1 (suite 0x01) ratchet behavior in any observable way.
- Batch this task with PQC-08 in one commit — land and review separately.
- Deviate from the root formula or contexts without human sign-off (PQC-00 rule 8).
