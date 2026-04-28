# Transition Plan: Programmatic AI Optimization with DSPy

This document outlines the roadmap for transitioning the SCMessenger Swarm from raw LLM prompt orchestration to a **deterministic programmatic framework** using DSPy.

## The Problem: Probabilistic Hallucination
Current agentic workflows (like CrewAI or raw loops) rely on "prompt engineering" to control behavior. This leads to:
- **Flakiness**: Minor prompt changes cause systemic logic shifts.
- **Unpredictability**: No mathematical guarantees on tool-use consistency.
- **Optimization Wall**: Prohibitive cost/complexity to manually "tune" prompts for 20+ models.

## The Solution: DSPy Integration
DSPy (Declarative Self-optimizing Language Programs) allows us to treat LLM calls as **Optimizer Modules** within a regular Python program.

### Key Components for SCMessenger

1.  **Signatures (The "Types")**:
    Define the input/output schema for every swarm role (e.g., `RustFeatureRequest -> Code + TestSuite`).
    
2.  **Modules (The "Pipeline")**:
    Instead of agents, we build **Chain-of-Thought (CoT)** or **Multi-Hop Recall** modules that handle logic flow programmatically.

3.  **Teleprompters (The "Optimizers")**:
    DSPy will automatically "compile" the best prompts for specific models (Qwen, DeepSeek, Google) by running against a small dataset of "Golden SCM Examples".

## Implementation Roadmap

### Phase 1: Scaffolding (In Progress)
- [ ] Define the `SCM_Protocol_Signature` in Python.
- [ ] Identify the "Golden Examples" (existing verified Rust/Crypto code).

### Phase 2: Hybrid Orchestration
- [ ] Integrate DSPy modules into `swarm.py` for high-precision tasks (Security Audit, Crypto Validation).
- [ ] Keep CrewAI for general, non-critical task routing.

### Phase 3: Total Programmatic Control
- [ ] Replace raw prompting with compiled DSPy programs.
- [ ] Implement a **Self-Correction Loop**: Failed `cargo test` runs are fed back into the DSPy optimizer to refine the "Coder" signature dynamically.

---

*Goal: Transitioning from "AI that tries to code" to "A program that uses AI to solve specialized system constraints with mathematical rigor."*
