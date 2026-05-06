#!/bin/bash
# Delete a Git worktree after merge or cancellation
# Usage: ./delete-worktree.sh <issue-id>
# Example: ./delete-worktree.sh WOR-214

set -e

ISSUE_ID="$1"
WORKTREE_BASE="../worktrees"

if [ -z "$ISSUE_ID" ]; then
    echo "Error: Issue ID is required"
    echo "Usage: $0 <issue-id>"
    echo "Example: $0 WOR-214"
    exit 1
fi

# Convert to lowercase for directory name
WORKTREE_DIR=$(echo "$ISSUE_ID" | tr '[:upper:]' '[:lower:]')
WORKTREE_PATH="${WORKTREE_BASE}/${WORKTREE_DIR}"

if [ ! -d "$WORKTREE_PATH" ]; then
    echo "Error: Worktree not found at $WORKTREE_PATH"
    exit 1
fi

# Get the branch name from worktree
BRANCH=$(git worktree list "$WORKTREE_PATH" --porcelain | grep "^branch " | sed 's/^branch //')

if [ -z "$BRANCH" ]; then
    echo "Warning: Could not detect branch name. Assuming it matches the directory."
    BRANCH="$WORKTREE_DIR/*"
fi

echo "Deleting worktree at $WORKTREE_PATH..."
echo "  Branch: $BRANCH"

# Remove the worktree
git worktree remove "$WORKTREE_PATH"

echo "✓ Removed worktree $WORKTREE_PATH"

# Optionally delete the branch (uncomment if desired)
# echo "Deleting branch $BRANCH..."
# git branch -D "$BRANCH" 2>/dev/null || true
