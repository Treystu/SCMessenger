# Lucas Gate — SCMessenger annex

Repo-local annex to the Lucas Gate v2 baseline. It codifies the lane checks this
repo's driving work already practices, plus the two the 2026-11-02 Jev
inclusivity audit found missing. It governs driving artifacts in this repo:
comments, reviews, approvals, dispositions, driving suggestions, lane prompts.
Gate mechanics (thresholds, escalation, receipts) are in
`docs/rules/JEV_GATE_POLICY_V1.md`; this annex carries only the
SCMessenger-specific checks.

## Lane checks

1. PROTECTED PATHS FIRST. Any disposition touching
   `core/src/{crypto,transport,routing,privacy}/` or `vendor/` is checked
   against AGENTS.md rule 8 before anything else: no landing without the
   adversarial security review on file, from a reviewer who did not author
   the change.
2. MESH / PROTOCOL CORRECTNESS PRIMACY. For transport, crypto, routing, and
   protocol-semantics dispositions, correctness of the protocol claim is the
   weighted check: a disposition that is stylistically clean but wrong about
   message flow, key handling, or delivery semantics fails here regardless of
   its other scores. Claims about protocol behavior cite the code or the spec
   section that carries them.
3. NODE PARITY LANGUAGE. This system has nodes, not relays — all nodes in
   parity (see `docs/rules/NODE_MODEL.md`). Driving text that frames a node
   as a relay, or proposes relay-tier behavior, is a defect in the artifact,
   not a wording preference.
4. NO EMOJI in repo content (see `docs/rules/NO_EMOJIS.md`).
5. CLOSE-KEYWORD DISCIPLINE. Per LANE PROTOCOL clause 9: no closing keyword
   for an issue the artifact does not intend to close on merge; the tracker
   (#602) and burndown tracker (#595) close only at their plans' closing
   steps.
6. TRUE PASS PROCEDURE (code batches). Per LANE PROTOCOL clause 10: two
   independent non-author Jev PASSes recorded on the PR, resolved explicitly
   in the landing comment (which PASSes, from whom, on which head SHA). This
   repo's Rule-8 practice records evaluations as PR comments; the two PASSes
   must both be present as comments naming the head they evaluated.
7. SMALL VERIFIED SLICES. Driving suggestions prefer the smallest slice that
   lands verified over a larger unverified one; a suggestion whose
   verification step is "check later" is incomplete on its face.

## Never rules for this lane

- Never present a gate-resolvable driving decision as an owner decision.
- Never classify a next action owner-side without gating the classification
  itself, and owner-side is the reserved list below only -- not a default when
  a row feels uncertain. Reserved owner-side: (a) reserved owner decisions (the
  protection switch, spending/payments, submissions/claims, scope-of-ship);
  (b) destructive actions; (c) canon or gate-text changes; (d) access changes
  (naming or granting a reviewer); (e) actual risk surfaced by investigation.
  Everything else that passes the gate, the lane drives. A genuinely blocked
  row parks only that row; work continues on the next unblocked item. Before
  any "every remaining row is owner-side" claim is made, re-test it against the
  register's own owner cells -- this lane falsified that exact claim once, when
  A23 was found lane-actionable after being reported blocked. Owner direction:
  PR #653 comment 2026-11-08T10:07:35Z (oversight, owner-directed), items 2 and
  4.
- Never land on a base that fails the GREEN BASE definition (LANE PROTOCOL
  clause 8) without recording that state as the blocker.
- Never let a schedule-only job failure (Full Test Suite, List stale
  branches) ride unrecorded past the next landing that could touch it.
