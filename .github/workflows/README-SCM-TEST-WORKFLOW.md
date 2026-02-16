# SCMessenger Test-Diagnose-Fix Workflow

## Prerequisites

Before using this workflow, ensure the following:

### Required GitHub Secrets

The workflow requires the following secret to be configured in your repository:

- **`COPILOT_GITHUB_TOKEN`**: A GitHub token with appropriate permissions for GitHub Copilot CLI
  - Navigate to: Repository Settings → Secrets and variables → Actions → New repository secret
  - Name: `COPILOT_GITHUB_TOKEN`
  - Value: **One of the following options:**
    
    **Option 1 (Recommended): Use the automatic token**
    - Leave this secret unconfigured - the workflow will automatically use `GITHUB_TOKEN`
    - This works if your repository has the default Actions token with sufficient permissions
    
    **Option 2: Use a Personal Access Token (PAT)**
    - Create a PAT at: https://github.com/settings/tokens
    - Required scopes: `repo`, `read:org`, `workflow`
    - Copy the token value (starts with `ghp_` or `github_pat_`)
    - Paste the actual token string as the secret value
    - **Important**: Do NOT use workflow expressions like `${{ secrets.GITHUB_TOKEN }}` - store the actual token value
    
    **Option 3: Use a GitHub App token**
    - Create a GitHub App with appropriate permissions
    - Install the app on your repository
    - Use the app's installation token as the secret value

### Required Repository Permissions

The workflow uses `safe-outputs` to handle PR/issue/comment creation, which means the workflow itself runs with read-only permissions for security. The `safe-outputs` jobs automatically get the necessary write permissions.

However, you should ensure your repository has the following Actions permissions configured:

- **Workflow permissions**: Read and write permissions
  - Navigate to: Repository Settings → Actions → General → Workflow permissions
  - Select: "Read and write permissions"
  - Check: "Allow GitHub Actions to create and approve pull requests"

This allows the `safe-outputs` jobs to create PRs, issues, and comments on behalf of the workflow.

## Overview

This GitHub Agentic Workflow (`scm-test-diagnose-fix.md`) provides automated testing, diagnosis, and fixing for SCMessenger across all platforms and components.

## Quick Start

### Installation

The workflow requires the `gh-aw` CLI extension. Install it:

```bash
# Check if installed
gh aw version

# Install if needed
curl -sL https://raw.githubusercontent.com/github/gh-aw/main/install-gh-aw.sh | bash

# Verify installation
gh aw version
```

### Running the Workflow

The workflow runs automatically:
- **Daily**: Via scheduled runs (fuzzy schedule to avoid load spikes)
- **Manual**: Via GitHub Actions UI → "Run workflow" button

### Monitoring

Check workflow status:
```bash
gh aw status scm-test-diagnose-fix
```

View recent runs:
```bash
gh run list --workflow=scm-test-diagnose-fix.lock.yml
```

## How It Works

### Full-Sweep Testing

The workflow tests ALL 12 domains sequentially in every run:

1. **Core Unit Tests** - Full workspace test suite (638+ tests)
2. **Core Module-by-Module** - Individual module testing (identity, crypto, message, store, transport, drift, routing, relay, privacy)
3. **CLI Build & Tests** - CLI binary build and tests
4. **WASM Build** - WebAssembly target compilation
5. **UniFFI Bindings** - Kotlin and Swift binding generation
6. **Android Build & Tests** - Gradle tests with MockK
7. **Docker Core Tests** - Containerized Rust tests
8. **Docker Network Simulation** - Multi-network P2P messaging tests
9. **Docker NAT Traversal** - NAT gateway simulation tests
10. **Cross-Compatibility** - API consistency across platforms
11. **Clippy & Formatting** - Linting and code style
12. **Security Audit** - Dependency vulnerability scanning

### Workflow Phases

Each run executes all domains through these phases:

1. **Phase 1: Run All Domains Sequentially** - Process all 12 domains in order (1-12)
2. **Phase 2: Test** - Run domain-specific test commands for each domain
3. **Phase 3: Diagnose** - Analyze failures with root cause analysis
4. **Phase 4: Fix** - Apply fixes following code conventions
5. **Phase 5: Re-Test** - Verify fixes and check for regressions (max 5 iterations per domain)
6. **Phase 6: Report** - Create consolidated PR/issue or call noop after all domains complete

### Automated Outputs

#### Pull Requests (Fixes Applied)
- **Prefix**: `[SCM-Fix] Multi-domain:`
- **Labels**: `automated`, `scm-test-fix`
- **Content**: Consolidated root cause analysis across all domains, fixes, test results before/after
- **Consolidation**: One PR per run containing all fixes from all domains
- **Example**: `[SCM-Fix] Multi-domain: Fixes in Core Unit Tests, CLI Build & Tests`

#### Issues (Unfixable Problems)
- **Prefix**: `[SCM-Diag] Multi-domain:`
- **Labels**: `automated`, `scm-diagnosis`
- **Content**: Consolidated breakdown by domain with root cause analysis, manual fix suggestions, reproduction steps
- **Max**: 1 issue per run (consolidates all unfixable problems across domains)
- **Example**: `[SCM-Diag] Multi-domain: Android SDK missing, Docker network timeout`

#### Comments
- Max: 5 comments per run

#### Noop (All Tests Pass)
- Called only after ALL 12 domains complete successfully
- **Example**: `"All 12 domains passed: 900+ tests passed across all domains, no fixes needed"`

## Cache-Memory State

The workflow maintains state across runs using `cache-memory`:

```json
{
  "last_run": "2026-02-15T10:30:00Z",
  "run_number": 42,
  "all_domains_status": {
    "Core Unit Tests": {
      "status": "pass",
      "tests_run": 638,
      "tests_passed": 638,
      "tests_failed": 0,
      "fixes_applied": 0
    },
    "Core Module-by-Module": {
      "status": "pass",
      "tests_run": 145,
      "tests_passed": 145,
      "tests_failed": 0,
      "fixes_applied": 2
    }
  },
  "total_fixes_applied": 2,
  "domains_with_fixes": ["Core Module-by-Module"]
}
```

Each run processes all 12 domains and updates the complete status for all domains.

## Code Conventions

The workflow follows SCMessenger's strict code conventions:

- **All new code is Rust** (no TypeScript/JavaScript)
- **Error handling**: `thiserror` for error types, `anyhow` for binaries
- **Logging**: `tracing` (never `println!` in library code)
- **Concurrency**: `parking_lot::RwLock` over `std::sync::RwLock`
- **Tests**: `#[cfg(test)] mod tests` in same file, integration tests in `tests/`
- **Security**: Zeroize-on-drop for crypto intermediate buffers
- **API stability**: No breaking changes without backward compatibility

## Modifying the Workflow

### Editing Instructions

The workflow file has two parts:

1. **YAML Frontmatter** (between `---` markers): Configuration requiring recompilation
2. **Markdown Body**: Agent instructions that can be edited without recompilation

### Making Changes

**For markdown body edits** (agent instructions):
```bash
# Edit the .md file directly
vim .github/workflows/scm-test-diagnose-fix.md

# Changes take effect on next run (no compilation needed)
```

**For frontmatter edits** (triggers, tools, permissions):
```bash
# Edit the .md file
vim .github/workflows/scm-test-diagnose-fix.md

# Recompile to generate new .lock.yml
gh aw compile scm-test-diagnose-fix

# Commit both files
git add .github/workflows/scm-test-diagnose-fix.md
git add .github/workflows/scm-test-diagnose-fix.lock.yml
git commit -m "Update workflow configuration"
```

### Testing Changes Locally

```bash
# Validate syntax
gh aw compile scm-test-diagnose-fix --strict

# Run security scanners
gh aw compile scm-test-diagnose-fix --actionlint --zizmor --poutine

# Check workflow status
gh aw status scm-test-diagnose-fix
```

## Troubleshooting

### Workflow Not Running

Check trigger configuration:
```bash
gh workflow view scm-test-diagnose-fix.lock.yml
```

Manually trigger:
```bash
gh workflow run scm-test-diagnose-fix.lock.yml
```

### Compilation Errors

Read error details:
```bash
gh aw compile scm-test-diagnose-fix 2>&1 | less
```

Common issues:
- **Unknown property**: Invalid YAML frontmatter field
- **Pinning failed**: Action version resolution issue (usually safe to ignore)
- **Syntax error**: Malformed YAML or markdown

### Cache-Memory Issues

Clear cache to restart cycle:
- Cache-memory is managed by the workflow itself
- To reset: wait for workflow run to complete a full cycle
- Manual intervention not typically needed

## Documentation

- **gh-aw Documentation**: https://github.github.com/gh-aw/
- **Quick Start**: https://github.github.com/gh-aw/setup/quick-start/
- **Reference**: `.github/aw/github-agentic-workflows.md` (comprehensive schema)
- **Workflow Source**: `.github/workflows/scm-test-diagnose-fix.md`

## Support

For issues with:
- **Workflow logic**: Edit `.github/workflows/scm-test-diagnose-fix.md`
- **gh-aw CLI**: https://github.com/github/gh-aw/issues
- **SCMessenger tests**: Refer to test-specific documentation in `docs/`
