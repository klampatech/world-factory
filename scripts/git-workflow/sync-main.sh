#!/bin/bash
# Sync worktree with latest changes from main
# Usage: ./sync-main.sh [worktree-name]
# If no worktree name provided, syncs current directory

set -e

# Determine which worktree to sync
if [ -n "$1" ]; then
    WORKTREE_NAME="$1"
else
    # Get worktree path from PWD
    WORKTREE_NAME=$(basename "$(pwd)")
fi

WORKTREE_BASE="../worktrees"
WORKTREE_PATH="${WORKTREE_BASE}/${WORKTREE_NAME}"

# Verify we're in or can find a worktree
if [ ! -d "$WORKTREE_PATH" ]; then
    # Try current directory
    if [ -d ".git" ]; then
        WORKTREE_PATH="$(pwd)"
        WORKTREE_NAME=$(basename "$WORKTREE_PATH")
        echo "Syncing current worktree: $WORKTREE_PATH"
    else
        echo "Error: Worktree not found at $WORKTREE_PATH"
        echo "Usage: $0 [worktree-name]"
        exit 1
    fi
fi

cd "$WORKTREE_PATH"

echo "Syncing worktree: $WORKTREE_NAME"

# Stash any local changes
if git status --porcelain | grep -q .; then
    echo "Stashing local changes..."
    git stash push -m "Auto-stash before sync"
fi

# Fetch latest from origin
echo "Fetching latest from origin..."
git fetch origin

# Check if we're on the expected branch
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || git rev-parse --abbrev-ref HEAD)
echo "Current branch: $CURRENT_BRANCH"

# Rebase onto latest main
echo "Rebasing onto latest main..."
git rebase origin/main

echo "✓ Sync complete for $WORKTREE_NAME"
echo ""
echo "If you stashed changes, apply them with:"
echo "  cd $WORKTREE_PATH && git stash pop"