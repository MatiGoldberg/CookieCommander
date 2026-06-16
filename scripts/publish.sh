#!/bin/bash
# CookieCommander Publish Script
# Runs a clean release build and publishes the portable binary executable with the version

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default behavior
INCREMENT_PART="minor"

show_help() {
    echo -e "${BLUE}CookieCommander Publish Script${NC}"
    echo ""
    echo "Usage: ./scripts/publish.sh [options]"
    echo ""
    echo "Options:"
    echo "  --major, -m             Increment the Major version (and zero the Minor and Patch)"
    echo "  --help, -h              Show this help message"
    echo ""
    echo "Example:"
    echo "  ./scripts/publish.sh"
    echo "  ./scripts/publish.sh --major"
}

# Parse options
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --major|-m)
            INCREMENT_PART="major"
            shift
            ;;
        --help|-h|help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# Step 1: Increment the version
echo -e "${GREEN}▶ Incrementing version (${INCREMENT_PART})...${NC}"
VERSION=$("$SCRIPT_DIR/increment_version.sh" "$INCREMENT_PART")
echo -e "${BLUE}ℹ️  New release version: $VERSION${NC}"

# Step 2: Clean and build in release mode
echo -e "${GREEN}▶ Running clean release build...${NC}"
cargo clean
cargo build --release

# Verify the release binary exists
BINARY_PATH="$PROJECT_ROOT/target/release/cookie_commander"
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Release binary not found at $BINARY_PATH${NC}" >&2
    exit 1
fi

# Step 3: Publish artifact
DIST_DIR="$PROJECT_ROOT/dist"
mkdir -p "$DIST_DIR"

PUBLISH_PATH="$DIST_DIR/cookie_commander-v$VERSION"
echo -e "${GREEN}▶ Publishing artifact to $PUBLISH_PATH...${NC}"
cp "$BINARY_PATH" "$PUBLISH_PATH"
chmod +x "$PUBLISH_PATH"

SIZE_BYTES=$(wc -c < "$PUBLISH_PATH" | tr -d ' ')
SIZE_KB=$((SIZE_BYTES / 1024))

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}Publish Successful!${NC}"
echo -e "${BLUE}  Published: $PUBLISH_PATH${NC}"
echo -e "${BLUE}  Size:      ${SIZE_KB} KB${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
