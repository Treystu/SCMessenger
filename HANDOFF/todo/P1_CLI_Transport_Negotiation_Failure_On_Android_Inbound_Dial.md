# TASK: P1-CLI-TRANSPORT — Windows CLI fails to negotiate transport protocol on inbound dials from Android

## Context

Found during a live LAN discovery test (2026-07-04): a Windows CLI daemon
(`scmessenger-cli.exe`, built from current `main`, release profile) was
listening on `192.168.0.121:9001` (raw TCP) and `:9002/ws` (WebSocket). A
physical Pixel 6a (`192.168.0.148`) on the same private WiFi correctly
discovered the Windows node via mDNS (`libp2p_mdns` on the Windows side
logged `mDNS discovered peer: 12D3KooWJJmBsLVA1rPsuPY6xWMSTRD427bnWUa7GxR496S8PxuU
at /ip4/192.168.0.148/tcp/9001/...` — confirming discovery works both ways
in this instance) and then actively dialed back to the Windows CLI. Both
inbound connection attempts failed at the transport-negotiation stage:

```
WARN scmessenger_core::transport::swarm: Incoming connection error from /ip4/192.168.0.148/tcp/50746 -> /ip4/192.168.0.121/tcp/9001: Listen error: Failed to negotiate transport protocol(s)
WARN scmessenger_core::transport::swarm: Incoming connection error from /ip4/192.168.0.148/tcp/53196/ws -> /ip4/192.168.0.121/tcp/9002/ws: Listen error: Failed to negotiate transport protocol(s)
```

Both the raw-TCP and WebSocket listeners failed identically, on the same
peer, within milliseconds of each other — this rules out a WS-specific or
TCP-specific bug and points at something shared across both transports'
negotiation stack (most likely candidates: Noise handshake version/config
mismatch, multistream-select protocol-id mismatch, or a libp2p dependency
version skew between the Android core build (`libscmessenger_core.so`,
bundled in the APK) and this Windows CLI build, if the two were built from
different points in the dependency tree — note that a related crate,
`desktop_bridge`, was independently found to be **failing to build entirely**
in this same session due to missing `zbus`/`web_time` dependencies, which is
separate but raises the general question of whether all workspace members
are being built against a consistent, verified dependency set).

This is a genuine functional blocker: even with the mDNS
self-loopback/peer-count issues on the Android side fixed (see companion
tickets), a real remote peer cannot currently establish a working connection
to this CLI build over LAN at all, based on this evidence.

## Progress (2026-07-04, native /scm session, session ended on API-limit)

Confirmed via live tandem debugging (real CLI daemon on this Windows box +
real Pixel 6a at 192.168.0.148, adb readonly): failure is **100%
reproducible, not intermittent** — 12 occurrences of "Failed to negotiate
transport protocol(s)" in the single hour-log `scm.log.2026-07-04-22` alone,
recurring roughly every ~3 minutes for both TCP/9001 and WS/9002, every
single time, since at least 21:12 that day.

Artifacts saved locally (gitignored `tmp/`, not committed, still on this
machine for the next session):
- `tmp/work_files/parity_debug_2026-07-04/logcat_full_dump_initial.txt` — full historical logcat dump
- `tmp/work_files/parity_debug_2026-07-04/logcat_live_capture2.txt` — live logcat during a trace-logging CLI run
- `tmp/work_files/parity_debug_2026-07-04/cli_trace_log2.txt` — CLI daemon run with `RUST_LOG=libp2p_swarm=trace,libp2p_noise=trace,libp2p_core=trace,libp2p_tcp=trace,libp2p_mdns=trace,libp2p_websocket=trace,multistream_select=trace,scmessenger_core=debug` — captured a fresh mDNS discovery event but the session ended (API limit) before this run's window caught a fresh negotiation failure with full trace detail.
- Real daemon logs (not gitignored artifact, persistent app data):
  `C:\Users\SCM\AppData\Local\scmessenger\logs\scm.log.2026-07-04-21` and
  `-22` already contain the 12 plain WARN-level occurrences referenced
  above (generic wrapper message only, no Noise/multistream-select detail
  — that's what the trace rerun was for).

**Next session should:** start the CLI fresh with the same RUST_LOG line
above (`./target/release/scmessenger-cli.exe start`, adb device serial is
`192.168.0.148:43759` — note two adb entries currently resolve to the same
physical device, must pass `-s 192.168.0.148:43759` explicitly or `adb`
errors "more than one device/emulator"), let it run **at least 6-8 minutes
uninterrupted** (observed retry cadence ~3min, want 2+ occurrences), then
grep both the CLI trace log and the tandem logcat capture for the actual
underlying Noise/multistream-select error around the WARN "Incoming
connection error" timestamps — that specific underlying error is still not
captured yet. Cross-check `Cargo.lock`'s libp2p pins against what's in the
Android APK's bundled `.so` per Acceptance Criteria below (not yet done).

## Acceptance Criteria

- Identify the specific negotiation failure point: reproduce with
  `RUST_LOG=libp2p_swarm=trace,libp2p_noise=trace,libp2p_core=trace` (or
  whatever the equivalent trace-level targets are for this libp2p version)
  on the CLI side while a real Android device dials in, and capture the
  actual multistream-select/Noise error (the current log line is a generic
  "Listen error" wrapper — get to the underlying cause, e.g. protocol
  mismatch, version mismatch, or a specific Noise/TLS negotiation failure).
- Confirm whether the Android APK's bundled `libscmessenger_core.so` and
  this CLI's compiled dependencies (specifically `libp2p`, `libp2p-noise`,
  `libp2p-tcp`, `libp2p-websocket` versions) match `Cargo.lock` at the
  commit both were built from. If they diverge (e.g. Android was built from
  an older/newer commit with a different libp2p pin), that mismatch is
  likely the root cause and the fix is a rebuild/re-sync, not a code change.
- If the versions DO match and there's a genuine protocol/config bug, fix
  the actual negotiation mismatch (exact fix depends on what step 1 reveals
  — do not guess a fix without first getting the real underlying error from
  trace logging).
- Add or extend an integration test exercising the negotiation path this bug
  affects if the root cause turns out to be a code-level config issue (not
  applicable if it's purely a build/version-skew issue caught by process,
  not code).
- **This touches `core/src/transport/` — the mandatory
  `crypto-security-auditor` adversarial review applies before this is
  considered done**, per `.claude/rules/security.md`, regardless of how
  small the eventual fix turns out to be.

## Implementation Plan

1. Reproduce with trace-level libp2p logging enabled on the CLI
   (`RUST_LOG` env var, see Acceptance Criteria) while dialing from a real
   Android device on the same LAN, and capture the specific negotiation
   error.
2. Cross-check `Cargo.lock`'s pinned `libp2p*` versions against what the
   Android build was actually compiled against (check
   `android/app/build.gradle` / CI artifacts / whatever records which Rust
   commit produced the currently-installed APK's `.so`, if determinable).
3. Based on findings, either: (a) document the version-skew finding and
   trigger a rebuild-and-redeploy of the Android APK from current `main` as
   the fix (process fix, not code), or (b) implement the actual protocol/
   config fix if it's a genuine code bug.
4. If a code fix is needed, add test coverage; either way, get the mandatory
   `crypto-security-auditor` review before closing.

## Files to Touch

- `core/src/transport/swarm.rs` (negotiation/listener setup — read first to scope the actual diff)
- Possibly `Cargo.toml`/`Cargo.lock` if a version-pin fix is needed
- `core/tests/` (new/extended integration test, if a code fix is needed)

## Verification Commands

```bash
export CARGO_INCREMENTAL=0
cargo build --workspace
cargo test -p scmessenger-core --lib
cargo test -p scmessenger-core --test integration_e2e
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
```

Manual verification (required, this is fundamentally a live-network bug):
run the CLI daemon on Windows, dial in from a physical Android device on the
same LAN (real device, not emulator — this bug was found against a Pixel 6a),
confirm the connection completes (`ConnectionEstablished` logged, not
`Incoming connection error`) and a message can be sent end-to-end.
