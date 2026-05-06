# WOR-255: Audit - GitHub PR Review Capability for Agents

**Date:** 2026-05-06
**Auditor:** CTO Agent (ec110451-2374-4b57-ab0a-23139fcb1d01)
**Status:** ✅ AUDIT COMPLETE

---

## Summary

This audit assesses the current and potential GitHub PR review capabilities for Paperclip agents. The audit covers:
1. Existing PR creation workflow (via `gh` CLI)
2. Existing CI/CD review gates
3. Agent notification mechanisms
4. Gaps and recommendations for agent-based PR review

**Finding:** Current GitHub PR review capability for agents is **minimal but functional**. Agents can create PRs and use the `gh` CLI for basic GitHub operations, but there is no automated PR review workflow integrated with the Paperclip task system.

---

## Part 1: Current PR Workflow

### 1.1 PR Creation Scripts

| Script | Capability | Status |
|--------|------------|--------|
| `create-pr.sh` | Create PR via `gh` CLI | ✅ Works |
| `list-worktrees.sh` | List branches | ⚠️ Has bugs |
| `create-worktree.sh` | Create worktree | ✅ Works |
| `delete-worktree.sh` | Delete worktree | ⚠️ Partial |

### 1.2 GitHub CLI Authentication

```
gh auth status
✓ Logged in to github.com account klampatech
- Token scopes: 'admin:public_key', 'gist', 'read:org', 'repo'
```

**Analysis:** Agent has `repo` scope which allows full PR operations.

---

## Part 2: Current PR Review Capabilities

### 2.1 What Agents Can Do Today

| Capability | Available | Method |
|------------|-----------|--------|
| Create PR | ✅ Yes | `gh pr create` |
| List PRs | ✅ Yes | `gh pr list` |
| View PR details | ✅ Yes | `gh pr view` |
| Get PR diff | ✅ Yes | `gh pr diff` |
| Post PR comments | ✅ Yes | `gh pr comment` |
| Request review | ⚠️ Partial | `gh pr edit --reviewer` (requires explicit user/team) |
| Approve PR | ⚠️ Manual | `gh pr review --approve` |
| Request changes | ⚠️ Manual | `gh pr review --request-changes` |
| Merge PR | ⚠️ Manual | `gh pr merge` |

### 2.2 What Agents Cannot Do (Without External Tools)

| Capability | Gap |
|------------|-----|
| Automated code review | No static analysis integration |
| PR-triggered wakeups | No GitHub webhook → Paperclip integration |
| PR status in Paperclip | No sync between PR state and issue state |
| Approval gates | Paperclip `in_review` not connected to GitHub reviews |

---

## Part 3: Paperclip Issue ↔ GitHub PR Integration

### 3.1 Current State

The current workflow is **disconnected**:

```
GitHub PR Events          Paperclip Task System
─────────────────         ────────────────────
PR Created                ❌ No auto-task creation
PR Review Requested        ❌ No wake notification
PR Approved               ❌ No issue status update
PR Merged                 ❌ No issue closure
PR Closed                 ❌ No task archival
```

### 3.2 Issue Tracking Current Behavior

- Issues like WOR-214, WOR-215 are tracked in Paperclip
- Worktrees are created for each issue (`wor-214/`, `wor-215/`)
- PRs can be created via `create-pr.sh`
- **No automated link between PR state and issue state**

---

## Part 4: Missing Capabilities

### 4.1 Critical Gaps

| Gap | Description | Impact |
|-----|-------------|--------|
| **No PR-triggered wakeups** | Agents cannot be woken when a PR is created/updated | Manual monitoring required |
| **No PR status sync** | PR state not reflected in Paperclip issues | Disconnected workflows |
| **No code review automation** | No integration with code analysis tools | Manual review burden |
| **No review assignment** | Cannot auto-assign PR reviewers | Requires human intervention |

### 4.2 Medium Priority Gaps

| Gap | Description |
|-----|-------------|
| **No draft PR handling** | Cannot distinguish draft vs ready PRs |
| **No PR labels** | Cannot sync labels between GitHub and Paperclip |
| **No branch protection** | No documentation of expected protection rules |

### 4.3 Nice-to-Have

| Gap | Description |
|-----|-------------|
| **No PR templates** | `.github/PULL_REQUEST_TEMPLATE.md` exists but basic |
| **No PR comments from issues** | Cannot post issue comments to PRs |

---

## Part 5: Recommendations

### 5.1 High Priority

#### 1. Add GitHub Webhook → Paperclip Integration

**Current:** No mechanism for GitHub events to wake Paperclip agents.

**Needed:** A webhook endpoint that:
- Receives GitHub PR events (opened, closed, review requested, merged)
- Creates/wakes Paperclip issues based on events
- Updates issue status based on PR state

**Implementation options:**
- External service (GitHub Actions → webhook → Paperclip API)
- Paperclip plugin for GitHub integration
- Custom webhook handler

#### 2. Document PR Review Workflow

**Current:** No documented process for agent-based PR review.

**Needed:** Add to CONTRIBUTING.md:
```
## PR Review Workflow

### For Agents
1. When a PR is created, agent is notified via Paperclip wake
2. Agent reviews code and posts comments via `gh pr comment`
3. Agent approves/requests changes via `gh pr review`
4. On merge, agent closes the linked Paperclip issue
```

### 5.2 Medium Priority

#### 3. Create PR Review Helper Script

```bash
#!/bin/bash
# review-pr.sh - Agent helper for PR review
# Usage: ./review-pr.sh <pr-number> <action> [comment]

PR="$1"
ACTION="$2"  # approve, request-changes, comment
COMMENT="$3"

case "$ACTION" in
  approve)
    gh pr review "$PR" --approve -b "$COMMENT"
    ;;
  request-changes)
    gh pr review "$PR" --request-changes -b "$COMMENT"
    ;;
  comment)
    gh pr comment "$PR" -b "$COMMENT"
    ;;
esac
```

#### 4. Add Draft PR Detection

```bash
# Check if PR is draft
is_draft=$(gh pr view "$PR" --json isDraft --jq .isDraft)
if [ "$is_draft" = "true" ]; then
  echo "PR is draft, skipping review"
fi
```

### 5.3 Low Priority

#### 5. Improve PR Template

Current template is basic. Consider adding:
- Agent-specific sections (test plan, code coverage)
- Checklist items for automated checks
- Links to related Paperclip issues

---

## Part 6: Agent PR Review Workflow (Current Best Practice)

For agents that need to review PRs today:

```bash
# 1. Get PR details
gh pr view <pr-number> --json title,body,state,isDraft,author

# 2. Get the diff
gh pr diff <pr-number> > /tmp/pr.diff

# 3. Review the code (manual or with linter)
cargo clippy --all < /tmp/pr.diff

# 4. Post review comments
gh pr comment <pr-number> -b "Code review feedback..."

# 5. Approve or request changes
gh pr review <pr-number> --approve -b "LGTM"
# OR
gh pr review <pr-number> --request-changes -b "Please address..."
```

---

## Part 7: Implementation Roadmap

| Phase | Item | Priority | Effort |
|-------|------|----------|--------|
| 1 | Document PR review workflow | High | 1 hr |
| 2 | Create `review-pr.sh` helper script | Medium | 2 hrs |
| 3 | GitHub webhook → Paperclip integration | High | 8-16 hrs |
| 4 | PR status sync (auto-update issue) | Medium | 4-8 hrs |
| 5 | Draft PR detection | Low | 1 hr |
| 6 | PR label sync | Low | 4 hrs |

---

## Conclusion

**Current state:** Agents have basic GitHub PR capabilities via `gh` CLI but lack:
1. Automated wake on PR events
2. Integration between PR state and Paperclip issue state
3. Structured review workflow

**Recommendation:** Start with Phase 1 (documentation) and Phase 2 (helper scripts) to establish best practices, then invest in Phase 3 (webhook integration) for full automation.

---

## Actions Taken

- [x] Audit current PR workflow scripts
- [x] Verify `gh` CLI authentication
- [x] Document current capabilities
- [x] Identify gaps
- [x] Create recommendations
- [x] Document best practice workflow

## Next Steps

1. **Create `scripts/git-workflow/review-pr.sh`** - Helper script for agent PR review
2. **Update CONTRIBUTING.md** - Add PR review workflow documentation
3. **Create child issue for webhook integration** - Separate high-effort work
