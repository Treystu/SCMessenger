# Prompt: Fable 5 Codebase Readiness Assessment

This briefs Fable on where SCMessenger actually stands — with evidence, not
assumptions — and hands it the decision of what to do next. It deliberately
does not prescribe a fixed sequence of steps: Fable is capable enough to
triage a problem of this shape better than a scripted checklist can, and a
rigid sequence written in advance (by someone who hasn't read the actual
failure logs) risks locking it into the wrong order of operations. Trust its
judgment on sequencing and scope. The one fixed requirement is the shape of
the handoff at the end: anything left for further implementation must be
specified precisely enough for a Sonnet or Haiku model to execute
mechanically.

---

## Prompt text

> You're assessing SCMessenger for release readiness. Read the briefing
> below in full before deciding what to do — it separates what's actually
> verified from what's still unknown, because this repo has already shown at
> least one instance of a confident claim being false on the commit where it
> was made. Then use your own judgment about where to start, what to fix
> yourself, and what to hand off.
>
> ## What's verified true right now
>
> - `main` is at `bc9b25e`. The master backlog is `fable5plan.md` (Tracks
>   T1–T5), mirrored per-task in `tasks/T<id>/progress.md` checklists.
>   `CHANGELOG.md` claims `1.0.0-rc2` is a complete release with "all gates
>   green."
> - **`main`'s CI has failed on every run in its visible history.** I pulled
>   the last 15 workflow runs on `main` (`CI`, `Mobile`, `Cross` workflows,
>   covering the 5 most recent commits back to `53370fe` on 2026-06-15):
>   15 out of 15 are `failure`. That includes `0a49d32` — "docs: update
>   CHANGELOG for v1.0.0-rc2" — the exact commit whose own message claims
>   "the Rust gatekeeper suite passes" and "Android/iOS/WASM builds are
>   verified." Both `CI` and `Cross` failed on that same SHA. The claim was
>   false the moment it was written.
> - There's one open PR: **#1**, `claude/v1-0-0-code-gaps-7d849x` → `main`
>   (head `1f52b425`), 22 commits, 68 files changed (+11023/-805). It claims
>   to close T1.4 (WiFi Direct group-owner-intent from battery state, plus a
>   new task `T1.8` it introduces), T4.5 (Argon2id identity backup
>   export/import + audit events), safety-number verification UI, and CLI
>   JSON-RPC message-request handling. It directly edits the progress
>   checklists for `T1.2`, `T1.3`, `T1.4`, `T1.8`, `T2.4`, `T2.5`, `T4.5`,
>   `T5.7`.
> - Of 16 CI check runs on PR #1's head commit: only "cubic · AI code
>   reviewer" passed, two binding-generation jobs were skipped, and the rest
>   — `Test (ubuntu-latest)`, `Lint`, `WASM`, `iOS`, `iOS Build`, all three
>   `Android` ABI builds, `Android Debug APK`, `FFI Surface Contract`, `Docs`
>   — are `failure` or `cancelled`. GitHub reports `mergeable_state:
>   unstable`.
> - On `main` as of this writing, six tasks still have unchecked boxes in
>   their `tasks/T*/progress.md`: `T1.2` (3 unchecked), `T1.3` (2), `T1.4`
>   (4), `T2.4` (4), `T2.5` (3), `T4.5` (4). PR #1 touches several of these
>   exact files — check the PR branch's version, not `main`'s, since it may
>   have already flipped boxes (correctly or not).
> - The commits `5df71b9` ("gemini Updates") and `bc9b25e` ("update for
>   cloud orchestrator") landed after those checklists were last edited on
>   `main`, adding `core/tests/integration_wifi_aware.rs`,
>   `core/tests/integration_retry_lifecycle.rs`, drift-mule test additions,
>   two new CI workflows, and a large new `cloud/` orchestrator subsystem
>   (Python + Terraform + Docker) with no lint/test gate of its own —
>   including committed `cloud/orchestrator/__pycache__/*.pyc` files that
>   shouldn't be tracked.
> - `docs/device-testing.md` exists. Whether it documents runnable
>   procedures or placeholders is unverified.
>
> ## What's not known — get evidence before acting on it, don't assume
>
> - **Why every CI job has failed for two-plus weeks straight is unknown to
>   me.** I tried to pull the job logs and they'd already expired (404). It
>   could be a genuine code/build break, or it could be something no amount
>   of code editing fixes — an expired secret, exhausted Actions minutes,
>   macOS-runner unavailability for the iOS jobs, a permissions problem.
>   Don't assume it's code-fixable until you've actually seen the failure.
> - Whether PR #1's specific failures are regressions the PR introduced, or
>   just `main`'s pre-existing breakage carried forward unchanged, is
>   unknown — the 100% failure rate on `main` makes the latter plausible.
> - Whether the new `cloud/` subsystem has hygiene problems beyond the
>   committed `.pyc` files (e.g. secrets in the Terraform/scripts) is
>   unverified.
> - Whether any task *not* listed above as still-open actually has
>   passing, meaningful tests behind its checked boxes, or was just checked
>   by habit, is unverified.
>
> ## Your call
>
> You have the full picture now. Decide the order of operations, what's
> worth fixing yourself and proving with a real passing run versus what's
> better left as a precisely specified task for another model, and whether
> anything here is genuinely outside code's reach and needs to be flagged to
> a human rather than worked around. Some things worth weighing — not steps
> to follow in order:
>
> - Cheap, fast signals (lint/unit tests on Linux) are worth exhausting
>   before expensive ones (macOS/iOS/Android cross-builds) if you end up
>   iterating blind.
> - Work that doesn't depend on CI being green — e.g. reading PR #1's diff
>   for correctness or security issues in the crypto (Argon2id backup) and
>   identity-verification (safety numbers) code it touches — doesn't need to
>   wait on CI triage finishing first.
> - If something needs org/repo-level access you don't have (secrets,
>   billing, runner quota), say so plainly rather than working around it or
>   declaring success anyway.
> - Don't take any commit message, PR description, checklist checkbox, or
>   CHANGELOG line at face value in this repo — verify it yourself before
>   relying on it or repeating it.
>
> ## What to leave behind when you're done
>
> - An honest written account of what you found and verified, including
>   anything you could not verify and why.
> - Any fix you made yourself, applied and proven with a real passing
>   run — not merely asserted.
> - For everything still open that needs further implementation: an
>   ordered, atomic task list precise enough for a Sonnet or Haiku model to
>   execute each item mechanically — exact files, exact anchors
>   (`path:line`), the exact change (you make any remaining design
>   decision yourself; don't leave options open for the next model to
>   resolve), the exact command to verify it, and an explicit done
>   condition. Reconcile `tasks/T*/progress.md` and `CHANGELOG.md` to
>   verified reality as part of this, not as an afterthought.
> - Anything that isn't a code problem, called out separately and
>   explicitly, so a human deals with it instead of it silently blocking
>   everything downstream.

---

## How to run it

Local Fable session:

```
fable "$(cat fable5_readiness_prompt.md | sed -n '/^## Prompt text/,/^---$/p')"
```

Or via the cloud orchestrator's Telegram bot:

```
/sprint <paste the prompt text block above>
```
