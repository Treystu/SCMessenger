# Wave H -- Human-Only Gates

Status: Active. Last updated: 2026-07-17.

These items cannot be delegated to any agent. Each requires direct operator
action. No agent may move these to done/ without a recorded operator
confirmation.

| ID   | Gate                             | Operator Action Required                                                                                                                              | Unblocks                                         |
|------|----------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------|
| H-01 | GitHub Actions billing           | Fix billing on personal account OR transfer repo to Sovereign-Communication org with Enterprise trial                                                  | D-07 (CI runners), D-03 (iOS xcodebuild), iOS lane |
| H-02 | Physical two-device field trials | WiFi Aware two-Android test; BLE sneakernet (tractor route) test; results logged to HANDOFF/review/                                                    | T-02 final verification, Farm FD-2/FD-3 drills   |
| H-03 | Three P1-10 sign-offs            | Approve: (1) peer_exchange semantics for GroupInfo sharing, (2) GroupInfo.port FFI field addition, (3) transport_memory privacy fingerprint design     | C-02, C-03, C-04 (adaptive port selection)       |
| H-04 | AWS relay resume                 | Decide: activate B4 cloud relay (run infra/aws/setup_credentials.sh) or defer to v1.1                                                                 | B4 AWS rig, 12-node farm-sim on real WAN          |
| H-05 | Final release sign-off           | All waves A/D/C/E complete; farm FD drills logged; PQC adversarial reviews on file; release notes approved; Apple Developer account decision resolved  | v1.0.0 tag                                       |

## Notes

- H-03 is the only gate that blocks code work (C-02/C-03/C-04 adaptive port
  implementation). All other H gates are terminal or parallel to active work.
- H-01 is on the critical path for the iOS lane; the rest of v1.0.0 is not
  blocked by CI availability.
- H-04 and H-05 are sequenced last; no agent work is blocked on them until
  near release.

Status: Pending operator action. No agent may act on these items.
