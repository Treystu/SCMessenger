#!/usr/bin/env bash
# reap_worktrees.sh -- report (and optionally remove) worktrees nobody is using.
#
# WHY THIS EXISTS
# AGENTS.md and CLAUDE.md tell every concurrent agent to take its own
# `git worktree`. Nothing ever told anyone to give it back. On 2026-08-15 there
# were 8 worktrees on a disk at 97% (7.5 GB free), two of them abandoned
# Antigravity subagent trees 344 commits behind, and low disk on this machine
# manifests as rustc crashes that read like source corruption
# (STATUS_STACK_BUFFER_OVERRUN, "can't find crate") -- so worktree litter shows
# up as fake compiler bugs, hours later, in someone else's session.
#
# A creation rule without a reaping rule is a leak with extra steps. This is the
# reaping rule, as a script rather than a paragraph, because the paragraph
# version of this lesson has already failed once.
#
#   scripts/reap_worktrees.sh            # report only (default, safe)
#   scripts/reap_worktrees.sh --remove   # remove only the ones judged SAFE
#
# SAFE means ALL of: not the main checkout, no uncommitted changes, no untracked
# files, and its HEAD is an ancestor of origin/main OR its branch is fully
# merged. Anything else is reported and left alone -- deleting a worktree with
# unpushed work is the same unrecoverable mistake as `git checkout -- .`.

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

REMOVE=0
[ "${1:-}" = "--remove" ] && REMOVE=1

MAIN=$(git rev-parse --show-toplevel)
printf '%-58s %-26s %-10s %s\n' "WORKTREE" "BRANCH" "STATE" "DISPOSITION"
printf '%.0s-' {1..118}; echo

safe_list=()
git worktree list --porcelain 2>/dev/null > /tmp/_wt.txt
path=""; branch=""; head=""
while IFS= read -r line; do
  case "$line" in
    worktree\ *) path="${line#worktree }" ;;
    HEAD\ *)     head="${line#HEAD }" ;;
    branch\ *)   branch="${line#branch refs/heads/}" ;;
    detached)    branch="(detached)" ;;
    "")
      [ -z "$path" ] && continue
      short="${path#"$(dirname "$MAIN")"/}"
      [ ${#short} -gt 56 ] && short="...${short: -53}"

      if [ "$path" = "$MAIN" ]; then
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "MAIN" "keep (this is the checkout)"
        path=""; branch=""; head=""; continue
      fi
      if [ ! -d "$path" ]; then
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "MISSING" "prunable: git worktree prune"
        path=""; branch=""; head=""; continue
      fi

      dirty=$(git -C "$path" status --porcelain 2>/dev/null | head -1)
      if [ -n "$dirty" ]; then
        n=$(git -C "$path" status --porcelain 2>/dev/null | wc -l)
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "DIRTY" "KEEP -- $n uncommitted path(s); ask the owner"
        path=""; branch=""; head=""; continue
      fi

      if git merge-base --is-ancestor "$head" origin/main 2>/dev/null; then
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "clean" "SAFE -- merged into origin/main"
        safe_list+=("$path")
      else
        ahead=$(git rev-list --count origin/main.."$head" 2>/dev/null || echo "?")
        printf '%-58s %-26s %-10s %s\n' "$short" "${branch:-?}" "clean" "KEEP -- $ahead commit(s) not in main"
      fi
      path=""; branch=""; head=""
      ;;
  esac
done < /tmp/_wt.txt

echo
echo "safe to remove: ${#safe_list[@]}"
if [ "${#safe_list[@]}" -eq 0 ]; then
  echo "nothing to reap."
  exit 0
fi
for p in "${safe_list[@]}"; do echo "    $p"; done

if [ "$REMOVE" -eq 0 ]; then
  echo
  echo "report only. re-run with --remove to delete the SAFE ones."
  exit 0
fi

for p in "${safe_list[@]}"; do
  if git worktree remove "$p" 2>/dev/null; then
    echo "[OK]   removed $p"
  else
    echo "[FAIL] could not remove $p (left in place)"
  fi
done
git worktree prune
echo "[OK] pruned stale administrative entries"
