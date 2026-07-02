# T5.8 — Top-level README, LICENSE, CHANGELOG, agent map

**Status:** completed
**Track:** 5 (CI/CD, FFI Stability & Repo Hygiene)
**Dependencies:** T5.1
**Blocks:** none

## Technical Context
- Module READMEs exist (`core/`, `cli/`, `wasm/`, `ios/`, `android/`) but no root docs
- Workspace says MIT but no LICENSE file

## Implementation
1. Root `README.md` (architecture diagram from the audit: bridge -> IronCore -> message/crypto/routing/transport/drift/store layers; build prerequisites incl. NDK env var from T5.2)
2. `LICENSE` (MIT)
3. `CHANGELOG.md` seeded at 0.3.4
4. `ARCHITECTURE.md` mapping every subsystem to its files — this is the swarm agents' navigation chart

## Edge Cases
- Keep claims accurate to code (no aspirational features — acoustic is explicitly listed as post-v1.0 in a roadmap section)

## Verification
- [x] `test -f README.md LICENSE CHANGELOG.md ARCHITECTURE.md`
- [x] Every path referenced in ARCHITECTURE.md exists (scriptable link-check)
