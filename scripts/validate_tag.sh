#!/usr/bin/env bash
# Version tag validation script
# Validates: Requirements 8.5, 8.9

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "🔍 Validating version tag..."

# Check if tag is provided
if [ $# -eq 0 ]; then
    echo -e "${RED}❌ Usage: $0 <tag>${NC}"
    echo "Example: $0 v0.2.2"
    exit 1
fi

TAG="$1"

# Extract version from workspace Cargo.toml
VERSION=$(grep -m 1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [ -z "$VERSION" ]; then
    echo -e "${RED}❌ Failed to extract version from Cargo.toml${NC}"
    exit 1
fi

echo -e "${GREEN}📦 Cargo.toml version: $VERSION${NC}"
echo -e "${GREEN}🏷️  Tag to validate: $TAG${NC}"

# Check if tag matches Cargo.toml version
EXPECTED_TAG="v$VERSION"
if [ "$TAG" != "$EXPECTED_TAG" ]; then
    echo -e "${RED}❌ Tag mismatch!${NC}"
    echo "  Expected: $EXPECTED_TAG"
    echo "  Got: $TAG"
    exit 1
fi

echo -e "${GREEN}✓ Tag matches Cargo.toml version${NC}"

# Validate semantic versioning format
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.]+)?(\+[a-zA-Z0-9\.]+)?$ ]]; then
    echo -e "${RED}❌ Invalid semantic version format: $VERSION${NC}"
    echo "  Expected format: MAJOR.MINOR.PATCH[-prerelease][+build]"
    exit 1
fi

echo -e "${GREEN}✓ Valid semantic versioning format${NC}"

# Parse version components
IFS='.-+' read -r MAJOR MINOR PATCH PRERELEASE BUILD <<< "$VERSION"

# Check for pre-release tags (alpha, beta, rc)
if [[ -n "$PRERELEASE" ]]; then
    echo -e "${YELLOW}⚠  Pre-release version detected: $PRERELEASE${NC}"
    
    # Validate pre-release identifier
    if [[ ! "$PRERELEASE" =~ ^(alpha|beta|rc)(\.[0-9]+)?$ ]]; then
        echo -e "${YELLOW}⚠  Non-standard pre-release identifier: $PRERELEASE${NC}"
        echo "  Standard identifiers: alpha, beta, rc"
    else
        echo -e "${GREEN}✓ Valid pre-release identifier${NC}"
    fi
else
    echo -e "${GREEN}✓ Stable release version${NC}"
fi

# Validate version components are numbers
if ! [[ "$MAJOR" =~ ^[0-9]+$ ]] || ! [[ "$MINOR" =~ ^[0-9]+$ ]] || ! [[ "$PATCH" =~ ^[0-9]+$ ]]; then
    echo -e "${RED}❌ Version components must be numbers${NC}"
    echo "  Major: $MAJOR, Minor: $MINOR, Patch: $PATCH"
    exit 1
fi

echo -e "${GREEN}✓ Version components are numeric${NC}"

# Check for version 0.x.x (initial development)
if [ "$MAJOR" -eq 0 ]; then
    echo -e "${YELLOW}⚠  Initial development version (0.x.x)${NC}"
    echo "  Note: API may change in incompatible ways"
fi

# Check Android versionCode calculation
VERSION_CODE=$((MAJOR * 10000 + MINOR * 100 + PATCH))
echo -e "${GREEN}📱 Android versionCode: $VERSION_CODE${NC}"

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠  Tag already exists in git history${NC}"
    
    # Get commit hash for existing tag
    EXISTING_COMMIT=$(git rev-parse "$TAG")
    CURRENT_COMMIT=$(git rev-parse HEAD)
    
    if [ "$EXISTING_COMMIT" != "$CURRENT_COMMIT" ]; then
        echo -e "${RED}❌ Tag exists but points to different commit${NC}"
        echo "  Tag commit: $EXISTING_COMMIT"
        echo "  Current commit: $CURRENT_COMMIT"
        exit 1
    else
        echo -e "${GREEN}✓ Tag already points to current commit${NC}"
    fi
else
    echo -e "${GREEN}✓ Tag does not exist yet (ready to create)${NC}"
fi

echo ""
echo -e "${GREEN}✅ Version tag validation passed!${NC}"
echo ""
echo "Summary:"
echo "  - Version: $VERSION"
echo "  - Tag: $TAG"
echo "  - Android versionCode: $VERSION_CODE"
if [ -n "$PRERELEASE" ]; then
    echo "  - Pre-release: $PRERELEASE"
fi
echo ""
echo "Next steps:"
echo "  1. Create tag: git tag $TAG"
echo "  2. Push tag: git push origin $TAG"
echo "  3. Release workflow will trigger automatically"