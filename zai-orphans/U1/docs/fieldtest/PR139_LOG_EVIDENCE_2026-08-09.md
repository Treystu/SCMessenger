# PR #139 pre-Run-1 log evidence -- Windows lane capture, 2026-08-09

Status: Active
Last updated: 2026-08-09
Lane: Windows/Claude
Requested by: PR #139 comment, 2026-08-09T08:16:04Z
Extends: `docs/fieldtest/PR139_ANDROID_WINDOWS_CORRELATION_2026-08-09.md`

## Summary

The requested Android window (13:35-13:55 HST, 2026-08-08) is **unrecoverable**.
Reporting it as an evidence gap rather than substituting a later capture, as the
request directed.

The cause is measurable and it is our own defect: a BLE L2CAP accept-spin
consumed 70% of the device log ring buffer in under three minutes and evicted
everything before 14:09:01 HST. The device is still running the pre-fix APK, so
Run 1 would reproduce it and destroy the evidence Run 1 exists to produce.

## Android -- Pixel 6a, `com.scmessenger.android`

Capture taken 2026-08-08 23:04-23:09 HST over wireless ADB
(`adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp`).

| Fact | Value |
|---|---|
| Buffer span (`logcat -b all`) | 08-08 14:09:01.598 -> 08-08 23:08:10.510 HST |
| Total lines captured | 3,203,018 |
| Requested window | 08-08 13:35-13:55 HST -- **evicted, 14 min before buffer start** |
| Installed build | versionName 0.4.0, versionCode 14 |
| Install time | 2026-08-08 12:47:45 HST |
| App running at capture | No |

### The accept-spin that ate the buffer

| Fact | Value |
|---|---|
| Tag | `BleL2capManager$startListening` |
| Lines | 2,239,045 (**70% of the entire buffer**) |
| Window | 08-08 18:30:08.574 -> 18:33:00.781 HST (172 s) |
| Distinct failures | 172,234 (**~1,000 per second**) |
| Error | `java.io.IOException: read failed, socket might closed or timeout, read ret: -1` |
| Lines per failure | ~13 (message + stack trace) |
| Spinning process | PID 1358 (started 18:13:08 "for service") |
| Recovery | PID 6116 started 18:32:59.631; logged `L2CAP server listening on PSM: 128` at 18:33:00.781 |

Recovery came from a **new process**, not from in-process re-arming. This is the
failure `0ed11b62` characterises as "socket is born broken".

### App process lifecycle in the retained window

| Time (HST) | Event |
|---|---|
| 08-08 17:05:42.378 | PID 16775 died |
| 08-08 17:56:01.906 | PID 30318 start (next-top-activity) |
| 08-08 18:13:08.518 | PID 1358 start (for service) |
| 08-08 18:30:08.574 | PID 1358 begins accept-spin |
| 08-08 18:30:12.985 | PID 5157 start (next-top-activity) |
| 08-08 18:32:59.631 | PID 6116 start (next-top-activity) |
| 08-08 18:33:00.781 | PID 6116 L2CAP listening on PSM 128; spin ends |

## Blocker for Run 1: the Pixel has never run the BLE fix

| Artifact | Timestamp (HST) |
|---|---|
| Installed APK (versionCode 14) | 2026-08-08 12:47:45 |
| `fdb32e7d` recover failed BLE L2CAP listeners | 2026-08-08 20:00:02 |
| `35c9a2db` keep BLE listener recovery armed | 2026-08-08 20:04:12 |
| `15f09049` normalize BLE sources for CI hygiene | 2026-08-08 20:11:19 |

The install predates every BLE recovery fix by more than seven hours. Starting
Run 1 on this APK reproduces the spin, burns the ring buffer again, and makes
post-run forensics impossible. **The Pixel must be reinstalled from a
candidate-SHA build before Run 1.**

## AWS always-on node -- `54.226.67.101`

Probed 2026-08-09 from the Windows lane. Address taken fresh from
`HANDOFF/gpt/AWS_RELAY_CURRENT_ADDRESS.md` per the standing IP policy.

| Endpoint | Status | Body |
|---|---|---|
| `GET /health` | 200 | `{"status":"healthy"}` |
| `GET /version` | **404** | -- |
| `GET /api/listeners` | 200 | concrete per-interface addresses |
| `GET /api/identity` | 200 | `initialized: false`, `libp2p_peer_id: null` |

**The 404 is provenance evidence.** `/version` exists in the candidate at
`cli/src/api.rs:1213` and returns `version`, `git_hash`, `build_time` and
`core_provenance`. The deployed image demonstrably predates that route, so the
node is still the `6b2573fa` image and is **not** a candidate build. The
`/version` provenance gate will work once the node is rebuilt from the digest.

Listener output (relevant excerpt): `/ip4/172.31.19.216/tcp/9001`,
`/ip4/172.17.0.1/tcp/9001`, `/ip4/127.0.0.1/tcp/9001`, plus IPv6 loopback and
link-local forms. No `0.0.0.0` entry, which confirms libp2p expands a wildcard
bind into concrete per-interface addresses on Linux -- the precondition the new
RFC1918 disclosure predicate depends on. Android and iOS still need the same
confirmation.

Two items to settle before Run 1:

1. The node reports no initialized identity, so its role among the five must be
   stated explicitly -- relay-only, or participant.
2. There is consequently no peer identity to record on the rebuild provenance
   line. Decide whether one is required.

## Evidence gaps (stated, not substituted)

- Android 13:35-13:55 HST window: **gone**, cause above.
- No iOS-side log for that window.
- No same-window macOS or Windows process/socket capture.
- Whether every desktop node was actually running during that window remains
  unproven.

## Raw artifacts

Preserved on the Windows host under the session scratchpad:

- `android_logcat_2026-08-08_full.log` -- 3,203,018 lines, full buffer
- `android_app.log` -- package-scoped extract
- `ble_spin.log` -- 2,239,045-line isolated accept-spin

These are not committed; they exceed sane repo size. Request them from the
Windows lane if a correlation needs the raw lines.

## Required before Run 1

1. Reinstall the Pixel from a candidate-SHA build.
2. Rebuild the AWS node from the `docker-publish.yml` image **digest**, not
   `:latest`, and gate on `GET /version` reporting the candidate commit.
3. Raise log buffer sizes on every node before start (`logcat -G` on Android).
4. Capture `/api/listeners` on all five nodes at start, to close observation O3
   in `docs/security/PR139_REVIEW_15dbcde0_2026-08-09.md`.
5. Correlate by message UUID, peer ID, identity ID, UTC timestamp, and transport
   -- not by log severity.
