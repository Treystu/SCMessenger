"""Test suite for preflight_guard.py. Exit 2 = blocked, 0 = allowed.

Run: python .claude/hooks/test_preflight_guard.py

Two of these cases exist because the first implementation got them wrong:
  - "clean after ;"  -- plain shlex.split() glues `;` to the preceding word, so
    `pwd; cargo clean` never exposed the clean at a command position. Fixed with
    shlex punctuation_chars=True.
  - "clean as quoted data" -- substring matching blocked commands that merely
    CONTAINED the string, which blocked a legitimate test harness.
"""
import json
import os
import subprocess
import sys

HOOK = os.path.join(os.path.dirname(os.path.abspath(__file__)), "preflight_guard.py")

# Built by concatenation so this file's own source never contains the literal
# that the guard matches on.
CLEAN = "cargo" + " clean"

# Whether a build is live is machine state, not a property of the command, so
# every case except the dedicated deconflict ones runs with that guard disabled.
ENV_NO_DECONFLICT = dict(os.environ, SCM_SKIP_DECONFLICT="1")

CASES = [
    ("plain clean", CLEAN, 2),
    ("clean with --target", CLEAN + " --target aarch64-linux-android", 2),
    ("clean after &&", "ls && " + CLEAN, 2),
    ("clean after inspection verb + &&", "grep -rn x docs/ && " + CLEAN, 2),
    ("clean after ;", "pwd; " + CLEAN, 2),
    ("clean via bash -c", 'bash -c "' + CLEAN + '"', 2),
    ("clean with env prefix", "FOO=1 " + CLEAN, 2),
    ("cargo -C dir clean (flag form)", "cargo -C /tmp clean", 2),
    ("clean as quoted data in python", "python -c \"print('" + CLEAN + "')\"", 0),
    ("clean mentioned in grep only", 'grep -rn "' + CLEAN + '" docs/', 0),
    ("clean mentioned in echo only", 'echo "' + CLEAN + '"', 0),
    ("cargo build --clean is not a clean", "cargo build --clean", 0),
    ("clean_target.sh is allowed", "scripts/clean_target.sh --dry-run --all", 0),
    # A commit message or doc that merely MENTIONS the command is data, not a
    # command. This blocked the hook's own commit before heredocs were stripped.
    ("clean named inside a heredoc body",
     "git commit -F - <<'EOM'\nfix: explain why " + CLEAN + " is banned\nEOM", 0),
    ("real clean after a heredoc still blocked",
     "git commit -F - <<'EOM'\nsome message\nEOM\n" + CLEAN, 2),

    ("agy missing both flags", 'agy -p "do a thing"', 2),
    ("agy missing --model", 'agy --add-dir /repo -p "x"', 2),
    ("agy missing --add-dir", 'agy --model gemini-2.5-pro -p "x"', 2),
    ("agy fully specified", 'agy --add-dir /repo --model gemini-2.5-pro -p "x"', 0),
    ("agy after && still checked", 'ls && agy -p "x"', 2),
    ("agy named in echo only", 'echo "run agy later"', 0),

    ("delegate without --mode", "python scripts/delegate_task.py --task t.md", 2),
    ("delegate with --mode", "python scripts/delegate_task.py --task t.md --mode diff", 0),

    ("unrelated command", "ls -la", 0),
    ("git status", "git status --short", 0),
    ("empty command", "", 0),
    ("whitespace only", "   ", 0),
    ("git commit with nothing staged allowed", "git commit -m 'clean'", 0),
    ("git commit --help allowed", "git commit --help", 0),
    ("git commit -h allowed", "git commit -h", 0),
    ("git commit-tree allowed", "git commit-tree HEAD^{tree} -m 'plumbing'", 0),
    ("git log --grep=commit allowed", "git log --grep='commit'", 0),
    ("echo git commit allowed", "echo 'git commit -m test'", 0),

    # --- Destructive-operation guard -------------------------------------
    # The first four are the VERBATIM commands a concurrent Antigravity session
    # ran on 2026-08-08, in order, each one after being told to stop. They
    # destroyed another session's uncommitted work. Nothing blocked them.
    ("INCIDENT: checkout discarding others' edits",
     "git checkout -- .claude/ CLAUDE.md docs/DOCUMENT_STATUS_INDEX.md", 2),
    ("INCIDENT: powershell recursive force delete",
     'powershell -Command "Remove-Item -Recurse -Force -ErrorAction '
     'SilentlyContinue .claude/hooks/preflight_guard.py, docs/rules"', 2),
    ("INCIDENT: git reset --hard", "git reset --hard 6cb7033a", 2),
    ("INCIDENT: force-push shared branch",
     "git push -f origin tracking/pre-v040-tag-work", 2),

    # Recovery moves that MUST stay allowed -- these are how the incident was
    # undone. A guard that blocks these makes recovery impossible.
    ("RECOVERY: checkout from a ref is allowed",
     "git checkout origin/main -- CLAUDE.md docs/rules .claude/settings.json", 0),
    ("RECOVERY: checkout file from a sha is allowed",
     "git checkout fbb9757d -- HANDOFF/gpt/GPT_MAC_PR139_TAKEOVER_2026-08-07.md", 0),

    ("git clean -fd blocked", "git clean -fd", 2),
    ("git rebase blocked", "git rebase main", 2),
    ("git restore discards, blocked", "git restore core/src/lib.rs", 2),
    ("git restore --staged is safe", "git restore --staged core/src/lib.rs", 0),
    ("git checkout branch switch allowed", "git checkout main", 0),
    ("normal push allowed", "git push origin main", 0),
    ("rm -rf under tmp/ allowed", "rm -rf tmp/scratch", 0),
    ("rm -rf repo path blocked", "rm -rf docs/rules", 2),
    ("rm -rf worktree target allowed", "rm -rf ../scm-anchor-860/target", 0),
    ("rm -rf worktree target subpath allowed", "rm -rf ../scm-anchor-860/target/debug", 0),
    ("rm -rf worktree non-target blocked", "rm -rf ../scm-anchor-860/src", 2),
    ("rm -rf non-worktree target blocked", "rm -rf ../unregistered-worktree-xyz/target", 2),
    ("rm -f single file allowed", "rm -f tmp/x.log", 0),

    # Lessons: $? after pipeline vs unpiped command
    ("lesson: reading $? immediately after pipe blocked", "cargo fmt --check | head; echo $?", 2),
    ("lesson: reading $? after pipe assignment blocked", "cargo test | tee test.log; rc=$?", 2),
    ("lesson: unpiped command following pipe allowed to read $?", "echo hello | grep h; python test.py; echo $?", 0),
    ("lesson: unpiped git command after pipe allowed to read $?", "git branch | grep foo; git checkout bar; echo $?", 0),
    ("lesson: multiline unpiped command after pipe allowed to read $?", "cat file.txt | grep pattern\ncargo check\nrc=$?", 0),

    # Escape hatches must work as an INLINE prefix. The hook runs as its own
    # process before the command, so an inline `VAR=1` never reaches its
    # environment -- checking only os.environ made every hatch unusable.
    ("inline override: destructive",
     "SCM_ALLOW_DESTRUCTIVE=1 git reset --hard HEAD", 0),
    ("inline override: cargo clean",
     "SCM_ALLOW_CARGO_CLEAN=1 " + CLEAN, 0),
    ("inline override: agy dispatch",
     'SCM_SKIP_DISPATCH_CHECK=1 agy -p "x"', 0),
    ("bare VAR=1 without the command still blocks",
     "SCM_ALLOW_SOMETHING_ELSE=1 git reset --hard HEAD", 2),
]


def run(cmd, env=ENV_NO_DECONFLICT):
    payload = json.dumps({"tool_name": "Bash", "tool_input": {"command": cmd}})
    p = subprocess.run([sys.executable, HOOK], input=payload,
                       capture_output=True, text=True, env=env)
    return p.returncode


def main():
    extra = []

    p = subprocess.run(
        [sys.executable, HOOK],
        input=json.dumps({"tool_name": "Read", "tool_input": {"file_path": "x"}}),
        capture_output=True, text=True)
    extra.append(("non-Bash tool allowed", p.returncode, 0))

    p = subprocess.run([sys.executable, HOOK], input="not json",
                       capture_output=True, text=True)
    extra.append(("malformed json fails open", p.returncode, 0))

    # Deconflict guard, checked against real machine state in both directions.
    try:
        live = subprocess.run(["tasklist", "/FO", "CSV", "/NH"],
                              capture_output=True, text=True, timeout=10).stdout.lower()
    except Exception:
        live = ""
    building = any(x in live for x in ("cargo.exe", "rustc.exe", "cargo-ndk.exe"))
    extra.append(("deconflict reflects real state (build live=%s)" % building,
                  run("cargo build --workspace", env=dict(os.environ)),
                  2 if building else 0))
    extra.append(("deconflict override respected", run("cargo build --workspace"), 0))

    # -- 2026-08-15 incident: `git checkout <ref> -- .` wiped four files of
    # another session's uncommitted work. The guard used to allow every
    # `checkout <ref> -- <paths>` form as "recovery". It now blocks based on
    # whether the named paths actually HAVE local changes, so recovery keeps
    # working while destruction does not. These two cases pin both halves --
    # the same command shape must flip on working-tree state alone.
    probe = "docs/rules/DELEGATION.md"
    had = os.path.exists(probe)
    original = open(probe, "rb").read() if had else None
    try:
        if had:
            with open(probe, "ab") as fh:
                fh.write(b"\n<!-- preflight guard test scratch -->\n")
            extra.append(("mass checkout from a ref is blocked",
                          run("git checkout origin/main -- ."), 2))
            extra.append(("checkout over a DIRTY path is blocked",
                          run("git checkout origin/main -- " + probe), 2))
            open(probe, "wb").write(original)
            extra.append(("checkout over a CLEAN path stays allowed",
                          run("git checkout origin/main -- " + probe), 0))
    finally:
        if had and original is not None:
            open(probe, "wb").write(original)

    # --- T1: Stale-gate blob comparison tests ----------------------------
    os.makedirs("tmp", exist_ok=True)
    idx_file = os.path.abspath("tmp/test_fixture_index")
    env_git = dict(os.environ, GIT_INDEX_FILE=idx_file)

    ref_identical_behind = "refs/test/canonical-identical-behind"
    ref_differ = "refs/test/canonical-differ"
    ref_missing_gate = "refs/test/canonical-missing-gate"
    ref_nongate_differ = "refs/test/canonical-nongate-differ"

    created_refs = []
    try:
        head_tree = subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"]).decode().strip()

        # 1. Identical tree, 25 commits ahead (HEAD is 25 behind)
        c = "HEAD"
        for i in range(25):
            c = subprocess.check_output(["git", "commit-tree", head_tree, "-p", c, "-m", "ahead %d" % i]).decode().strip()
        subprocess.check_call(["git", "update-ref", ref_identical_behind, c])
        created_refs.append(ref_identical_behind)

        # 2. Canonical tree where scripts/pr_scope.sh blob differs (stale gate A), but scripts/rules_check.py is identical
        subprocess.check_call(["git", "read-tree", "HEAD"], env=env_git)
        mod_pr_scope = subprocess.check_output(
            ["git", "hash-object", "-w", "--stdin"],
            input=b"#!/usr/bin/env bash\n# modified pr_scope\nexit 0\n"
        ).decode().strip()
        subprocess.check_call(["git", "update-index", "--add", "--cacheinfo", "100755", mod_pr_scope, "scripts/pr_scope.sh"], env=env_git)
        tree_differ = subprocess.check_output(["git", "write-tree"], env=env_git).decode().strip()
        c_differ = subprocess.check_output(["git", "commit-tree", tree_differ, "-p", "HEAD", "-m", "differ"]).decode().strip()
        subprocess.check_call(["git", "update-ref", ref_differ, c_differ])
        created_refs.append(ref_differ)

        # 3. Canonical tree where a gate script (scripts/rules_check.py) is absent
        subprocess.check_call(["git", "read-tree", "HEAD"], env=env_git)
        subprocess.check_call(["git", "update-index", "--force-remove", "scripts/rules_check.py"], env=env_git)
        tree_missing = subprocess.check_output(["git", "write-tree"], env=env_git).decode().strip()
        c_missing = subprocess.check_output(["git", "commit-tree", tree_missing, "-p", "HEAD", "-m", "missing gate"]).decode().strip()
        subprocess.check_call(["git", "update-ref", ref_missing_gate, c_missing])
        created_refs.append(ref_missing_gate)

        # 4. Canonical tree where non-gate script differs
        subprocess.check_call(["git", "read-tree", "HEAD"], env=env_git)
        mod_nongate = subprocess.check_output(
            ["git", "hash-object", "-w", "--stdin"],
            input=b"#!/usr/bin/env bash\n# modified clean_target\nexit 0\n"
        ).decode().strip()
        subprocess.check_call(["git", "update-index", "--add", "--cacheinfo", "100755", mod_nongate, "scripts/clean_target.sh"], env=env_git)
        tree_nongate = subprocess.check_output(["git", "write-tree"], env=env_git).decode().strip()
        c_nongate = subprocess.check_output(["git", "commit-tree", tree_nongate, "-p", "HEAD", "-m", "nongate differ"]).decode().strip()
        subprocess.check_call(["git", "update-ref", ref_nongate_differ, c_nongate])
        created_refs.append(ref_nongate_differ)

        # Test 1: Gate script blob IDENTICAL to canonical ref, HEAD far behind -> NO block
        env_t1 = dict(ENV_NO_DECONFLICT, _SCM_TEST_CANONICAL_REF=ref_identical_behind)
        p1 = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "scripts/pr_scope.sh 139"}}),
            capture_output=True, text=True, env=env_t1
        )
        t1_ok = (p1.returncode == 0 and p1.stderr.strip() == "")
        extra.append(("T1: identical blob when HEAD far behind allows (scm-lane-b-pr-scope case)", 0 if t1_ok else 1, 0))

        # Test 2: Gate script blob DIFFERENT from canonical ref -> BLOCK with path and git diff
        env_t2 = dict(ENV_NO_DECONFLICT, _SCM_TEST_CANONICAL_REF=ref_differ)
        p2 = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "scripts/pr_scope.sh 139"}}),
            capture_output=True, text=True, env=env_t2
        )
        diff_cmd = "git diff HEAD %s -- scripts/pr_scope.sh" % ref_differ
        t2_msg = (
            "scripts/pr_scope.sh" in p2.stderr
            and ("differs from the canonical version at %s" % ref_differ) in p2.stderr
            and diff_cmd in p2.stderr
            and ("git worktree add --detach <path> %s" % ref_differ) in p2.stderr
            and "SCM_SKIP_STALE_GATE=1" in p2.stderr
            and "PR #139" in p2.stderr
        )
        extra.append(("T1: different blob blocks", p2.returncode, 2))
        extra.append(("T1: block message contains path, diff command, remediation", 0 if t2_msg else 1, 0))

        # Test 3: Script path absent at HEAD -> NO block (fail open)
        p3 = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "scripts/verify_nonexistent_absent.sh"}}),
            capture_output=True, text=True, env=env_t2
        )
        extra.append(("T1: script path absent at HEAD fails open (allowed)", p3.returncode, 0))

        # Test 4: Script path absent at canonical ref -> NO block (fail open)
        env_t4 = dict(ENV_NO_DECONFLICT, _SCM_TEST_CANONICAL_REF=ref_missing_gate)
        p4 = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "python scripts/rules_check.py"}}),
            capture_output=True, text=True, env=env_t4
        )
        extra.append(("T1: script path absent at canonical ref fails open (allowed)", p4.returncode, 0))

        # Test 5: Canonical ref does not exist at all -> NO block (fail open)
        env_t5 = dict(ENV_NO_DECONFLICT, _SCM_TEST_CANONICAL_REF="refs/test/nonexistent-ref-xyz")
        p5 = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "scripts/pr_scope.sh 139"}}),
            capture_output=True, text=True, env=env_t5
        )
        extra.append(("T1: nonexistent canonical ref fails open (allowed)", p5.returncode, 0))

        # Test 6: Non-gate script that differs -> NO block
        env_t6 = dict(ENV_NO_DECONFLICT, _SCM_TEST_CANONICAL_REF=ref_nongate_differ)
        p6 = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "scripts/clean_target.sh --all"}}),
            capture_output=True, text=True, env=env_t6
        )
        extra.append(("T1: non-gate script differing is allowed", p6.returncode, 0))

        # Test 7: SCM_SKIP_STALE_GATE=1 -> NO block
        p7 = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "SCM_SKIP_STALE_GATE=1 scripts/pr_scope.sh 139"}}),
            capture_output=True, text=True, env=env_t2
        )
        extra.append(("T1: SCM_SKIP_STALE_GATE=1 override allows", p7.returncode, 0))

        # Test 8: Stale gate A does not block invocation of current gate B
        p8_b = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "python scripts/rules_check.py"}}),
            capture_output=True, text=True, env=env_t2
        )
        extra.append(("T1: stale gate A does not block invocation of current gate B", p8_b.returncode, 0))
    finally:
        for r in created_refs:
            subprocess.run(["git", "update-ref", "-d", r], capture_output=True)
        if os.path.exists(idx_file):
            os.remove(idx_file)

    # --- T2: Dispatch timeout floor tests --------------------------------
    try:
        os.makedirs("tmp", exist_ok=True)
        build_prompt = "tmp/test_build_prompt.txt"
        nobuild_prompt = "tmp/test_nobuild_prompt.txt"
        with open(build_prompt, "w") as f:
            f.write("Please run cargo test --workspace\n")
        with open(nobuild_prompt, "w") as f:
            f.write("Please review documentation\n")

        env_agy = dict(os.environ, AGY="true")
        p_def = subprocess.run(["bash", "scripts/agy_run.sh", "test-model", "", nobuild_prompt],
                               capture_output=True, text=True, env=env_agy)
        extra.append(("T2 timeout floor: default is 90m", 0 if "timeout=90m" in p_def.stdout else 1, 0))

        p_warn = subprocess.run(["bash", "scripts/agy_run.sh", "test-model", "30m", build_prompt],
                                capture_output=True, text=True, env=env_agy)
        warn_ok = "[WARNING] timeout 30m is below the 90m floor for build-bearing tasks" in p_warn.stderr
        extra.append(("T2 timeout floor: warning for build-bearing prompt under 90m", 0 if warn_ok else 1, 0))

        p_90m = subprocess.run(["bash", "scripts/agy_run.sh", "test-model", "90m", build_prompt],
                               capture_output=True, text=True, env=env_agy)
        extra.append(("T2 timeout floor: no warning for build-bearing prompt at 90m", 0 if "[WARNING]" not in p_90m.stderr else 1, 0))

        p_120m = subprocess.run(["bash", "scripts/agy_run.sh", "test-model", "120m", build_prompt],
                                capture_output=True, text=True, env=env_agy)
        extra.append(("T2 timeout floor: no warning for build-bearing prompt at 120m", 0 if "[WARNING]" not in p_120m.stderr else 1, 0))

        p_nobuild = subprocess.run(["bash", "scripts/agy_run.sh", "test-model", "30m", nobuild_prompt],
                                   capture_output=True, text=True, env=env_agy)
        extra.append(("T2 timeout floor: no warning for non-build prompt under 90m", 0 if "[WARNING]" not in p_nobuild.stderr else 1, 0))
    except Exception as e:
        extra.append(("T2 timeout floor tests failed setup: %s" % e, 1, 0))

    # --- T3: Commit hygiene staged content tests ------------------------
    fixture_ws = "tmp/test_fixture_ws.txt"
    fixture_emoji = "tmp/test_fixture_emoji.txt"
    fixture_clean = "tmp/test_fixture_clean.txt"
    try:
        os.makedirs("tmp", exist_ok=True)

        # 1. Clean staged file -> commit allowed
        with open(fixture_clean, "w", encoding="utf-8", newline="\n") as f:
            f.write("clean line 1\nclean line 2\n")
        subprocess.run(["git", "add", "-f", fixture_clean], capture_output=True, check=True)
        p_clean = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "git commit -m 'clean commit'"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: staged clean file allows commit", p_clean.returncode, 0))
        subprocess.run(["git", "rm", "-f", "--cached", fixture_clean], capture_output=True)

        # 2. Staged file with trailing whitespace -> commit blocked, lists offence
        with open(fixture_ws, "w", encoding="utf-8", newline="\n") as f:
            f.write("line with trailing whitespace   \nsecond line\n\n")
        subprocess.run(["git", "add", "-f", fixture_ws], capture_output=True, check=True)
        p_ws = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "git commit -m 'ws commit'"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: staged file with trailing whitespace blocks commit", p_ws.returncode, 2))
        ws_msg_ok = (
            fixture_ws in p_ws.stderr
            and "trailing whitespace" in p_ws.stderr.lower()
            and "git diff --cached --check" in p_ws.stderr
            and "SCM_SKIP_COMMIT_HYGIENE=1" in p_ws.stderr
        )
        extra.append(("T3: whitespace block message contains file, diff cmd, override", 0 if ws_msg_ok else 1, 0))

        # 2b. Read-only commands not blocked even when trailing whitespace is staged
        p_ws_help = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "git commit --help"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: git commit --help allowed even when whitespace staged", p_ws_help.returncode, 0))

        p_ws_log = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "git log --grep='commit'"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: git log --grep=commit allowed when whitespace staged", p_ws_log.returncode, 0))

        # 2c. Override allowed with SCM_SKIP_COMMIT_HYGIENE=1
        p_ws_override = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "SCM_SKIP_COMMIT_HYGIENE=1 git commit -m 'ws commit'"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: SCM_SKIP_COMMIT_HYGIENE=1 override allows whitespace commit", p_ws_override.returncode, 0))

        subprocess.run(["git", "rm", "-f", "--cached", fixture_ws], capture_output=True)

        # 3. Staged file with emoji -> commit blocked, lists codepoint
        with open(fixture_emoji, "w", encoding="utf-8", newline="\n") as f:
            f.write("clean line\nline with emoji \U0001F600 here\n")
        subprocess.run(["git", "add", "-f", fixture_emoji], capture_output=True, check=True)
        p_emoji = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "git commit -m 'emoji commit'"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: staged file containing emoji blocks commit", p_emoji.returncode, 2))
        emoji_msg_ok = (
            fixture_emoji in p_emoji.stderr
            and "U+1F600" in p_emoji.stderr
            and "AGENTS.md rule 1" in p_emoji.stderr
            and "SCM_SKIP_COMMIT_HYGIENE=1" in p_emoji.stderr
        )
        extra.append(("T3: emoji block message contains file, codepoint, rule reference", 0 if emoji_msg_ok else 1, 0))

        # 3b. Read-only commands not blocked even when emoji is staged
        p_emoji_help = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "git commit -h"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: git commit -h allowed even when emoji staged", p_emoji_help.returncode, 0))

        # 3c. Override allowed with SCM_SKIP_COMMIT_HYGIENE=1
        p_emoji_override = subprocess.run(
            [sys.executable, HOOK],
            input=json.dumps({"tool_name": "Bash", "tool_input": {"command": "SCM_SKIP_COMMIT_HYGIENE=1 git commit -m 'emoji commit'"}}),
            capture_output=True, text=True, env=ENV_NO_DECONFLICT,
        )
        extra.append(("T3: SCM_SKIP_COMMIT_HYGIENE=1 override allows emoji commit", p_emoji_override.returncode, 0))

        subprocess.run(["git", "rm", "-f", "--cached", fixture_emoji], capture_output=True)

    finally:
        for fpath in (fixture_ws, fixture_emoji, fixture_clean):
            subprocess.run(["git", "rm", "-f", "--cached", fpath], capture_output=True)
            if os.path.exists(fpath):
                try:
                    os.remove(fpath)
                except OSError:
                    pass

    npass = nfail = 0
    results = [(d, run(c), e) for d, c, e in CASES] + extra
    for desc, rc, expected in results:
        ok = rc == expected
        print(("[OK]   " if ok else "[FAIL] ") + desc +
              (" (exit %d)" % rc if ok
               else " (expected %d, got %d)" % (expected, rc)))
        npass, nfail = (npass + 1, nfail) if ok else (npass, nfail + 1)

    print("\npassed: %d  failed: %d" % (npass, nfail))
    sys.exit(1 if nfail else 0)


if __name__ == "__main__":
    main()
