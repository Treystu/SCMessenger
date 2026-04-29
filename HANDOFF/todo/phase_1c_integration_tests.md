# Phase 1C: Integration Test Pass

**Priority:** P0
**Assigned Agent:** rust-coder (glm-5.1:cloud)
**Fallback:** precision-validator (deepseek-v3.2:cloud)
**Status:** TODO
**Depends On:** phase_1b_core_module_wiring

## Objective
All integration tests pass.

## Integration Tests to Verify
```bash
cargo test -p scmessenger-core --test integration_e2e
cargo test -p scmessenger-core --test integration_contact_block
cargo test -p scmessenger-core --test integration_offline_partition_matrix
cargo test -p scmessenger-core --test integration_ironcore_roundtrip
cargo test -p scmessenger-core --test integration_registration_protocol
cargo test -p scmessenger-core --test integration_nat_reflection
cargo test -p scmessenger-core --test integration_relay_custody
cargo test -p scmessenger-core --test integration_retry_lifecycle
cargo test -p scmessenger-core --test integration_receipt_convergence
cargo test -p scmessenger-core --test integration_all_phases
cargo test -p scmessenger-core --test test_address_observation
cargo test -p scmessenger-core --test test_multiport
cargo test -p scmessenger-core --test test_persistence_restart
cargo test -p scmessenger-core --test test_mesh_routing
```

## Success Criteria
- `cargo test --workspace` exits 0
- All named integration tests pass individually
- No flaky test failures on retry

## Rules
- Test naming convention: `integration_<domain>_<scenario>`
- Property-based testing: `proptest` harness in `core/src/crypto/proptest_harness.rs`
- New features require: unit test + integration test + property test (for crypto/routing)
