# SCMessenger Commit Schedule - Official Policy

**Status:** Active | Authority: Lead Orchestrator
**Effective:** 2026-04-15 | Enforcement: Mandatory

## 🎯 Commit Philosophy

**"Commit Validated Work Immediately"** - Never batch fixes. Each validated change deserves its own commit with proper documentation.

## 📅 Commit Tiers & Triggers

### Tier 1: P0 Critical Fixes (IMMEDIATE COMMIT)
```bash
# Trigger: Any P0 bug fix validation
# Scope: Single issue resolution  
# Verification: cargo check + platform build PASS
# Message: "Fix: [Task-ID] - [Description] - [Verification]"

git add [affected_files]
git add MASTER_BUG_TRACKER.md  # Required for bug fixes
git commit -m "Fix: AND-SEND-BTN-001 - Send button responsiveness - Kotlin compile PASS"
```

### Tier 2: Security Hardening (IMMEDIATE + DOCS)
```bash
# Trigger: PHIL rule implementation completion
# Scope: Security module changes
# Verification: cargo test + security review  
# Message: "Security: [PHIL-ID] - [Feature] - [Tests PASS]"

git add core/src/[module]/ *.rs
git add REMAINING_WORK_TRACKING.md  # Required for PHIL compliance
git commit -m "Security: PHIL-005 - Bounded retention - 734/734 tests PASS"
```

### Tier 3: Swarm Coordination (BATCH COMMIT)
```bash
# Trigger: Swarm completion with multiple coordinated changes
# Scope: Thematically related fixes
# Verification: Full test suite PASS
# Message: "Swarm: [Theme] - [Components] - [Validation]"

git add .  # All swarm changes
git commit -m "Swarm: Core Storage - Retention + Encryption + Audit - All integration tests PASS"
```

### Tier 4: Documentation Updates (WITH CODE CHANGES)
```bash
# Trigger: Tracking file updates with related code changes
# Scope: MASTER_BUG_TRACKER.md + REMAINING_WORK_TRACKING.md
# Verification: Content accuracy review
# Message: "Docs: Update [File] - [Changes Made]"

git add MASTER_BUG_TRACKER.md REMAINING_WORK_TRACKING.md
git commit -m "Docs: Update MASTER_BUG_TRACKER - Closed AND-SEND-BTN-001, AND-CONTACTS-WIPE-001"
```

## ⚡ Verification Requirements

| Tier | Required Verification | Timeframe |
|------|---------------------|-----------|
| P0 Fix | `cargo check` + platform build | Immediate |
| Security | `cargo test --workspace --lib` | Immediate |
| Swarm | Full test suite + integration tests | < 15 min |
| Docs | Content review + accuracy check | With code |

## 🔧 Enforcement Mechanisms

### Pre-Commit Checklist (MANDATORY)
- [ ] Code changes validated (appropriate verification tier)
- [ ] Tracking files updated (MASTER_BUG_TRACKER.md / REMAINING_WORK_TRACKING.md)
- [ ] Commit message follows template
- [ ] No breaking changes to CI
- [ ] Documentation reflects current reality

### Post-Commit Validation
- [ ] CI pipeline remains green
- [ ] No regression in functionality
- [ ] Documentation changes accurate
- [ ] Commit message descriptive and searchable

## 🚫 Prohibited Practices

1. **NO** batching unrelated fixes
2. **NO** committing without verification  
3. **NO** breaking CI intentionally
4. **NO** undocumented security changes
5. **NO** vague commit messages

## 📊 Compliance Monitoring

The Lead Orchestrator will:
- Audit commit history every 15 minutes
- Verify tracking file accuracy
- Ensure verification requirements met
- Enforce commit message standards
- Maintain CI pipeline integrity

## 🔄 Exception Process

**Temporary Exceptions** require:
1. Written justification in commit message
2. Explicit risk acknowledgment 
3. Remediation plan with timeline
4. Lead Orchestrator approval

---

*This policy ensures traceability, accountability, and continuous delivery of production-ready code.*