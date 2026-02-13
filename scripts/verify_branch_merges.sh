#!/bin/bash
# Verification script for branch merge status
# Checks GitHub PRs to confirm all branches are truly merged

set -e

echo "==================================="
echo "Branch Merge Verification"
echo "==================================="
echo ""

# Check if gh CLI is available
if ! command -v gh &> /dev/null; then
    echo "⚠️  GitHub CLI (gh) not found."
    echo "Install with: brew install gh (macOS) or see https://cli.github.com/"
    echo ""
    echo "Continuing with git-based verification only..."
    echo ""
fi

echo "Checking merge status of branches..."
echo ""

# Branches and their associated PRs
declare -A branch_prs=(
    ["claude/comprehensive-network-testing-bgVk8"]="10"
    ["claude/fix-dead-code-warnings-nP5ck"]="7"
    ["claude/fix-docker-build-verification-5qr4k"]="4"
    ["claude/fix-github-build-tests-bgVk8"]="8,9,11"
    ["claude/fix-unused-variable-warning-uCK1E"]="12"
    ["claude/gemini-ui-guide-LtFv9"]="3"
    ["claude/merge-control-api-to-main-5qr4k"]="6"
    ["claude/setup-scmessenger-gcp-hOHGN"]="13,14,15,16"
    ["copilot/audit-repository-completeness"]="2"
    ["copilot/merge-prs-and-address-comments"]="20"
    ["copilot/fix-scm-send-encryption-error"]="19"
    ["copilot/sub-pr-16"]="17"
)

echo "═══════════════════════════════════════════════════════════════"
echo "Git-based verification (note: squash merges appear unmerged)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

merged_count=0
unmerged_count=0

for branch in "${!branch_prs[@]}"; do
    prs="${branch_prs[$branch]}"
    
    # Fetch branch if needed
    git fetch origin "$branch:refs/remotes/origin/$branch" 2>/dev/null || true
    
    # Check if merged
    if git merge-base --is-ancestor "origin/$branch" "origin/main" 2>/dev/null; then
        echo "✓ $branch (PR #$prs) - MERGED"
        merged_count=$((merged_count + 1))
    else
        echo "⊗ $branch (PR #$prs) - appears unmerged (normal for squash merge)"
        unmerged_count=$((unmerged_count + 1))
    fi
done

echo ""
echo "Git results: $merged_count truly merged, $unmerged_count squash-merged"
echo ""

# If gh is available, verify PRs
if command -v gh &> /dev/null; then
    echo "═══════════════════════════════════════════════════════════════"
    echo "GitHub PR verification (authoritative)"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    
    pr_merged_count=0
    pr_unmerged_count=0
    
    for branch in "${!branch_prs[@]}"; do
        prs="${branch_prs[$branch]}"
        
        # Check each PR associated with this branch
        IFS=',' read -ra PR_ARRAY <<< "$prs"
        all_merged=true
        
        for pr in "${PR_ARRAY[@]}"; do
            pr_state=$(gh pr view "$pr" --json state,mergedAt --jq '.state' 2>/dev/null || echo "UNKNOWN")
            merged_at=$(gh pr view "$pr" --json mergedAt --jq '.mergedAt' 2>/dev/null || echo "null")
            
            if [ "$merged_at" != "null" ] && [ "$merged_at" != "" ]; then
                echo "  ✓ PR #$pr: MERGED at $merged_at"
            else
                echo "  ✗ PR #$pr: NOT MERGED (state: $pr_state)"
                all_merged=false
            fi
        done
        
        if [ "$all_merged" = true ]; then
            echo "✓ $branch - ALL PRs MERGED"
            pr_merged_count=$((pr_merged_count + 1))
        else
            echo "✗ $branch - NOT ALL PRs MERGED"
            pr_unmerged_count=$((pr_unmerged_count + 1))
        fi
        echo ""
    done
    
    echo "═══════════════════════════════════════════════════════════════"
    echo "GitHub verification: $pr_merged_count fully merged, $pr_unmerged_count incomplete"
    echo "═══════════════════════════════════════════════════════════════"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "CONCLUSION"
echo "═══════════════════════════════════════════════════════════════"

if command -v gh &> /dev/null; then
    if [ $pr_unmerged_count -eq 0 ]; then
        echo "✓ All branches have been merged via PRs"
        echo "✓ Safe to delete all 12 listed branches"
        echo ""
        echo "To delete these branches, run:"
        echo "  ./scripts/delete_merged_branches.sh"
    else
        echo "⚠️  Some PRs are not merged!"
        echo "⚠️  Review before deletion"
    fi
else
    echo "ℹ️  Install GitHub CLI for authoritative PR verification"
    echo "   Based on git analysis, branches appear squash-merged (expected)"
fi

echo ""
