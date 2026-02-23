# SCMessenger Triple-Check Report

Last run: **2026-02-23**

This report captures the three-pass repository verification requested for full-context planning.

## Pass 1: Full Inventory Coverage

Scope note: inventory uses `rg --files` (source-relevant files) and excludes transient hidden build outputs such as `.build/`.

- Total tracked files (`rg --files`): **357**
- Tracker entries (`docs/DOC_PASS_TRACKER.md`): **357**
- Checked entries: **357**
- Pending entries: **0**

Directory counts:

- `core/`: 88
- `android/`: 90
- `iOS/`: 55
- `docs/`: 22
- `docker/`: 21
- `cli/`: 11
- `mobile/`: 4
- `wasm/`: 8
- `ui/`: 1
- `reference/`: 7
- `scripts/`: 7

## Pass 2: Global Risk Scan

Pattern set:

- `TODO|FIXME|XXX|WIP|HACK|panic!|unimplemented!|fatalError(|try!|as!|@Ignore|ignored|placeholder|DEPRECATED|deprecated`

Results:

- Total hits: **279**
- Highest-concentration roots:
  - `iOS`: 98
  - `core`: 29
  - `android`: 23
  - `docs`: 47
  - `docker`: 17

Interpretation:

- Most iOS hits are expected in generated UniFFI files (`Generated/api.swift`) and historical/reference docs.
- Most core hits are test assertions (`panic!` in test match arms), not production runtime panics.
- Android high-value hits remain around test `@Ignore` status mismatch and a small number of TODOs.

TODO/FIXME sync outcome:

- Canonical backlog was refreshed to include a recurring TODO/FIXME accuracy sync pass:
  - `REMAINING_WORK_TRACKING.md` -> Priority 1, item 16.

## Pass 3: Cross-Platform Parity Signal Scan

Keyword distribution (`core`, `android`, `iOS`, `wasm`, `ui`, `cli`, `mobile`, `docs`):

- `relay`: `1455, 167, 390, 144, 4, 36, 0, 171`
- `bootstrap`: `162, 53, 117, 0, 3, 135, 0, 116`
- `public_key_hex`: `52, 1, 6, 8, 0, 15, 2, 12`
- `identity_id`: `40, 9, 5, 2, 0, 11, 1, 8`
- `libp2p_peer_id`: `9, 17, 14, 0, 0, 2, 0, 7`
- `privacy`: `9, 18, 72, 1, 0, 0, 0, 45`
- `qr`: `17, 68, 79, 0, 0, 0, 0, 9`
- `history`: `16, 71, 291, 0, 1, 74, 0, 45`

Interpretation:

- Relay/bootstrap semantics are implemented broadly across core + mobile app layers.
- Identity fields remain mixed (`public_key_hex`, `identity_id`, `libp2p_peer_id`) and still need canonicalization hardening.
- Web/WASM surface is present but materially thinner than Android/iOS app surfaces.
- History and QR flows are strongly represented on Android/iOS.

## Confirmed Outcome

- Repository-wide file coverage is complete (**357/357 checked**).
- No unreviewed tracker entries remain.
- Remaining work is implementation/parity/operational hardening, not discovery.

Canonical execution plan: `docs/UNIFIED_GLOBAL_APP_PLAN.md`
