#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo -e "${GREEN}Dotfiles Release Helper${NC}"
echo "Current version: ${YELLOW}v${CURRENT_VERSION}${NC}"
echo ""

# Prompt for new version
read -p "Enter new version (e.g., 0.2.0): " NEW_VERSION

if [ -z "$NEW_VERSION" ]; then
    echo -e "${RED}Error: Version cannot be empty${NC}"
    exit 1
fi

# Validate version format
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Invalid version format. Use X.Y.Z (e.g., 0.2.0)${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Release Plan:${NC}"
echo "  1. Update Cargo.toml version to ${NEW_VERSION}"
echo "  2. Update Cargo.lock"
echo "  3. Commit changes"
echo "  4. Create git tag v${NEW_VERSION}"
echo "  5. Push tag to GitHub (triggers release workflow)"
echo ""

read -p "Continue? [y/N] " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${RED}Aborted${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Step 1:${NC} Updating Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml && rm Cargo.toml.bak

echo -e "${GREEN}Step 2:${NC} Updating Cargo.lock..."
cargo build --release

echo -e "${GREEN}Step 3:${NC} Creating commit..."
git add Cargo.toml Cargo.lock
git commit -m "Bump version to v${NEW_VERSION}"

echo -e "${GREEN}Step 4:${NC} Creating tag v${NEW_VERSION}..."
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

echo ""
echo -e "${YELLOW}Ready to push!${NC}"
echo "This will:"
echo "  - Push the commit to GitHub"
echo "  - Push the tag to GitHub"
echo "  - Trigger the release workflow"
echo "  - Build binaries for macOS (Intel + ARM) and Linux"
echo "  - Create GitHub release with binaries"
echo ""

read -p "Push now? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${GREEN}Step 5:${NC} Pushing to GitHub..."
    git push origin main
    git push origin "v${NEW_VERSION}"

    echo ""
    echo -e "${GREEN}✅ Release initiated!${NC}"
    echo ""
    echo "Monitor progress at:"
    echo "  https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
    echo ""
    echo "Release will be available at:"
    echo "  https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/releases/tag/v${NEW_VERSION}"
else
    echo ""
    echo -e "${YELLOW}Not pushed.${NC} You can push manually later:"
    echo "  git push origin main"
    echo "  git push origin v${NEW_VERSION}"
fi
