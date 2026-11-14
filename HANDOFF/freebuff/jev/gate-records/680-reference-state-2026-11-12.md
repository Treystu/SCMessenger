<!-- HANDOFF-SCOPE-BEGIN -->
scope: SCMessenger
owner: Sovereign-Communication/SCMessenger
purpose: SCMessenger-only findings and remediation handoff
foreign_material: NONE
boundary: No foreign-repository findings, evidence, status, or remediation are included.
<!-- HANDOFF-SCOPE-END -->
Repository: scmessenger

# Gate record -- #680 reference-state edit (Lucas Gate v2)

Date: 2026-11-12. Item: #680. Decision: edit the checked-in reference state so the
canonical completion gate reproduces a pass on it.

## Classification through the Gate (before filing)

- **Choice under test.** Three options were put on #680 by the issue itself:
  (1) recalibrate/reference-state fix, (2) re-admit a different harness tag,
  (3) scoped clause-10 waiver.
- **Category.** Tooling/evidence-integrity, not architecture, not a security or
  privacy trade-off, not an API contract. It is gate-tooling calibration.
- **Destructive score.** Low. The change adds citations (`evidence_map`) and moves
  `canon_rows` inside `evidence` so the citation index can see it. No question,
  criterion, threshold or scoring rule is altered; no history is rewritten; no
  prior verdict is deleted. Nothing is removed from the repository.
- **Lead.** The evidence leads to exactly one option, so `lead >= 0.20`. The root
  cause was read from the gate's own code: `_citation_problem()` returns
  `uncited: no evidence_map entry for this question` when a state lacks the key,
  and a missing citation downgrades an affirmative answer to score 0.00. That is a
  property of the state file, not of the model or of the work. Options 2 and 3 do
  not touch the defect: a different harness tag still meets the same missing key,
  and a waiver accepts a broken instrument instead of repairing it.
- **Concrete actual risk.** The residual risk is real and recorded rather than
  waved: the measured margin is thin (0.72 and 0.76 against the 0.70 `noul` floor),
  so a future reading below the floor fails this state again. That is a property of
  a sampled primitive, disclosed on #680, and it does not make the *state* wrong --
  it makes the instrument's sensitivity visible.
- **Verdict.** Gate-settled for option 1. The choice was already decided by
  oversight on #680 and by PROMPT.md's queue ("the residual choice is DECIDED by
  the gates: edit the reference state, the lane-endorsed option"), so the lane
  executes it and does not re-present it as a question.

## Jev surface on this host

Named blocker, recorded once on #602 (c.6808294084): no model key is present on
this host (`OPENROUTER_API_KEY`, `JEV_API_KEY`, `TYPESAFE_API_KEY` all absent), so
this gate note is a classification from the recorded evidence and the gate's own
code, NOT a model-scored Lucas Gate run. It is filed as UNVERIFIED telemetry and
must be re-run on a surface holding a key before it is treated as canon. The two
keyed runs quoted on #680 (0.72 / 0.76, `is_fallback=False`, `jev-1.14.0`) were
made in an earlier session on the surface that held the key; this session did not
reproduce them and does not claim to.

## What this record does not claim

- It does not claim any parked PR now passes clause 10. Each still needs its own
  two non-author passes on its own head.
- It does not claim the gate is calibrated in general; it claims the reference
  state now carries the citations the gate requires.
