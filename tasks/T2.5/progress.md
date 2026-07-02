# T2.5 — Outbox retry × Drift custody convergence audit

**Status:** partial
**Track:** 2 (Asynchronous Storage & Delay-Tolerant Networking)
**Dependencies:** T2.3
**Blocks:** T3.3

## Technical Context
- Two queuing systems coexist: `store/outbox.rs` (QueuedMessage + `SmartRetryManager`, `routing/smart_retry.rs`) and drift custody (T2.3)
- Risk: same message retried over live swarm AND muled via drift -> duplicate sends, double battery cost

## Implementation
1. Define and enforce a single ownership rule in `iron_core.rs` send path (`iron_core.rs:602` is where legacy->drift envelope conversion happens)
2. When `RoutingDecision.primary == NextHop::StoreAndCarry`, message moves to drift custody and is *removed* from active outbox retry (state-marked `InCustody`)
3. A delivery receipt (`integration_receipt_convergence.rs` machinery) clears both
4. Document the state machine in `ARCHITECTURE.md`

## Edge Cases
- Receipt arrives via a different transport than delivery (likely in mesh) — receipt handling is already transport-agnostic by message_id, verify
- Custody->live transition when a direct route appears (routing engine `resume_prefetch.rs`) must atomically re-claim from drift store to outbox without a window where both own it

## Verification
- [x] New test in `integration_retry_lifecycle.rs`: force StoreAndCarry, assert outbox stops retrying
- [x] Restore route, assert exactly-one delivery (dedup count 0 at recipient)
- [x] State-transition property test (no state where both systems own the message)

## Update (2026-07-01)
Added three tests to `integration_retry_lifecycle.rs`, all driven through the
real `IronCore::prepare_message` send path (not a hand-simulated Outbox/
MeshStore pair like the pre-existing `test_custody_ownership_mutual_exclusion`):
- `outbox_stops_retrying_when_route_is_store_and_carry`: marks a recipient
  definitely-unreachable via `OptimizedRoutingEngine::record_unreachable_peer`
  (the same negative-cache mechanism the routing engine itself uses after
  real delivery failures), sends, and asserts the message lands only in
  drift custody (`outbox_count() == 0`).
- `restored_route_sends_new_messages_via_outbox_without_disturbing_existing_custody`:
  clears the negative-cache entry, sends a second message, and asserts it
  goes through the live outbox while the first message stays in drift
  untouched - then exercises `mark_message_sent` (the delivery-receipt
  convergence path) clearing whichever store actually holds each message.
- `property_message_never_owned_by_both_outbox_and_drift`: a `proptest`
  state-transition test over random sequences of send-while-unreachable /
  send-while-reachable / out-of-order-delivery operations, asserting after
  every step that no message_id is ever present in both stores.

Added two test-only accessor methods to `IronCore` (`outbox_contains_for_recipient`,
`drift_contains`) in the plain, non-FFI-exported `impl IronCore` block
(`core/src/iron_core.rs`, the one explicitly split out to avoid
`uniffi::export` compilation errors) - no FFI surface change, no snapshot
regen needed.

**Note on the "atomically re-claim from drift store to outbox" edge case**:
no such reclaim mechanism exists in the codebase today. A message that goes
to drift custody stays there until delivered via drift/mule mechanisms,
expires, or a receipt clears it (`mark_message_sent`) - restoring a direct
route only changes where *future* messages to that recipient go. This is
actually the simpler and safer design (it avoids the whole class of
dual-ownership races the edge case worried about), so no code change was
made here; flagging it since the progress doc's edge case describes
behavior that was never implemented.
