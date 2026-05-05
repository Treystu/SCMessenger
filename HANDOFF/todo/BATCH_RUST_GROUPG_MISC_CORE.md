# BATCH: Rust Group G — miscellaneous core wiring (2 tasks)
# AGENT: rust-coder
# MODEL: glm-5.1:cloud
# FALLBACK: qwen3-coder-next:cloud
# TARGET FILES: core/src/abuse/reputation.rs, core/src/dspy/signatures.rs

1. **overall_score** — Wire into abuse reputation scoring aggregation and routing negative cache decisions.
2. **get_signature** — Wire into crypto signature verification and identity authentication flows (already in Group B but duplicated here for separate dspy/signatures.rs targeting).

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.