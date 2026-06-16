#!/bin/bash
# Helper script to automatically parse, increment, and write back version in Cargo.toml

set -e

ACTION=$1 # "major", "minor", or "patch"

if [ -z "$ACTION" ]; then
    echo "Usage: $0 [major|minor|patch]" >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

if [ ! -f "$CARGO_TOML" ]; then
    echo "Error: Cargo.toml not found at $CARGO_TOML" >&2
    exit 1
fi

# Get current version from Cargo.toml (only matching the package version line, which starts with 'version =')
# Using awk to get the first matching line
VERSION=$(awk -F'"' '/^version =/ {print $2; exit}' "$CARGO_TOML")

if [ -z "$VERSION" ]; then
    echo "Error: Could not find version in $CARGO_TOML" >&2
    exit 1
fi

# Split version into major, minor, patch
IFS='.' read -r major minor patch <<< "$VERSION"

# Perform increment based on action
case "$ACTION" in
    major)
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    minor)
        minor=$((minor + 1))
        patch=0
        ;;
    patch)
        patch=$((patch + 1))
        ;;
    *)
        echo "Error: Invalid action '$ACTION'. Must be major, minor, or patch." >&2
        exit 1
        ;;
esac

NEW_VERSION="${major}.${minor}.${patch}"

# Replace version in Cargo.toml safely using a temp file
sed "s/^version = \"$VERSION\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML" > "$CARGO_TOML.tmp"
mv "$CARGO_TOML.tmp" "$CARGO_TOML"

# Output only the new version to stdout
echo -n "$NEW_VERSION"
