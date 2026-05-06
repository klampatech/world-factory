#!/bin/bash
# Validate branch name follows project conventions
# Usage: ./validate-branch.sh [branch-name]
# If no branch name provided, uses current branch

set -e

# Determine which branch to validate
if [ -n "$1" ]; then
    BRANCH_NAME="$1"
else
    BRANCH_NAME=$(git branch --show-current 2>/dev/null || git rev-parse --abbrev-ref HEAD)
fi

if [ -z "$BRANCH_NAME" ]; then
    echo "Error: Could not determine branch name"
    exit 1
fi

echo "Validating branch: $BRANCH_NAME"

ERRORS=0

# Check for issue-based branches: wor-XXX/type-slug
if [[ "$BRANCH_NAME" =~ ^wor-[0-9]+/[a-z]+-.+$ ]]; then
    # Extract the type (first segment after wor-XXX/)
    REST="${BRANCH_NAME#*/}"  # Everything after first /
    TYPE="${REST%%-*}"        # First segment before first -
    
    VALID_TYPES=("feat" "fix" "docs" "style" "refactor" "test" "chore")
    VALID=false
    
    for valid_type in "${VALID_TYPES[@]}"; do
        if [ "$TYPE" = "$valid_type" ]; then
            VALID=true
            break
        fi
    done
    
    if [ "$VALID" = false ]; then
        echo "✗ Invalid type suffix: $TYPE"
        echo "  Valid types: feat, fix, docs, style, refactor, test, chore"
        ERRORS=$((ERRORS + 1))
    fi
    
    # Check slug format (everything after type-)
    SLUG="${REST#${TYPE}-}"
    if ! [[ "$SLUG" =~ ^[a-z0-9-]+$ ]]; then
        echo "✗ Invalid slug format: $SLUG"
        echo "  Use only lowercase letters, numbers, and hyphens"
        ERRORS=$((ERRORS + 1))
    fi
    
    echo "✓ Issue branch format valid"

# Check for wor-XXX without type (allowed but warn)
elif [[ "$BRANCH_NAME" =~ ^wor-[0-9]+$ ]]; then
    echo "⚠ Warning: Branch name missing type prefix"
    echo "  Expected: wor-XXX/type-slug (e.g., wor-254/fix-pr-workflow)"
    ERRORS=$((ERRORS + 1))

# Check for other known patterns
elif [[ "$BRANCH_NAME" =~ ^hotfix-[0-9]+-[a-z0-9-]+$ ]]; then
    echo "✓ Hotfix branch format valid"
elif [[ "$BRANCH_NAME" =~ ^release-[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "✓ Release branch format valid"
elif [ "$BRANCH_NAME" = "main" ] || [ "$BRANCH_NAME" = "develop" ]; then
    echo "✓ Protected branch"
else
    echo "✗ Unknown branch format: $BRANCH_NAME"
    echo "  Expected: wor-XXX/type-description"
    echo "  Example:  wor-254/fix-pr-workflow"
    ERRORS=$((ERRORS + 1))
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "✓ Branch name is valid"
    exit 0
else
    echo "✗ Branch name has $ERRORS error(s)"
    exit 1
fi