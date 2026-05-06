#!/bin/bash
# Create a Git worktree for a new issue branch
# Usage: ./create-worktree.sh <issue-id> <description>
# Example: ./create-worktree.sh WOR-214 implement-source-control-workflow

set -e

ISSUE_ID="$1"
DESCRIPTION="$2"
WORKTREE_BASE="../worktrees"

if [ -z "$ISSUE_ID" ]; then
    echo "Error: Issue ID is required"
    echo "Usage: $0 <issue-id> <description>"
    echo "Example: $0 WOR-214 implement-source-control-workflow"
    exit 1
fi

if [ -z "$DESCRIPTION" ]; then
    echo "Error: Description is required"
    echo "Usage: $0 <issue-id> <description>"
    echo "Example: $0 WOR-214 implement-source-control-workflow"
    exit 1
fi

# Convert issue ID to lowercase for worktree directory
WORKTREE_DIR=$(echo "$ISSUE_ID" | tr '[:upper:]' '[:lower:]')

# Create branch name: wor-XXX/description-slug
SLUG=$(echo "$DESCRIPTION" | sed 's/[^a-zA-Z0-9]/-/g' | sed 's/\-+/-/g' | sed 's/^-//' | sed 's/-$//' | tr '[:upper:]' '[:lower:]')
BRANCH_NAME="${WORKTREE_DIR}/${SLUG}"

# Full path to worktree
WORKTREE_PATH="${WORKTREE_BASE}/${WORKTREE_DIR}"

# Check if worktree already exists
if [ -d "$WORKTREE_PATH" ]; then
    echo "Error: Worktree already exists at $WORKTREE_PATH"
    exit 1
fi

# Create parent directory if needed
mkdir -p "$WORKTREE_BASE"

# Create the worktree with new branch
git worktree add "$WORKTREE_PATH" -b "$BRANCH_NAME"

echo "✓ Created worktree at $WORKTREE_PATH"
echo "  Branch: $BRANCH_NAME"
echo "  Issue: $ISSUE_ID"

# Initialize README in worktree if it doesn't exist
if [ ! -f "$WORKTREE_PATH/README.md" ]; then
    echo "# $ISSUE_ID: $DESCRIPTION" > "$WORKTREE_PATH/README.md"
    echo "" >> "$WORKTREE_PATH/README.md"
    echo "Status: In Progress" >> "$WORKTREE_PATH/README.md"
    echo "Created: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$WORKTREE_PATH/README.md"
fi
