# SCMessenger Documentation Hub

This is the documentation index and governance map for the repository.

## Canonical Current Docs (Read First)

- [Repository Overview](README.md)
- [Repository Context (Cross-Component Map)](docs/REPO_CONTEXT.md)
- [Current Verified State](docs/CURRENT_STATE.md)
- [Active Gap Backlog](REMAINING_WORK_TRACKING.md)
- [Global Rollout Plan](docs/GLOBAL_ROLLOUT_PLAN.md)
- [Unified Global App Plan (Android+iOS+Web)](docs/UNIFIED_GLOBAL_APP_PLAN.md)
- [Full File Documentation Tracker](docs/DOC_PASS_TRACKER.md)
- [Non-Markdown Followup Classification](docs/NON_MD_FOLLOWUP_CLASSIFICATION.md)
- [Implementation Path Buckets](docs/IMPLEMENTATION_PATH_BUCKETS.md)
- [Triple-Check Verification Report](docs/TRIPLE_CHECK_REPORT.md)
- [High-Impact Section Action Matrix](docs/HIGH_IMPACT_SECTION_ACTIONS.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Transport Architecture](docs/TRANSPORT_ARCHITECTURE.md)
- [Protocol Specification](docs/PROTOCOL.md)
- [Testing Guide](docs/TESTING_GUIDE.md)

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
- [NAT Traversal Guide](docs/NAT_TRAVERSAL_GUIDE.md)
- [SwarmBridge Integration Notes](docs/SWARMBRIDGE_INTEGRATION.md)
- [Bootstrap Notes](BOOTSTRAP.md)

## Development and Policy

- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Network Testing Notes](NETWORK_TESTING.md)

## Historical-Heavy Docs (Mixed-Status, Non-Canonical)

Use these as mixed-status context. Do not treat the entire file as current or deprecated by default; evaluate sections individually.

- `PRODUCTION_READY.md`
- `INTEGRATION_COMPLETE.md`
- `INTEGRATION_VERIFICATION.md`
- `ANDROID_BUILD_RESOLUTION.md`
- `ANDROID_DISCOVERY_ISSUES.md`
- `AUDIT_QUICK_REFERENCE.md`
- `AUDIT_RESOLUTIONS.md`
- `BRANCH_AUDIT_REPORT.md`
- `DOCKER_TEST_SETUP_COMPLETE.md`
- `DOCKER_TEST_QUICKREF.md`
- `DOCKER_QUICKSTART.md`
- `docker/IMPLEMENTATION_SUMMARY.md`
- `FEATURE_PARITY.md`
- `FEATURE_WORKFLOW.md`
- `HANDOFF_NEARBY_PEERS.md`
- `NAT_REFACTOR_PLAN.md`
- `SOVEREIGN_MESH_PLAN.md`
- `DRIFTNET_MESH_BLUEPRINT.md`
- `SECURITY_AUDIT_NOTES.md`
- `docs/REMEDIATION_PLAN.md`
- `android/IMPLEMENTATION_STATUS.md`
- `iOS/IMPLEMENTATION_STATUS.md`
- `iOS/IMPLEMENTATION_SUMMARY.md`
- `iOS/FINAL_STATUS.md`
- `iOS/COMPLETE_STATUS.md`
- `iOS/PHASE4_IMPLEMENTATION.md`
- `iOS/PHASES_4-15_GUIDE.md`
- `iOS/PLAN_REVIEW.md`

## Documentation Placement Rules

- Cross-cutting architecture, protocol, and verification docs belong in `docs/`.
- Package-specific usage docs belong in local module READMEs (`core/`, `cli/`, `mobile/`, `wasm/`, `android/`, `iOS/`), with active iOS app code under `iOS/SCMessenger/`.
- Path-case governance: `iOS/` is the only canonical iOS root path for docs, scripts, and references; do not introduce new `ios/` references.
- Backlog and delivery tracking belongs in `REMAINING_WORK_TRACKING.md`.
- Repo-wide rollout and execution plans belong in `docs/GLOBAL_ROLLOUT_PLAN.md` and `docs/UNIFIED_GLOBAL_APP_PLAN.md`, and should reference `docs/DOC_PASS_TRACKER.md`.
- If a document becomes outdated, update canonical docs first, then keep the old file marked as historical rather than creating a competing "final" document.

## Reference

- [Legacy Crypto Porting Reference](reference/README.md)
