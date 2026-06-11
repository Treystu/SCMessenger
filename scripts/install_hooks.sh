#!/usr/bin/env bash
# Install Git hooks for SCMessenger

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "Installing Git hooks..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "Error: Not in a git repository"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Install pre-commit hook
if [ -f "scripts/pre-commit" ]; then
    cp scripts/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✓${NC} Installed pre-commit hook"
else
    echo -e "${YELLOW}⚠${NC} scripts/pre-commit not found"
fi

# Install commit-msg hook
if [ -f "scripts/commit-msg" ]; then
    cp scripts/commit-msg .git/hooks/commit-msg
    chmod +x .git/hooks/commit-msg
    echo -e "${GREEN}✓${NC} Installed commit-msg hook"
else
    echo -e "${YELLOW}⚠${NC} scripts/commit-msg not found"
fi

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "The following checks will run before each commit:"
echo "  • Rust formatting (cargo fmt)"
echo "  • Clippy linting (cargo clippy)"
echo "  • Unit tests (cargo test --lib --bins)"
echo "  • No unwrap() in library code"
echo "  • No println! in library code"
echo "  • Conventional commit message format"
echo ""
echo "To skip hooks (not recommended), use: git commit --no-verify"
