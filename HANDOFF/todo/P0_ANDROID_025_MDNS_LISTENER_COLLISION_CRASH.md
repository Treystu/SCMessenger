# MODEL: qwen3-coder-next:cloud
# BUDGET: 1200
# token_budget: 12000

# P0_ANDROID_025_MDNS_LISTENER_COLLISION_CRASH

**Status:** VERIFIED REMAINING WORK
**Agent:** implementer
**Budget:** 1200s (LIGHT tier)
**Phase:** v0.2.1 P0 Android stability
**Source:** Live end-to-end test 2026-06-05 21:18 PT (Overseer session, auto-discovered)
**Depends on:** none (independent of P0_024, P1_022)
**Branch:** fix/p0-android-025-mdns-listener-collision (off origin/main dd109707)
**Worktree:** create a new worktree at `E:\SCMessenger-build-p0-025\` based on origin/main
**Assignee:** worker
**Note:** Per 2026-06-05 21:22 PT user directive, future work pivots to local models. The MODEL: line above is the historical default; the actual dispatch should be against the local `scm-coder:7b` or `scm-thinker:14b` model unless the user explicitly approves a cloud call.

---

# P0 — Android: mDNS "listener already in use" crash on Android<->Windows discovery

**Status:** OPEN — NEWLY DISCOVERED
**Severity:** P0 (crash on any discovered mDNS peer; blocks Android<->Windows LAN discovery)
**Reported by:** Overseer session, 2026-06-05 21:19 PT (auto-discovered during end-to-end test)
**Detected in:** v0.2.3-debug on Pixel 6a (installed 21:18 PT from `E:\SCMessenger-build-p0-024\android\app\build\outputs\apk\debug\app-debug.apk`)
**Reporter's claim:** "Android app crashes when it discovers the Windows CLI over mDNS."

## Symptom (verified, logcat-attached)

After `adb install` of the v0.2.3-debug build, the app launched and successfully:
- Started mDNS discovery (mDNS advertiser on `_p2p._udp.local`).
- Received the Windows CLI relay's mDNS broadcast (Peer ID `12D3KooWFjyBaagUcyuweT26YVoAUtyM1u2K8YnKRgkMJ59zY8fD` at `192.168.0.230:9101`).
- Attempted to resolve the service via `NsdManager.resolveService(...)`.

It then **crashed** with a `FATAL EXCEPTION: ConnectivityThread` and `java.lang.IllegalArgumentException: listener already in use`:

```
06-05 21:19:45.462 25810 25865 E AndroidRuntime: FATAL EXCEPTION: ConnectivityThread
06-05 21:19:45.462 25810 25865 E AndroidRuntime: Process: com.scmessenger.android, PID: 25810
06-05 21:19:45.462 25810 25865 E AndroidRuntime: java.lang.IllegalArgumentException: listener already in use
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager.putListener(NsdManager.java:1312)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager.putListener(NsdManager.java:1295)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager.resolveService(NsdManager.java:1781)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at com.scmessenger.android.transport.MdnsServiceDiscovery.resolveService(MdnsServiceDiscovery.kt:476)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at com.scmessenger.android.transport.MdnsServiceDiscovery.onServiceFound(MdnsServiceDiscovery.kt:106)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at com.scmessenger.android.transport.MdnsServiceDiscovery$startDiscovery$1.onServiceFound(MdnsServiceDiscovery.kt:445)
06-05 21:19:45.462 25810 25865 E AndroidRuntime: 	at android.net.nsd.NsdManager$ServiceHandler.lambda$handleMessage$2(NsdManager.java:1203)
```

`06-05 21:19:45.482 ActivityTaskManager: Force finishing activity com.scmessenger.android/.ui.MainActivity`
`06-05 21:19:47.855 Process com.scmessenger.android (pid 25810) has died: cch CRE`

## Root cause (confirmed in source)

`E:\SCMessenger-build-p0-024\android\app\src\main\java\com\scmessenger\android\transport\MdnsServiceDiscovery.kt:476`

```kotlin
private fun resolveService(serviceInfo: NsdServiceInfo) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
        nsdManager?.resolveService(serviceInfo, context.getMainExecutor(), getResolveListener())
    } else {
        @Suppress("DEPRECATION")
        nsdManager?.resolveService(serviceInfo, getResolveListener())
    }
}
```

`getResolveListener()` returns a **singleton** listener. `NsdManager.resolveService()` reuses it for every `onServiceFound` callback. Once the first resolve is in flight (or has not yet been detached), the second call throws `IllegalArgumentException: listener already in use`.

**This bug pre-exists my v0.2.3-debug fix set.** It was latent because the previous v0.2.3 install on the phone had not seen a peer yet, and the mDNS path on v0.2.2 was on port 9001 (which the Windows CLI's 9101 was not). The new test setup with `cli relay --listen /ip4/0.0.0.0/tcp/9101 --http-port 9102` finally produced a real `onServiceFound` callback chain and exposed the bug.

## Repro (deterministic, on this hardware)

1. `adb -s adb-26261JEGR01896-6pHTac._adb-tls-connect._tcp uninstall com.scmessenger.android` (clean).
2. `adb install -r <debug-apk>` (any build with `MdnsServiceDiscovery.kt:476`).
3. `E:\SCMessenger-Github-Repo\SCMessenger\target\debug\scmessenger-cli.exe relay --listen /ip4/0.0.0.0/tcp/9101 --http-port 9102` (any Windows SCMessenger CLI on the same LAN, mDNS must reach the phone).
4. Launch the app via `am start -n com.scmessenger.android/.ui.MainActivity`.
5. Within ~30 sec, the app will crash with the listener-in-use exception when it discovers the Windows CLI.

## Files to investigate (precise)

| File | Why |
|---|---|
| `android/app/src/main/java/com/scmessenger/android/transport/MdnsServiceDiscovery.kt:476` | The crash site. `getResolveListener()` returns a singleton. |
| `MdnsServiceDiscovery.kt` (whole file, 490 lines) | The `ResolveListener` impl and its cleanup on `onResolveSucceeded` / `onResolveFailed` must detach the listener before allowing reuse. |
| `BleScanner.kt` (sibling cache) | After P1_ANDROID_022 fix, `clearPeerCache()` is the model. mDNS needs the analogous `markResolveComplete()` or listener-pool pattern. |
| `core/src/transport/` (Rust) | Not the cause — this is a pure Android NSD bug. Cross-check: does libp2p's mDNS ever trigger the same path? It does not (uses UDP multicast directly, not the Android NSD daemon). |

## Hypothesis (entry point for the subagent)

The most likely fix is one of:

1. **Per-service listener** — Create a fresh `NsdManager.ResolveListener` for each `resolveService()` call, and let Android GC it. Cheap, but loses error context. (smallest patch)
2. **Pending-resolve set + onComplete cleanup** — Track which `(serviceName, listener)` pairs are in flight, and only return the listener to the pool on `onResolveSucceeded` / `onResolveFailed`. (canonical fix)
3. **Debounce onServiceFound** — If the same `serviceName` is found twice within N seconds, ignore the duplicate. Doesn't address root cause, but masks it. (anti-pattern, do not use)

The fix is small (10-20 LoC) and lives entirely in `MdnsServiceDiscovery.kt`.

## Acceptance criteria

- [ ] App survives `onServiceFound` callbacks for at least 2 distinct mDNS peers without crashing.
- [ ] App survives the same peer re-broadcasting (the common case on Android 14+).
- [ ] No FATAL EXCEPTION in logcat during a 5-min Android<->Windows LAN session.
- [ ] The Windows CLI's `discovery peers` output shows the Android phone's peer-id within 60 sec.
- [ ] Build verification: `./gradlew :app:assembleDebug -x lint` succeeds.
- [ ] Hand off a post-mortem to `HANDOFF/STATE/`.

## Out of scope (do NOT do in this ticket)

- Refactoring the entire transport layer.
- Adding new mDNS features.
- The P0_ANDROID_024 / P1_ANDROID_022 fixes already in the worktree at `E:\SCMessenger-build-p0-024\` — they are SEPARATE and ship in a different commit.
- Cross-OS triangulation work.

## Build environment (for the subagent)

Use the verified env from `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md`:
- `JAVA_HOME=E:\build-tools\jdk17\jdk-17.0.14`
- `ANDROID_NDK_HOME=E:\build-tools\android-sdk\ndk\26.1.10909125`
- Override: `-Pandroid.ndkVersion=26.1.10909125`
- Build dir: `E:\SCMessenger-build-p0-024\android`

## Reference

- P0_ANDROID_024 (separate): identity-generation re-entrancy
- P1_ANDROID_022 (separate): BLE stale-cache cleanup
- `HANDOFF/STATE/2026-06-05_ORCHESTRATION_INDEX.md` — build env, ticket queue
- `HANDOFF/STATE/2026-06-05_ANDROID_P0_024_P1_022_BUILD_VERIFIED.md` — verified build, env
