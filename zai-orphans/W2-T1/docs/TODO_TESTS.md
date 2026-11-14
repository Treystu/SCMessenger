# TODO_TESTS.md

This file tracks ignored/skipped tests in the SCMessenger test suite.

## Core Ignored Tests

The following tests are marked `#[ignore]` and need to be addressed:

### `scmessenger-core`

1. `drift::sync::tests::test_sync_large_symmetric_difference`
2. `transport::nat::tests::test_detect_nat_type_with_peers`
3. `transport::nat::tests::test_get_external_address_from_peer`
4. `transport::nat::tests::test_get_hole_punch_status`
5. `transport::nat::tests::test_hole_punch_disabled`
6. `transport::nat::tests::test_hole_punch_start`
7. `transport::nat::tests::test_peer_discovery_no_peers`
8. `transport::nat::tests::test_probe_nat`

Additionally, note that the test `tests::test_mismatched_sender_receipt_is_ignored` is not ignored but has "ignored" in its name; it is actually run.

## Next Steps

- Each ignored test should be investigated and either fixed, removed, or have the ignore reason documented.
- Consider if these tests are ignored due to flakiness, environment dependencies (e.g., NAT simulation), or performance.
- Plan to re‑enable them as part of the v0.2.1 stabilization.

## Last Updated

2026‑04‑13 (after workspace deep‑clean)