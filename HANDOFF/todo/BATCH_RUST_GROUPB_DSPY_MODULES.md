# BATCH: Rust Group B — dspy/modules.rs wiring (6 tasks)
# AGENT: implementer
# MODEL: qwen3-coder-next:cloud
# FALLBACK: glm-5.1:cloud
# TARGET FILES: core/src/dspy/modules.rs, core/src/dspy/signatures.rs

1. **build_security_audit_pipeline** — Wire into IronCore initialization and relay custody verification.
2. **create_multihop** — Wire into relay multi-hop routing and transport manager path selection.
3. **run_optimization** — Wire into OptimizedRoutingEngine optimization cycle and routing tick.
4. **create_optimizer** — Wire into OptimizedRoutingEngine initialization and adaptive TTL setup.
5. **add_step** — Wire into multi-hop chain-of-thought pipeline and relay routing assembly.
6. **get_signature** — Wire into crypto signature verification and identity authentication flows.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.