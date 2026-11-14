#!/usr/bin/env python3
"""Pre-push gate: block history-rewriting pushes from ANY tool.

Called by .githooks/pre-push, so Claude Code, Cowork, Gemini/Antigravity, agy,
Codex, Copilot and humans all hit the same gate. Reads the standard pre-push
stdin format:

    <local ref> <local sha> <remote ref> <remote sha>

Blocks two classes of push:

  1. Non-fast-forward (force) push -- the remote tip is not an ancestor of what
     is being pushed, so commits that exist on the remote would be discarded.
  2. Remote branch deletion.

Why this exists: on 2026-08-08 a concurrent Antigravity session ran
`git push -f origin tracking/pre-v040-tag-work` against the head branch of an
OPEN pull request while trying to undo its own mistake. Nothing stopped it.
AGENTS.md rule 5 already forbade force-pushing shared branches; it was
documentation with no enforcement behind it. This is the enforcement.

A force-push is occasionally legitimate (an operator cleaning up their own
unshared branch). That case is the operator's call, not an agent's:

    SCM_ALLOW_FORCE_PUSH=1 git push --force origin <branch>

Exit 0 = allowed, exit 1 = blocked.
"""
import os
import subprocess
import sys

ZERO = "0" * 40


def is_ancestor(old: str, new: str) -> bool:
    """True if `old` is reachable from `new` (i.e. the push fast-forwards)."""
    result = subprocess.run(
        ["git", "merge-base", "--is-ancestor", old, new],
        capture_output=True,
    )
    return result.returncode == 0


def have_object(sha: str) -> bool:
    result = subprocess.run(
        ["git", "cat-file", "-e", sha + "^{commit}"], capture_output=True
    )
    return result.returncode == 0


def main() -> int:
    if os.environ.get("SCM_ALLOW_FORCE_PUSH") == "1":
        return 0

    violations = []
    for raw in sys.stdin:
        parts = raw.split()
        if len(parts) != 4:
            continue
        local_ref, local_sha, remote_ref, remote_sha = parts

        if local_sha.strip("0") == "":
            violations.append(
                "[FAIL] %s: remote branch DELETION is blocked." % remote_ref
            )
            continue

        if remote_sha.strip("0") == "":
            continue  # brand-new remote branch, nothing can be lost

        # If the remote tip is unknown locally we cannot prove the push is
        # safe. Fetch first rather than guessing.
        if not have_object(remote_sha):
            violations.append(
                "[FAIL] %s: remote tip %s is not present locally, so this push "
                "cannot be verified as fast-forward. Run `git fetch` first."
                % (remote_ref, remote_sha[:8])
            )
            continue

        if not is_ancestor(remote_sha, local_sha):
            violations.append(
                "[FAIL] %s: NON-FAST-FORWARD push. The remote is at %s, which "
                "is not an ancestor of %s -- commits on the remote would be "
                "discarded." % (remote_ref, remote_sha[:8], local_sha[:8])
            )

    if not violations:
        return 0

    print("pre-push: BLOCKED -- history-rewriting push refused.", file=sys.stderr)
    for line in violations:
        print(line, file=sys.stderr)
    print("", file=sys.stderr)
    print(
        "AGENTS.md rule 5: no capability class may force-push a shared branch.\n"
        "A shared branch includes the head of an open pull request.\n"
        "\n"
        "If you are trying to UNDO your own mistake, do not rewrite history --\n"
        "push a new commit that reverts it, so nobody else's work is discarded.\n"
        "\n"
        "Operator override (human decision, not an agent's):\n"
        "  SCM_ALLOW_FORCE_PUSH=1 git push --force origin <branch>",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
