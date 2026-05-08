# WOR-215: Worktree-Per-Issue Workflow Design

## Status
**Draft v1** — Needs review and approval

## Problem Statement

We need a systematic workflow for using Paperclip's worktree system on a per-issue basis. Currently, worktrees are created ad-hoc for feature branches, but we lack a consistent process for:
- Creating worktrees tied to specific issues
- Tracking worktree→issue relationships
- Managing the lifecycle (create → work → merge → cleanup)
- Handling parallel worktrees without conflict

## Design

### Core Principle

**One issue, one worktree, full isolation.** Each worktree contains:
- A dedicated git branch (named `issue/{issue-id}`)
- An isolated Paperclip instance with its own database
- Complete separation from other worktrees and the primary instance

### Naming Convention

Worktrees follow the pattern: `pc-{issue-id}-{short-name}`

Examples:
- `pc-WOR-215-worktree-design`
- `pc-WOR-220-auth-refactor`
- `pc-WOR-225-fix-sidebar`

This ensures:
- Easy visual identification in `worktree:list`
- Machine-parseable for automation
- Human-readable for quick context switching

### Lifecycle States

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   PLANNED   │───▶│   ACTIVE    │───▶│   MERGING   │───▶│  COMPLETED  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                         │                   │
                         ▼                   ▼
                   ┌─────────────┐    ┌─────────────┐
                   │   BLOCKED   │───▶│   ACTIVE    │
                   └─────────────┘    └─────────────┘
```

| State | Description |
|-------|-------------|
| **PLANNED** | Issue assigned, worktree not yet created |
| **ACTIVE** | Worktree created, actively working |
| **BLOCKED** | Worktree exists but work paused (waiting on review, etc.) |
| **MERGING** | Work complete, merging history back |
| **COMPLETED** | Merged and cleaned up |

### Standard Workflow

#### 1. Start Work on Issue

```bash
# Automatically creates:
# - Git branch: issue/WOR-215
# - Worktree: pc-WOR-215-worktree-design
# - Isolated Paperclip instance with seeded database

npx paperclipai worktree:make WOR-215 --start-point origin/main
```

The CLI will:
1. Create the worktree directory under `~/.paperclip-worktrees/{name}`
2. Create a branch named `issue/{issue-id}` from `--start-point`
3. Initialize a Paperclip instance with seeded data
4. Print environment variables for the new instance

#### 2. Work in the Worktree

```bash
cd ~/.paperclip-worktrees/pc-WOR-215-worktree-design

# Source the worktree environment (sets API_URL, DATABASE_URL, etc.)
eval "$(npx paperclipai worktree env)"

# Start the dev server
pnpm dev

# OR for persistent server (outlives heartbeat):
tmux new-session -d -s WOR-215 'pnpm dev'
```

#### 3. Track Progress

Update the issue status in Paperclip as work progresses:
- Set to `in_progress` when starting
- Add comments with findings, decisions, blockers
- Document any significant changes in the issue thread

#### 4. Complete Work

```bash
# 1. Merge history back to the parent instance
npx paperclipai worktree:merge-history --from pc-WOR-215-worktree-design --to current --apply

# 2. Push the branch
git push -u origin issue/WOR-215

# 3. Create PR (if applicable)
gh pr create --repo paperclipai/paperclip --head issue/WOR-215 ...

# 4. Cleanup the worktree
npx paperclipai worktree:cleanup pc-WOR-215-worktree-design
```

### Handling Blocked Issues

When work is blocked (waiting on another issue, review, or external input):

```bash
# Keep the worktree but mark as blocked
npx paperclipai worktree:pause pc-WOR-215-worktree-design

# Resume later
npx paperclipai worktree:resume pc-WOR-215-worktree-design
```

### Parallel Worktrees

Multiple worktrees can run simultaneously. Each gets:
- Unique port (incremented from base, e.g., 3000, 3001, 3002)
- Isolated database
- Independent server process

To list active worktrees:
```bash
npx paperclipai worktree:list
```

Example output:
```
┌─────────────────────────┬────────┬────────┬──────────────────────────┐
│ Name                    │ Issue  │ Status │ Port │ Branch              │
├─────────────────────────┼────────┼────────┼───────┼─────────────────────┤
│ pc-WOR-215-worktree     │ WOR-215│ ACTIVE │ 3001  │ issue/WOR-215       │
│ pc-WOR-220-auth         │ WOR-220│ ACTIVE │ 3002  │ issue/WOR-220       │
│ pc-WOR-225-fix-sidebar  │ WOR-225│ BLOCKED│ 3003  │ issue/WOR-225       │
└─────────────────────────┴────────┴────────┴───────┴─────────────────────┘
```

### Error Handling

| Problem | Resolution |
|---------|------------|
| Worktree creation fails | Run `npx paperclipai doctor --repair`, then retry |
| Server won't start | Check port conflict with `lsof -i :3001`, restart if needed |
| Database out of sync | Run `worktree reseed --seed-mode incremental` |
| Worktree instance broken | Run `worktree repair <name>` |
| Cleanup fails with unmerged commits | Merge/push first, or use `--force` flag |

### Automation Opportunities

The following could be automated via CLI enhancements:

1. **`worktree:make --issue {id}`** — Auto-generate name, create branch, seed from issue
2. **`worktree:status`** — Show all worktrees with issue references and current state
3. **`worktree:checkpoint`** — Create a named save point within a worktree
4. **`worktree:auto-cleanup`** — Remove worktrees for completed/merged issues

## Implementation Plan

### Phase 1: Document & Adopt (Current)
- [x] Create this design document
- [ ] Get approval on the workflow
- [ ] Train agents on the workflow

### Phase 2: CLI Enhancements (Current Sprint)
- [x] Add `worktree:make --issue {id}` for auto-naming (via `scripts/worktree-manage.sh`)
- [x] Add `worktree:list` command with issue mapping (via `scripts/worktree-manage.sh list`)
- [x] Add `worktree:pause` and `worktree:resume` commands (via `scripts/worktree-manage.sh`)
- [ ] Add `worktree:status` command with issue mapping
- [ ] Add `worktree:checkpoint` command (partially implemented)
- [ ] Add `worktree:auto-cleanup` for completed/merged issues

### Phase 3: Integration (Future)
- [ ] Auto-create worktree when issue status changes to `in_progress`
- [ ] Auto-update issue status based on worktree state
- [ ] Integration with GitHub PR creation workflow

## Scripts Implementation

### worktree-manage.sh

Located at `scripts/worktree-manage.sh`, provides a CLI for issue-centric worktree management:

```bash
# Create worktree for an issue
./scripts/worktree-manage.sh make WOR-215 worktree-design --start-point origin/main

# List all worktrees with states
./scripts/worktree-manage.sh list

# Show status of a specific worktree
./scripts/worktree-manage.sh status pc-WOR-215-worktree-design

# Mark worktree as paused/blocked
./scripts/worktree-manage.sh pause pc-WOR-215-worktree-design

# Resume a paused worktree
./scripts/worktree-manage.sh resume pc-WOR-215-worktree-design

# Cleanup worktree
./scripts/worktree-manage.sh cleanup pc-WOR-215-worktree-design --force

# Create a named checkpoint
./scripts/worktree-manage.sh checkpoint pc-WOR-215-worktree-design "before-refactor"
```

### worktree-lib.sh

Located at `scripts/worktree-lib.sh`, provides shared library functions:
- Worktree registry management at `~/.paperclip/worktree-registry/registry.json`
- Functions for state management, path generation, and registry queries
- Works with or without `jq` (uses fallback parsing when jq unavailable)

## Open Questions

1. **Should we auto-create worktrees on issue assignment?**
   - Pros: Zero friction, always ready
   - Cons: Wastes resources on issues that never start

2. **How should we handle worktrees for issues that span multiple PRs?**
   - Option A: One worktree per PR, all linked to the issue
   - Option B: One persistent worktree with multiple commits/PRs

3. **Should worktrees auto-cleanup on PR merge?**
   - Pros: No manual cleanup
   - Cons: Lose local state before CI passes

## Related Documents

- `doc/DEVELOPING.md` — Paperclip worktree CLI reference
- Paperclip Dev Skill — Agent guidance for worktree operations

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| v1 | 2026-05-06 | Initial draft |
