# BATCH: Rust Group A  resume_prefetch.rs wiring (6 tasks)
# AGENT: implementer
# MODEL: qwen3-coder-next:cloud
# FALLBACK: glm-5.1:cloud
# TARGET FILES: core/src/routing/resume_prefetch.rs, core/src/drift/frame.rs

1. **is_prefetch_complete**  Wire into OptimizedRoutingMaintenance tick cycle and routing status reporting.
2. **is_prefetch_in_progress**  Wire into OptimizedRoutingEngine prefetch state queries and transport manager.
3. **mark_refresh_failed**  Wire into relay delegate prewarm failure handling and retry scheduling.
4. **next_refresh_hint**  Wire into OptimizedRoutingEngine refresh scheduling and TTL budget display.
5. **start_refresh**  Wire into OptimizedRoutingMaintenance periodic refresh trigger.
6. **read_with_timeout**  Wire into drift::frame::FrameReader timeout handling and relay custody reads.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.