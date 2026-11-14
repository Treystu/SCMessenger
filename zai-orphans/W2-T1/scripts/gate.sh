#!/usr/bin/env bash
# gate.sh -- run a build/gate command with an unfaked exit status.
#
# WHY THIS EXISTS (AGENTS.md Zai addendum rule R-Z1, loop rule 7):
# on 2026-08-16 the zai coordinator piped cargo/gradle output through
# `tail`/`grep` twice in one session and read the PIPELINE's exit code as
# the BUILD's -- recreating, in the new repo within its first hour, the
# documented failure "a piped gate can never fail"
# (docs/rules/BUILD_AND_CI.md, plausible-emptiness traps). A lesson stored
# as prose gets re-learned; this wrapper is the gate.
#
# Usage:
#   scripts/gate.sh <label> <command> [args...]
#
# Behavior:
#   - full output captured to tmp/gate-logs/<label>-<UTC stamp>.log
#   - prints the last 15 lines, the log path, and GATE_EXIT=<real code>
#   - exits with the command's REAL exit code (never the printer's)
#   - refuses labels containing '/' or '..' (log-path injection)
#
# Every cargo/gradle/test/gate invocation in this repo runs through this
# wrapper, or as a direct invocation whose exit code is echoed explicitly
# with full output saved. There is no third option.

set -u

if [ "$#" -lt 2 ]; then
    echo "[ERROR] usage: scripts/gate.sh <label> <command> [args...]" >&2
    exit 64
fi

label="$1"
shift

case "$label" in
    */*|*..*|"")
        echo "[ERROR] label must be a flat slug, got: '$label'" >&2
        exit 64
        ;;
esac

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
log_dir="$repo_root/tmp/gate-logs"
mkdir -p "$log_dir"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
log_file="$log_dir/${label}-${stamp}.log"

"$@" > "$log_file" 2>&1
real_exit=$?

echo "---- gate[$label] last 15 lines (full log: $log_file) ----"
tail -15 "$log_file"
echo "---- GATE_EXIT=$real_exit ----"

exit $real_exit
