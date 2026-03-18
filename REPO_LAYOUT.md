# SCMessenger Repository Layout Guide

Status: Active
Last updated: 2026-03-16

This document defines the **approved locations** for files in this repository to prevent duplicates and ensure consistency.

---

## 1. XCFramework (iOS Core Bindings)

**APPROVED LOCATION:** `iOS/SCMessengerCore.xcframework/`

- This is the canonical location for the prebuilt iOS framework
- The Xcode project (`iOS/SCMessenger.xcodeproj`) references this path
- **NEVER** place a duplicate in the repo root

**If you need to regenerate:**
```bash
cd iOS && ./copy-bindings.sh
```

---

## 2. Build Artifacts

| Artifact Type | Approved Location | .gitignore Pattern |
|---------------|-------------------|-------------------|
| Rust target/ | `core/target/` | `/target`, `core/target/` |
| Android build | `android/app/build/` | `android/app/build/` |
| iOS DerivedData | `iOS/SCMessenger/DerivedData/` | `iOS/SCMessenger/DerivedData/` |
| iOS Build | `iOS/SCMessenger/Build/` | `iOS/SCMessenger/Build/` |
| WASM target | `wasm/target/` | `/target` |

**Rule:** Build artifacts must NEVER be committed. If found, remove with `git rm --cached`.

---

## 3. Logs and Debug Data

| Log Type | Approved Location | .gitignore Pattern |
|----------|-------------------|-------------------|
| Mesh test logs | `logs/5mesh/<timestamp>/` | `logs/*` |
| Android logcat | `android/*.logcat` | `*.logcat` |
| iOS diagnostics | Temporary only | `*.log` |
| Validation logs | `validation_logs_<timestamp>/` | `validation_logs_*/` |

**Rule:** All log files are gitignored by default. Use `logs/` for test runs, but clean up after sessions.

---

## 4. Scripts

| Script Type | Approved Location |
|-------------|-------------------|
| Shell scripts | `scripts/*.sh` |
| Python utilities | `scripts/*.py` |
| Node.js tools | `scripts/*.mjs` or `log-visualizer/` |

**Rule:** No scripts in repo root. Move any stray scripts to `scripts/`.

---

## 5. Documentation

### 5.1 Active Docs (Root Level - Keep Minimal)

Only these files belong in root:
- `README.md` - Entry point
- `DOCUMENTATION.md` - Docs hub
- `AGENTS.md` - Agent policy
- `CONTRIBUTING.md` - Contributor guide
- `SECURITY.md` - Security policy
- `SUPPORT.md` - Support routing
- `CODE_OF_CONDUCT.md` - Code of conduct
- `LICENSE` - License
- `REMAINING_WORK_TRACKING.md` - Active backlog
- `MASTER_BUG_TRACKER.md` - Bug tracker
- `REPO_LAYOUT.md` - This file

### 5.2 Active Docs (docs/)

Keep in `docs/`:
- `CURRENT_STATE.md` - Verified state
- `REPO_CONTEXT.md` - Architecture context
- `DOCUMENT_STATUS_INDEX.md` - Doc lifecycle map
- Milestone plans, risk registers, matrices
- Protocol specs, testing guides

### 5.3 Historical Docs (docs/historical/)

| Category | Location |
|----------|----------|
| Session reports | `docs/historical/session-reports/` |
| Platform audits | `docs/historical/platform-audits/` |
| Bug RCAs | `docs/historical/audits/` |
| Superseded plans | `docs/historical/plans/` |

**Rule:** If a doc is >7 days old and superseded, move it to `docs/historical/`.

---

## 6. PID Files

**APPROVED LOCATION:** None (gitignored)

All `*.pid` files are gitignored. They are runtime artifacts only.

---

## 7. Python Scripts in Root

**APPROVED LOCATION:** `scripts/`

The following were moved from root to `scripts/`:
- `analyze_mesh.py`
- `check_logs.py`
- `patch_api.py`
- `patch_api_rm_nonisolated.py`

**Rule:** No `.py` files in repo root.

---

## 8. iOS Data Directories

**APPROVED LOCATION:** None (gitignored)

The following were removed and gitignored:
- `ios_apps_data/`
- `ios_container/`

These are debug artifacts from device testing.

---

## 9. Duplicate Prevention Checklist

Before committing, verify:

- [ ] No XCFramework in root (only `iOS/SCMessengerCore.xcframework/`)
- [ ] No `.py` files in root (use `scripts/`)
- [ ] No `.log` files tracked (check with `git ls-files "*.log"`)
- [ ] No `.pid` files tracked (check with `git ls-files "*.pid"`)
- [ ] No build artifacts (check `core/target/`, `android/app/build/`)
- [ ] Session reports in `docs/historical/session-reports/`
- [ ] Platform audits in `docs/historical/platform-audits/`
- [ ] Superseded plans in `docs/historical/plans/`

---

## 10. Quick Reference Commands

```bash
# Check for tracked files that should be gitignored
git ls-files "*.log" "*.pid" "*.logcat"

# Remove incorrectly tracked files
git rm --cached <file>

# Check for duplicates in root
ls -la *.py *.framework 2>/dev/null

# Verify doc sync
./scripts/docs_sync_check.sh

# Clean merged branches
git branch --merged main | grep -v "^\*" | grep -v "main" | xargs git branch -d
```

---

## 11. History

| Date | Change |
|------|--------|
| 2026-03-16 | Initial layout guide created during hygiene cleanup |
| 2026-03-16 | Moved 68+ files to `docs/historical/` |
| 2026-03-16 | Removed duplicate XCFramework from root |
| 2026-03-16 | Moved 4 Python scripts to `scripts/` |
| 2026-03-16 | Updated .gitignore with `*.logcat`, `*.pid`, debug dirs |
