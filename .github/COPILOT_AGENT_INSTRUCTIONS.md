# Copilot Agent Explicit Instructions

## Critical Repository Rules

⚠️ **ALL AGENTS AND TOOLS MUST FOLLOW THESE RULES**

### 1. File Storage - STRICT
- ❌ **NEVER** use `/tmp` outside the repo (e.g., `/tmp/file.txt`)
- ❌ **NEVER** use `/var/tmp`, `/dev/shm`, or other system temp locations
- ✅ **ALWAYS** use `/tmp/` at repo root: `/Users/.../SCMessenger/tmp/`
- ✅ Session files go in: `/tmp/session_logs/`, `/tmp/work_files/`, etc.
- ✅ All temp files must be gitignored (already in `.gitignore`)

### 2. Documentation - MANDATORY
- When behavior/scope/risk changes: Update canonical docs in SAME run
- Canonical docs location: See `AGENTS.md` sources 1-7
- Never delete working code unless explicitly required
- Always validate builds after code changes

### 3. Git Operations
- All work must stay in repo
- Use `git add -A` before commits
- Commit messages must be descriptive and include:
  - Which issues fixed
  - Files modified
  - Test status (build, fresh install, etc.)
  - Canonical docs updated

### 4. Build & Test Verification
- After ANY code changes: Run build verification
- Android: `./gradlew assembleDebug -x lint --quiet`
- Never push without successful build
- Document build status in commit message

### 5. Logging & Debugging
- Session work files: `/tmp/session_logs/YYYYMMDD_HHMM/`
- Debug logs: `/tmp/work_files/debug_logs/`
- Audit trails: `/tmp/audit_reports/`
- All must be in repo `/tmp/` subdirectory

---

## Example: Correct File Paths

### ❌ WRONG
```
/tmp/my_session.log
/var/tmp/debug_output.txt
/dev/shm/cache_file
/Users/username/Desktop/temp.txt
```

### ✅ CORRECT
```
/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/tmp/session_logs/my_session.log
/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/tmp/work_files/debug_output.txt
/Users/christymaxwell/Desktop/Luke_Stuff/GitHub/SCMessenger/tmp/audit_reports/cache_file
```

---

## Canonical Documentation Sources (Priority Order)

1. `AGENTS.md` - Agent coordination & rules
2. `DOCUMENTATION.md` - Project documentation index
3. `docs/DOCUMENT_STATUS_INDEX.md` - Doc status tracking
4. `docs/CURRENT_STATE.md` - Current architecture state
5. `REMAINING_WORK_TRACKING.md` - Work tracking & status
6. `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md` - Milestone plan
7. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` - Risk tracking

---

## Session File Organization

When running a session, organize files as:

```
/tmp/
  ├── session_logs/
  │   ├── 20260314_0519_contact_debug/
  │   │   ├── logcat_output.txt
  │   │   ├── build_log.txt
  │   │   └── test_results.txt
  │   └── ...
  ├── work_files/
  │   ├── debug_output.txt
  │   ├── test_data.json
  │   └── ...
  ├── audit_reports/
  │   ├── code_audit_20260314.md
  │   └── ...
  └── notes.txt (session planning, NOT in git)
```

---

## Enforcement

✅ This file is referenced in `.github/copilot-instructions.md`  
✅ All custom agents must acknowledge these rules  
✅ Violations = session termination + manual review required  

---

**Last Updated:** 2026-03-14  
**Status:** ACTIVE - ALL SESSIONS MUST COMPLY
