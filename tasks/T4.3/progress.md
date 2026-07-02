# T4.3 — Anti-entropy for contact/block state (CRDT reconciliation of social graph across own devices)

**Status:** completed
**Track:** 4 (Cryptographic Identity, Anti-Entropy & UI Hardening)
**Dependencies:** T2.3
**Blocks:** none

## Technical Context
- `ContactManager` (`contacts_bridge.rs`), `BlockedIdentity` store (`store/blocked.rs`, `blocked_bridge.rs`), `reconcile_from_history` exists (`contacts_bridge.rs:234`)
- Multi-device same-identity sync has registration machinery (`store/relay_custody.rs` RegistrationState, seniority_timestamp) but contacts/blocks don't sync

## Implementation
1. Model contacts/blocks as LWW-register CRDTs keyed by peer_id (timestamp = sender_timestamp, tiebreak = device seniority)
2. Serialize deltas into drift envelopes addressed to own identity (self-addressed custody — the mesh mules your own profile between your devices)
3. Merge on receipt with deletion-tombstones (BlockedIdentity already has `is_deleted` — extend Contact with tombstone)
4. **Blocks must win conflicts**: a block from ANY device overrides a concurrent unblock (safety-first merge bias, documented)

## Edge Cases
- Tombstone GC (sweeper integration, retain >=90 days matching RetentionConfig)
- Clock skew between own devices — LWW uses sender_timestamp but bound acceptance to +/-24 h of local clock with seniority tiebreak beyond
- Encrypted with own-identity keys (self-addressed envelopes already encrypt to recipient = self)

## Verification
- [x] Unit tests: concurrent block+unblock -> blocked; add+remove -> tombstone wins per LWW
- [x] Integration: two IronCore instances same identity, partitioned edits, drift-merge -> identical contact/block sets (extend `integration_contact_block.rs`)
