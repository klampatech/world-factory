#!/bin/bash
# Create a pull request from a worktree branch to main
# Usage: ./create-pr.sh <issue-id> <title> [body]
# Example: ./create-pr.sh WOR-214 "Fix: Implement source control workflow"

set -e

ISSUE_ID="$1"
TITLE="$2"
BODY="${3:-}"
WORKTREE_BASE="../worktrees"

if [ -z "$ISSUE_ID" ]; then
    echo "Error: Issue ID is required"
    echo "Usage: $0 <issue-id> <title> [body]"
    exit 1
fi

if [ -z "$TITLE" ]; then
    echo "Error: PR title is required"
    echo "Usage: $0 <issue-id> <title> [body]"
    exit 1
fi

# Convert to lowercase for branch name
BRANCH_NAME=$(echo "$ISSUE_ID" | tr '[:upper:]' '[:lower:]')

# Check if we're in the worktree
if git rev-parse --git-dir > /dev/null 2>&1; then
    CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || git rev-parse --abbrev-ref HEAD)
    # If current branch matches, use it; otherwise look for matching worktree
    if [[ "$CURRENT_BRANCH" == "$BRANCH_NAME"* ]] || [ -d "$WORKTREE_BASE/$BRANCH_NAME" ]; then
        echo "Using branch: $CURRENT_BRANCH"
    fi
fi

# Get default body from branch if not provided
if [ -z "$BODY" ]; then
    BODY="## Summary

## Changes Made

## Testing

## Checklist
- [ ] Tests pass
- [ ] Code follows style guidelines
- [ ] Documentation updated"
fi

# Attempt to create PR using gh CLI if available
if command -v gh &> /dev/null; then
    echo "Creating PR with gh CLI..."
    gh pr create \
        --title "[$ISSUE_ID] $TITLE" \
        --body "$BODY" \
        --base main \
        --head "$BRANCH_NAME"
else
    echo "gh CLI not found. Please create PR manually:"
    echo ""
    echo "  Title: [$ISSUE_ID] $TITLE"
    echo "  Branch: $BRANCH_NAME"
    echo "  Base: main"
    echo "  Body:"
    echo ""
    echo "$BODY"
    echo ""
    echo "URL: https://github.com/<owner>/<repo>/pull/new/$BRANCH_NAME"
fi
