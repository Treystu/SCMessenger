# Optimized `orchestrate` Command Definition — v4.0

## Overview
This document defines the optimized orchestration logic for the SCMessenger hybrid swarm.
It replaces the v3.3 `orchestrator_manager.sh` pool-launch model with a unified orchestrate
command that handles task decomposition, hybrid routing, context compression, parallel execution,
and artifact merging.

---

## Command: `orchestrate`

```bash
orchestrate <manifest_file> [--mode cloud|local|hybrid] [--parallel N] [--budget tokens]
```

### Input: Manifest File (YAML)

```yaml
project: SCMessenger
phase: "Phase 1 — KMP Scaffolding & Rust Integration"
default_route: hybrid    # cloud | local | hybrid
max_parallel: 3
token_budget: 500000     # total swarm budget for this run

agents:
  - id: rust-uniffi-linux
    role: Rust & UniFFI Linux Specialist
    route: cloud
    model_primary: minimax-m3:cloud
    model_fallback: glm-5.1:cloud
    task_file: TASK_KMP_RUST_UNIFFI_LINUX.md
    depends_on: []
    output_schema: rust_crate

  - id: compose-architect
    role: Compose Multiplatform Architect
    route: cloud
    model_primary: minimax-m3:cloud
    model_fallback: qwen3-coder:480b:cloud
    task_file: TASK_KMP_COMPOSE_ARCHITECT.md
    depends_on: []
    output_schema: kotlin_module

  - id: devops-packaging
    role: DevOps & Packaging Engineer
    route: local
    model_primary: scm-coder:7b
    model_fallback: qwen2.5-coder:7b
    task_file: TASK_KMP_DEVOPS_PACKAGING.md
    depends_on: ["rust-uniffi-linux"]   # needs cargo.toml structure
    output_schema: ci_yaml

  - id: qa-interop
    role: QA & Interoperability Tester
    route: local
    model_primary: scm-thinker:14b
    model_fallback: scm-coder:7b
    task_file: TASK_KMP_QA_INTEROP.md
    depends_on: ["compose-architect"]   # needs shared module structure
    output_schema: test_plan
```

---

## Core Logic (Pseudocode)

```
FUNCTION orchestrate(manifest, mode, parallel, budget):
    
    # ═══════════════════════════════════════════════════════════════
    # PHASE 1: LOAD & VALIDATE MANIFEST
    # ═══════════════════════════════════════════════════════════════
    agents = LOAD_MANIFEST(manifest)
    VALIDATE_AGENT_CONFIGS(agents)
    BUILD_DEPENDENCY_GRAPH(agents)        # DAG for parallel scheduling
    
    # ═══════════════════════════════════════════════════════════════
    # PHASE 2: QUOTA-AWARE HYBRID ROUTING
    # ═══════════════════════════════════════════════════════════════
    quota = CHECK_QUOTA_STATE()           # Read .claude/quota_state.json
    tier = COMPUTE_TIER(quota)            # Tier 1-6 from quota percentages
    
    FOR EACH agent IN agents:
        IF agent.route == "cloud":
            # Architectural/high-context → Cloud API (minimax-m3:cloud)
            agent.model = SELECT_CLOUD_MODEL(agent, tier)
            agent.context_limit = 128000    # Full cloud context
        ELSE IF agent.route == "local":
            # Iterative/high-volume → Local LLM (scm-coder:7b, scm-thinker:14b)
            agent.model = SELECT_LOCAL_MODEL(agent)
            agent.context_limit = 8000      # nb: scm-coder:7b effective context
            
        # Apply context compression for local agents
        IF agent.route == "local":
            COMPRESS_TASK_CONTEXT(agent, 
                strategy="architectural_summary",
                token_budget=agent.context_limit * 0.7)  # Leave 30% headroom
    
    # ═══════════════════════════════════════════════════════════════
    # PHASE 3: PARALLEL EXECUTION ENGINE
    # ═══════════════════════════════════════════════════════════════
    READY_QUEUE = TOPOLOGICAL_SORT(agents)    # DAG → execution order
    ACTIVE_SET = {}
    COMPLETED_SET = {}
    ARTIFACT_REGISTRY = {}
    
    WHILE READY_QUEUE NOT EMPTY OR ACTIVE_SET NOT EMPTY:
        
        # Launch ready agents up to parallelism limit
        WHILE LEN(ACTIVE_SET) < parallel AND READY_QUEUE NOT EMPTY:
            agent = READY_QUEUE.pop()
            IF ALL_DEPENDENCIES_MET(agent, COMPLETED_SET):
                child_pid = SPAWN_AGENT(
                    model = agent.model,
                    task = agent.task_file,
                    context = agent.compressed_context,
                    agent_id = agent.id
                )
                ACTIVE_SET[agent.id] = {
                    pid: child_pid,
                    model: agent.model,
                    route: agent.route,
                    start_time: NOW(),
                    output_schema: agent.output_schema
                }
            ELSE:
                REQUEUE(agent)   # Dependencies not met, retry next round
        
        # Poll active agents for completion
        FOR EACH (id, info) IN ACTIVE_SET:
            IF PROCESS_COMPLETED(info.pid):
                output = COLLECT_OUTPUT(id)
                ARTIFACT_REGISTRY[id] = VALIDATE_AND_PARSE(output, info.output_schema)
                COMPLETED_SET[id] = {
                    output: ARTIFACT_REGISTRY[id],
                    route: info.route,
                    model: info.model,
                    duration: NOW() - info.start_time
                }
                DELETE(ACTIVE_SET, id)
                
                # Check if any waiting agents are now unblocked
                FOR EACH agent IN READY_QUEUE:
                    IF ALL_DEPENDENCIES_MET(agent, COMPLETED_SET):
                        PROMOTE_TO_READY(agent)
        
        SLEEP(10)    # Poll interval
    
    # ═══════════════════════════════════════════════════════════════
    # PHASE 4: ARTIFACT MERGING & STATE SYNCHRONIZATION
    # ═══════════════════════════════════════════════════════════════
    merged_state = {}
    
    FOR EACH agent_id IN TOPOLOGICAL_ORDER(agents):
        artifact = ARTIFACT_REGISTRY[agent_id]
        
        # Merge strategy depends on artifact type
        SWITCH artifact.schema:
            case "rust_crate":
                MERGE_SOURCE_TREE(merged_state, artifact.files)
                MERGE_CARGO_TOML(merged_state, artifact.cargo_changes)
            case "kotlin_module":
                MERGE_SHARED_MODULE(merged_state, artifact.shared_files)
                MERGE_GRADLE_FILES(merged_state, artifact.gradle_changes)
            case "ci_yaml":
                MERGE_CI_WORKFLOW(merged_state, artifact.workflow_files)
            case "test_plan":
                MERGE_TEST_FILES(merged_state, artifact.test_files)
        
        # Cross-artifact validation
        VALIDATE_INTER_ARTIFACT_CONSISTENCY(merged_state)
    
    # ═══════════════════════════════════════════════════════════════
    # PHASE 5: CHECKPOINT & HANDOFF
    # ═══════════════════════════════════════════════════════════════
    WRITE_MANIFEST_STATE(merged_state, "HANDOFF/STATE/current.json")
    UPDATE_DONE_TRACKING(COMPLETED_SET)
    
    # Git checkpoint
    GIT_ADD_ALL()
    GIT_COMMIT(f"swarm: {manifest.phase} — {LEN(COMPLETED_SET)}/{LEN(agents)} agents complete")
    
    RETURN {
        status: "complete",
        agents_completed: LEN(COMPLETED_SET),
        agents_total: LEN(agents),
        artifacts: ARTIFACT_REGISTRY,
        duration: TOTAL_ELAPSED
    }
```

---

## Key Optimizations Over v3.3

### 1. Hybrid Routing Logic
```
┌─────────────────────────────────────────────────────────────────┐
│                    HYBRID ROUTING TABLE                         │
├─────────────────────────┬──────────────┬────────────────────────┤
│ Task Characteristic     │ Route        │ Model                  │
├─────────────────────────┼──────────────┼────────────────────────┤
│ Architecture/design     │ Cloud        │ minimax-m3:cloud       │
│ Large codebase (>50K    │ Cloud        │ minimax-m3:cloud       │
│   tokens context)       │              │ (128K context)         │
│ Protocol/crypto logic   │ Cloud        │ deepseek-v3.2:cloud    │
│ Gradle/CI config        │ Local        │ scm-coder:7b           │
│ Test file generation    │ Local        │ scm-thinker:14b        │
│ Lint/fix/format         │ Local        │ qwen2.5-coder:1.5b     │
│ Docs/CHANGELOG          │ Local        │ scm-coder:7b           │
└─────────────────────────┴──────────────┴────────────────────────┘

Route selection in manifest:  route: "cloud" | "local" | "hybrid"
  — "cloud": always use cloud API (highest quality, higher cost)
  — "local": always use local Ollama (fastest, no API cost)
  — "hybrid": auto-select based on task complexity + quota tier
```

### 2. Context Summarization Protocol
```
FUNCTION COMPRESS_TASK_CONTEXT(agent, strategy, budget):
    
    raw_context = READ(agent.task_file)
    repo_map = LOAD_REPO_MAP()
    
    IF strategy == "architectural_summary":
        # Instead of dumping full file contents, produce:
        # 1. Module dependency graph (text)
        # 2. Key type/function signatures (not bodies)
        # 3. Relevant Cargo.toml / build.gradle sections only
        # 4. Interface contracts (UDL, protobuf, API schemas)
        
        compressed = {
            "module_map": EXTRACT_MODULE_GRAPH(repo_map, agent.relevant_files),
            "signatures": EXTRACT_SIGNATURES(agent.relevant_files),
            "config_snippets": EXTRACT_CONFIG_SECTIONS(agent.relevant_files),
            "contracts": EXTRACT_INTERFACE_CONTRACTS(agent.relevant_files),
            "task_instructions": raw_context.task_section
        }
        
        token_count = ESTIMATE_TOKENS(compressed)
        IF token_count > budget:
            # Further compression: remove signatures, keep only interfaces
            compressed = REMOVE_DETAIL_LEVEL(compressed)
        
        agent.compressed_context = FORMAT_AS_MARKDOWN(compressed)
    
    # Cross-agent context passing:
    # When Agent A completes, its output becomes input context for dependent Agent B.
    # Instead of passing full source code, pass:
    #   — File paths and line ranges of changes
    #   — New/modified function signatures
    #   — Updated dependency declarations
    #   — Schema/interface changes
    
    IF agent.has_dependencies:
        FOR dep_id IN agent.depends_on:
            dep_output = ARTIFACT_REGISTRY[dep_id]
            agent.compressed_context += FORMAT_DEPENDENCY_SUMMARY(dep_output)

TOKEN SAVINGS: 3-5x reduction vs. full source dump for local agents.
Context budget usage: ~60-70% for instructions, ~30-40% for agent output.
```

### 3. Asynchronous Execution Engine
```
┌─────────────────────────────────────────────────────────────────┐
│                 DEPENDENCY GRAPH (DAG)                          │
│                                                                 │
│  ┌──────────────────┐     ┌────────────────────┐               │
│  │ rust-uniffi-linux│     │ compose-architect  │  PARALLEL   │
│  │ (Cloud)          │     │ (Cloud)            │  BATCH 1    │
│  └────────┬─────────┘     └─────────┬──────────┘               │
│           │                          │                          │
│           ▼                          ▼                          │
│  ┌──────────────────┐     ┌────────────────────┐               │
│  │ devops-packaging │     │ qa-interop         │  PARALLEL   │
│  │ (Local)          │     │ (Local)            │  BATCH 2    │
│  └──────────────────┘     └────────────────────┘               │
│                                                                 │
│  Parallel batch 1: Agents 1 & 2 run simultaneously             │
│  Parallel batch 2: Agents 3 & 4 start when deps are met        │
│  Max concurrent: 3 (configurable)                               │
└─────────────────────────────────────────────────────────────────┘

Execution modes:
  — Sequential: depend_on chain forces order
  — Parallel: independent agents run concurrently
  — Batched: parallel within dependency level
```

### 4. Artifact Passing Protocol (JSON/Markdown Schema)
```
┌─────────────────────────────────────────────────────────────────┐
│                   ARTIFACT SCHEMA DEFINITIONS                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  "rust_crate": {                                                │
│    "files": [{"path": "...", "content": "...", "op":           │
│               "create|modify|delete"}],                         │
│    "cargo_changes": [{"crate": "...", "version": "..."}],      │
│    "udl_exports": ["..."],                                      │
│    "build_commands": ["cargo check -p ..."],                   │
│    "verification": "cargo test -p ..."                          │
│  }                                                              │
│                                                                 │
│  "kotlin_module": {                                             │
│    "shared_files": [{"path": "...", "content": "..."}],        │
│    "gradle_changes": [{"file": "...", "changes": [...]}],      │
│    "entry_point": "linuxX64Main.kt",                           │
│    "dependencies": ["jvm:...", "native:..."],                  │
│    "verification": "./gradlew :shared:compileKotlinLinuxX64"    │
│  }                                                              │
│                                                                 │
│  "ci_yaml": {                                                   │
│    "workflow_files": [{"path": "...", "content": "..."}],      │
│    "trigger": "push|pull_request",                              │
│    "artifacts": ["debian_pkg", "appimage"],                     │
│    "verification": "act -j build_desktop"                       │
│  }                                                              │
│                                                                 │
│  "test_plan": {                                                 │
│    "test_files": [{"path": "...", "content": "..."}],          │
│    "ui_parity_matrix": "markdown_table",                        │
│    "interop_scenarios": ["desktop↔android", "desktop↔wasm"],   │
│    "verification": "cargo test + gradlew test"                  │
│  }                                                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Quota-Aware Execution

```
FUNCTION SELECT_CLOUD_MODEL(agent, tier):
    SWITCH tier:
        case TIER_1_HEAVY:    # <= 25% 5hr — unlimited budget
            RETURN agent.model_primary    # minimax-m3:cloud
        case TIER_2_EXECUTE:  # <= 50% — standard
            RETURN agent.model_primary
        case TIER_3_MIXED:    # <= 75% — reduce cloud usage
            IF agent.route == "local":
                RETURN agent.model_primary    # keep local
            ELSE IF agent.can_downgrade:
                RETURN agent.model_fallback   # cheaper cloud
            ELSE:
                RETURN agent.model_primary    # keep as-is
        case TIER_4_LIGHT:    # <= 90% — critical only
            IF agent.priority == "critical":
                RETURN agent.model_primary
            ELSE:
                RETURN agent.model_fallback   # downgrade or skip
        case TIER_5_MICRO:    # <= 99.5% — emergency only
            RETURN agent.model_fallback       # smallest model
        case TIER_6_HARDLOCK: # > 99.5% — STOP
            RAISE EmergencyStop("Quota exhausted. Pause all dispatch.")
```

---

## State Synchronization Plan

```
ARTIFACT MERGING STRATEGY:
═════════════════════════

1. TOPOLOGICAL MERGE ORDER:
   rust-uniffi-linux → compose-architect → devops-packaging → qa-interop
   (Respects dependency graph even if agents complete out of order)

2. THREE-WAY MERGE FOR OVERLAPPING FILES:
   If Agent 1 and Agent 2 both modify Cargo.toml:
     BASE = git HEAD version
     AGENT1 = rust bridge adds desktop_bridge crate
     AGENT2 = compose adds shared module reference
     RESULT = BASE + AGENT1.cargo_changes + AGENT2.gradle_changes
     (No conflict — they modify different sections)

3. CONFLICT RESOLUTION:
   — Same file, same section:Flag for manual review in HANDOFF/REVIEW/
   — Same file, different sections: auto-merge with markers
   — Binary/lock files: regenerate (cargo metadata, gradle sync)

4. CONTINUOUS INTEGRATION CHECKPOINT:
   After each agent completes:
     — Run `cargo check --workspace` (if Rust files changed)
     — Run `./gradlew :shared:compileKotlinLinuxX64` (if Gradle changed)
     — Run `bash scripts/docs_sync_check.sh` (if docs changed)
     — Log results in HANDOFF/STATE/{agent_id}/build_status.json

5. FINAL MERGED STATE:
   Written to HANDOFF/STATE/phase1_complete.json containing:
     — All file changes with git diff references
     — Build verification status per agent
     — Remaining TODOs for next phase
     — Known issues / conflicts needing human review
```

---

## Comparison: v3.3 vs v4.0

| Feature | v3.3 | v4.0 |
|---------|------|------|
| Task input | Ad-hoc task files | Structured YAML manifest |
| Model routing | Config-only, no logic | Hybrid routing table + quota-aware |
| Context injection | Full repo scan (slow) | Architectural summary (3-5x faster) |
| Execution | Sequential CLI launches | DAG-based parallel batches |
| Artifact passing | Implicit (file system) | Explicit schema definitions |
| Quota management | Basic 5-tier | 6-tier with auto-downgrade |
| Conflict detection | File domain matching | Three-way merge + auto-resolution |
| Checkpoint | Manual git add/commit | Automatic per-agent + final |
| State sync | HANDOFF file moves | JSON state + build verification |
