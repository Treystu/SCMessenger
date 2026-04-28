#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

FAILED=0

fail() {
  echo "docs-sync-check: $1" >&2
  FAILED=1
}

check_header_fields() {
  local file="$1"
  local head_block

  if [[ ! -f "$file" ]]; then
    fail "missing required file: $file"
    return
  fi

  head_block="$(head -n 12 "$file")"
  if ! printf "%s\n" "$head_block" | grep -qE '^Status:'; then
    fail "$file is missing a Status header near top-of-file"
  fi
  if ! printf "%s\n" "$head_block" | grep -qE '^Last updated:'; then
    fail "$file is missing a Last updated header near top-of-file"
  fi
}

check_links_in_file() {
  local file="$1"
  local dir
  dir="$(cd "$(dirname "$file")" && pwd)"

  while IFS= read -r target; do
    local cleaned
    local resolved

    cleaned="$target"
    cleaned="${cleaned%%#*}"
    cleaned="${cleaned%%\?*}"
    cleaned="$(printf "%s" "$cleaned" | perl -pe 's/:(\d+(-\d+)?)$//')"

    [[ -z "$cleaned" ]] && continue
    [[ "$cleaned" =~ ^https?:// ]] && continue
    [[ "$cleaned" =~ ^mailto: ]] && continue
    [[ "$cleaned" =~ ^# ]] && continue

    if [[ "$cleaned" == /* ]]; then
      resolved="$ROOT_DIR$cleaned"
    else
      resolved="$dir/$cleaned"
    fi

    if [[ ! -e "$resolved" ]]; then
      fail "broken markdown link in $file -> $target"
    fi
  done < <(perl -nE 'while(/\[[^\]]+\]\(([^)]+)\)/g){say $1}' "$file")
}

check_no_machine_local_paths() {
  local file="$1"

  if rg -n '/Users/|/home/[^/]+/|[A-Za-z]:\\\\Users\\\\' "$file" >/dev/null 2>&1; then
    fail "machine-local path found in $file"
  fi
}

HEADER_FILES=(
  "DOCUMENTATION.md"
  "docs/DOCUMENT_STATUS_INDEX.md"
  "docs/REPO_CONTEXT.md"
  "docs/CURRENT_STATE.md"
  "REMAINING_WORK_TRACKING.md"
  "docs/TESTING_GUIDE.md"
  "docs/MILESTONE_PLAN_V0.2.0_ALPHA.md"
  "docs/V0.2.0_RESIDUAL_RISK_REGISTER.md"
  "docs/EDGE_CASE_READINESS_MATRIX.md"
  "docs/ARCHITECTURE.md"
  "SECURITY.md"
  "SUPPORT.md"
  ".github/copilot-instructions.md"
  "docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md"
)

LINK_CHECK_FILES=(
  "README.md"
  "DOCUMENTATION.md"
  "CONTRIBUTING.md"
  "SECURITY.md"
  "SUPPORT.md"
  "docs/DOCUMENT_STATUS_INDEX.md"
  "docs/REPO_CONTEXT.md"
  "docs/CURRENT_STATE.md"
  "docs/TESTING_GUIDE.md"
  "docs/ARCHITECTURE.md"
  "docs/V0.2.0_RESIDUAL_RISK_REGISTER.md"
  ".github/copilot-instructions.md"
  "docs/releases/RELEASE_SYNC_PLAN_V0.1.2_TO_V0.2.0.md"
)

for file in "${HEADER_FILES[@]}"; do
  check_header_fields "$file"
  check_no_machine_local_paths "$file"
done

for file in "${LINK_CHECK_FILES[@]}"; do
  check_links_in_file "$file"
done

if [[ "${DOC_SYNC_REQUIRE_DOC_UPDATES:-0}" == "1" ]]; then
  BASE_REF="${DOC_SYNC_BASE_REF:-}"
  if [[ -z "$BASE_REF" ]]; then
    fail "DOC_SYNC_REQUIRE_DOC_UPDATES=1 requires DOC_SYNC_BASE_REF"
  else
    local_code_changes="$(git diff --name-only "$BASE_REF"...HEAD -- core android iOS wasm mobile cli ui scripts || true)"
    local_doc_changes="$(git diff --name-only "$BASE_REF"...HEAD -- README.md DOCUMENTATION.md CONTRIBUTING.md SECURITY.md SUPPORT.md REMAINING_WORK_TRACKING.md docs .github/CODEOWNERS .github/ISSUE_TEMPLATE .github/pull_request_template.md .github/dependabot.yml .github/copilot-instructions.md || true)"

    if [[ -n "$local_code_changes" && -z "$local_doc_changes" ]]; then
      fail "code changed since $BASE_REF but no docs changed in canonical docs surface"
    fi
  fi
fi

if [[ "$FAILED" -ne 0 ]]; then
  echo "docs-sync-check: FAIL" >&2
  exit 1
fi

echo "docs-sync-check: PASS"
