# NEXT_ITER_04: Live-Device Retest — Android <-> Windows Pairing

**Priority:** P0 for the parity effort, but OPERATOR-GATED (needs the physical
Pixel + Windows CLI daemon on the same LAN; a headless worker cannot do this alone)
**Recommended worker:** any model, driven interactively with the operator, AFTER
NEXT_ITER_01 and 02 pass and the sprint commit is installed on both sides
**Source:** Fable 5 sprint 2026-07-05/06 fixed the four live-device pairing
failures from `FABLE_5_COMPREHENSIVE_AUDIT.md`; this is the verification pass.
Raw prior logs: `HANDOFF/todo/P1_ANDROID_FABLE_5_DISCOVERY_REPORT.md`.

## Test matrix (repeat of the 2026-07-04 interop test)

Build/install: `./android/install-clean.sh` on the Pixel; rebuild + restart the
Windows CLI daemon from the sprint commit.

1. **Listener bind (Issue 1 fix):** After app start with internet enabled,
   `adb shell cat /proc/net/tcp` must show a socket in LISTEN on port 9001
   (hex 2329). Logcat must show `[IronCore] [OK] Swarm listening on ...`.
   Kill test: occupy port 9001 on the device first (or start two instances) —
   the app must now log `Swarm startup failed` / `Listener ... failed` instead
   of silently claiming RUNNING.
2. **Rescan ANR (Issue 2 fix):** In Nearby Contacts, hammer "Rescan" repeatedly.
   `adb shell dumpsys dropbox --print data_app_anr` must show NO new ANR from
   `SubnetProbe`.
3. **Outbound dial (Issue 4 fix):** With the Windows daemon at LISTEN on 9001,
   Android must establish a real libp2p connection (Windows `daemon.log` shows
   ConnectionEstablished, NOT `Failed to negotiate transport protocol(s)`
   within ~1s of the SubnetProbe ping — the 500ms dial delay should separate them).
4. **BLE backoff (Issue 3, previously verified by cargo check only):** Run the
   Windows CLI 10+ minutes near an unreachable BLE device;
   `GattCommunicationStatus(1)` warnings must back off (2s, 4s, ... 60s cap),
   not repeat every ~30s indefinitely.
5. **End-to-end:** pair, exchange messages both directions, verify receipts.

## Output

Write results (with log excerpts) into this file, move to `HANDOFF/done/` if all
pass; file new P0 tasks for anything that fails, citing exact log evidence.
