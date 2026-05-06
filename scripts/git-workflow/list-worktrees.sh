#!/bin/bash
# List all Git worktrees with their status
# Usage: ./list-worktrees.sh

echo "=== Git Worktrees ==="
echo ""

git worktree list

echo ""
echo "=== Pending PRs ==="
# List branches that likely have open PRs (those with wor- prefix)
git branch -r 2>/dev/null | grep -E 'origin/wor-[0-9]+' | while read -r branch; do
    echo "  $branch"
done

echo ""
echo "=== Recent Commits on Worktrees ==="
git log --oneline -10 --all --source --worktree 2>/dev/null || echo "  (no worktrees or no recent commits)"
