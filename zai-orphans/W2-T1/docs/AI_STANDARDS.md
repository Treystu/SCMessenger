# SCMessenger AI Standards and Governance

Status: Active
Last updated: 2026-08-10

This document records the active governance boundary. Canonical orchestration
authority is `docs/ORCHESTRATION.md` and `orchestration/manifest.yaml` version
2.0.0. Agent-specific instructions must defer to those sources.

The controller is a coordinator, not a final approver or source author. It must
dispatch all substantive implementation, test implementation, compile repairs,
and investigation to a fresh scoped worker. It may collect evidence, run
deterministic gates, route reviews, maintain durable state, and integrate a
verified worker-produced patch. It may not choose a substantive architecture
winner, perform a small direct fix, or make a release/product decision.

Independent validation is mandatory for consequential work. Protected
crypto/transport/routing/privacy paths require a CRITICAL_VALIDATOR; delivery
changes require the protocol's independent delivery review. A gatekeeper is an
independent evidence role, never the persistent controller.

Provider/model assignments are capability mappings, not authority mappings.
Model unavailability selects a compatible provider or escalates; it does not
turn a controller into a planner, validator, or implementer. The operator owns
unresolved architecture, security/privacy trade-offs, API breaks, technology
changes, release/version policy, and material scope changes.
