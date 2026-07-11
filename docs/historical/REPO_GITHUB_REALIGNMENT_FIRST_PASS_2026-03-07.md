# SCMessenger GitHub + Repository Realignment Audit (First Pass)

Status: Active
Last updated: 2026-03-07
Scope: Planning-only audit and execution blueprint for repository/GitHub realignment

This document is the planning-first pass requested for `Treystu/SCMessenger`. It audits the repository's current documentation, GitHub operating surface, issue tracker, CI/workflow topology, and contributor/agent context so a follow-up execution pass can cleanly implement the required changes.

## 1. Executive summary

### Biggest repository hygiene problems

1. The intended canonical documentation chain exists, but the actual doc surface is still crowded with mixed-status root docs, stale snapshots, and audit artifacts that read like active truth.
2. Canonical docs are partially current, but contributor-facing entrypoints still drift from verified reality. Examples include stale test counts in `README.md` and `docs/ARCHITECTURE.md`, old scale claims in `CONTRIBUTING.md`, and mixed-status policy language in `SECURITY.md`.
3. Branch hygiene is poor: `main` is not reported as protected, and the repo still contains a large number of stale long-lived branches.

### Biggest GitHub operating-model problems

1. The issue tracker is not functioning as a backlog: all 4 open issues are automation-generated (`#38`, `#39`, `#40`, `#42`) rather than curated engineering work.
2. GitHub feature usage is incomplete or fragmented: issue templates exist, but there is no issue-form/config layer, no `CODEOWNERS`, no Dependabot config, no visible support policy doc, no canonical Copilot instructions file, and Discussions currently return `404`.
3. Repo health signals are weak: releases/tags exist (`v0.1.0`, `v0.1.1`), but release workflow/output does not clearly match the multi-platform product scope.

### Biggest CI/docs drift problems

1. Local baseline validation is healthy (`cargo fmt --all -- --check`, `cargo build --workspace`, `cargo test --workspace`, and `./scripts/docs_sync_check.sh` all pass), but the documentation system still carries stale verification timestamps and counts.
2. Current PR-triggered GitHub Actions runs `22787446333` (`CI`) and `22787446353` (`Docker Build & Push`) concluded `action_required` with no job data exposed, indicating an approval/permissions gate or workflow-trigger policy problem that reduces trust in PR CI evidence.
3. `scripts/docs_sync_check.sh` only validates a narrow subset of the canonical docs chain, so high-value files like `REMAINING_WORK_TRACKING.md`, `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, and `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` can drift without being caught.

### Biggest issue-tracker problems

1. There is no credible active backlog in GitHub Issues right now.
2. The current open issues mix workflow bookkeeping with stale diagnosis, so issue count understates real work while overstating automation noise.
3. Labels/milestones are not operating as a prioritization system.

### Overall cleanup strategy

1. Re-establish a tight canonical docs chain and demote audit/history/process artifacts to supporting or historical roles.
2. Reset the issue tracker around a fresh taxonomy, then close/recreate stale automation issues instead of preserving them as backlog truth.
3. Simplify GitHub operating surfaces so Actions, issues, docs, releases, and agent guidance all point at the same operating model.
4. Treat CI topology and repository governance as release-readiness work, not optional polish.

## 2. Current-state inventory

### Documentation system

- Canonical-entry docs exist: `README.md`, `DOCUMENTATION.md`, `docs/DOCUMENT_STATUS_INDEX.md`, `docs/REPO_CONTEXT.md`, `docs/CURRENT_STATE.md`, and `REMAINING_WORK_TRACKING.md`.
- Additional active execution docs also exist: `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`, `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`, `docs/TESTING_GUIDE.md`, `docs/PLATFORM_SUPPORT_MATRIX.md`, `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`, `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`, and `scripts/README.md`.
- Many root-level docs remain mixed-status or audit-style artifacts: `INSTALL.md`, `NETWORK_TESTING.md`, `QUICKCONNECT.md`, `FEATURE_WORKFLOW.md`, `AUDIT_QUICK_REFERENCE.md`, `BRANCH_AUDIT_REPORT.md`, `AUDIT_RESOLUTIONS.md`, `PRODUCTION_READY.md`, `Latest_Updates.md`, `ANDROID_DISCOVERY_ISSUES.md`, `ANDROID_BUILD_RESOLUTION.md`, `FEATURE_PARITY.md`, `HANDOFF_NEARBY_PEERS.md`, and others.
- `docs/historical/*` is populated and generally acting as the archive, but some newer audit-style documents still live in active root/doc locations.

### GitHub features in use

Verified in use:
- Issues
- Pull requests
- Releases/tags (`v0.1.0`, `v0.1.1`)
- GitHub Actions workflows (`CI`, `Docker Build & Push`, `Docker Test Suite`, `Release Binaries`, disabled `SCMessenger Comprehensive Test → Diagnose → Fix → Re-Test`, plus dynamic Copilot workflows)
- Issue templates (`bug_report.md`, `feature_request.md`, `documentation.md`)
- Pull request template (`.github/pull_request_template.md`)

Verified absent or not aligned:
- No `CODEOWNERS`
- No Dependabot config (`.github/dependabot.yml` missing)
- No issue config / issue forms (`.github/ISSUE_TEMPLATE/config.yml` missing)
- No visible `.github/copilot-instructions.md`
- No visible support policy doc (`SUPPORT.md` missing)
- Discussions return `404`
- No evidence of an active Projects board from public surface
- `main` is not reported as protected by branch metadata

### Issue tracker state

- Open issues: 4 total
- All 4 open issues are automation-generated (`github-actions`)
- Open issues observed:
  - `#38` `[agentics] Failed runs`
  - `#39` `[agentics] SCMessenger Comprehensive Test → Diagnose → Fix → Re-Test failed`
  - `#40` `[agentics] No-Op Runs`
  - `#42` `[SCM-Diag] Multi-domain: Environment restrictions + security audit findings`

### Actions/workflows

- `.github/workflows/ci.yml`
- `.github/workflows/docker-publish.yml`
- `.github/workflows/docker-test-suite.yml`
- `.github/workflows/release.yml`
- `.github/workflows/scm-test-diagnose-fix.lock.yml` (disabled)
- `.github/workflows/scm-test-diagnose-fix.md` (instruction source)
- `.github/workflows/README-SCM-TEST-WORKFLOW.md` (workflow runbook)

### Health/community files

Present:
- `LICENSE`
- `CODE_OF_CONDUCT.md`
- `CONTRIBUTING.md`
- `SECURITY.md`

Missing or incomplete for a world-class public repo:
- `CODEOWNERS`
- `SUPPORT.md`
- issue template configuration / contact links
- Dependabot configuration
- clear branch-protection/status-check documentation aligned to repo reality

### Agent-context files

- `AGENTS.md` - active Codex run policy with mandatory documentation-sync behavior
- `CLAUDE.md` - mixed-status operator/agent context, broad philosophy + codebase guidance
- `SCMessengerSKILL.md` - mixed-status “skill” file with overlapping repo context and implementation guidance
- `.github/workflows/scm-test-diagnose-fix.md` - agentic workflow instructions (operationally related, but not canonical contributor guidance)

## 3. Findings by domain

### 3.1 Documentation

| Finding | Severity | Evidence | Consequence | Proposed fix |
| --- | --- | --- | --- | --- |
| Canonical docs exist, but active navigation still mixes source-of-truth docs with audit artifacts and one-off plans. | High | `DOCUMENTATION.md` lists `docs/global_viability_audit.md` and `docs/implementation_cheatsheet_3.4.2026.md` alongside canonical current-state docs. | Readers cannot tell what is authoritative versus advisory. | Reduce canonical set to current-state + backlog + testing + repo-context docs; move audits/cheatsheets into supporting-audit classification. |
| Entrypoint docs contain stale verified data. | High | `README.md` still shows a 2026-02-23 snapshot of `324 passed`; `docs/ARCHITECTURE.md` repeats `324 passed`; `docs/CURRENT_STATE.md` and `docs/TESTING_GUIDE.md` show `367 passed`. | New contributors and agents can trust the wrong baseline. | Refresh canonical entrypoints, then demote or rewrite stale supporting docs. |
| Supporting and mixed-status docs still contain direct operational claims that should live in canonical docs. | High | `INSTALL.md`, `NETWORK_TESTING.md`, `QUICKCONNECT.md`, `FEATURE_WORKFLOW.md`, `AUDIT_QUICK_REFERENCE.md`, and `BRANCH_AUDIT_REPORT.md` all carry `[Needs Revalidation]` operational guidance. | Operators follow stale procedures; audit outputs masquerade as instructions. | Reclassify by purpose and rewrite only the runbooks that remain actively needed. |
| Doc-validation automation is too narrow. | Critical | `scripts/docs_sync_check.sh` validates headers for only 3 docs and link-checks only `README.md`, `DOCUMENTATION.md`, `docs/DOCUMENT_STATUS_INDEX.md`, and `docs/CURRENT_STATE.md`. | Drift in milestone, backlog, risk, contributor, and security docs can accumulate silently. | Expand doc-sync checks to cover the full canonical chain and reject absolute workstation paths. |
| Some active docs contain machine-local absolute links. | High | `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` contains `/Users/...` file links in evidence sections. | Public docs are not portable and cannot be trusted outside one machine. | Convert all active-doc evidence links to repo-relative paths during execution pass. |

### 3.2 Issues / backlog governance

| Finding | Severity | Evidence | Consequence | Proposed fix |
| --- | --- | --- | --- | --- |
| The open issue tracker is automation noise, not product/project management. | Critical | All 4 open issues are automation-generated; no curated bug/docs/CI issues are open. | Maintainers cannot use Issues as a credible backlog. | Perform a tracker reset: triage every open issue, recreate only validated work, and introduce a fresh taxonomy. |
| Workflow bookkeeping is being stored as issues. | High | `#38`, `#39`, and `#40` are agentic workflow state containers rather than engineering work items. | Backlog hygiene and search quality degrade immediately. | Close or retire workflow-bookkeeping issues and move workflow health into Actions/reports/discussions if needed. |
| Issue templates are too generic for the repo’s actual complexity. | Medium | Only three Markdown templates exist; no CI, security/privacy, audit-follow-up, or regression-report specific intake path. | Important issue classes will keep being filed inconsistently. | Replace with issue forms or stronger templates aligned to the new taxonomy. |

### 3.3 GitHub features / configuration

| Finding | Severity | Evidence | Consequence | Proposed fix |
| --- | --- | --- | --- | --- |
| `main` is not protected. | Critical | Branch listing reports `main` with `protected: false`. | No guaranteed status checks, review requirements, or accidental-push guardrails. | Define and document branch protection as part of the execution pass before backlog reset. |
| Branch hygiene is poor. | High | Branch list shows many stale `claude/*`, `copilot/*`, `v0/*`, and duplicate `main-*` branches. | Repo state appears abandoned/noisy; branch review becomes costly. | Run a post-triage branch cleanup with explicit keep/delete rules. |
| Community-health surface is incomplete. | High | `CODEOWNERS`, Dependabot config, support policy, and issue config are missing. | Maintainer load stays manual and contributor expectations remain ambiguous. | Add minimal health/config files after canonical docs are fixed. |
| Discussions are not currently a usable support surface. | Medium | `https://github.com/Treystu/SCMessenger/discussions` returns `404`. | `CONTRIBUTING.md` points users to a feature that is not active. | Either enable and curate Discussions or remove/replace the reference. |
| PR template is broad but not aligned to repo governance needs. | Medium | `.github/pull_request_template.md` lacks explicit docs-sync scope, canonical-doc references, and issue taxonomy mapping. | Review quality depends on contributor memory. | Update PR template after docs/CI operating model is finalized. |

### 3.4 CI/CD / Actions

| Finding | Severity | Evidence | Consequence | Proposed fix |
| --- | --- | --- | --- | --- |
| CI topology exists but is not yet a coherent operating model. | High | `ci.yml`, `docker-test-suite.yml`, `docker-publish.yml`, `release.yml`, and the disabled agentic workflow all overlap partially. | It is unclear which checks are required, optional, nightly, or release-only. | Re-map workflows into required PR gates, optional heavy/nightly suites, and release-only automation. |
| Current PR CI evidence is blocked by `action_required` runs. | High | Runs `22787446333` (`CI`) and `22787446353` (`Docker Build & Push`) ended `action_required`; no jobs were exposed via job/log APIs. | Maintainers cannot rely on PR CI for fast feedback. | Investigate approval/permissions policy and remove unnecessary PR-triggered workflows that require manual intervention. |
| Docker Build & Push is misaligned as a PR workflow. | Medium | `docker-publish.yml` runs on `pull_request` even though push is disabled there. | PR signal is noisy and expensive without producing release artifacts. | Restrict image publishing workflow to `main`, tags, or manual dispatch; use a separate image-build verification job if needed. |
| Release workflow scope is CLI-only while repo scope is multi-platform. | Medium | `release.yml` only builds `scmessenger-cli` assets for desktop targets. | Releases can appear “complete” without representing Android/iOS/WASM readiness. | Document release surface honestly and add platform-ready release criteria before broad releases. |
| Disabled agentic workflow still leaks issue noise and maintenance surface. | Medium | Workflow `SCMessenger Comprehensive Test → Diagnose → Fix → Re-Test` is disabled manually, but related docs/issues remain active. | Users see stale automation as part of the operating model. | Either fully retire it (docs + issues + labels) or reintroduce it with explicit ownership and guardrails. |

### 3.5 Contributor experience

| Finding | Severity | Evidence | Consequence | Proposed fix |
| --- | --- | --- | --- | --- |
| Build/test entrypoints are discoverable, but surrounding instructions drift quickly. | High | `README.md` quick-start commands are fine, but `CONTRIBUTING.md` cites `~638 tests` and points to Discussions that 404. | New contributors get mixed trust signals immediately. | Rebuild contributor docs around verified commands, canonical doc links, and actual support/reporting channels. |
| Security reporting is not cleanly presented. | High | `SECURITY.md` is mixed-status and claims “CodeQL security scanning in CI/CD” without a visible CodeQL workflow. | Security posture reads less trustworthy than it should. | Rewrite `SECURITY.md` as a concise, current policy and move older context elsewhere. |
| Role-specific workflows are not explicitly documented. | Medium | Maintainer/release-manager/operator paths are spread across `README.md`, `DOCUMENTATION.md`, `scripts/README.md`, release docs, and mixed root docs. | Work is slower and more person-dependent than necessary. | Add a lean maintainer-operations guide after the docs architecture is simplified. |

### 3.6 Agent / instruction context

| Finding | Severity | Evidence | Consequence | Proposed fix |
| --- | --- | --- | --- | --- |
| Agent guidance is fragmented. | High | `AGENTS.md`, `CLAUDE.md`, and `SCMessengerSKILL.md` all contain overlapping repo philosophy, architecture, and workflow rules. | Different agents can receive contradictory or differently-aged guidance. | Create one canonical repo-agent context file; demote platform-specific adapters to support docs. |
| There is no canonical GitHub Copilot instructions file in `.github/`. | Medium | No `.github/copilot-instructions.md` exists. | Copilot-specific guidance is harder to discover and standardize. | Add a concise Copilot instructions file that points at the canonical repo-agent doc. |
| Agentic workflow docs and repo guidance are blended. | Medium | `.github/workflows/README-SCM-TEST-WORKFLOW.md` acts like contributor guidance for a disabled automation system. | Repo operators can mistake one workflow’s runbook for general repo policy. | Reclassify it as internal/automation support or archive it. |

### 3.7 Claims that require correction or explicit revalidation

1. `README.md` still presents a **2026-02-23** workspace test snapshot of **324 passed**, while current canonical verification docs show **367 passed**.
2. `docs/ARCHITECTURE.md` repeats the same stale **324 passed** claim.
3. `CONTRIBUTING.md` says “We currently have ~638 tests across all modules,” which no longer matches the canonical current-state verification snapshot.
4. `CONTRIBUTING.md` sends users to GitHub Discussions, but Discussions currently return `404`.
5. `SECURITY.md` says “CodeQL security scanning in CI/CD,” but no repository CodeQL workflow is visible in `.github/workflows/`.
6. `docs/REPO_CONTEXT.md` still says “Last reviewed: 2026-02-23” even though it is part of the active canonical chain.
7. `docs/GLOBAL_ROLLOUT_PLAN.md`, `docs/global_viability_audit.md`, and `docs/implementation_cheatsheet_3.4.2026.md` contain active planning value, but should not be treated as current-state truth without explicit supporting-status labeling.
8. `QUICKCONNECT.md` advertises embedded bootstrap/GCP node assumptions that require explicit revalidation against the current relay/bootstrap operating model.
9. `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md` contains workstation-local absolute links and should not be treated as portable evidence until rewritten.

## 4. Issue tracker re-evaluation plan

### Diagnosis of why the current issues are stale or misaligned

1. The repo used GitHub Issues as an output sink for automation rather than a curated engineering backlog.
2. Agentic workflow failure/no-op parent issues (`#38`, `#39`, `#40`) were never separated from real product work.
3. The remaining open diagnosis issue (`#42`) mixes environment blockers, dependency security findings, and stale workflow-specific remediation into one large bundle.
4. Labels currently visible in the open set (`agentic-workflows`, `automated`, `scm-diagnosis`) do not form a reusable issue taxonomy.

### Proposed disposition for each current open issue

| Issue | Current role | Proposed action | Why |
| --- | --- | --- | --- |
| `#38` `[agentics] Failed runs` | Parent tracker for workflow failures | Close as obsolete after recording the disabled-workflow decision in docs | This is workflow bookkeeping, not backlog work. |
| `#39` `[agentics] ... failed` | Single failed workflow run | Close as obsolete/superseded | It tracks one expired secret-validation incident for a disabled automation path. |
| `#40` `[agentics] No-Op Runs` | No-op tracker | Close as obsolete | No-op logging does not belong in product backlog. |
| `#42` `[SCM-Diag] Multi-domain...` | Bundled diagnosis issue | Close and recreate cleanly | It mixes stale environment constraints, dependency findings, and workflow-specific reproduction notes; only validated follow-up items should survive. |

### Proposed new issue taxonomy

Use a small stable taxonomy with one primary `type/*` label and one or more secondary axes.

**Primary type labels**
- `type/bug`
- `type/docs`
- `type/ci-cd`
- `type/platform-parity`
- `type/release-blocker`
- `type/maintenance`
- `type/audit-follow-up`
- `type/security-privacy`
- `type/infrastructure-relay`
- `type/ux-product`

**Secondary area labels**
- `area/core`
- `area/android`
- `area/ios`
- `area/wasm-web`
- `area/cli`
- `area/docs`
- `area/github`
- `area/actions`
- `area/release`

**Priority/state labels**
- `priority/p0`
- `priority/p1`
- `priority/p2`
- `status/needs-triage`
- `status/blocked`
- `status/ready`
- `status/in-progress`
- `status/decision-needed`

**Process labels**
- `source/audit`
- `source/runtime-evidence`
- `source/ci`
- `good-first-issue`
- `help-wanted`

### Proposed milestone plan

1. `ops-reset` - docs architecture, GitHub hygiene, issue tracker reset, branch cleanup, templates, CODEOWNERS, support policy.
2. `v0.2.0-alpha-closeout` - remaining runtime/reliability/release-blocking work already tracked in current canonical execution docs.
3. `v0.2.1-post-alpha` - deferred scope already represented by planned docs (`WS13` / `WS14`).

Do **not** use milestones for one-off automation noise or generic backlog dumping.

### Rules for close vs recreate vs rewrite

- **Close as obsolete** when the issue tracks workflow bookkeeping, expired incidents, or already-disabled automation.
- **Close and recreate** when an issue bundles multiple unrelated root causes, stale environment assumptions, or out-of-date repro steps.
- **Rewrite in place** only when the issue number already has active discussion/history that is still valuable.
- **Split** when one issue spans multiple owners or validation paths (for example docs+CI+security in one ticket).
- **Convert to doc checklist instead of issue** when the work is purely canonical-doc synchronization and does not need separate lifecycle tracking.

### Migration sequence

1. Freeze issue creation rules by adding the new issue taxonomy/templates first.
2. Audit every existing open issue and label it with a proposed disposition.
3. Close `#38`, `#39`, and `#40` with a short note pointing to Actions/workflow docs.
4. Extract still-valid work from `#42` into discrete new issues (security/dependency, CI environment policy, or workflow retirement) only after revalidation against current repo state.
5. Create fresh audit-follow-up issues for the highest-priority execution tasks from this document.
6. Only then enable normal backlog intake again.

### Risks of doing nothing

- The issue tracker will remain untrustworthy.
- Maintainers will keep using docs and PRs as an ad-hoc backlog.
- Automation noise will continue to mask real release blockers.
- External contributors will not know where valid work actually lives.

## 5. Documentation realignment plan

### Intended documentation architecture

1. **Canonical current-state docs** - current truth only.
2. **Active supporting docs** - detailed matrices, release/risk artifacts, and bounded audit references that support but do not override canonical docs.
3. **Operational runbooks** - setup and execution instructions that must be revalidated regularly.
4. **Contributor-facing guides** - contributing, security, code of conduct, support.
5. **Release/process docs** - milestone, residual risk, release notes, operator workflows.
6. **Historical/archive docs** - old audits, superseded status reports, legacy plans.
7. **Agent/internal instruction docs** - AI/operator guidance and automation-specific runbooks.

### Actual documentation architecture today

- The intended categories exist, but active navigation still over-promotes audits and mixed-status documents.
- Root-level docs contain too many overlapping runbooks, audits, and historical snapshots.
- Some active docs still contain stale or machine-local evidence.
- Contributor/support/security docs are not cleanly separated from mixed historical context.

### Proposed classification by document set

#### Canonical current-state docs (retain as canonical)
- `README.md`
- `DOCUMENTATION.md`
- `docs/DOCUMENT_STATUS_INDEX.md`
- `docs/REPO_CONTEXT.md`
- `docs/CURRENT_STATE.md`
- `REMAINING_WORK_TRACKING.md`
- `docs/TESTING_GUIDE.md`

#### Active supporting docs (keep, but not canonical truth)
- `AGENTS.md`
- `docs/MILESTONE_PLAN_V0.2.0_ALPHA.md`
- `docs/V0.2.0_RESIDUAL_RISK_REGISTER.md`
- `docs/PLATFORM_SUPPORT_MATRIX.md`
- `docs/INTEROP_MATRIX_V0.2.0_ALPHA.md`
- `docs/WS12.29_KNOWN_ISSUES_BURNDOWN_PLAN.md`
- `scripts/README.md`
- `docs/PROTOCOL.md`
- `docs/EDGE_CASE_READINESS_MATRIX.md`
- `docs/REPO_GITHUB_REALIGNMENT_FIRST_PASS_2026-03-07.md`

#### Operational runbooks (keep only if revalidated)
- `INSTALL.md`
- `QUICKCONNECT.md`
- `NETWORK_TESTING.md`
- `DOCKER_QUICKSTART.md`
- `DOCKER_TEST_QUICKREF.md`
- `GCP_DEPLOY_GUIDE.md`
- `BOOTSTRAP.md`
- `docs/RELAY_OPERATOR_GUIDE.md`
- `docs/BOOTSTRAP_GOVERNANCE.md`
- `docs/NAT_TRAVERSAL_GUIDE.md`
- `FEATURE_WORKFLOW.md`
- component READMEs (`core/README.md`, `cli/README.md`, `mobile/README.md`, `wasm/README.md`, `android/README.md`, `iOS/README.md`)

#### Contributor-facing guides (rewrite for clarity)
- `CONTRIBUTING.md`
- `SECURITY.md`
- `CODE_OF_CONDUCT.md`

#### Release/process docs (keep, but tighten scope)
- `docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md`
- `docs/releases/RELEASE_NOTES_V0.2.0_DRAFT.md`
- `docs/releases/RELEASE_NOTES_V0.1.2_GH.md`
- `docs/GLOBAL_ROLLOUT_PLAN.md`
- `docs/UNIFIED_GLOBAL_APP_PLAN.md`

#### Agent-context / internal-instructions docs
- `AGENTS.md`
- `CLAUDE.md`
- `SCMessengerSKILL.md`
- `.github/workflows/scm-test-diagnose-fix.md`
- `.github/workflows/README-SCM-TEST-WORKFLOW.md`

#### Historical/archive docs (explicit archive)
- `docs/historical/*`
- `reference/historical/*`
- Root audit/history docs that should be archived or reclassified:
  - `AUDIT_QUICK_REFERENCE.md`
  - `AUDIT_RESOLUTIONS.md`
  - `BRANCH_AUDIT_REPORT.md`
  - `Latest_Updates.md`
  - `PRODUCTION_READY.md`
  - `ANDROID_DISCOVERY_ISSUES.md`
  - `ANDROID_BUILD_RESOLUTION.md`
  - `HANDOFF_NEARBY_PEERS.md`
  - `GEMINI_UI_GUIDE.md`
  - `DRIFTNET_MESH_BLUEPRINT.md`
  - `INTEGRATION_VERIFICATION.md`
  - `FEATURE_PARITY.md`

### Docs to merge / remove / archive / rewrite

- **Merge into canonical docs:** stale verification data from `README.md`, `docs/ARCHITECTURE.md`, `CONTRIBUTING.md`, and `SECURITY.md`.
- **Archive or explicitly demote:** `docs/global_viability_audit.md`, `docs/implementation_cheatsheet_3.4.2026.md`, `AUDIT_QUICK_REFERENCE.md`, `BRANCH_AUDIT_REPORT.md`, `PRODUCTION_READY.md`.
- **Rewrite as active runbooks:** `INSTALL.md`, `QUICKCONNECT.md`, `NETWORK_TESTING.md`.
- **Rewrite contributor-facing:** `CONTRIBUTING.md`, `SECURITY.md`.
- **Rewrite/replace agent docs:** `CLAUDE.md`, `SCMessengerSKILL.md`, `.github/workflows/README-SCM-TEST-WORKFLOW.md`.

### New docs to create in execution pass

1. `.github/copilot-instructions.md` - short adapter pointing at canonical repo-agent guidance.
2. `SUPPORT.md` - clear support/reporting routing (security vs bugs vs questions).
3. `docs/MAINTAINER_OPERATIONS.md` or similar - maintainer/release/operator workflow map.
4. `docs/RELEASE_PROCESS.md` or equivalent if release docs continue to spread across multiple files.

## 6. GitHub feature realignment plan

### Target GitHub operating model

#### Issues
- Keep Issues enabled.
- Reset the tracker around the new taxonomy.
- Use issue forms/config so support/security questions are routed away from the backlog.

#### Labels
- Introduce the type/area/priority/status/source label system from Section 4.
- Retire automation-only labels from day-to-day backlog views unless the corresponding automation remains active.

#### Milestones
- Use only 2-3 active milestones at a time (`ops-reset`, `v0.2.0-alpha-closeout`, `v0.2.1-post-alpha`).
- Avoid milestone sprawl.

#### Templates
- Replace generic Markdown issue templates with structured issue forms for:
  - bug
  - docs
  - CI/CD
  - audit follow-up
  - security/privacy report redirection
- Rewrite PR template to require:
  - canonical docs touched or explicitly not needed
  - local/CI validation summary
  - linked issue or rationale for no issue
  - risk/security notes

#### CODEOWNERS / community health
- Add minimal `CODEOWNERS`.
- Keep `CODE_OF_CONDUCT.md`.
- Rewrite `SECURITY.md` and add `SUPPORT.md`.
- Keep `CONTRIBUTING.md`, but simplify and align it to real build/test flows.

#### Discussions / Projects
- **Discussions:** either enable intentionally with limited categories (`Announcements`, `Q&A`, `Operator reports`) or stop referencing them. Default recommendation: do not enable until maintainer bandwidth exists.
- **Projects:** do not introduce a GitHub Project until the issue taxonomy reset is complete. Milestones + labels are sufficient first.

#### Release process surfaces
- Keep GitHub Releases.
- Align release notes, tag strategy, and release workflow with actual supported deliverables.
- Document clearly whether releases are CLI-only, operator artifacts, or full product milestones.

#### Branch strategy / protections
- Protect `main`.
- Require the final required CI set before merge.
- Clean up stale branches after open PR/issue triage.

#### Dependency/update automation
- Add minimal Dependabot coverage for:
  - Cargo
  - GitHub Actions
  - Gradle
- Only after maintainers are ready to triage update PRs.

## 7. CI/CD and Actions remediation plan

### Current workflow map

| Workflow | Current role | Current assessment |
| --- | --- | --- |
| `ci.yml` | Primary repo validation | Closest thing to the required gate; should remain the main PR validator. |
| `docker-test-suite.yml` | Containerized test coverage + NAT simulation | Valuable, but too heavy/redundant for every PR. Better as scheduled/manual or selective. |
| `docker-publish.yml` | Docker image build/publish | Misaligned on PRs; likely should not run for every pull request. |
| `release.yml` | Desktop CLI release artifacts | Useful, but too narrow to represent full product readiness. |
| `scm-test-diagnose-fix.lock.yml` | Agentic diagnose/fix automation | Disabled; related issue/docs noise should be triaged decisively. |
| Dynamic Copilot workflows | GitHub platform-managed automation | Fine to keep, but not part of SCMessenger’s core operating model. |

### Target workflow map

#### Required PR checks
1. `repo-hygiene`
   - path governance
   - docs sync / canonical doc validation
   - workflow/actionlint validation if added
2. `rust-core`
   - fmt
   - clippy
   - build
   - workspace tests
   - deterministic core integration suites
3. `wasm-web`
   - target build
   - `wasm-pack` browser tests
4. `android`
   - build
   - targeted unit/parity suites
5. `ios`
   - `iOS/verify-test.sh`

#### Optional / heavy / scheduled checks
6. `docker-integration`
   - rust/docker integration and NAT simulations
   - scheduled, manual, or merge-queue only until stabilized
7. `dependency-audit`
   - security/dependency scanning if maintainers want it in Actions rather than ad hoc

#### Release-only checks
8. `release-cli` or renamed release workflow
   - only on tags/manual dispatch
   - clearly scoped to desktop CLI artifacts unless expanded

### Broken or risky workflows

1. PR runs ending in `action_required` without jobs should be investigated first.
2. `docker-publish.yml` on PR is a noisy non-gate.
3. Disabled agentic workflow still has public operational residue (issues/docs).
4. Release workflow name/scope is broader than its actual output.

### Mandatory vs optional checks

**Mandatory**
- path/docs governance
- Rust fmt/clippy/build/tests
- wasm build/tests
- Android targeted build/tests
- iOS verification script

**Optional until stabilized**
- Docker NAT/network simulation
- image publishing verification
- agentic diagnose/fix automation

### Docs/CI coupling improvements

1. Make `docs/TESTING_GUIDE.md` the single source for local equivalents of required PR checks.
2. Update `README.md` and `CONTRIBUTING.md` to point to the same command set.
3. Expand `scripts/docs_sync_check.sh` or add a broader repo-doc validation layer.
4. Document which workflows are required for merge and which are advisory/manual.

### Local-dev command parity improvements

Document one standard local validation ladder:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -A clippy::empty_line_after_doc_comments
cargo build --workspace
cargo test --workspace
./scripts/docs_sync_check.sh
```

Then platform-specific validation:
- Android targeted Gradle suite
- `bash ./iOS/verify-test.sh`
- `cd wasm && wasm-pack test --headless --firefox`

## 8. Execution roadmap

### Phase 0: inventory + guardrails
- **Goals:** freeze the plan, verify current repo/GitHub evidence, define canonical operating model boundaries.
- **Likely files/surfaces:** `DOCUMENTATION.md`, `docs/DOCUMENT_STATUS_INDEX.md`, planning audit doc, issue/branch/workflow inventory.
- **Dependencies:** none.
- **Risk notes:** avoid editing many supporting docs before the target model is agreed.
- **Recommended validation:** docs link checks; re-run GitHub inventory after any repo-config changes.

### Phase 1: canonical docs + repo health
- **Goals:** tighten canonical docs chain, refresh stale entrypoint claims, rewrite contributor/security/support surfaces, archive obvious audit/history docs.
- **Likely files/surfaces:** `README.md`, `DOCUMENTATION.md`, `docs/DOCUMENT_STATUS_INDEX.md`, `docs/CURRENT_STATE.md`, `REMAINING_WORK_TRACKING.md`, `CONTRIBUTING.md`, `SECURITY.md`, `SUPPORT.md`, `docs/ARCHITECTURE.md`.
- **Dependencies:** Phase 0 agreement on canonical/supporting categories.
- **Risk notes:** doc churn is high; use small PRs.
- **Recommended validation:** `./scripts/docs_sync_check.sh`; manual link review; contributor path sanity walk-through.

### Phase 2: issue tracker reset + GitHub feature realignment
- **Goals:** add templates/config, labels, milestones, CODEOWNERS, support routing; re-triage and recreate issues.
- **Likely files/surfaces:** `.github/ISSUE_TEMPLATE/*`, `.github/ISSUE_TEMPLATE/config.yml`, `.github/pull_request_template.md`, `.github/CODEOWNERS`, `SUPPORT.md`, GitHub labels/milestones/issues.
- **Dependencies:** Phase 1 contributor/support docs.
- **Risk notes:** close/recreate rules must be applied consistently or history will get noisier before it gets cleaner.
- **Recommended validation:** create dry-run issue examples; verify templates/routes in GitHub UI.

### Phase 3: CI/CD repair + docs/CI coupling
- **Goals:** define required vs optional workflows, remove PR-noisy jobs, repair any approval/permission gating, align docs to the real workflow map.
- **Likely files/surfaces:** `.github/workflows/*.yml`, `.github/workflows/README-SCM-TEST-WORKFLOW.md`, `docs/TESTING_GUIDE.md`, `CONTRIBUTING.md`, `scripts/docs_sync_check.sh`.
- **Dependencies:** Phase 1 docs cleanup and Phase 2 protection/template decisions.
- **Risk notes:** changing required checks before branch protection is defined can break merge flow.
- **Recommended validation:** workflow lint/validation, targeted Actions runs, local command parity verification.

### Phase 4: agent-context consolidation
- **Goals:** unify repo-agent guidance, add Copilot instructions, demote stale duplicated agent files.
- **Likely files/surfaces:** `AGENTS.md`, `CLAUDE.md`, `SCMessengerSKILL.md`, `.github/copilot-instructions.md`, workflow-specific agent docs.
- **Dependencies:** canonical doc architecture from Phase 1.
- **Risk notes:** avoid deleting useful platform-specific instructions without replacements.
- **Recommended validation:** agent read-path sanity check against one representative coding task.

### Phase 5: final validation + cleanup
- **Goals:** close stale branches/issues, verify branch protections/status checks, ensure release/process docs match reality.
- **Likely files/surfaces:** GitHub branches, protections, issues, releases, milestone docs, residual risk register.
- **Dependencies:** prior phases complete.
- **Risk notes:** branch cleanup should happen only after open PR decisions are made.
- **Recommended validation:** end-to-end maintainer workflow rehearsal from issue → PR → CI → merge → release notes.

## 9. Recommended first execution PR(s)

Recommendation: **a staged sequence of a few structured PRs**, not one large PR.

### PR 1 - Canonical docs + community health reset
**Title:** `docs: realign canonical docs and contributor health surfaces`
- README/current-state/testing/contributing/security/support cleanup
- doc architecture tightening
- archive/demote obvious audit artifacts

### PR 2 - GitHub operating model reset
**Title:** `github: reset issue intake, labels, milestones, and ownership rules`
- issue forms/config
- PR template rewrite
- CODEOWNERS
- label/milestone setup
- issue migration execution

### PR 3 - CI topology and workflow cleanup
**Title:** `ci: align required checks with repo scope and local validation`
- workflow role cleanup
- docs/CI coupling
- docs sync enforcement expansion
- retirement or repair of disabled/noisy automation

### PR 4 - Agent-context consolidation
**Title:** `docs: consolidate agent instructions and archive duplicated guidance`
- canonical agent context
- `.github/copilot-instructions.md`
- CLAUDE/skill doc rewrite or demotion

## 10. Immediate no-regret actions

1. Protect `main` and define required status checks.
2. Reclassify `docs/global_viability_audit.md` and `docs/implementation_cheatsheet_3.4.2026.md` as supporting audits instead of canonical current-state docs.
3. Refresh stale test-count claims in `README.md`, `docs/ARCHITECTURE.md`, and `CONTRIBUTING.md`.
4. Rewrite `SECURITY.md` into a current, portable policy and remove non-portable/stale claims.
5. Add `SUPPORT.md` and stop pointing contributors at disabled/missing support surfaces.
6. Add `CODEOWNERS`.
7. Stop using open issues for agentic workflow bookkeeping; close `#38`, `#39`, and `#40`.
8. Split or recreate `#42` into validated, current issues only.
9. Remove `pull_request` triggering from `docker-publish.yml` unless there is a real PR gate need.
10. Expand `scripts/docs_sync_check.sh` to cover the full canonical docs chain and reject absolute-machine links.
