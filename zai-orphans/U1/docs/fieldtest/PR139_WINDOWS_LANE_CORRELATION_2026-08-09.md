# PR #139 Windows-lane correlation -- Windows / Android / AWSUbuntu

Status: Active
Last updated: 2026-08-09
Lane: Windows/Claude (Windows, ADB/Pixel, AWSUbuntu)
Requested by: PR #139 comments 2026-08-09T08:16:04Z and 10:18:25Z
Supersedes the evidence-gap section of: `docs/fieldtest/PR139_LOG_EVIDENCE_2026-08-09.md`

All wall-clock times are HST (UTC-10) unless a line is quoted verbatim from a
Windows log, which timestamps in UTC. Both are given where it matters.

## Correction to the earlier gap report

An earlier version of this lane's report said the 13:35-13:55 HST window was
unrecoverable. That was too broad and is corrected here.

- **Device-wide logcat for the window IS preserved** at
  `tmp/logs/window_1330_1402.log` -- a 4.3 MB capture taken at 14:02, spanning
  13:30:00.433 -> 14:02:29.452. The live ring buffer has since rolled past it;
  the on-disk capture has not.
- **App-internal mesh logs for the window are still absent.** The earliest
  `MeshRepository` line in that capture is 14:01:32.416. The app's own lines for
  13:35-13:55 had already been evicted when the capture was taken.
- **What the capture does give us for the window is OS-level process lifecycle**,
  and it is decisive. See the timeline below.

The narrower gap stands: no app-internal mesh telemetry for 13:35-13:55.

## Node identities

| Node | Peer ID | LAN address |
|---|---|---|
| Windows CLI | `12D3KooWD6vZQrUqpyGaCqY3tNSK8p44BS78TvxpGpwhdPJ1T9mw` | -- |
| Android (Pixel 6a) | `12D3KooWNnPi9wqUJ7Jypj6g4jHmW2PUTmynUs9sJY1h6SQbjLrG` | `192.168.0.141` |
| Apple side ("ChristyLove") | `12D3KooWJUJ1koSWwSEAX32z6SGaepikyqpJawpojoy6gvQ8k688` | `192.168.0.142` |
| Apple-side identity id | `8094de3c9dda917c7413e4f14ac6f79e28aed2a76a208c2e690498787942d699` | -- |
| Android identity id | `a43772fe` (cached), beacon `c0a682ef...` | -- |
| AWS always-on node | none -- `initialized: false` | `54.226.67.101` |

## The requested window was not a valid test window

This is the headline finding. Both desktop-class participants were absent or
short-lived for nearly all of 13:35-13:55.

### Android: up for 2m50s, then task-killed by the user

From `tmp/logs/window_1330_1402.log`:

| Time (HST) | Event |
|---|---|
| 13:40:37.842 | `wm_create_activity` MainActivity |
| 13:40:37.854 | `am_proc_start` PID 15783, `next-top-activity` |
| 13:43:27.696 | `am_kill: [0,15783,com.scmessenger.android,905,remove task,577444]` |
| 13:43:28.566 | `am_proc_died` PID 15783 |
| 13:43:41.127 | relaunch, PID 16775 |

The kill reason is **`remove task`** at adj 905 -- a user swipe from recents, not
a crash, ANR, tombstone, or low-memory kill. This confirms and sharpens the
earlier correlation doc's item 3: the app had been running only **2 minutes 50
seconds** before it was dismissed, and was relaunched 13 seconds later.

### Windows: not running at all

From `~/AppData/Local/SCMessenger/logs/`:

| Time (UTC) | Time (HST) | Event |
|---|---|---|
| 23:03:00.499 | 13:03:00 | last line in `scm.log.2026-08-08-23` |
| -- | 13:03 - 14:09 | **no Windows log output whatsoever** |
| 00:09:38.883 | 14:09:38 | `SCMessenger CLI starting up... 0.4.0 (6cb7033a ...)` |

The Windows node was down for the entire requested window. This closes item 5 of
`PR139_ANDROID_WINDOWS_CORRELATION_2026-08-09.md` ("whether every desktop node
was actually running remains unproven"): it was not.

**Consequence.** Any conclusion drawn from 13:35-13:55 about mesh behaviour --
BLE intermittency, peer discovery, receipt gaps -- is unsupported. There was no
Windows peer present, and the Android app was up for under three minutes of it.
That window should not be cited as evidence for or against the mesh.

## Windows node: the UPnP panic is now proven reproducible

The prior correlation doc listed run 2 as "not allowed to run long enough to
prove reproducibility". It is proven now: **two independent runs, same panic,
same source line.**

| Run | Start (UTC) | Death (UTC) | Uptime | Panic |
|---|---|---|---|---|
| 1 | 00:09:38 | 00:15:20.618 | 5m 42s | `libp2p-upnp-0.5.0/src/behaviour.rs:497:38: mapping should exist` |
| 2 | 00:16:05 | 00:32:47.852 | 16m 42s | same, `tmp/logs/win_node_run2.log:1471` |

Both deaths follow an identical three-second sequence: relay circuit reservation
accepted -> burst of circuit `Listening on` registrations -> ledger exchange
response -> `swarm_event_loop_died`. The differing uptimes rule out a fixed
timer and are consistent with mapping expiry, which is what the upstream
`expect("mapping should exist")` asserts on.

The candidate removes the `upnp` behaviour and its event arm outright, so the
fix addresses the proven root cause rather than a suspected one. It remains
**unverified at runtime** -- a Windows soak on a candidate build is required
before Run 1, and it must exceed 17 minutes to clear both observed uptimes.

Note the CLI's own safety net worked correctly in both runs: it detected the
dead swarm loop and exited rather than lingering as a zombie.

## Classifications requested

### BLE intermittency

Every Android delivery in the retained capture went over BLE and only BLE, to
one peripheral (`4B:C0:2F:A8:76:AF`), with no LAN/TCP attempt logged -- despite
the Android node holding LAN listeners and having identified the LAN peer
`12D3KooWJUJ1...` at 14:01:35.739 ("TCP/mDNS: LAN peer detected ... with 13
local addresses").

One fallback was observed: msg `cdf35737-704b-4813-a223-9a76dc3014c7` at
14:01:42.675 logged `phase=local_fallback outcome=target_fallback` before
succeeding via `smart_router`. That is the intermittency signature, and it
resolved within 160 ms.

The severe BLE fault in this dataset is not intermittency but the L2CAP
accept-spin -- see the section below.

### Android cellular peer/ledger discovery

Android's listener snapshot at 14:01:35.639 carries both LAN and cellular
interfaces concurrently:

```
listeners=[/ip4/192.168.0.141/tcp/9090/ws, /ip4/10.16.109.218/tcp/9090/ws,
           /ip4/127.0.0.1/tcp/9090/ws, /ip6/2600:381:9b5e:...:17d4/tcp/9001, ...]
```

`10.16.109.218` and the `2600:381:...` globals are the carrier interfaces.
Ledger seeding used the LAN path: "Dialed seed relay from ledger:
`/ip4/192.168.0.142/tcp/443`" at 14:01:35.629, followed by discovery and
identification of `12D3KooWJUJ1...` at 14:01:35.635-.738.

**This also closes observation O3 from the security review for Android**: the
platform reports concrete per-interface listen addresses, not a wildcard, so the
new RFC1918 disclosure predicate has usable local-subnet evidence there. Same
already confirmed for Linux via the AWS node. iOS remains unconfirmed and is the
Apple lane's to check.

### Receipts

One complete, clean round trip:

| Time (HST) | Event |
|---|---|
| 14:01:37.312 | message `2add6ad7-98e4-4cee-8a52-9c82bfb40209` received from `8094de3c...` |
| 14:01:37.325 | `delivery_attempt medium=core phase=rx outcome=received` |
| 14:01:37.402 | history sync data processed -- **receiver-side decrypt confirmed** |
| 14:01:37.428 | `[RECEIPT-ENCODE] status=DELIVERED ts=1786233697` |
| 14:01:37.433 | `[RECEIPT-ENCODE] SUCCESS bytes=97` |
| 14:01:37.481 | `medium=receipt phase=aggregate outcome=acked` |

Round trip from receipt to ack: 169 ms.

**Scoring caveat for Run 1.** The outbound legs record `transport_ack=true` at
`phase=aggregate`, which is a BLE transport acknowledgement, not proof the
recipient decrypted anything. Per the standing fleet-run scoring rule, score on
receiver-side decrypt plus durable history plus receipt -- as the `2add6ad7`
inbound leg above actually demonstrates -- never on `transport_ack` alone.

### History / outbox

`sendHistorySyncIfNeeded shouldSend=true ... (age=248709ms)` at 14:01:35.843,
request sent at 14:01:36.171, response processed at 14:01:37.402. History sync
worked end to end on this exchange.

### Reconvergence

Windows run 1 reached the Apple side within 1 second of start
(`00:10:01`, `/ip4/192.168.0.142/tcp/9090`, promiscuous mode) but took until
`00:15:17` -- **5 minutes 39 seconds** -- to converge with Android, arriving
roughly 3 seconds before the UPnP panic killed it. Run 2 shows the same shape:
convergence with Android at `00:32:44`, panic at `00:32:47`.

Convergence to the Android peer is therefore both slow and, on this build,
almost perfectly anti-correlated with node lifetime. Whether the delay is
independent of UPnP is unknown and should be measured on the candidate soak.

### Custody

Windows relay custody audit log count went 140 (23:03 UTC, pre-restart) -> 149,
then held flat at 149 across every minute from 00:21:07 to 00:32:07 UTC. No
custody churn during the observed window; no custody failures logged.

## The BLE L2CAP accept-spin, and why it gates Run 1

Independent of the window above, the device buffer records a severe fault.

| Fact | Value |
|---|---|
| Tag | `BleL2capManager$startListening` |
| Lines | 2,239,045 -- **70% of the entire 3.2M-line device buffer** |
| Window | 18:30:08.574 -> 18:33:00.781 HST (172 s) |
| Distinct failures | 172,234 (**~1,000 per second**) |
| Error | `java.io.IOException: read failed, socket might closed or timeout, read ret: -1` |
| Spinning process | PID 1358 (started 18:13:08 "for service") |
| Recovery | new PID 6116 at 18:32:59.631, `L2CAP server listening on PSM: 128` at 18:33:00.781 |

Recovery required a **new process**, not in-process re-arming. This is the
failure `0ed11b62` characterises as "socket is born broken".

**The Pixel has never run the fix.** Installed versionCode 14 at 2026-08-08
**12:47:45 HST**; `fdb32e7d` and `35c9a2db` landed at **20:00:02** and
**20:04:12 HST**, over seven hours later. Run 1 on this APK reproduces the spin,
burns the ring buffer, and destroys its own evidence. The Pixel must be
reinstalled from the candidate Actions artifact before Run 1.

## AWSUbuntu

| Check | Result |
|---|---|
| SSH | works as **`ec2-user`** with `~/.ssh/scm-node-key.pem`; `ubuntu` is rejected |
| `GET /health` | 200 `{"status":"healthy"}` |
| `GET /version` | **404** |
| `GET /api/listeners` | 200, concrete per-interface addresses |
| `GET /api/identity` | `initialized: false`, `libp2p_peer_id: null` |

The 404 is positive provenance evidence: `/version` exists in the candidate at
`cli/src/api.rs:1213` returning `git_hash` and `core_provenance`, so the
deployed image demonstrably predates it. The node is still the `6b2573fa` image.

Listeners include `172.31.19.216` and `172.17.0.1` (docker bridge) with no
`0.0.0.0` entry, confirming libp2p expands a wildcard bind on Linux.

Open question: the node reports no initialized identity, so its role among the
five must be stated -- relay-only or participant -- and there is currently no
peer identity to record on the rebuild provenance line.

## Raw artifacts

In-repo (already preserved by the prior session):

- `tmp/logs/window_1330_1402.log` -- device-wide logcat 13:30-14:02 HST
- `tmp/logs/window_app.log`, `tmp/logs/app_only.log` -- app-filtered slices
- `tmp/logs/win_node.log`, `tmp/logs/win_node_run2.log` -- both Windows runs incl. panic text
- `tmp/logs/pixel_all_1402.log`, `tmp/logs/pixel_full_20260808_1939.log`

On the Windows host, not committed (size):

- `~/AppData/Local/SCMessenger/logs/scm.log.*` -- hourly Windows node logs
- session scratchpad: full 3.2M-line logcat, app-scoped extract, isolated spin log

## Required before Run 1

1. Reinstall the Pixel from the candidate Actions artifact (not a local build).
2. Soak the Windows node on a candidate build for **more than 17 minutes** to
   clear both observed UPnP uptimes.
3. Rebuild AWS from the `docker-publish.yml` image **digest**, never `:latest`,
   and gate on `GET /version` reporting the candidate commit.
4. Raise log buffers on every node before start (`logcat -G` on Android).
5. Capture `/api/listeners` on all five nodes at start.
6. Settle the AWS node's role and whether it needs an initialized identity.
7. Correlate by message UUID, peer ID, identity id, UTC timestamp, and transport
   -- and score on receiver decrypt plus durable history plus receipt, never on
   `transport_ack` alone.
