#!/usr/bin/env python3
"""Static gate audit for #619: every notification post site must be gated or exempt.

Defect #619 ("notifications-disabled setting is ignored") is a *coverage* problem
before it is a behaviour problem: the in-app toggle only means anything if every
path that can put a card in the Android notification bar consults the gate. The
post primitives are few and the gate is one field, so the invariant is checkable
without a device, an emulator or a Gradle build:

  INVARIANT 1  Only the files on POSTING_FILES may contain a post primitive.
  INVARIANT 2  Inside NotificationHelper.kt - the one place that is allowed to
               decide - every post primitive must sit in a function that reads
               `notificationsEnabled` before it posts.
  INVARIANT 3  Inside the service/worker files, every post primitive must sit in
               a function on MANDATORY_FGS_ALLOWLIST, i.e. a notification that
               Android or WorkManager requires in order for a foreground service
               or an expedited worker to run at all. Those cannot be suppressed
               and are the documented #619 exemption.

This is a source-level guard, deliberately narrow: it fails CLOSED on a new post
site (a human must classify it) and it prints every site it saw, not a sample,
so a reader can check the classification rather than trust the verdict.

Usage:
  python3 scripts/notification_gate_audit.py [--repo-root .] [--verbose]

Exit 0 = every post site classified. Exit 1 = at least one unclassified site.
Exit 2 = the scan itself could not run (missing tree, unreadable file).
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ANDROID_MAIN = Path("android/app/src/main")

# Files allowed to post at all. Adding a file here is the review decision.
POSTING_FILES = {
    "android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt",
    "android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt",
    "android/app/src/main/java/com/scmessenger/android/service/MeshAutoRestart.kt",
}

GATED_FILE = (
    "android/app/src/main/java/com/scmessenger/android/utils/NotificationHelper.kt"
)

# Mandatory foreground-service posts: Android requires a visible FGS
# notification, and WorkManager requires ForegroundInfo for expedited work.
# Neither can be suppressed while the mesh runs; #619 records them as the
# explicit exemption rather than pretending the toggle silences them.
MANDATORY_FGS_ALLOWLIST = {
    ("android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt", "createNotification"): "FGS status card required by startForeground()",
    ("android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt", "tryStartForeground"): "FGS status card required by startForeground()",
    ("android/app/src/main/java/com/scmessenger/android/service/MeshForegroundService.kt", "updateNotification"): "re-post of the same FGS status card id",
    ("android/app/src/main/java/com/scmessenger/android/service/MeshAutoRestart.kt", "getForegroundInfo"): "WorkManager expedited-worker ForegroundInfo",
    ("android/app/src/main/java/com/scmessenger/android/service/MeshAutoRestart.kt", "doWork"): "WorkManager setForeground(ForegroundInfo)",
}

POST_PATTERNS = (
    re.compile(r"\.notify\("),
    re.compile(r"NotificationManagerCompat\.from\([^)]*\)\.notify\("),
    re.compile(r"\bstartForeground\("),
    re.compile(r"ServiceCompat\.startForeground\("),
    re.compile(r"\bForegroundInfo\("),
    re.compile(r"\.setForeground\("),
)

FUN_RE = re.compile(
    r"^\s*(?:(?:private|internal|public|protected|override|actual|expect|open|abstract|external|suspend|inline|operator)\s+)*"
    r"fun\s+(?:<[^>]+>\s*)?(?:[\w.]+\.)?(\w+)\s*\("
)
GATE_FIELD = "notificationsEnabled"


def enclosing_function(lines: list[str], index: int) -> tuple[str, int]:
    """Nearest preceding `fun` declaration and its line index."""
    for i in range(index, -1, -1):
        m = FUN_RE.match(lines[i])
        if m:
            return m.group(1), i
    return "<file-scope>", -1


def scan(root: Path) -> tuple[list[dict], list[str]]:
    tree = root / ANDROID_MAIN
    if not tree.is_dir():
        raise FileNotFoundError(f"missing Android source tree: {tree}")

    sites: list[dict] = []
    for path in sorted(tree.rglob("*.kt")):
        rel = path.relative_to(root).as_posix()
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError as exc:  # pragma: no cover - unreadable file
            raise RuntimeError(f"cannot read {rel}: {exc}") from exc
        for idx, line in enumerate(lines):
            stripped = line.lstrip()
            if stripped.startswith("//") or stripped.startswith("*"):
                continue
            if not any(p.search(line) for p in POST_PATTERNS):
                continue
            func, func_line = enclosing_function(lines, idx)
            body = "\n".join(lines[func_line + 1 : idx + 1]) if func_line >= 0 else ""
            sites.append(
                {
                    "path": rel,
                    "line": idx + 1,
                    "func": func,
                    "gated_in_body": GATE_FIELD in body,
                    "text": stripped[:110],
                }
            )
    return sites, []


def classify(sites: list[dict]) -> tuple[list[dict], list[str]]:
    verdicts: list[str] = []
    for site in sites:
        key = (site["path"], site["func"])
        if site["path"] not in POSTING_FILES:
            site["verdict"] = "UNCLASSIFIED"
            site["reason"] = "file is not on POSTING_FILES"
        elif site["path"] == GATED_FILE:
            if site["gated_in_body"]:
                site["verdict"] = "GATED"
                site["reason"] = f"function reads {GATE_FIELD} before posting"
            else:
                site["verdict"] = "UNCLASSIFIED"
                site["reason"] = f"no {GATE_FIELD} check in enclosing function"
        elif key in MANDATORY_FGS_ALLOWLIST:
            site["verdict"] = "EXEMPT-MANDATORY-FGS"
            site["reason"] = MANDATORY_FGS_ALLOWLIST[key]
        else:
            site["verdict"] = "UNCLASSIFIED"
            site["reason"] = "not on the mandatory-FGS allowlist"
        if site["verdict"] == "UNCLASSIFIED":
            verdicts.append(f"{site['path']}:{site['line']} ({site['func']}): {site['reason']}")
    return sites, verdicts


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--verbose", action="store_true", help="print every line's text")
    args = parser.parse_args(argv)

    root = Path(args.repo_root).resolve()
    try:
        sites, _ = scan(root)
    except (FileNotFoundError, RuntimeError) as exc:
        print(f"[FAIL] {exc}")
        return 2

    sites, problems = classify(sites)
    gated = [s for s in sites if s["verdict"] == "GATED"]
    exempt = [s for s in sites if s["verdict"] == "EXEMPT-MANDATORY-FGS"]

    print("Notification post-site audit (#619)")
    print("=" * 78)
    print(f"  scanned : {ANDROID_MAIN}/**/*.kt")
    print(f"  post sites found : {len(sites)}  (all printed below, nothing truncated)")
    print()
    for site in sites:
        print(f"  [{site['verdict']}] {site['path']}:{site['line']}  fun={site['func']}")
        print(f"      {site['reason']}")
        if args.verbose:
            print(f"      {site['text']}")
    print()
    print(f"  gated                : {len(gated)}")
    print(f"  exempt (mandatory FGS): {len(exempt)}")
    for site in exempt:
        print(f"      {site['path'].split('/')[-1]}:{site['line']} {site['func']}")
    print()

    if problems:
        print("[FAIL] unclassified post sites - classify each before landing:")
        for problem in problems:
            print(f"  - {problem}")
        print()
        print("[WARNING] an unclassified site is a place the #619 toggle can be bypassed.")
        return 1

    print("[OK] every post site is either gated by notificationsEnabled or on the")
    print("     documented mandatory-foreground-service allowlist.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
