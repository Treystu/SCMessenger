# SCMessenger New Machine Setup Log

**Date:** 2026-05-17
**Machine:** SCM_Dev (LAPTOP-87OJFOSB)
**OS:** Windows 11 Pro

---

## 1. JDK Location (Auto-discovered by Gradle)

**Path:** `C:\Users\SCM_Dev\.gradle\jdks\jetbrains_s_r_o_-17-amd64-windows.2`

**JAVA_HOME must be set before any Android build:**
```bash
export JAVA_HOME="/c/Users/SCM_Dev/.gradle/jdks/jetbrains_s_r_o_-17-amd64-windows.2"
```

**Version verified:**
```
openjdk version "17.0.14" 2025-01-21
OpenJDK Runtime Environment JBR-17.0.14+1-1367.22-jcef
OpenJDK 64-Bit Server VM JBR-17.0.14+1-b1367.22-jcef
```

**Note:** JDK is NOT in standard locations (Program Files, registry, PATH). Gradle auto-downloaded it on first build attempt. Future agents must explicitly set JAVA_HOME to this path before running `./gradlew`.

---

## 2. Android SDK Location

**Path:** `C:\Users\SCM_Dev\AppData\Local\Android\Sdk`

Already present with build-tools, ndk, platforms, platform-tools.
ANDROID_HOME may also need to be set if not already configured.

---

## 3. Orchestrator Fixes Applied

The swarm orchestrator had several bugs on the new machine that prevented agent dispatch. All fixed in commit `85131bd6` and follow-ups:

| Fix | File | Description |
|-----|------|-------------|
| PYTHON env var | `orchestrator_manager.sh` | Script now respects externally set `PYTHON=` instead of overwriting |
| Robust python3 detection | `model_validation_template.sh`, `launch_agent.sh` | Tests that `python3` actually runs before selecting (prevents Windows Store stub) |
| Case-insensitive micro-task | `orchestrator_manager.sh` | `MICRO_*` tasks now recognized as micro-tasks regardless of filename casing |
| context_extractor guard | `orchestrator_manager.sh` | Failure of context extractor is non-fatal (`|| true`) |
| Deprecation warning | `freshness_gate.py` | Replaced `utcfromtimestamp()` with `timezone.utc` |

---

## 4. Cookie Refresh

**Ollama Cloud cookie updated in `OllamaQuotaScraper.ps1`:**
- `aid`: `bf5f45fb-b5ea-4b39-b61c-abacf9cc81bb`
- `__Secure-session`: `[current session token]`
- `baseDir`: Updated from old machine (`kanal`) to `SCM_Dev`

---

## 5. Build Config Updates

### `.cargo/config.toml`
- Old machine linker paths commented out (MSVC 14.50, mingw32-gcc)
- Now uses PATH-default linkers on new machine

### `android/gradle.properties`
- JVM heap reduced from 2048m to 1536m
- Gradle workers limited to 2 for lower-RAM machine

---

## 6. Quota State (as of 2026-05-17)

- **5-hour:** 6.7%
- **7-day:** 92.9%
- **Reset:** ~180 minutes
- **Phase:** TIER 5 MICRO (approaching HARDLOCK at 99.5%)

---

## 7. Active Work Queue

### Completed Today
- `MICRO_ANR_001`: MeshRepository.kt relay identity guard (IllegalStateException -> Timber.w + return)

### Pending (in `HANDOFF/todo/`)
- `MICRO_ANR_002`: MeshRepository.kt empty message ID guard
- `MICRO_ANR_003`: MeshForegroundService.kt RUNNING state guard
- `MICRO_DEPRECATION_001`: BleGattServer.kt API 31+ executor overload
- `MICRO_DEPRECATION_002`: MdnsServiceDiscovery.kt API 28 gate fix

### Blocker
Android build verification cannot proceed without JAVA_HOME being set in agent environment.

---

## 8. Next Steps for Resume

1. Set `JAVA_HOME="/c/Users/SCM_Dev/.gradle/jdks/jetbrains_s_r_o_-17-amd64-windows.2"`
2. Export `ANDROID_HOME="/c/Users/SCM_Dev/AppData/Local/Android/Sdk"` if not set
3. Run `./gradlew :app:assembleDebug -x lint --quiet` to verify Android build after agent changes
4. Resume agent dispatch for remaining 4 micro-tasks
5. After every 2-3 tasks: refresh quota, commit checkpoint

---

Saved at API limit boundary. Resume from here.
