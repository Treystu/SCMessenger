#!/usr/bin/env bash
# Script to audit unsafe Rust code blocks and verify SAFETY comments
# Requirements: 9.5 - Verify all unsafe Rust blocks have // SAFETY: comments

set -euo pipefail

echo "🔍 Auditing unsafe Rust code blocks..."

# Check if ripgrep is available
if ! command -v rg &> /dev/null; then
    echo "❌ ERROR: ripgrep (rg) is not installed"
    echo "Install with: cargo install ripgrep"
    exit 1
fi

# Find all unsafe blocks in Rust source code
# Exclude test files and target directory
UNSAFE_BLOCKS=$(rg -n 'unsafe\s*\{' \
    --glob '*.rs' \
    --glob '!**/target/**' \
    --glob '!**/tests/**' \
    --glob '!**/*test*.rs' \
    core/src mobile/src cli/src 2>/dev/null || true)

if [[ -z "$UNSAFE_BLOCKS" ]]; then
    echo "✅ No unsafe blocks found in library code"
    exit 0
fi

echo "Found unsafe blocks, checking for SAFETY comments..."
echo ""

FAILED=0
TOTAL=0

while IFS= read -r line; do
    if [[ -z "$line" ]]; then
        continue
    fi
    
    TOTAL=$((TOTAL + 1))
    
    # Parse file path and line number
    file=$(echo "$line" | cut -d: -f1)
    lineno=$(echo "$line" | cut -d: -f2)
    
    # Check if SAFETY comment exists within 5 lines before the unsafe block
    start_line=$((lineno - 5))
    if [[ $start_line -lt 1 ]]; then
        start_line=1
    fi
    end_line=$((lineno - 1))
    
    # Extract lines before unsafe block and check for SAFETY comment
    if sed -n "${start_line},${end_line}p" "$file" | grep -q "// SAFETY:"; then
        echo "✅ $file:$lineno - SAFETY comment found"
    else
        echo "❌ $file:$lineno - Missing SAFETY comment"
        echo "   Context:"
        sed -n "${start_line},${lineno}p" "$file" | sed 's/^/   /'
        echo ""
        FAILED=$((FAILED + 1))
    fi
done <<< "$UNSAFE_BLOCKS"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Summary: $TOTAL unsafe blocks found"

if [[ $FAILED -eq 0 ]]; then
    echo "✅ All unsafe blocks have SAFETY comments"
    exit 0
else
    echo "❌ $FAILED unsafe blocks lack SAFETY comments"
    echo ""
    echo "SAFETY comments should explain:"
    echo "  1. Why the unsafe code is necessary"
    echo "  2. What invariants must be maintained"
    echo "  3. Why the invariants are guaranteed to hold"
    echo ""
    echo "Example:"
    echo "  // SAFETY: The pointer is guaranteed to be valid because we just"
    echo "  // allocated it above and haven't freed it yet. The alignment is"
    echo "  // correct because we used Layout::from_size_align_unchecked."
    echo "  unsafe { ptr::write(ptr, value) }"
    exit 1
fi
