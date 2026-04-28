# Implementation Plan

[Overview]
Consolidate all legacy AI tool configurations into a unified `.clinerules` agentic swarm protocol and purge obsolete files.

The SCMessenger repository had accumulated configuration files from multiple AI tools (Roo Code, Claude, GitHub Copilot, LeansTral, custom agent markdowns). These scattered configs created conflicts, duplication, and maintenance burden. This plan extracts all knowledge from those legacy files, synthesizes it into a single authoritative `.clinerules`, creates a production roadmap based on codebase reality, and provides a purge command to eliminate the legacy files. The `.clinerules` defines the Cline Agentic Swarm Protocol using specific Ollama Cloud Pro models, establishes non-negotiable architectural constraints (Driftnet mesh, identity unification, Rust core dominance), mandates build verification, and declares itself the single source of truth for agent behavior.

[Types]
No new types, interfaces, enums, or data structures are introduced.

This is a configuration consolidation task. The existing type system in the Rust codebase (`core/src/`) remains unchanged. The "types" here are the agentic role definitions encoded in `.clinerules` §1:

- **Architect** role: `kimi-k2.5` / `kimi-k2-thinking` — system design, planning, PHILOSOPHY_CANON enforcement
- **Backend Coder** role: `glm-5.1` / `qwen3.5:397b` — massive refactors, complex Rust, multi-file edits
- **Mobile Executer** role: `glm-5.1` / `qwen3.5:397b` — Kotlin/Compose, SwiftUI, UniFFI bindings
- **Security Auditor** role: `deepseek-v3.2` — crypto audit, protocol review, penetration testing
- **Fast Executer** role: `gemma4:31b` / `minimax-m2.7` — CLI commands, quick tests, minor edits <50LOC

[Files]
Two new files created; 12+ legacy files/folders targeted for deletion.

- **NEW: `.clinerules`** (root) — Master agentic swarm protocol. 16 sections covering sovereignty declaration, model assignments, architectural constraints, cryptography, build verification, documentation sync, testing, Rust conventions, platform rules, file storage, log extraction, estimation, prohibited actions, dependencies, philosophy enforcement, integration patterns, canonical doc sources. Synthesizes knowledge from ALL legacy files listed below.
- **NEW: `PRODUCTION_ROADMAP.md`** (root) — Reality check against codebase + 5-phase roadmap to v1.0. Phase 1: Stability, Phase 2: Core Wiring, Phase 3: Privacy/Security, Phase 4: Platform Parity, Phase 5: Production Release.
- **DELETE: `.roo/`** — Entire directory (mcp.json, memory-bank/, rules/ with 8 rule files, rules-scm-rust/)
- **DELETE: `.roomodes/`** — Entire directory (8 JSON mode files: scm-android, scm-debug-mesh, scm-docs, scm-ios, scm-protocol, scm-release, scm-repo-agent, scm-rust)
- **DELETE: `skills/`** — Entire directory (5 skill subdirs: mesh-diagnostics, philosophy-enforcer, platform-parity-check, release-gate-validator, scm-repo-agent)
- **DELETE: `AGENTS.md`** — Legacy Codex/Copilot agent coordination file
- **DELETE: `CLAUDE.md`** — Legacy Claude-specific instructions
- **DELETE: `BOOTSTRAP.md`** — Legacy bootstrap node documentation (knowledge moved to `.clinerules` §8.4, §15)
- **DELETE: `SCMessengerSKILL.md`** — Legacy custom skill definition
- **DELETE: `scmessenger-agent.md`** — Legacy custom agent profile
- **DELETE: `leanstral-integration.md`** — Legacy LeansTral integration guide
- **DELETE: `leanstral-execution-workflow.md`** — Legacy LeansTral workflow
- **DELETE: `.github/copilot-instructions.md`** — Legacy GitHub Copilot instructions
- **DELETE: `.github/COPILOT_AGENT_INSTRUCTIONS.md`** — Legacy GitHub Copilot agent instructions
- **DELETE: `scmessenger-roo-code-features-plan.md`** — Legacy Roo Code features plan
- **DELETE: `workflow-improvements.md`** — Legacy LeansTral workflow improvements
- **DELETE: `docs/roo_task_mar-16-2026_5-17-28-pm.md`** — Legacy Roo task log

**PRESERVED (not deleted):**
- `.github/workflows/` — CI/CD pipelines (per `.clinerules` §0)
- `reference/PHILOSOPHY_CANON.md` — Product philosophy (per `.clinerules` §0, §14)
- All source code, documentation, and scripts

[Functions]
No function modifications.

This task does not modify any Rust, Kotlin, Swift, or JavaScript functions. It only creates configuration files and targets legacy AI config files for deletion.

[Classes]
No class modifications.

This task does not modify any classes, structs, or type definitions in the codebase.

[Dependencies]
No dependency modifications.

No `Cargo.toml`, `package.json`, `build.gradle`, or other dependency manifests are changed.

[Testing]
Verification is configuration-based, not unit-test-based.

- **`.clinerules` validation**: Verify all 16 sections present, all Ollama model strings exact, all PHIL rules enumerated, all build commands match actual scripts
- **`PRODUCTION_ROADMAP.md` validation**: Verify reality check matches actual `core/src/` file structure, verify known gaps align with `REMAINING_WORK_TRACKING.md`, verify LOC estimates are reasonable
- **Purge command validation**: Verify the command targets ONLY legacy AI config files, does NOT delete source code, docs, scripts, or `.github/workflows/`
- **Post-purge verification**: After running purge, `git status` should show only deleted legacy files + new `.clinerules` + `PRODUCTION_ROADMAP.md`

[Implementation Order]
Step-by-step execution sequence (ALL STEPS COMPLETED).

1. ✅ Read all files in `.roo/` directory (mcp.json, memory-bank/activeContext.md, projectbrief.md, techContext.md, rules/000-critical.md through 060-testing.md, rules-scm-rust/rust-specific.md)
2. ✅ Read all files in `.roomodes/` directory (scm-android.json, scm-debug-mesh.json, scm-docs.json, scm-ios.json, scm-protocol.json, scm-release.json, scm-repo-agent.json, scm-rust.json)
3. ✅ Read root-level legacy markdowns (AGENTS.md, CLAUDE.md, BOOTSTRAP.md, SCMessengerSKILL.md, scmessenger-agent.md, leanstral-integration.md, leanstral-execution-workflow.md)
4. ✅ Read GitHub Copilot configs (.github/copilot-instructions.md, .github/COPILOT_AGENT_INSTRUCTIONS.md)
5. ✅ Read skills/ directory (mesh-diagnostics/SKILL.md, philosophy-enforcer/SKILL.md, platform-parity-check/SKILL.md, release-gate-validator/SKILL.md, scm-repo-agent/SKILL.md)
6. ✅ Read reference/PHILOSOPHY_CANON.md for enforceable rules
7. ✅ Scan actual codebase structure (core/src/, iOS/, android/app/, wasm/src/)
8. ✅ Read REMAINING_WORK_TRACKING.md for known gaps and current status
9. ✅ Create `.clinerules` synthesizing all extracted knowledge into unified agentic swarm protocol
10. ✅ Create `PRODUCTION_ROADMAP.md` with reality check and 5-phase roadmap
11. ✅ Provide purge command for user to execute manually