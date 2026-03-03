# SCMessenger Documentation Hub

Status: Active  
Last updated: 2026-03-03

This is the primary documentation entrypoint.

For lifecycle classification (`Active`, `Planned`, `Mixed`, `Historical`, `Superseded`), use:

- [Document Status Index](docs/DOCUMENT_STATUS_INDEX.md)

## Active Canonical Docs (Read First)

- [Repository Overview](README.md)
- [Repository Context](docs/REPO_CONTEXT.md)
- [Current Verified State](docs/CURRENT_STATE.md)
- [Active Backlog](REMAINING_WORK_TRACKING.md)
- [v0.2.0 Milestone Plan](docs/MILESTONE_PLAN_V0.2.0_ALPHA.md)
- [v0.2.0 Phase Prompts](docs/V0.2.0_PHASE_EXECUTION_PROMPTS.md)
- [v0.2.0 Residual Risk Register](docs/V0.2.0_RESIDUAL_RISK_REGISTER.md)
- [Platform Support Matrix](docs/PLATFORM_SUPPORT_MATRIX.md)
- [Protocol Specification](docs/PROTOCOL.md)
- [Testing Guide](docs/TESTING_GUIDE.md)
- [Edge-Case Readiness Matrix](docs/EDGE_CASE_READINESS_MATRIX.md)

## Planned Docs (Future Scope)

- [v0.2.1 Tight Pair / WS13 Plan](docs/V0.2.1_SINGLE_ACTIVE_DEVICE_TIGHT_PAIR_PLAN.md)

## Component and Platform Docs

- [Core Crate](core/README.md)
- [CLI Crate](cli/README.md)
- [Mobile Bindings Crate](mobile/README.md)
- [WASM Bindings Crate](wasm/README.md)
- [Android App](android/README.md)
- [iOS App](iOS/README.md)

## Operations and Setup

- [Install](INSTALL.md)
- [Quick Connect](QUICKCONNECT.md)
- [Docker](docker/README.md)
- [GCP Deploy](GCP_DEPLOY_GUIDE.md)
- [Relay Operator Guide](docs/RELAY_OPERATOR_GUIDE.md)
- [Bootstrap Governance](docs/BOOTSTRAP_GOVERNANCE.md)
- [NAT Traversal Guide](docs/NAT_TRAVERSAL_GUIDE.md)

## Mixed and Historical Context

Use these for context only; do not treat as execution source of truth unless revalidated:

- [Transport Architecture (mixed-status)](docs/TRANSPORT_ARCHITECTURE.md)
- [Global Rollout Plan (strategy)](docs/GLOBAL_ROLLOUT_PLAN.md)
- [Unified Global App Plan (strategy)](docs/UNIFIED_GLOBAL_APP_PLAN.md)
- [Historical docs index](docs/historical/README.md)

Historical audit artifacts currently live under `docs/historical/`.

## Documentation Governance Rules

1. Execution truth must come from Active docs listed above.
2. Backlog updates must go to `REMAINING_WORK_TRACKING.md`.
3. Residual risk updates must go to `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` (or the active release's equivalent register).
4. Superseded status/audit reports should be moved to or referenced from `docs/historical/`, not duplicated as new "final" docs.
5. Use `iOS/` (uppercase-I) for all path references.
6. Run `./scripts/docs_sync_check.sh` before finalizing task output; fix failures in the same run.
