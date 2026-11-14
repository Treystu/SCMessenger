#!/usr/bin/env python
"""PreToolUse hook: block Bash commands that have previously caused expensive,
silent damage in this repo, and print the SOP instead.

Each guard replaces a prose rule that cost standing context tokens on every agent
spawn and could still be forgotten. A hook costs nothing until it fires and
cannot be skipped.

Guards:
  1. cargo clean       -- wipes ALL of target/ regardless of --target (44.7 GB
                          measured loss) and can destroy
                          core/target/generated-sources/.
  2. worker dispatch   -- agy without --add-dir times out; without a pinned
                          --model it can silently spend Anthropic quota;
                          delegate_task.py without --mode silently truncates.
  3. build deconflict  -- concurrent build tools corrupt target/ in ways that
                          read as source errors.
  4. destructive ops   -- `reset --hard`, `checkout -- <paths>`, `restore`,
                          `clean -f`, `rebase`, force-push, and recursive
                          force-deletes outside tmp//target/. security.md
                          already required operator approval for these; on
                          2026-08-08 a concurrent agent ran four of them in
                          sequence and destroyed another session's work.
  5. repeat mistakes   -- patterns proven to fail repeatedly in this repo.
  6. stale checkout    -- repo gates run from a stale checkout silently run the
                          stale gate script, and stale gates fail in the safe-
                          looking direction. On 2026-08-15, a stale pr_scope.sh
                          reported [OK] on PR #139 because of an old 100-file
                          API cap, missing 6 merge-blocked crypto/transport
                          files that the current gate caught.
  7. commit hygiene    -- git commit with staged trailing whitespace, blank-
                          line-at-EOF, or emoji. Repository Hygiene went red
                          twice on PR #139 from committing unverified artifacts.

MATCHING: commands are tokenized with shlex and split into shell segments, and a
guard only fires when the trigger sits at a COMMAND POSITION. Substring matching
was tried first and was wrong in both directions:
  - false positive: `python -c "print('cargo clean')"` blocked, because the
    string appears as data. This blocked a legitimate test harness.
  - false negative: `grep x docs/ && cargo clean` allowed, because the command
    merely STARTED with an inspection verb, so the whole line was skipped.
Segment-aware matching fixes both. `bash -c "..."` payloads are recursed into,
bounded by _MAX_DEPTH.

Contract: exit 0 allows, exit 2 blocks and feeds stderr back to the agent.
Fails OPEN -- any internal error allows the command through, because a broken
hook must never wedge the session. If shlex cannot parse the command, falls back
to conservative substring matching (blocks more, never less).

Escape hatches (deliberate, per-guard). Honoured both from the environment and
as an inline `VAR=1 <command>` prefix -- see override(); checking only
os.environ silently made every hatch unusable, since this hook runs as its own
process before the command does.
  SCM_ALLOW_CARGO_CLEAN=1   skip guard 1
  SCM_SKIP_DISPATCH_CHECK=1 skip guard 2
  SCM_SKIP_DECONFLICT=1     skip guard 3
  SCM_ALLOW_DESTRUCTIVE=1   skip guard 4
  SCM_SKIP_LESSONS=1        skip guard 5
  SCM_SKIP_STALE_GATE=1     skip guard 6
  SCM_SKIP_COMMIT_HYGIENE=1 skip guard 7
"""

import json
import os
import re
import shlex
import subprocess
import sys

_MAX_DEPTH = 3
_SEPARATORS = {"&&", "||", ";", "|", "&", "\n"}
_SHELLS = {"bash", "sh", "zsh", "dash", "ksh", "pwsh", "powershell", "cmd"}

# Paths where recursive deletion is sanctioned (AGENTS.md rule 2: temp files go
# in repo-local tmp/). Everything else needs an operator decision.
_DELETE_OK_PREFIXES = ("tmp/", "./tmp/", "target/", "./target/")

# Commands that read rather than execute. A segment led by one of these is
# skipped -- but ONLY that segment, never the rest of the line.
_INSPECT = {
    "grep", "rg", "echo", "cat", "less", "more", "head", "tail",
    "awk", "sed", "printf", "wc", "find", "ls", "diff",
}

# Read-only git subcommands. A segment led by `git <one of these>` is inspecting
# the repo, so a script NAME appearing in it is a search term, not an
# invocation. Without this, `git grep -l delegate_task.py` and
# `git show <ref>:scripts/delegate_task.py` both tripped the dispatch guard --
# twice in one session on 2026-08-15, each time pushing the operator toward
# SCM_SKIP_DISPATCH_CHECK=1. A guard that cries wolf trains people to silence
# it, which is worse than not having it.
_GIT_READONLY = {
    "grep", "show", "log", "diff", "status", "ls-files", "ls-tree",
    "cat-file", "blame", "rev-parse", "rev-list", "merge-base", "describe",
}

_ENV_ASSIGN = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")

# `cmd <<'MARK' ... MARK` -- the body is DATA, not commands. Without stripping
# it, writing a commit message or doc that merely mentions `cargo clean` trips
# the guard, which blocked this hook's own commit. Verified by test.
_HEREDOC_START = re.compile(r"<<-?\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")


def strip_heredocs(command):
    """Remove heredoc bodies and their `<<MARKER` introducers.

    Removing the introducer is what makes this IDEMPOTENT, and it must be: this
    runs both in main() and inside segments(). Keeping the introducer meant a
    second pass saw an orphaned `<<'EOM'` with no terminator left and swallowed
    every following line -- which silently disabled all guards for any command
    that used a heredoc. Verified by test.
    """
    out, marker = [], None
    for line in command.splitlines():
        if marker is not None:
            if line.strip() == marker:
                marker = None
            continue
        found = _HEREDOC_START.search(line)
        if found:
            marker = found.group(2)
            out.append(_HEREDOC_START.sub("", line))
        else:
            out.append(line)
    return "\n".join(out)

# Conservative fallbacks, used only when shlex cannot parse the command.
_RAW_CARGO_CLEAN = re.compile(r"\bcargo\s+clean\b")
_RAW_AGY = re.compile(r"\bagy(\.exe)?\b")
# Subcommands that only read state. No worker, no quota, so no flags required.
_AGY_READONLY = re.compile(
    r"\bagy(\.exe)?\b[^|;&]*?\s(--help|-h|--version|models?|agents?|changelog|help|install)\b"
)
_RAW_DELEGATE = re.compile(r"delegate_task\.py")
_RAW_BUILD = re.compile(
    r"\bcargo\s+(build|test|check|clippy|run)\b|\bgradlew\b|\bcargo-ndk\b"
)

_CARGO_BUILD_VERBS = {"build", "test", "check", "clippy", "run"}

# Process images that indicate a live build. Checked only when a build command
# is about to run, so the tasklist cost is not paid on every Bash call.
#
# Deliberately EXCLUDES java.exe and gradle.exe: the Gradle daemon idles in the
# background for hours after a build finishes, so treating it as a conflict
# would block every subsequent cargo invocation. rustc.exe and cargo-ndk.exe
# only exist during active compilation, which is the condition we care about.
_BUILD_PROCS = ("cargo.exe", "rustc.exe", "cargo-ndk.exe")


def block(message):
    sys.stderr.write(message.strip() + "\n")
    sys.exit(2)


def override(name, raw=""):
    """True if escape hatch `name` is set, in the environment OR inline.

    The inline form matters and is easy to get wrong. This hook runs as its own
    process BEFORE the command does, so `SCM_ALLOW_DESTRUCTIVE=1 git push ...`
    sets the variable for git, never for the hook -- checking only os.environ
    made every documented escape hatch unusable. Verified by test.
    """
    if os.environ.get(name) == "1":
        return True
    return re.search(r"(^|\s)%s=1(\s|$)" % re.escape(name), raw) is not None


def basename(token):
    """Strip directory and .exe so ./scripts/cargo.exe -> cargo."""
    name = re.split(r"[\\/]", token)[-1]
    return name[:-4].lower() if name.lower().endswith(".exe") else name.lower()


def segments(command, depth=0):
    """Split a command line into shell segments of tokens.

    Returns a list of token-lists, each representing one command position.
    Recurses into `bash -c "..."` payloads. Returns None if the command cannot
    be tokenized, signalling the caller to use conservative fallbacks.
    """
    if depth > _MAX_DEPTH:
        return []
    command = strip_heredocs(command)

    # Split on newlines FIRST. shlex treats a newline as ordinary whitespace, so
    # `git commit ... <<EOM ... EOM` followed by a real command on the next line
    # collapsed into one segment led by `git`, hiding the second command from
    # every guard. Verified by test.
    out = []
    for line in command.splitlines():
        if not line.strip():
            continue
        try:
            # punctuation_chars=True is required: plain shlex.split() glues `;`
            # to the preceding word, so `pwd; cargo clean` tokenized as
            # ['pwd;','cargo',...] and the clean was never seen at a command
            # position. Verified by test.
            lexer = shlex.shlex(line, posix=True, punctuation_chars=True)
            lexer.whitespace_split = True
            tokens = list(lexer)
        except ValueError:
            return None
        current = []
        for token in tokens:
            if token in _SEPARATORS:
                if current:
                    out.append(current)
                current = []
            else:
                current.append(token)
        if current:
            out.append(current)

    expanded = []
    for seg in out:
        # Drop leading VAR=value assignments so `FOO=1 cargo clean` still matches.
        while seg and _ENV_ASSIGN.match(seg[0]):
            seg = seg[1:]
        if not seg:
            continue
        expanded.append(seg)
        # Recurse into shell payloads: `bash -c "..."`, and PowerShell's
        # `-Command "..."`. PowerShell matters specifically: the 2026-08-08
        # incident used `powershell -Command "Remove-Item -Recurse -Force ..."`,
        # which hides the destructive verb one level down.
        if basename(seg[0]) in _SHELLS:
            for i, tok in enumerate(seg[1:], start=1):
                if tok.lower() in ("-c", "-command", "/c") and i + 1 < len(seg):
                    nested = segments(seg[i + 1], depth + 1)
                    if nested:
                        expanded.extend(nested)
                    break
    return expanded


def is_readonly_git(seg):
    """True for `git <read-only subcommand> ...`.

    `git grep -l delegate_task.py` and `git show <ref>:scripts/delegate_task.py`
    name a script as a SEARCH TERM or a PATH, never as something to run.
    """
    if basename(seg[0]) != "git" or len(seg) < 2:
        return False
    for tok in seg[1:]:
        if tok.startswith("-"):
            continue          # skip global flags like -C <dir>, --no-pager
        return tok in _GIT_READONLY
    return False


def actionable(command):
    """Segments that actually execute something, inspection segments removed."""
    segs = segments(command)
    if segs is None:
        return None
    return [
        s for s in segs
        if basename(s[0]) not in _INSPECT and not is_readonly_git(s)
    ]


def is_cargo_clean(seg):
    return basename(seg[0]) == "cargo" and "clean" in seg[1:]


def is_cargo_build(seg):
    if basename(seg[0]) == "cargo":
        return any(v in _CARGO_BUILD_VERBS for v in seg[1:])
    name = basename(seg[0])
    return "gradlew" in name or name == "cargo-ndk"


def is_agy(seg):
    return basename(seg[0]) == "agy"


def is_delegate(seg):
    return any(t.endswith("delegate_task.py") for t in seg)


def guard_cargo_clean(segs, raw):
    if override("SCM_ALLOW_CARGO_CLEAN", raw):
        return
    hit = _RAW_CARGO_CLEAN.search(raw) if segs is None else any(
        is_cargo_clean(s) for s in segs
    )
    if not hit:
        return
    block(
        "[BLOCKED] `cargo clean` is not scoped in this repo.\n"
        "\n"
        "`cargo clean --target <triple>` does NOT clean one triple -- it wipes ALL\n"
        "of target/. Measured: 44.7 GB destroyed when ~4 GB was intended. Running\n"
        "it from inside core/ also destroys core/target/generated-sources/, after\n"
        "which ffi_surface.sh --update reports a vacuous success with no bindings.\n"
        "\n"
        "Use the script, which removes directories by explicit path, backs up\n"
        "generated-sources, and refuses to run while a build tool is live:\n"
        "\n"
        "  scripts/clean_target.sh --dry-run --all   # always look first\n"
        "  scripts/clean_target.sh --triples         # cross-compile outputs only\n"
        "  scripts/clean_target.sh --deps            # intermediates, KEEPS binaries\n"
        "  scripts/clean_target.sh --all\n"
        "\n"
        "Detail: docs/rules/BUILD_AND_CI.md\n"
        "Override (only with a stated reason): SCM_ALLOW_CARGO_CLEAN=1"
    )


def guard_dispatch(segs, raw):
    if override("SCM_SKIP_DISPATCH_CHECK", raw):
        return

    if segs is None:
        agy_segs = [None] if _RAW_AGY.search(raw) else []
        delegate_hit = bool(_RAW_DELEGATE.search(raw)) and "--mode" not in raw
    else:
        agy_segs = [s for s in segs if is_agy(s)]
        delegate_hit = any(is_delegate(s) and "--mode" not in s for s in segs)

    for seg in agy_segs:
        text = raw if seg is None else " ".join(seg)

        # Read-only subcommands and help are NOT dispatches: they spend no
        # quota and start no worker. Blocking them was a live bug -- the
        # guard's own message tells you to run `agy models` to find the exact
        # name to pin, and the guard then refused to let you run it.
        if _AGY_READONLY.search(text):
            continue

        missing = []
        if "--add-dir" not in text:
            missing.append(
                "  --add-dir <repo path>  : without it agy re-discovers the repo\n"
                "                           every dispatch and often bails before\n"
                "                           finishing. This is the root cause of\n"
                "                           what looks like a random timeout."
            )
        if "--model" not in text:
            missing.append(
                "  --model '<exact name>' : without an explicit pin, agy can route\n"
                "                           to claude-sonnet/opus and silently spend\n"
                "                           Anthropic quota. Shorthand names\n"
                "                           substitute a different model with no\n"
                "                           error -- use the exact quoted name from\n"
                "                           `agy models`."
            )
        if missing:
            block(
                "[BLOCKED] agy dispatch is missing required flags:\n\n"
                + "\n".join(missing)
                + "\n\nSOP: docs/rules/DELEGATION.md\n"
                "Override: SCM_SKIP_DISPATCH_CHECK=1"
            )

    if delegate_hit:
        block(
            "[BLOCKED] delegate_task.py called without --mode.\n"
            "\n"
            "Full-file mode silently truncates past roughly 300-500 lines, and\n"
            "flash-tier Qwen returns a vacuous success (exit 3) when handed a small\n"
            "edit in full-file form. Pass --mode explicitly:\n"
            "\n"
            "  --mode diff        scoped edits, and anything over ~300 lines\n"
            "  --mode full-file   only for genuinely small whole-file rewrites\n"
            "\n"
            "Also state the wanted output length in the prompt -- max_tokens resolves\n"
            "to the model maximum and is not a brevity constraint.\n"
            "\n"
            "SOP: docs/rules/DELEGATION.md\n"
            "Override: SCM_SKIP_DISPATCH_CHECK=1"
        )


def running_build_processes():
    """Return live build-tool process names, or [] if undetectable."""
    try:
        out = subprocess.run(
            ["tasklist", "/FO", "CSV", "/NH"],
            capture_output=True,
            text=True,
            timeout=10,
        ).stdout.lower()
    except Exception:
        return []
    return [p for p in _BUILD_PROCS if p in out]


def guard_deconflict(segs, raw):
    if override("SCM_SKIP_DECONFLICT", raw):
        return
    hit = _RAW_BUILD.search(raw) if segs is None else any(
        is_cargo_build(s) for s in segs
    )
    if not hit:
        return
    live = running_build_processes()
    if not live:
        return
    block(
        "[BLOCKED] a build tool is already running: " + ", ".join(live) + "\n"
        "\n"
        "Multiple agent sessions share this repo. Two concurrent build-tool\n"
        "invocations corrupt target/ in ways that surface as source errors, and\n"
        "Gradle spawns cargo-ndk upstream, so an Android build counts as a cargo\n"
        "build. Wait for the live build, or confirm the process is stale before\n"
        "proceeding -- check real process age, not just output-file existence.\n"
        "\n"
        "Also worth checking before a big build: `df -h /c` (this box runs near\n"
        "97% full; a disk-full failure reads as a code bug).\n"
        "\n"
        "Detail: docs/rules/BUILD_AND_CI.md\n"
        "Override: SCM_SKIP_DECONFLICT=1"
    )


def _would_discard_uncommitted(paths):
    """True if `git checkout <ref> -- <paths>` would overwrite real work.

    Asks git directly rather than guessing from the path shape. Restoring a
    path with no local modifications discards nothing and is exactly the
    recovery move the 2026-08-08 incident needed; restoring one that HAS
    modifications throws them away with no undo, which is the 2026-08-15
    incident. Same command, opposite consequence -- only the working-tree
    state distinguishes them, so that is what we check.

    Returns the list of paths that would be lost, or [] if none.
    Fails OPEN (returns []) if git cannot be consulted.
    """
    args = [p.strip("'\"") for p in paths if p.strip("'\"")]
    if not args:
        return []
    try:
        out = subprocess.run(
            ["git", "status", "--porcelain", "--"] + args,
            capture_output=True, text=True, timeout=15,
        )
        if out.returncode != 0:
            return []
        lost = []
        for line in out.stdout.splitlines():
            if len(line) < 4:
                continue
            code, name = line[:2], line[3:].strip()
            if code == "??":          # untracked: checkout does not touch it
                continue
            lost.append(name)
        return lost
    except Exception:
        return []


def discards_working_tree(seg):
    """True for any git invocation that throws away uncommitted work.

    Covers `git checkout -- <paths>`, `git checkout .`, bare `git restore`, AND
    `git checkout <ref> -- .` / `<ref> -- <dir>`.

    That last form used to be allowed outright, on the reasoning that restoring
    FROM a commit is the standard recovery move. It is -- for ONE FILE. On
    2026-08-15 `git checkout tracking/pre-v040-tag-work -- .` was run to get a
    clean tree for a grep and silently destroyed four files of another
    session's uncommitted work (core/Cargo.toml, scripts/build_wiring_graph.py,
    and two generated JSON files). Unstaged changes never enter the object
    store, so there was no recovery path: not reflog, not fsck, not stash.

    The distinction is CONSEQUENCE, not path shape. Restoring a clean path
    discards nothing; restoring a dirty one is unrecoverable. Both are spelled
    identically, so the guard asks git which case it is:
      ALLOWED  git checkout <ref> -- <paths>   when those paths are clean
      BLOCKED  git checkout <ref> -- <paths>   when any has local changes
    """
    if basename(seg[0]) != "git":
        return False
    if "checkout" in seg:
        rest = seg[seg.index("checkout") + 1:]
        rest = [t for t in rest if t == "--" or not t.startswith("-")]
        if rest and rest[0] in ("--", "."):
            return True
        # `git checkout <ref> -- <paths>`: block only if it would destroy work.
        if "--" in rest:
            paths = rest[rest.index("--") + 1:]
            if not paths:
                return True
            if _would_discard_uncommitted(paths):
                return True
    if "restore" in seg and "--staged" not in seg:
        if not any(t == "-s" or t.startswith("--source") for t in seg):
            return True
    return False


_WORKTREE_TARGETS_CACHE = None


def _registered_worktree_targets():
    global _WORKTREE_TARGETS_CACHE
    if _WORKTREE_TARGETS_CACHE is not None:
        return _WORKTREE_TARGETS_CACHE
    targets = set()
    try:
        p = subprocess.run(
            ["git", "worktree", "list", "--porcelain"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if p.returncode == 0:
            for line in p.stdout.splitlines():
                if line.startswith("worktree "):
                    wt = line.split(" ", 1)[1].strip()
                    if wt:
                        norm_wt = os.path.abspath(wt).replace("\\", "/").rstrip("/").lower()
                        targets.add(norm_wt + "/target")
    except Exception:
        pass
    _WORKTREE_TARGETS_CACHE = targets
    return _WORKTREE_TARGETS_CACHE


def _sanctioned_delete_path(path):
    norm = path.replace("\\", "/").rstrip(",").strip("'\"")
    low = norm.lower()
    if "appdata/local/temp" in low or low.startswith("/tmp/"):
        return True  # scratchpad / system temp, outside the repo
    stripped = norm[2:] if norm.startswith("./") else norm
    if stripped.startswith(_DELETE_OK_PREFIXES) or stripped.startswith(
        ("tmp/", "target/")
    ):
        return True
    abs_norm = os.path.abspath(norm).replace("\\", "/").rstrip("/").lower()
    for wt_target in _registered_worktree_targets():
        if abs_norm == wt_target or abs_norm.startswith(wt_target + "/"):
            return True
    return False


def recursive_force_delete(seg):
    """Return risky paths for a recursive+force delete, else None."""
    name = basename(seg[0])
    paths, recursive, force = [], False, False
    if name == "rm":
        for t in seg[1:]:
            if t == "--recursive":
                recursive = True
            elif t == "--force":
                force = True
            elif t.startswith("-") and not t.startswith("--"):
                if "r" in t[1:].lower():
                    recursive = True
                if "f" in t[1:]:
                    force = True
            elif not t.startswith("-"):
                paths.append(t)
    elif name in ("remove-item", "ri", "rd", "rmdir"):
        recursive = name in ("rd", "rmdir")
        for t in seg[1:]:
            low = t.lower().rstrip(",")
            if low.startswith("-recurse"):
                recursive = True
            elif low.startswith("-force"):
                force = True
            elif not t.startswith("-"):
                paths.append(t)
        if name in ("rd", "rmdir"):
            force = True
    else:
        return None
    if not (recursive and force):
        return None
    risky = [p for p in paths if not _sanctioned_delete_path(p)]
    return risky or None


def guard_destructive(segs, raw):
    """Enforce the destructive-operation rules that security.md already states.

    security.md: "Git operations that modify history (rebase, reset,
    force-push) require explicit human trust dialog" and "NEVER execute
    `rm -rf` without explicit human approval". Those were documentation with no
    enforcement until a concurrent agent ran checkout/Remove-Item/reset --hard/
    push -f in sequence on 2026-08-08 and destroyed another session's work.
    """
    if override("SCM_ALLOW_DESTRUCTIVE", raw) or segs is None:
        return

    def fail(title, detail):
        block(
            "[BLOCKED] " + title + "\n\n" + detail + "\n\n"
            "This is an OPERATOR decision, not an agent's. If you are undoing\n"
            "your own mistake, prefer a forward fix: restore from a ref\n"
            "(`git checkout <ref> -- <path>`) or add a revert commit. Never\n"
            "discard state you did not create.\n"
            "\n"
            "Rules: docs/rules/SECURITY_PROTOCOL.md, AGENTS.md rule 5\n"
            "Override: SCM_ALLOW_DESTRUCTIVE=1"
        )

    for seg in segs:
        if basename(seg[0]) == "git":
            if "reset" in seg and "--hard" in seg:
                fail(
                    "`git reset --hard` discards uncommitted work.",
                    "Another agent may have uncommitted changes in this shared\n"
                    "checkout. A hard reset destroys them with no undo, and it\n"
                    "does NOT spare untracked files if paired with a delete.",
                )
            if "rebase" in seg:
                fail(
                    "`git rebase` rewrites history.",
                    "security.md requires an explicit human trust dialog for\n"
                    "history-modifying git operations.",
                )
            if "push" in seg and any(
                t == "-f" or t.startswith("--force") for t in seg
            ):
                fail(
                    "Force-push refused.",
                    "Force-pushing a shared branch -- including the head of an\n"
                    "open pull request -- discards commits other people pushed.\n"
                    "The pre-push hook blocks this for every tool; this is the\n"
                    "earlier, clearer stop.",
                )
            if "clean" in seg and any(
                t.startswith("-") and "f" in t.lower() for t in seg[1:]
            ):
                fail(
                    "`git clean -f` deletes untracked files.",
                    "Untracked files are often another agent's in-progress work\n"
                    "that no commit or reflog can recover.",
                )
            if discards_working_tree(seg):
                fail(
                    "This discards uncommitted working-tree changes.",
                    "`git checkout -- <paths>` / `git restore <paths>` throw away\n"
                    "edits with no undo, including edits made by another session\n"
                    "sharing this checkout.\n"
                    "\n"
                    "To RESTORE a file from a commit instead (allowed):\n"
                    "  git checkout <ref> -- <path>",
                )
        risky = recursive_force_delete(seg)
        if risky:
            fail(
                "Recursive force-delete outside sanctioned paths: "
                + ", ".join(risky[:4]),
                "security.md: never `rm -rf` without explicit human approval.\n"
                "Recursive deletes are permitted under `tmp/`, `target/`, and\n"
                "the session scratchpad only. Untracked files deleted this way\n"
                "are NOT recoverable from git.",
            )


# --- Guard 5: repeat mistakes ------------------------------------------------
#
# A mistake made ONCE is a lesson. A mistake made TWICE is a missing hook.
# Everything here was made at least twice by an agent in this repo, each time
# costing a failed run and a re-diagnosis. The guard fires BEFORE the command,
# states what went wrong last time, and gives the working form -- so the lesson
# is recalled at the moment it is needed rather than written in a doc nobody
# re-reads.
#
# These block (exit 2) rather than warn. A warning printed into a transcript is
# a doc with extra steps; being made to reissue the command is what makes the
# lesson land. Override per-command with SCM_SKIP_LESSONS=1.
_LESSONS = [
    (
        # python -c '...' containing an f-string with escaped double quotes.
        re.compile(r"python3?\s+(-u\s+)?-c\s+'[^']*f\"[^']*\\\""),
        "f-string with escaped quotes inside a single-quoted python -c",
        "Bash single-quoting turns \\\" inside an f-string into a SyntaxError.\n"
        "Made twice on 2026-08-15: scripts/agy_stream_watch.py and\n"
        "scripts/pr_scope.sh, one failed run each.\n\n"
        "Use %-formatting, or put the script in a file:\n"
        "  print(\"%s\" % d[\"key\"])              # works\n"
        "  python3 - \"$ARG\" <<'PYEOF' ... PYEOF  # works, no quoting at all",
    ),
    (
        # A python invocation that reads or writes an absolute /tmp path.
        # Deliberately loose: the first version used [^|;&]* to stay within one
        # segment, which excluded the semicolon in `import json;d=...` and so
        # missed the very case it was written for. A false positive here costs
        # one override; a false negative costs a silent wrong answer.
        re.compile(r"python3?\b.*['\"]/tmp/"),
        "/tmp path passed to python on Windows",
        "Git Bash maps /tmp; python3 does not. The open() raises, the caller\n"
        "sees an empty string, and a numeric guard like ${VAR:-0} silently\n"
        "defaults -- turning a failure into a false PASS.\n"
        "Made twice on 2026-08-15; the second one made a merge-safety check\n"
        "report 'all checks green' while five checks were still running.\n\n"
        "Use the repo-local tmp/ (AGENTS.md rule 2):\n"
        "  T=\"$(git rev-parse --show-toplevel)/tmp\"; mkdir -p \"$T\"",
    ),
    (
        # Broad staging in a shared checkout.
        re.compile(r"\bgit\s+add\s+(-A\b|--all\b|-u\b|\.(\s|$))"),
        "git add -A / -u / . in a shared checkout",
        "This stages files you did not create. Other agents and the operator\n"
        "work in this checkout concurrently, and their untracked or modified\n"
        "files end up in your commit.\n"
        "Made on 2026-08-15: `git add -A scripts/` swept in five untracked\n"
        "files belonging to another session. Same class as the `git checkout\n"
        "<ref> -- .` that destroyed four files earlier the same day -- a broad\n"
        "path operator applied to a shared tree.\n\n"
        "Stage explicit paths (AGENTS.md):\n"
        "  git add path/one.rs path/two.md\n"
        "  git status --short          # confirm ONLY your files are staged",
    ),
    (
        # Reading $? after a pipeline.
        re.compile(r"\|[^;&|\n]+[;&|\n]+[^;&|\n]*\$\?"),
        "reading $? after a pipe",
        "The pipeline's exit status is the LAST command's, so a piped gate can\n"
        "never fail. `cargo fmt --check | head; echo $?` always prints 0.\n\n"
        "Capture first, then test:\n"
        "  cargo fmt --check > out.txt; rc=$?; head out.txt; exit $rc",
    ),
]


def guard_lessons(segs, raw):
    if override("SCM_SKIP_LESSONS", raw):
        return
    for pattern, title, lesson in _LESSONS:
        if pattern.search(raw):
            block(
                "[REMEMBER] %s\n\n%s\n\n"
                "This has been done before in this repo. Reissue the command in\n"
                "the working form above.\n"
                "Override: SCM_SKIP_LESSONS=1" % (title, lesson)
            )


# --- Guard 6: stale checkout ------------------------------------------------
#
# A repo gate run from a stale checkout silently runs the STALE gate script,
# and stale gates fail in the safe-looking direction.
#
# On 2026-08-15, two CTO sessions ran concurrently. The shared checkout sat 23
# commits behind tracking. `scripts/pr_scope.sh` there was therefore the OLD
# version, which derived its file list from `gh pr view --json files` -- an API
# that silently capped at 100 files. PR #139 changed 253 files. Run from the
# stale checkout, the gate printed:
#     [OK]      clear of core/src/{crypto,transport,routing,privacy}
# Run from a current checkout, the SAME gate on the SAME PR printed:
#     [BLOCKER] touches merge-blocked directories (AGENTS.md rule 8):
#                 core/src/crypto/backup.rs
#                 core/src/transport/addr_filter.rs
#                 core/src/transport/behaviour.rs
#                 core/src/transport/dial_policy.rs
#                 core/src/transport/observation.rs
#                 core/src/transport/swarm.rs
#
# The gate reported PASS while six merge-blocked files were invisible to it.
# The repair was already committed (#158), but was not in the checkout where the
# runbook instructed people to run it.
#
# This guard blocks when a repo gate/verification script is invoked from a
# working tree whose HEAD is behind base refs without performing network operations.

_GATE_SCRIPT_NAMES = {
    "pr_scope.sh",
    "docs_sync_check.sh",
    "docs_sync_check.ps1",
    "triage_lane.sh",
    "preflight.sh",
    "preflight_disk.sh",
    "preflight_disk.ps1",
    "build_verify.sh",
    "rules_check.py",
    "prepush_check.py",
    "orchestration_contract.py",
    "repo_audit.sh",
    "audit_unsafe.sh",
    "audit_session_logs.sh",
    "validate_tag.sh",
    "validate_v020_final.sh",
    "verify_incremental_gate.py",
    "verify_swift_violations.py",
}

_GATE_SCRIPT_RE = re.compile(
    r"(?:^|[/\\])"
    r"(?:pr_scope\.sh|docs_sync_check\.(?:sh|ps1)|triage_lane\.sh|preflight(?:\.sh|_disk\.(?:sh|ps1))|"
    r"build_verify\.sh|rules_check\.py|prepush_check\.py|orchestration_contract\.py|"
    r"repo_audit\.sh|audit_unsafe\.sh|audit_session_logs\.sh|validate_tag\.sh|validate_v020_final\.sh|"
    r"verify_[a-zA-Z0-9_-]+\.(?:sh|py|ps1)|check_[a-zA-Z0-9_-]+\.(?:sh|py|ps1)|"
    r"[a-zA-Z0-9_-]+_(?:check|verify)[a-zA-Z0-9_-]*\.(?:sh|py|ps1))$",
    re.IGNORECASE,
)

_RAW_GATE_RE = re.compile(
    r"(?:^|[\s/\\\"'])(?:scripts/|\.claude/skills/|\.agents/skills/)?"
    r"(pr_scope\.sh|docs_sync_check\.(?:sh|ps1)|triage_lane\.sh|preflight(?:\.sh|_disk\.(?:sh|ps1))|"
    r"build_verify\.sh|rules_check\.py|prepush_check\.py|orchestration_contract\.py|"
    r"repo_audit\.sh|audit_unsafe\.sh|audit_session_logs\.sh|validate_tag\.sh|validate_v020_final\.sh|"
    r"verify_[a-zA-Z0-9_-]+\.(?:sh|py|ps1)|check_[a-zA-Z0-9_-]+\.(?:sh|py|ps1)|"
    r"[a-zA-Z0-9_-]+_(?:check|verify)[a-zA-Z0-9_-]*\.(?:sh|py|ps1))\b",
    re.IGNORECASE,
)

_CANONICAL_REF = "origin/tracking/pre-v040-tag-work"


def is_gate_script(tok):
    clean = tok.strip("'\"")
    bname = os.path.basename(clean.replace("\\", "/")).lower()
    return bname in _GATE_SCRIPT_NAMES or _GATE_SCRIPT_RE.search(clean) is not None


def _get_blob_oid(ref, path):
    """Return git blob object ID for path at ref, or None if missing or on error."""
    try:
        p = subprocess.run(
            ["git", "rev-parse", "--verify", "%s:%s" % (ref, path)],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if p.returncode == 0:
            oid = p.stdout.strip()
            if oid:
                return oid
    except Exception:
        pass
    return None


def resolve_script_path(tok):
    clean = tok.strip("'\"").replace("\\", "/")
    if clean.startswith("./"):
        clean = clean[2:]
    return clean


def _resolve_repo_path(path):
    """Resolve relative path against HEAD if a bare script name was used."""
    norm = resolve_script_path(path)
    if _get_blob_oid("HEAD", norm) is not None:
        return norm
    if "/" not in norm:
        cand = "scripts/" + norm
        if _get_blob_oid("HEAD", cand) is not None:
            return cand
    return norm


def guard_stale_checkout(segs, raw):
    if override("SCM_SKIP_STALE_GATE", raw):
        return

    gate_scripts = []
    if segs is not None:
        for seg in segs:
            for tok in seg:
                if is_gate_script(tok):
                    gate_scripts.append(tok)
    else:
        for m in _RAW_GATE_RE.finditer(raw):
            gate_scripts.append(m.group(1))

    if not gate_scripts:
        return

    canonical_ref = (
        os.environ.get("_SCM_TEST_CANONICAL_REF")
        or os.environ.get("_SCM_TEST_STALE_BASE_REFS")
        or _CANONICAL_REF
    )

    for tok in gate_scripts:
        script_path = _resolve_repo_path(tok)
        head_oid = _get_blob_oid("HEAD", script_path)
        if not head_oid:
            continue  # Path missing at HEAD or git error -> fail open

        canonical_oid = _get_blob_oid(canonical_ref, script_path)
        if not canonical_oid:
            continue  # Path missing at ref, ref missing, or git error -> fail open

        if head_oid != canonical_oid:
            block(
                "[BLOCKED] repo gate differs from canonical version.\n"
                "\n"
                "`%s` differs from the canonical version at %s.\n"
                "\n"
                "A repo gate run from a stale checkout silently runs the STALE gate script,\n"
                "and stale gates fail in the safe-looking direction. On 2026-08-15, a stale\n"
                "`scripts/pr_scope.sh` (lacking #158) reported [OK] on PR #139 because of an old\n"
                "100-file API cap, missing 6 merge-blocked crypto/transport files that the current\n"
                "gate caught.\n"
                "\n"
                "To see what changed:\n"
                "  git diff HEAD %s -- %s\n"
                "\n"
                "Create a worktree at the canonical ref and run it there:\n"
                "  git worktree add --detach <path> %s\n"
                "\n"
                "Detail: docs/rules/BUILD_AND_CI.md, AGENTS.md rule 13\n"
                "Override: SCM_SKIP_STALE_GATE=1"
                % (script_path, canonical_ref, canonical_ref, script_path, canonical_ref)
            )


# --- Guard 7: commit hygiene ------------------------------------------------
#
# Repository Hygiene went red on PR #139 on 2026-08-15 and again on 2026-08-16
# from committing files verbatim carrying trailing whitespace.
# AGENTS.md rule 1 bans emoji across all code and docs; staged commits were a gap.
#
# This guard blocks `git commit` when staged content contains whitespace errors
# (trailing whitespace, blank-line-at-EOF) or emoji characters.

_EMOJI_EXEMPT_PREFIXES = ("docs/historical/",)
_BINARY_SUFFIXES = (
    ".png", ".jpg", ".jpeg", ".gif", ".ico", ".webp", ".so", ".a", ".dll",
    ".dylib", ".jar", ".aar", ".apk", ".keystore", ".jks", ".zip", ".gz",
    ".xcframework", ".ttf", ".otf", ".woff", ".woff2", ".bin", ".exe",
)

_RAW_COMMIT = re.compile(r"\bgit\b(?:\s+-[^\s]+(?:\s+[^\s]+)?)*\s+commit\b")
_RAW_COMMIT_HELP = re.compile(
    r"\bgit\b(?:\s+-[^\s]+(?:\s+[^\s]+)?)*\s+commit\b.*?\s(--help|-h)\b"
)


def is_blocked_emoji_codepoint(codepoint: int) -> bool:
    """Return whether a code point is within the repository's blocked ranges."""
    return (
        0x1F300 <= codepoint <= 0x1FAFF
        or 0x1F1E6 <= codepoint <= 0x1F1FF
        or 0x2600 <= codepoint <= 0x27BF
    )


def parse_git_commit(seg):
    """Return (True, target_cwd) if seg is an actual git commit invocation, else (False, None)."""
    if not seg or basename(seg[0]) != "git":
        return False, None
    idx = 1
    subcmd = None
    target_cwd = None
    while idx < len(seg):
        tok = seg[idx]
        if tok == "-C" and idx + 1 < len(seg):
            target_cwd = seg[idx + 1]
            idx += 2
            continue
        if (
            tok in ("-c", "--git-dir", "--work-tree", "--namespace", "--super-prefix", "--exec-path")
            and idx + 1 < len(seg)
        ):
            idx += 2
            continue
        if tok.startswith("-"):
            idx += 1
            continue
        subcmd = tok
        break

    if subcmd != "commit":
        return False, None

    commit_args = seg[idx + 1:]
    if any(t in ("--help", "-h") for t in commit_args):
        return False, None

    return True, target_cwd


def check_staged_whitespace(cwd=None):
    """Run `git diff --cached --check` to detect whitespace errors in staged content.

    Returns all offending lines reported by git (empty if clean).

    Exempts a pure line-ending renormalisation. `git add --renormalize .`
    converts CRLF to LF across already-committed files; any line that was
    `<text><space><CR><LF>` becomes `<text><space><LF>`, which git then reports
    as newly-added trailing whitespace even though not one byte of content
    changed. This repo has 546 such files and 1,387 such lines, 291 of them
    legitimate Markdown hard line breaks that MUST NOT be stripped.

    The exemption is deliberately narrow: `--ignore-cr-at-eol` reports any
    difference other than the line terminator itself, so a commit that actually
    ADDS trailing whitespace still fails. Only a commit that changes nothing but
    CR bytes is skipped.
    """
    try:
        cr_only = subprocess.run(
            ["git", "diff", "--cached", "--ignore-cr-at-eol", "--numstat"],
            capture_output=True, text=True, timeout=15, cwd=cwd,
        )
        if cr_only.returncode == 0 and not (cr_only.stdout or "").strip():
            staged = subprocess.run(
                ["git", "diff", "--cached", "--numstat"],
                capture_output=True, text=True, timeout=15, cwd=cwd,
            )
            if (staged.stdout or "").strip():
                return []
    except Exception:
        pass
    try:
        cmd = ["git", "diff", "--cached", "--check"]
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=15, cwd=cwd)
        combined = (p.stdout or "") + (p.stderr or "")
        lines = [line.strip() for line in combined.splitlines() if line.strip()]
        if p.returncode != 0 and lines:
            return lines
        elif lines and any(
            "whitespace" in l.lower() or "blank line" in l.lower() for l in lines
        ):
            return lines
        return []
    except Exception:
        return []


def check_staged_emojis(cwd=None):
    """Scan staged text content for emoji characters.

    Returns a list of violation lines, e.g. 'path/file.txt:12: contains emoji (U+1F600)'.
    """
    try:
        cmd = ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR"]
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=15, cwd=cwd)
        if p.returncode != 0 or not p.stdout.strip():
            return []
        staged = [line.strip() for line in p.stdout.splitlines() if line.strip()]
        violations = []
        for path in staged:
            norm = path.replace("\\", "/")
            if any(norm.startswith(pfx) for pfx in _EMOJI_EXEMPT_PREFIXES):
                continue
            if any(norm.endswith(sfx) for sfx in _BINARY_SUFFIXES):
                continue
            p_show = subprocess.run(
                ["git", "show", ":" + norm],
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="ignore",
                timeout=10,
                cwd=cwd,
            )
            if p_show.returncode != 0 or not p_show.stdout:
                continue
            for lineno, line in enumerate(p_show.stdout.splitlines(), start=1):
                hits = [c for c in line if is_blocked_emoji_codepoint(ord(c))]
                if hits:
                    cps = ", ".join(f"U+{ord(c):04X}" for c in hits)
                    violations.append(f"{norm}:{lineno}: contains emoji ({cps})")
        return violations
    except Exception:
        return []


def guard_commit_hygiene(segs, raw):
    if override("SCM_SKIP_COMMIT_HYGIENE", raw) or override("SCM_ALLOW_COMMIT_HYGIENE", raw):
        return

    commit_found = False
    target_cwd = None

    if segs is not None:
        for seg in segs:
            is_commit, cwd = parse_git_commit(seg)
            if is_commit:
                commit_found = True
                target_cwd = cwd
                break
    else:
        if (
            _RAW_COMMIT.search(raw)
            and not _RAW_COMMIT_HELP.search(raw)
            and not re.search(r"\bgit\s+commit-(?:tree|graph)\b", raw)
        ):
            commit_found = True

    if not commit_found:
        return

    ws_errors = check_staged_whitespace(cwd=target_cwd)
    emoji_errors = check_staged_emojis(cwd=target_cwd)

    if not ws_errors and not emoji_errors:
        return

    sections = []
    if ws_errors:
        sections.append(
            "Trailing whitespace or blank-line-at-EOF found in staged content:\n"
            + "\n".join("  " + err for err in ws_errors)
            + "\n\n"
            "Committing trailing whitespace turns Repository Hygiene red on CI\n"
            "(AGENTS.md rule 15, HANDOFF/CTO_STATE.md section 8).\n\n"
            "To inspect:\n"
            "  git diff --cached --check\n\n"
            "To fix trailing whitespace in a file:\n"
            "  # Remove trailing whitespace and re-stage:\n"
            "  git add <file>"
        )

    if emoji_errors:
        sections.append(
            "Emoji character(s) found in staged content:\n"
            + "\n".join("  " + err for err in emoji_errors)
            + "\n\n"
            "Per AGENTS.md rule 1, no emoji anywhere (code, docs, comments, logs).\n"
            "Use plain-text tags ([OK], [ERROR], [WARNING], [INFO]) instead.\n\n"
            "To fix:\n"
            "  Replace emoji with plain-text tags and re-stage:\n"
            "  git add <file>"
        )

    block(
        "[BLOCKED] staged content violates commit hygiene rules.\n\n"
        + "\n\n".join(sections)
        + "\n\nDetail: AGENTS.md rule 1 & rule 15, HANDOFF/CTO_STATE.md section 8\n"
        "Override: SCM_SKIP_COMMIT_HYGIENE=1"
    )


def main():
    try:
        payload = json.load(sys.stdin)
    except Exception:
        sys.exit(0)  # fail open

    if payload.get("tool_name") != "Bash":
        sys.exit(0)

    raw = (payload.get("tool_input") or {}).get("command", "")
    if not raw.strip():
        sys.exit(0)

    try:
        # Strip heredoc bodies for the fallback path too, so an unparseable
        # command does not false-positive on documentation text.
        raw = strip_heredocs(raw)
        segs = actionable(raw)  # None => unparseable, use raw fallbacks
        if segs is not None and not segs:
            sys.exit(0)  # inspection-only command line
        guard_destructive(segs, raw)
        guard_cargo_clean(segs, raw)
        guard_dispatch(segs, raw)
        guard_deconflict(segs, raw)
        guard_lessons(segs, raw)
        guard_stale_checkout(segs, raw)
        guard_commit_hygiene(segs, raw)
    except SystemExit:
        raise
    except Exception:
        sys.exit(0)  # fail open

    sys.exit(0)


if __name__ == "__main__":
    main()
