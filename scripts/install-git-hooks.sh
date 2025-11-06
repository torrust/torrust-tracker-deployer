#!/usr/bin/env bash
#
# Install Git hooks for this repository.
# This script creates a symbolic link from .git/hooks/pre-commit to scripts/pre-commit.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOK_SOURCE="$REPO_ROOT/scripts/pre-commit.sh"
HOOK_TARGET="$REPO_ROOT/.git/hooks/pre-commit"

echo "Installing Git hooks..."

# Check if .git directory exists
if [ ! -d "$REPO_ROOT/.git" ]; then
    echo "Error: .git directory not found. Are you in a Git repository?"
    exit 1
fi

# Check if source script exists
if [ ! -f "$HOOK_SOURCE" ]; then
    echo "Error: Pre-commit script not found at $HOOK_SOURCE"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$REPO_ROOT/.git/hooks"

# Remove existing hook if present
if [ -e "$HOOK_TARGET" ]; then
    echo "Removing existing pre-commit hook..."
    rm "$HOOK_TARGET"
fi

# Create symbolic link to pre-commit script
echo "Creating symbolic link to scripts/pre-commit.sh..."
ln -s "$HOOK_SOURCE" "$HOOK_TARGET"

# Verify the hook is executable
if [ ! -x "$HOOK_SOURCE" ]; then
    echo "Warning: Making scripts/pre-commit.sh executable..."
    chmod +x "$HOOK_SOURCE"
fi

echo ""
echo "✓ Git hooks installed successfully"
echo ""
echo "The pre-commit hook is now linked to ./scripts/pre-commit.sh"
echo "Any changes to scripts/pre-commit.sh will automatically affect the Git hook."
