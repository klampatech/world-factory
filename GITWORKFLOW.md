# Source Control Workflow for Agents

## Status
**Active** — Current working policy for all engineering agents

## Overview

This document defines the source control workflow for agents working on the ProceduralWorld project. It establishes how agents interact with git, worktrees, repositories, and the Paperclip workspace system.

**Parent issue:** [WOR-444](/WOR/issues/WOR-444)  
**Related:** [WOR-215](/WOR/issues/WOR-215) (Worktree-Per-Issue Workflow)

---

## Core Principle: Repository-First, Workspace-Supplemental

The **repository** (`/WOR`) is the source of truth for all production code. The **workspace** is an isolated Paperclip instance used for issue execution. Agents must keep these synchronized:

```
┌─────────────────────────────────────────────────────────────────┐
│                     Repository (/WOR)                            │
│  • Main branch: production-ready code                           │
│  • Issue branches: feature/fix work                              │
│  • PRs: require review before merge                             │
│  • ALL code must eventually land here                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (push on completion)
┌─────────────────────────────────────────────────────────────────┐
│               Paperclip Workspace (execution)                    │
│  • Isolated Paperclip instance per issue                         │
│  • Worktree with issue-specific branch                           │
│  • Agents implement code here                                    │
│  • NOT the permanent home for code                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Git Workflow

### Branching Strategy

We use a **worktree-per-issue branching model**:

```
main ──────────────────────────────────────────────────────────────
         │                           │
         ▼                           ▼
   issue/WOR-446              issue/WOR-447
   (this workflow)            (parallel work)
         │                           │
         ▼                           ▼
   pc-WOR-446-worktree        pc-WOR-447-worktree
   (worktree)                 (worktree)
```

| Branch Pattern | Purpose | Lifetime |
|---------------|---------|----------|
| `main` | Production-ready code | Permanent |
| `issue/{id}` | Per-issue feature/fix branches | Until PR merge |
| `hotfix/{id}` | Emergency fixes | Short-lived |

### Per-Issue Workflow

1. **Checkout/Claim Issue** — Agent claims via Paperclip checkout
2. **Create Worktree** — Agent creates worktree with issue branch from `main`:
   ```bash
   npx paperclipai worktree:make {issue-id} --start-point origin/main
   ```
3. **Implement** — Agent writes code in the worktree
4. **Commit** — Agent commits with proper message and Co-Author
5. **Push Branch** — Agent pushes issue branch to origin
6. **Create PR** — Agent creates PR for review (or merges if auto-merge)
7. **Cleanup** — Agent cleans up worktree after PR merge

### When to Use Worktree vs Direct Repo Access

| Context | Method | Reason |
|---------|--------|--------|
| Implementing feature/fix assigned to you | **Worktree** | Isolation, no conflicts |
| Reading/analyzing code | **Worktree or main** | Both work |
| Hotfix on production | **Direct branch** | Speed, low risk |
| Documentation only | **Direct branch** | Simple, low risk |
| Emergency concurrent work | **Worktree** | Branch isolation |

---

## Workspace and Repository Relationship

### Lifecycle

```
Issue Claimed → Worktree Created → Code Written → Committed → Pushed → PR Merged → Worktree Cleaned
```

### Workspace (Paperclip Instance)

The workspace is **temporary execution context**, not a permanent home:

- Contains isolated Paperclip instance for the issue
- Houses working code until ready to push
- Should be cleaned up after PR merge
- **Does NOT replace the repository**

### Repository (Git)

The repository is **permanent record**:

- All code must eventually land here
- PRs are the gate for merges to `main`
- History is immutable once committed
- **Is the source of truth**

### Sync Responsibilities

| Action | Agent Responsibility |
|--------|---------------------|
| On issue claim | Create worktree from latest `main` |
| During work | Commit frequently to issue branch |
| Before PR | Rebase on latest `main` if diverged |
| On PR merge | Push to repo, then cleanup worktree |
| On PR close (no merge) | Push branch for future reference |

---

## Commit Standards

### Required: Co-Authored-By

Every commit from an agent **MUST** include:

```
Co-Authored-By: Paperclip <noreply@paperclip.ing>
```

This applies to ALL commits regardless of:
- Commit type (feat, fix, docs, refactor, etc.)
- Branch (issue, hotfix, etc.)
- Size (tiny fix or large feature)

### Commit Message Format

```
<type>(<scope>): <short summary>

[optional body]

[optional footer]
```

**Types:**
- `feat` — New feature
- `fix` — Bug fix
- `docs` — Documentation only
- `refactor` — Code change without feature/fix
- `test` — Adding or updating tests
- `chore` — Maintenance, dependencies, config

**Examples:**

```
feat(simulation): add hydrological erosion model

Implements basic water flow simulation across terrain tiles.
Uses simplified Navier-Stokes for river pathfinding.

Co-Authored-By: Paperclip <noreply@paperclip.ing>
```

```
fix(api): correct species endpoint response shape

Returns Vec<Species> instead of single Species for list operations.

Co-Authored-By: Paperclip <noreply@paperclip.ing>
```

### Avoiding Conflict

To minimize merge conflicts:

1. **Short-lived branches** — Complete issue work quickly
2. **Rebase before PR** — `git rebase origin/main`
3. **Small, focused commits** — Each commit = one logical change
4. **Communicate with other agents** — Check issue assignment before parallel work
5. **Avoid shared file edits when possible** — Separate concerns across files

---

## Pull Request Standards

### PR Requirements

| Field | Requirement |
|-------|-------------|
| Title | `feat/fix/docs(scope): short description` |
| Description | Summary of changes, motivation, testing notes |
| Reviewers | At least one human reviewer |
| Labels | `agent-work`, relevant domain labels |
| Linked Issue | `Fixes #WOR-{id}` or `Closes #WOR-{id}` |

### PR Description Template

```markdown
## Summary
Brief description of what this PR does.

## Motivation
Why is this change needed?

## Changes
- List of specific changes made
- Include file paths when relevant

## Testing
How was this tested?

## Related Issues
- Fixes #WOR-{id}

## Co-Authorship
Co-Authored-By: Paperclip <noreply@paperclip.ing>
```

### Auto-Merge vs Review Gate

| PR Type | Merge Strategy |
|---------|---------------|
| Documentation | Auto-merge after CI passes |
| Config/Low-risk | Auto-merge after CI passes |
| Feature code | Human review required |
| Production/bypass | Requires explicit approval |

---

## Agent Coordination

### Avoiding Conflicts

1. **Check before starting** — Look at issue assignments, claim yours
2. **Check branch existence** — `git branch -r | grep issue/WOR-xxx`
3. **Short branches** — Rebase daily on `main` if work spans multiple days
4. **Communicate blockers** — Update issue with conflicts found

### Conflict Resolution

| Scenario | Resolution |
|----------|------------|
| Local branch conflicts with main | Rebase: `git rebase origin/main` |
| Two agents on same issue | First claimed wins, second reassigns |
| Cross-issue file collision | Coordinate via issue comments, split work |
| Stale branch from abandon | Close PR, create new branch from fresh main |

### Coordination via Paperclip

- **Issue comments** — Use for coordination notes
- **Status updates** — `in_progress`, `blocked`, `in_review`
- **Blocked-by** — Set when waiting on another issue
- **@-mentions** — Use sparingly for urgent coordination

---

## CI/CD Expectations

### Automated Checks (GitHub Actions)

| Check | Trigger | Purpose |
|-------|---------|---------|
| `cargo build` | Every push | Compilation |
| `cargo test` | Every push | Unit tests |
| `cargo clippy` | Every push | Linting |
| `cargo fmt --check` | Every push | Style |
| `rustsec` | Weekly | Security vulnerabilities |

### Agent Expectations

1. **Run tests before pushing** — `cargo test` must pass
2. **Run linter before pushing** — `cargo clippy` should pass (or document waivers)
3. **Format code** — `cargo fmt` applied
4. **No secrets in commits** — Credentials stay in env, never in code
5. **Small PRs preferred** — Easier to review, fewer conflicts

### Release Process

```
main branch → Tag → GitHub Release → (future: deploy)
```

Agents do not directly push to `main`. All merges go through PR.

---

## Quick Reference

### Common Commands

```bash
# Create worktree for issue
npx paperclipai worktree:make WOR-446 --start-point origin/main

# Commit with required co-author
git commit -m "feat(scope): description

Co-Authored-By: Paperclip <noreply@paperclip.ing>"

# Push issue branch
git push -u origin issue/WOR-446

# Rebase on latest main
git fetch origin
git rebase origin/main

# Create PR
gh pr create --repo owner/repo --head issue/WOR-446 --title "feat(scope): description"

# Cleanup worktree after merge
npx paperclipai worktree:cleanup pc-WOR-446-worktree
```

### Ticket Linking in Comments

When referencing issues in git commits or PRs:

```markdown
- Related: #WOR-446
- Closes #WOR-446
- Fixes #WOR-446
```

In Markdown (Paperclip comments):

```markdown
- Related: [WOR-446](/WOR/issues/WOR-446)
- Parent: [WOR-444](/WOR/issues/WOR-444)
```

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| v1 | 2026-05-07 | Initial document from WOR-446 |
