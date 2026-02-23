# SCMessenger Triple-Check Report

Last run: **2026-02-23**

This report captures the three-pass repository verification requested for full-context planning.

## Pass 1: Full Inventory Coverage

Scope note: inventory uses `rg --files` (source-relevant files) and excludes transient hidden build outputs such as `.build/`.

- Total tracked files (`rg --files`): **392**
- Tracker entries (`docs/DOC_PASS_TRACKER.md`): **392**
- Checked entries: **392**
- Pending entries: **0**

Directory counts:

- `core/`: 90
- `android/`: 90
- `iOS/`: 88
- `docs/`: 19
- `docker/`: 21
- `cli/`: 12
- `mobile/`: 4
- `wasm/`: 8
- `ui/`: 1
- `reference/`: 9
- `scripts/`: 6

## Pass 2: Global Risk Scan

Pattern set:

- `TODO|FIXME|XXX|WIP|HACK|panic!|unimplemented!|fatalError(|try!|as!|@Ignore|ignored|placeholder|DEPRECATED|deprecated`

Results:

- Total hits: **357**
- Highest-concentration roots:
  - `iOS`: 192
  - `core`: 29
  - `android`: 25
  - `docs`: 20
  - `docker`: 17

Interpretation:

- Most iOS hits are expected in generated UniFFI files (`Generated/api.swift`) and historical/reference docs.
- Most core hits are test assertions (`panic!` in test match arms), not production runtime panics.
- Android high-value hits remain around test `@Ignore` status mismatch and a small number of TODOs.

## Pass 3: Cross-Platform Parity Signal Scan

Keyword distribution (`core`, `android`, `iOS`, `wasm`, `ui`, `cli`, `mobile`, `docs`):

- `relay`: `1574, 167, 499, 129, 4, 20, 0, 142`
- `bootstrap`: `141, 44, 50, 0, 3, 120, 0, 106`
- `public_key_hex`: `45, 1, 9, 8, 0, 22, 2, 12`
- `identity_id`: `37, 9, 6, 2, 0, 14, 1, 8`
- `libp2p_peer_id`: `6, 17, 16, 0, 0, 2, 0, 7`
- `privacy`: `29, 18, 85, 1, 0, 0, 0, 35`
- `qr`: `16, 68, 81, 0, 0, 0, 0, 9`
- `history`: `15, 71, 428, 0, 1, 67, 0, 42`

Interpretation:

- Relay/bootstrap semantics are implemented broadly across core + mobile app layers.
- Identity fields remain mixed (`public_key_hex`, `identity_id`, `libp2p_peer_id`) and still need canonicalization hardening.
- Web/WASM surface is present but materially thinner than Android/iOS app surfaces.
- History and QR flows are strongly represented on Android/iOS.

## Confirmed Outcome

- Repository-wide file coverage is complete (**392/392 checked**).
- No unreviewed tracker entries remain.
- Remaining work is implementation/parity/operational hardening, not discovery.

Canonical execution plan: `docs/UNIFIED_GLOBAL_APP_PLAN.md`
