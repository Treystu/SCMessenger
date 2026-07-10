# BATCH: Rust Group C  notification.rs + relay wiring (4 tasks)
# AGENT: implementer
# MODEL: qwen3-coder-next:cloud
# FALLBACK: glm-5.1:cloud
# TARGET FILES: core/src/notification.rs, core/src/relay/delegate_prewarm.rs, core/src/relay/invite.rs

1. **unregister_endpoint**  Wire into notification cleanup on peer disconnect and transport shutdown.
2. **touch_endpoint**  Wire into relay endpoint health tracking and delegate prewarm keep-alive.
3. **update_keepalive**  Wire into delegate prewarm heartbeat and relay endpoint liveness checks.
4. **refresh_delegate_routes**  Wire into relay bootstrap route refresh and transport manager periodic update.
5. **get_signable_data**  Wire into identity signing flows and invite protocol verification.
6. **federated_nickname**  Wire into contact resolution and identity display across mesh nodes.

CRITICAL: You are forbidden from considering a task 'complete' until you execute the `mv` or `Rename-Item` command to move the task markdown file from `todo/` (or `IN_PROGRESS/`) to `done/`. If you do not move the file, the Orchestrator assumes you failed.