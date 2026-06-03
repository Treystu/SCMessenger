# SCMessenger Local-First Orchestrator Configuration
# Version: 1.0 — 2026-06-02
#
# Philosophy: Maximize local LLM usage for code quality. Use cloud only when
# local models lack capacity (large context, complex reasoning, quota allows).
#
# Two-tier model routing:
#   TIER 1 — LOCAL (preferred): scm-coder:7b, scm-thinker:14b, qwen2.5-coder:7b,
#           deepseek-r1-14b, qwen2.5-coder:1.5b, llama3.2:3b
#   TIER 2 — CLOUD (fallback): qwen3-coder:480b, glm-5.1, deepseek-v4-pro, etc.
#
# Quota-aware: checks quota_state.json before dispatching cloud tasks.
# If quota > 90%, force-local mode (local models only, smaller scope).

# ──────────────────────────────────────────────────────────────────────────────
# LOCAL MODEL ROSTER — models installed on this WSL host
# ──────────────────────────────────────────────────────────────────────────────

## Tier 1A — Primary Local Coders (best quality for implementation)
LOCAL_CODER_PRIMARY="scm-coder:7b"         # 4.4 GB — best local code quality
LOCAL_CODER_FALLBACK="qwen2.5-coder:7b"    # 4.4 GB — backup coder

## Tier 1B — Local Reasoners (architecture, planning, validation)
LOCAL_REASONER_PRIMARY="scm-thinker:14b"   # 4.4 GB — best local reasoning
LOCAL_REASONER_FALLBACK="deepseek-r1-distill-14b-iq2xs:latest"  # 4.4 GB

## Tier 1C — Local Quick/Triage (fast, low-latency)
LOCAL_QUICK="qwen2.5-coder:1.5b"          # 0.9 GB — fast, small tasks
LOCAL_SIMPLE="llama3.2:3b"                 # 1.9 GB — simplest tasks

# ──────────────────────────────────────────────────────────────────────────────
# CLOUD MODEL ROSTER — available via Ollama Cloud proxy
# ──────────────────────────────────────────────────────────────────────────────

## Tier 2A — Cloud Flagship (complex architecture, multi-file reasoning)
CLOUD_ARCHITECTURE="qwen3-coder:480b:cloud"       # 475 GB — complex planning
CLOUD_REASONING="qwen3.5:397b:cloud"               # 370 GB — deep reasoning

## Tier 2B — Cloud Implementation (large features, refactoring)
CLOUD_CODER="qwen3-coder-next:cloud"               # 76 GB — next-gen coding
CLOUD_RUST="glm-5.1:cloud"                         # 1404 GB — Rust specialist

## Tier 2C — Cloud Validation (security, crypto, protocol audit)
CLOUD_SECURITY="deepseek-v3.2:cloud"               # 641 GB — crypto/math
CLOUD_VALIDATION="deepseek-v4-pro:cloud"           # 1490 GB — deep analysis

## Tier 2D — Cloud Review (pre-merge, gatekeeping)
CLOUD_REVIEW="kimi-k2-thinking:cloud"              # 1042 GB — extended thinking
CLOUD_GATE="kimi-k2.6:cloud"                      # 554 GB — fast review

## Tier 2E — Cloud Quick (triage, lint, docs)
CLOUD_QUICK="gemini-3-flash-preview:cloud"         # fast triage
CLOUD_DOCS="gemma4:31b:cloud"                      # 58 GB — docs/tests

# ──────────────────────────────────────────────────────────────────────────────
# TASK-TO-MODEL ROUTING TABLE
# Format: TASK_TYPE = "LOCAL_MODEL | CLOUD_MODEL"
# The orchestrator tries LOCAL first. Falls back to CLOUD if:
#   (a) local model is OOM / unavailable, OR
#   (b) task context > local model's context window, OR
#   (c) quota < 90% AND task complexity > local model capability
# ──────────────────────────────────────────────────────────────────────────────

# ── ORCHESTRATION (always local — orchestrator IS the local model) ──
ROUTE_OVERSEER="scm-thinker:14b|qwen3-coder:480b:cloud"
ROUTE_DELEGATOR="scm-thinker:14b|minimax-m3:cloud"
ROUTE_COORDINATION="scm-thinker:14b|mistral-large-3:675b:cloud"

# ── ARCHITECTURE & PLANNING ──
# Small/medium scope → local thinker. Large scope → cloud flagship.
ROUTE_ARCHITECTURE="scm-thinker:14b|qwen3-coder:480b:cloud"
ROUTE_PLANNING="scm-thinker:14b|qwen3-coder:480b:cloud"
ROUTE_DESIGN="scm-thinker:14b|qwen3-coder-next:cloud"
ROUTE_MULTI_FILE_REASONING="deepseek-r1-distill-14b-iq2xs:latest|qwen3-coder:480b:cloud"

# ── IMPLEMENTATION & CODING ──
# Default to local coder. Cloud for large features or Rust core.
ROUTE_IMPLEMENTATION="scm-coder:7b|qwen3-coder-next:cloud"
ROUTE_CODING="scm-coder:7b|qwen3-coder-next:cloud"
ROUTE_FEATURES="scm-coder:7b|minimax-m2.7:cloud"
ROUTE_BUG_FIX="scm-coder:7b|qwen3-coder-next:cloud"
ROUTE_REFACTORING="scm-coder:7b|minimax-m2.7:cloud"
ROUTE_RUST_CORE="scm-coder:7b|glm-5.1:cloud"
ROUTE_PROTOCOLS="scm-coder:7b|glm-5.1:cloud"
ROUTE_WASM="scm-coder:7b|glm-5.1:cloud"
ROUTE_MOBILE_BRIDGE="scm-coder:7b|glm-5.1:cloud"

# ── SECURITY & VALIDATION ──
# Local reasoner for small audits. Cloud for deep crypto/protocol work.
ROUTE_SECURITY="deepseek-r1-distill-14b-iq2xs:latest|deepseek-v3.2:cloud"
ROUTE_CRYPTO="deepseek-r1-distill-14b-iq2xs:latest|deepseek-v3.2:cloud"
ROUTE_VALIDATION="deepseek-r1-distill-14b-iq2xs:latest|deepseek-v3.2:cloud"
ROUTE_PROTOCOL_VALIDATION="deepseek-r1-distill-14b-iq2xs:latest|deepseek-v3.2:cloud"
ROUTE_SECURITY_AUDIT="deepseek-r1-distill-14b-iq2xs:latest|deepseek-v4-pro:cloud"

# ── TESTS & DOCS ──
# Always local — these are small, well-scoped tasks.
ROUTE_TESTS="qwen2.5-coder:7b|gemma4:31b:cloud"
ROUTE_DOCS="qwen2.5-coder:7b|gemma4:31b:cloud"
ROUTE_BINDINGS="qwen2.5-coder:7b|gemma4:31b:cloud"
ROUTE_PLATFORM="qwen2.5-coder:7b|gemma4:31b:cloud"
ROUTE_GRADLE="qwen2.5-coder:7b|gemma4:31b:cloud"

# ── TRIAGE & QUICK FIXES ──
# Always local — fast, small, well-scoped.
ROUTE_TRIAGE="qwen2.5-coder:1.5b|gemini-3-flash-preview:cloud"
ROUTE_LINT="qwen2.5-coder:1.5b|gemini-3-flash-preview:cloud"
ROUTE_QUICK_FIX="qwen2.5-coder:1.5b|gemini-3-flash-preview:cloud"
ROUTE_MICRO="llama3.2:3b|ministral-3:8b:cloud"
ROUTE_FMT="llama3.2:3b|ministral-3:3b:cloud"

# ── REVIEW & GATEKEEPING ──
# Local for small PRs. Cloud for complex reviews.
ROUTE_REVIEW="scm-thinker:14b|kimi-k2-thinking:cloud"
ROUTE_GATEKEEPING="scm-thinker:14b|kimi-k2.6:cloud"
ROUTE_QUALITY="scm-thinker:14b|kimi-k2-thinking:cloud"
ROUTE_MERGE="scm-thinker:14b|kimi-k2-thinking:cloud"
ROUTE_FINAL_REVIEW="scm-thinker:14b|kimi-k2-thinking:cloud"

# ── SWARM & PIPELINE ──
ROUTE_ORCHESTRATION="scm-thinker:14b|mistral-large-3:675b:cloud"
ROUTE_SWARM="scm-thinker:14b|mistral-large-3:675b:cloud"
ROUTE_PIPELINE="scm-thinker:14b|mistral-large-3:675b:cloud"

# ── VISION & MULTIMODAL ──
ROUTE_VISION="qwen3-vl:235b:cloud"  # No local vision model available
ROUTE_SCREENSHOT="qwen3-vl:235b:cloud"
ROUTE_UI_REVIEW="qwen3-vl:235b:cloud"
ROUTE_DIAGRAM="qwen3-vl:235b:cloud"

# ── GENERAL / FALLBACK ──
ROUTE_GENERAL="scm-coder:7b|qwen3-coder-next:cloud"
ROUTE_SCAFFOLDING="qwen2.5-coder:7b|devstral-2:123b:cloud"
ROUTE_BOILERPLATE="qwen2.5-coder:1.5b|devstral-2:123b:cloud"
ROUTE_RAPID_CODING="scm-coder:7b|qwen3-coder-next:cloud"

# ──────────────────────────────────────────────────────────────────────────────
# COMPLEXITY THRESHOLDS — when to escalate from local to cloud
# ──────────────────────────────────────────────────────────────────────────────

# Task size (estimated lines of code or files touched):
LOCAL_MAX_FILES=5        # ≤5 files → local model
LOCAL_MAX_LOC=300        # ≤300 LOC → local model
LOCAL_MAX_CONTEXT=8192   # ≤8K tokens context → local model

# Above these thresholds, escalate to cloud:
CLOUD_MIN_FILES=6        # ≥6 files → consider cloud
CLOUD_MIN_LOC=301        # ≥301 LOC → consider cloud
CLOUD_MIN_CONTEXT=8193   # ≥8K tokens → consider cloud

# ──────────────────────────────────────────────────────────────────────────────
# QUOTA-AWARE DISPATCH RULES
# ──────────────────────────────────────────────────────────────────────────────

# Quota state file (updated by OllamaQuotaScraper.ps1)
QUOTA_STATE_FILE=".claude/quota_state.json"

# Quota thresholds (percentage of 5-hour window):
QUOTA_HEAVY_LIFT=25     # ≤25%: any model, any task
QUOTA_EXECUTE=50        # ≤50%: local preferred, cloud for complex
QUOTA_MIXED=75          # ≤75%: local only for small/medium, cloud for critical
QUOTA_LIGHT=90          # ≤90%: local only, small tasks only
QUOTA_MICRO=99.5        # ≤99.5%: local only, single-line/P0 only
QUOTA_HARDLOCK=100      # >99.5%: NO cloud dispatch

# ──────────────────────────────────────────────────────────────────────────────
# AGENT POOL — local-first configuration
# ──────────────────────────────────────────────────────────────────────────────
# Each agent has a local primary model and cloud fallback.
# The orchestrator dispatches to local first, escalates to cloud on:
#   - OOM / model unavailable
#   - Context window exceeded
#   - Quota available AND complexity exceeds local capability
# ──────────────────────────────────────────────────────────────────────────────
