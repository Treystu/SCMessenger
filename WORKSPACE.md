# SCMessenger Workspace — Source of Truth

> Single canonical map of where everything lives. If a file, binary, or
> config is referenced from a script, model, or agent, its location is
> documented here. Last updated: 2026-06-04 (post-deconfliction).

═══════════════════════════════════════════════════════════════
1. THE FOUR CANONICAL SCMESSENGER LOCATIONS
═══════════════════════════════════════════════════════════════

There are FOUR distinct SCMessenger-related trees. Do not conflate them.

  (1) SOURCE OF TRUTH (canonical code)
      /mnt/e/SCMessenger-Github-Repo/SCMessenger/
      Windows: E:\SCMessenger-Github-Repo\SCMessenger\
      Git remote: https://github.com/Treystu/SCMessenger.git
      HEAD: b6dacfa4  (branch: main)
      Status: clean working tree
      What lives here:
        - Cargo.toml (workspace: core, mobile, cli, desktop_bridge, wasm)
        - core/         Rust library (transport, crypto, mesh)
        - mobile/       Kotlin/Rust mobile bindings
        - cli/          Rust CLI
        - desktop_bridge/
        - wasm/         WASM bindings
        - android/      Android app (Kotlin + Gradle)
        - shared/       Kotlin multiplatform shared code
        - scripts/, .claude/, HANDOFF*, Latest_Updates.md, CLAUDE.md

  (2) LIVE RUNTIME (deployed binaries + runtime state)
      /mnt/e/SCMessenger/
      Windows: E:\SCMessenger\
      Git: NOT a repo. Do not git init.
      What lives here:
        - bin/scmessenger-cli.exe       20MB live CLI binary
        - bin/scmessenger-gateway.exe   live Hermes gateway
        - data/                         user data, message history
        - logs/                         runtime logs
        - config.json                   runtime config
        - start.bat, recover.bat        launch + recovery scripts
        - recover-wsl.sh, RECOVERY_INSTRUCTIONS.txt  (rebuild-from-scratch)
        - post-restart-checklist.txt    (post-restart tasks)
      Source: built from (1), copied to here by build-cli-release.bat etc.
      DO NOT EDIT FILES IN THIS TREE. To change behavior, edit (1) and rebuild.

  (3) BUILD TOOLCHAIN (Android SDK, NDK, Gradle, Cargo, JDK)
      /mnt/e/build-tools/
      Windows: E:\build-tools\
      Git: NOT a repo.
      What lives here:
        - android-sdk/                  Android SDK (cmdline-tools, build-tools, platforms)
        - android-sdk/ndk/26.1.10909125/  NDK r26b (the working NDK path)
        - jdk17/                        JDK 17 (renamed; do NOT include '+' in path)
        - .gradle/                      GRADLE_USER_HOME (prevents C: refill)
        - .cargo/                       CARGO_HOME
        - .rustup/                      rustup toolchain
        - platform-tools/               adb, fastboot
        - licenses/                     Android SDK licenses (accepted)
        - build-android-wsl.sh          WSL wrapper: cd android/ && gradlew assembleDebug
        - build-apk-wsl.sh              DEPRECATED, forwards to build-android-wsl.sh
        - android-env.sh                exports JAVA_HOME, ANDROID_HOME, etc.
        - *.bat, *.ps1                  Windows batch / PowerShell build entry points
        - build-apk-wsl-live.log        last successful build log (Jun 2 15:53)
        - diag.txt, gradle-build3.log   diagnostic dumps

  (4) BACKUPS (do not touch unless recovering)
      /mnt/e/backup-2026-06-04-pre-unify/
      Windows: E:\backup-2026-06-04-pre-unify\
      Contents:
        - scmessenger-repo-source.tar.gz    (4.1M, 2092 entries, removed stale fork)
        - sovereign-messenger-source.tar.gz (18K,  33 entries, removed unrelated project)
        - PRE-UNIFY-STATE.txt, POST-UNIFY-STATE.txt
      Keep until: 2026-07-04 (30 days) or until 2 successful builds pass.

═══════════════════════════════════════════════════════════════
2. SYMLINKS / JUNCTIONS (do not delete or recreate blindly)
═══════════════════════════════════════════════════════════════

  /mnt/e/Sdk/ndk/26.1.10909125   ->  /mnt/e/build-tools/android-ndk-r26b
  Windows: E:\Sdk\ndk\26.1.10909125 [JUNCTION] -> E:\build-tools\android-ndk-r26b
  Purpose: many Android tools default to E:\Sdk\ndk\...; this junction
  points them at the real NDK location.
  Maintainer: if NDK reinstalled at android-ndk-r26b, junction auto-resolves.
  Do NOT point the junction at /mnt/e/build-tools/android-sdk/ndk/26.1.10909125
  (the sdkmanager-managed NDK) unless you also update project local.properties.

  /mnt/e/Sdk/                     symlinks to /mnt/e/Android/ (or similar)
  Verify with:  ls -la /mnt/e/Sdk/

═══════════════════════════════════════════════════════════════
3. PROJECT CONFIG FILES (point at canonical locations)
═══════════════════════════════════════════════════════════════

  /mnt/e/SCMessenger-Github-Repo/SCMessenger/android/local.properties
    sdk.dir=E:/Sdk
    ndk.dir=E:/build-tools/android-sdk/ndk/26.1.10909125
    [NOTE: this is the sdkmanager-managed NDK path. If NDK is restored
     at android-ndk-r26b instead, update this line OR recreate the
     junction. As of Jun 4 2026, the working NDK lives at
     E:\build-tools\android-sdk\ndk\26.1.10909125\ per sdkmanager install.]

  /mnt/e/SCMessenger-Github-Repo/SCMessenger/Cargo.toml
    [workspace] members = ["core", "mobile", "cli", "desktop_bridge", "wasm"]

  /mnt/e/SCMessenger-Github-Repo/SCMessenger/.claude/
    claude config + HANDOFF docs, model capability mapping (17.5KB)

═══════════════════════════════════════════════════════════════
4. ENV VARS REQUIRED FOR ANDROID BUILDS
═══════════════════════════════════════════════════════════════

  Source these in WSL before any gradle/cargo ndk invocation:
    source /mnt/e/build-tools/android-env.sh

  Sets:
    JAVA_HOME       = /mnt/e/build-tools/jdk17
    ANDROID_HOME    = /mnt/e/build-tools/android-sdk
    ANDROID_SDK_ROOT = /mnt/e/build-tools/android-sdk
    ANDROID_NDK_HOME = /mnt/e/build-tools/android-sdk/ndk/26.1.10909125
    GRADLE_USER_HOME = /mnt/e/build-tools/.gradle
    CARGO_HOME      = /mnt/e/build-tools/.cargo
    PATH            += JDK/bin, SDK/platform-tools, SDK/cmdline-tools/latest/bin

  On Windows side (PowerShell), the equivalent is in
  /mnt/e/build-tools/install-android-sdk.ps1 (read for reference).

═══════════════════════════════════════════════════════════════
5. WHAT WAS REMOVED IN THE 2026-06-04 DECONFLICTION
═══════════════════════════════════════════════════════════════

  REMOVED: /mnt/e/scmessenger-repo/     (15GB; stale non-git copy from May 26)
  REMOVED: /mnt/e/sovereign-messenger/  (95MB; unrelated project)

  These were identified as either stale forks, regenerable artifacts, or
  unrelated projects. Tar.gz source backups are in (4).

  Reason: had two copies of SCMessenger code on E: with different content.
  Kept the live one with the git remote (1) and removed the other.

═══════════════════════════════════════════════════════════════
6. CANONICAL ANSWERS TO "WHERE IS X?"
═══════════════════════════════════════════════════════════════

  Q: "Where is the SCMessenger source code?"
  A: /mnt/e/SCMessenger-Github-Repo/SCMessenger/

  Q: "Where do I run gradle/cargo build?"
  A: /mnt/e/SCMessenger-Github-Repo/SCMessenger/  (workspace root)
     or /mnt/e/SCMessenger-Github-Repo/SCMessenger/android/  (for android)

  Q: "Where is the live CLI I just rebuilt?"
  A: /mnt/e/SCMessenger/bin/scmessenger-cli.exe

  Q: "Where is the Android NDK?"
  A: /mnt/e/build-tools/android-sdk/ndk/26.1.10909125
     Accessible via /mnt/e/Sdk/ndk/26.1.10909125 (junction)

  Q: "Where is the Android SDK?"
  A: /mnt/e/build-tools/android-sdk/

  Q: "Where is the JDK?"
  A: /mnt/e/build-tools/jdk17

  Q: "Where do I find the build logs?"
  A: /mnt/e/build-tools/build-apk-wsl-live.log (most recent success)
     and /mnt/e/build-tools/*.log (older runs)

  Q: "Where is Hermes gateway data?"
  A: /mnt/e/hermes-home/  (live config + kanban.db + state)
     /mnt/e/hermes-handoff/  (handoff protocol + c-distro backup)

  Q: "Where is the WSL Ubuntu distro?"
  A: /mnt/e/WSL/Ubuntu/ext4.vhdx  (37GB; do not touch)

  Q: "Where do I put a NEW thing (script, model, tool)?"
  A: If it's SCMessenger code:    /mnt/e/SCMessenger-Github-Repo/SCMessenger/<subdir>/
     If it's a build helper:      /mnt/e/build-tools/
     If it's a running service:   /mnt/e/SCMessenger/bin/
     If it's hermes infrastructure: /mnt/e/hermes-home/ or /mnt/e/hermes-handoff/
     NEVER create new top-level dirs under /mnt/e/ without updating this file.

═══════════════════════════════════════════════════════════════
7. INVARIANTS (any agent can rely on these)
═══════════════════════════════════════════════════════════════

  I1. There is exactly ONE SCMessenger source tree on E:.
      Path: /mnt/e/SCMessenger-Github-Repo/SCMessenger/
      Any other "scmessenger" or "scm" dir is a bug or backup.

  I2. The live CLI binary at /mnt/e/SCMessenger/bin/scmessenger-cli.exe
      is BUILT FROM (1). It is never edited directly. To update, rebuild.

  I3. /mnt/e/build-tools/ contains NO project source. Only toolchain.

  I4. /mnt/e/backup-2026-06-04-pre-unify/ is read-only historical state.

  I5. The Android NDK lives at
      /mnt/e/build-tools/android-sdk/ndk/26.1.10909125
      and is also reachable via the junction at /mnt/e/Sdk/ndk/26.1.10909125.
      If that path is missing, run:
        cmd.exe /C "E:\\build-tools\\android-sdk\\cmdline-tools\\latest\\bin\\sdkmanager.bat --sdk_root=E:\\build-tools\\android-sdk \"ndk;26.1.10909125\""

  I6. /mnt/e/SCMessenger-Github-Repo/SCMessenger/android/local.properties
      points sdk.dir at E:/Sdk and ndk.dir at E:/build-tools/android-sdk/ndk/26.1.10909125.

═══════════════════════════════════════════════════════════════
END OF WORKSPACE.md
═══════════════════════════════════════════════════════════════
