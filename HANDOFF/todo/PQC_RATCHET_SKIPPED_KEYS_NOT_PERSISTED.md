# TASK [MEDIUM]: Skipped ratchet keys don't survive session persistence

Status: TODO. Found 2026-07-12 during the PQC-05/06/07 adversarial review
checkpoint (`HANDOFF/review/PQC_05_06_07_ADVERSARIAL_REVIEW.md`), confirmed
by direct source read (not just the reviewing model's claim).

## The gap

`RatchetSession::reconstruct` (`core/src/crypto/ratchet.rs:158-185`) has no
`skipped_keys` parameter at all, and unconditionally initializes it as
`HashMap::new()` (line 185). `core/src/crypto/session_manager.rs` never
mentions `skipped_keys` anywhere (zero grep hits) — it was never added to
the persisted session format in the first place, so this isn't a case of
`reconstruct` discarding stored data; the data simply never gets that far.

## Why it matters

`skipped_keys` holds message keys for out-of-order messages the ratchet
has already advanced past (`ratchet.rs:610,631-636`, capped at
`MAX_SKIP_KEYS`). Every time a session gets persisted and reloaded (process
restart, app backgrounding on mobile, etc.), any key currently held there is
permanently lost. A message that was legitimately delayed in transit and
arrives after the reload will have no matching skipped key to decrypt with
and gets treated as behind the current chain position -- a real message
silently becomes undeliverable, not because of a transport bug but because
of this persistence gap.

## Fix direction

1. Add `skipped_keys` to whatever the serialized session-state format is in
   `session_manager.rs` (need to check the actual struct that gets
   `serde_json`'d for persistence -- this ticket doesn't yet know its name;
   first step is finding it via `rg -n "struct.*Session.*Serialize|derive.*Serialize" core/src/crypto/session_manager.rs`).
2. Add a `skipped_keys` parameter to `reconstruct` and restore it from the
   persisted value instead of hardcoding `HashMap::new()`.
3. Consider whether skipped key MATERIAL should be persisted at all from a
   security-hygiene standpoint (message keys are sensitive; persisting them
   to disk extends their exposure window) vs. accepting the current
   behavior (lose them on restart) as an intentional security/availability
   tradeoff -- if the latter, this ticket should be closed as WONTFIX with
   that reasoning recorded, not silently ignored. Escalate this specific
   tradeoff question to the operator if genuinely unsure; don't guess.
4. Add a regression test: skip a message (advance the ratchet past it),
   persist + reconstruct the session, then decrypt the skipped message and
   confirm it still succeeds.

## Gate

Touches `core/src/crypto/` -- mandatory adversarial review before merge per
`.claude/rules/security.md`, same as any ratchet/session-state change.
