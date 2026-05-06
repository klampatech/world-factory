#!/bin/bash
# review-pr.sh - Agent helper for PR review
# Usage: ./review-pr.sh <pr-number> <action> [comment-body]
#
# Actions:
#   approve      - Approve the PR
#   request-changes - Request changes with comment
#   comment      - Post a comment only
#   view         - Show PR details
#   diff         - Show PR diff (paginated)
#   checks       - Show CI/CD check status
#
# Examples:
#   ./review-pr.sh 42 view
#   ./review-pr.sh 42 approve "LGTM, good work!"
#   ./review-pr.sh 42 request-changes "Please fix the clippy warnings"
#   ./review-pr.sh 42 comment "Note: consider using Result::ok() instead"
#   ./review-pr.sh 42 diff | less -R
#   ./review-pr.sh 42 checks

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
if [ $# -lt 2 ]; then
    echo "Usage: $0 <pr-number> <action> [comment-body]"
    echo ""
    echo "Actions:"
    echo "  approve          Approve the PR"
    echo "  request-changes  Request changes with comment"
    echo "  comment          Post a comment only"
    echo "  view             Show PR details"
    echo "  diff             Show PR diff (paginated)"
    echo "  checks           Show CI/CD check status"
    echo ""
    echo "Examples:"
    echo "  $0 42 approve \"LGTM, good work!\""
    echo "  $0 42 request-changes \"Please fix the clippy warnings\""
    echo "  $0 42 diff | less -R"
    exit 1
fi

PR_NUMBER="$1"
ACTION="$2"
COMMENT="${3:-}"

# Check for draft status first
is_draft=$(gh pr view "$PR_NUMBER" --json isDraft --jq .isDraft 2>/dev/null || echo "false")

if [ "$is_draft" = "true" ]; then
    echo -e "${YELLOW}⚠ PR #$PR_NUMBER is a draft - skipping review${NC}"
    echo "Use 'gh pr ready $PR_NUMBER' to mark it ready for review when appropriate."
    exit 0
fi

case "$ACTION" in
    approve)
        if [ -z "$COMMENT" ]; then
            echo -e "${YELLOW}⚠ No comment provided for approval${NC}"
            echo "Consider adding a brief comment explaining your approval."
            gh pr review "$PR_NUMBER" --approve -b "LGTM"
        else
            gh pr review "$PR_NUMBER" --approve -b "$COMMENT"
        fi
        echo -e "${GREEN}✓ PR #$PR_NUMBER approved${NC}"
        ;;

    request-changes)
        if [ -z "$COMMENT" ]; then
            echo -e "${RED}✗ Error: Comment required for request-changes action${NC}"
            exit 1
        fi
        gh pr review "$PR_NUMBER" --request-changes -b "$COMMENT"
        echo -e "${YELLOW}✓ Changes requested on PR #$PR_NUMBER${NC}"
        ;;

    comment)
        if [ -z "$COMMENT" ]; then
            echo -e "${RED}✗ Error: Comment body required for comment action${NC}"
            exit 1
        fi
        gh pr comment "$PR_NUMBER" -b "$COMMENT"
        echo -e "${BLUE}✓ Comment posted to PR #$PR_NUMBER${NC}"
        ;;

    view)
        echo -e "${BLUE}=== PR #$PR_NUMBER Details ===${NC}"
        gh pr view "$PR_NUMBER" --json title,state,author,createdAt,url,body,headRefName,isDraft,additions,deletions,changedFiles \
            --jq '
            [
                ("Title", .title),
                ("State", .state),
                ("Draft", .isDraft),
                ("Author", .author.login),
                ("Branch", .headRefName),
                ("Created", .createdAt),
                ("+/-/files", "\(.additions)/\(.deletions)/\(.changedFiles)"),
                ("URL", .url)
            ] | .[] | "\(.[0]): \(.[1])"
            '
        echo ""
        echo -e "${BLUE}Body:${NC}"
        gh pr view "$PR_NUMBER" --json body --jq '.body'
        ;;

    diff)
        echo -e "${BLUE}=== PR #$PR_NUMBER Diff ===${NC}"
        gh pr diff "$PR_NUMBER"
        ;;

    checks)
        echo -e "${BLUE}=== PR #$PR_NUMBER CI/CD Checks ===${NC}"
        # Get status checks from the last commit
        gh pr checks "$PR_NUMBER" --watch 2>/dev/null || gh pr checks "$PR_NUMBER"
        ;;

    *)
        echo -e "${RED}✗ Unknown action: $ACTION${NC}"
        echo "Valid actions: approve, request-changes, comment, view, diff, checks"
        exit 1
        ;;
esac