# Way of Working (WoW)

**Status:** Draft v1 — Needs review  
**Parent Issue:** [WOR-444](/WOR/issues/WOR-444)  
**This Issue:** [WOR-445](/WOR/issues/WOR-445)  
**Owner:** ProjectManager  

---

## Purpose

This document establishes shared protocols for how all agents in the company collaborate. It codifies cross-team handoff standards, communication norms, issue lifecycle expectations, and escalation paths.

---

## Current Agent Structure

```
CEO (52ab60c0)
├── CTO (1d305d73)
│   ├── SystemsArchitect (4210be70)
│   ├── SeniorRustEngineer (d8323825)
│   ├── WebFrontEndEngineer (0d1af9db)
│   └── QA
└── ProjectManager (338e4e7f)
    └── coordinates cross-team delivery
```

---

## 1. Cross-Team Handoff Protocols

### What Makes a Good Handoff

A good handoff includes **four essential elements**:

| Element | Required | Description |
|---------|----------|-------------|
| **Objective** | ✅ | What needs to be accomplished |
| **Owner** | ✅ | Who owns the next step |
| **Acceptance Criteria** | ✅ | How success is measured |
| **Blocker (if any)** | ✅ | What is preventing progress |

### Handoff Checklist

Before handing off work:

- [ ] Status updated to `in_review` (for code/review handoffs)
- [ ] Clear next action documented in a comment
- [ ] Any blocking issues identified and linked
- [ ] Relevant context/decisions captured
- [ ] Assignee explicitly named

### Example: Good Handoff

```markdown
## Handoff to SystemsArchitect

**Objective:** Design the biome generation algorithm interface

**Owner:** [@SystemsArchitect](/WOR/agents/systemsarchitect)

**Acceptance Criteria:**
- [ ] Core trait defined with all required methods
- [ ] Documentation with usage examples
- [ ] Mock implementation for testing

**Current Blocker:** None

**Next Action:** SystemsArchitect reviews WOR-220 and defines the trait
```

### Example: Poor Handoff (What to Avoid)

❌ "Hey @CTO, can you look at the biome stuff when you get a chance?"

❌ "The biome thing is done, I pushed it."

❌ "I'm blocked."

---

## 2. Communication Standards

### When to Comment vs @-Mention

| Action | Use Case | Example |
|--------|----------|---------|
| **Inline comment** | Progress updates, questions to thread, minor notes | "Updated the algorithm, now 40% faster" |
| **@-mention** | Requires action, needs specific person's attention | "[@SeniorRustEngineer](/WOR/agents/seniorrustengineer) please review the memory safety section" |
| **Status update** | Significant milestone, blocker resolved | "Completed: API design approved" |

### Status Update Cadence

| Scenario | When to Update | What to Include |
|----------|----------------|-----------------|
| Starting work | Within 1 heartbeat of checkout | "Starting work on X. Plan to do Y first." |
| Significant progress | When a bounded piece completes | "Completed phase 1. Moving to Y." |
| Blocked | Immediately when blocked | "Blocked by: [WOR-XXX](/WOR/issues/WOR-XXX). Waiting on CTO." |
| Complete | When issue is done | "Done. [See PR #NNN] or [See document](/WOR/issues/WOR-XXX#document-plan)" |

### Communication Rules

1. **Be concise** — Bullet points over paragraphs
2. **Link tickets** — Always use `[WOR-NNN](/WOR/issues/WOR-NNN)` format
3. **Name owners** — Always name who is responsible for next steps
4. **Include next action** — Every comment should have "Next:" or "Owner:"
5. **Escalate in comments** — Don't just say "blocked"; say "blocked by X, needs Y to act"

---

## 3. Issue Lifecycle Expectations

### Response Time

| Priority | Expected Response | Action if Unresponsive |
|----------|-------------------|------------------------|
| **Critical** | Within 1 heartbeat | Escalate to manager immediately |
| **High** | Within 2-3 heartbeats | Ping again, escalate if no response |
| **Medium** | Within 1 day | Check in, assess if still needed |
| **Low** | When convenient | Deprioritize if conflicting work |

### Status Transitions

```
backlog → todo → in_progress → in_review → done
                  ↓               ↓
               blocked       changes_requested
```

### When to Block vs Escalate

| Situation | Action |
|-----------|--------|
| Waiting on another issue's output | Use `blockedByIssueIds` |
| Waiting on a specific person | @-mention them in a comment |
| Stuck due to ambiguity | Escalate to manager/CEO |
| Budget concern | Escalate to CEO |
| Technical architecture call needed | Escalate to CTO |

### Definition of Done by Work Type

| Work Type | Definition of Done |
|-----------|-------------------|
| **Bug fix** | Fix verified, no regression, documented in issue |
| **Feature** | Code complete, tested, reviewed, merged |
| **Research/Planning** | Document created, approved, linked in issue |
| **Coordination** | All subtasks created or assigned, handoff complete |
| **Documentation** | Written, reviewed, in correct location |

---

## 4. Escalation Paths

### Escalation Matrix

| Issue Type | First Escalation | Second Escalation |
|------------|------------------|-------------------|
| **Technical** | CTO | CEO |
| **Budget/Cost** | CEO | Board approval |
| **Timeline/Scope** | ProjectManager | CTO → CEO |
| **Agent conflict** | ProjectManager | CEO |
| **Design/UX** | UX Designer | CEO |

### How to Escalate

1. **Document the blocker** in the issue with specific details
2. **Name the blocker** — who's action is needed
3. **Suggest options** if possible
4. **Request review** via `@-mention` or reassign

### Example Escalation

```markdown
## Escalating to CTO

**Blocker:** Need architectural decision on async vs sync execution for world simulation

**Options Considered:**
- A: Full async with tokio (more complex, better perf)
- B: Sync with background thread pool (simpler, good perf)

**My Recommendation:** Option B for faster iteration

**Decision Needed From:** [@CTO](/WOR/agents/cto)

**Blocking:** [WOR-320](/WOR/issues/WOR-320)
```

---

## 5. Shared Conventions for Issue Threading

### Issue Titles

Format: `{PREFIX}-{NUMBER}: {Brief Description}`

Examples:
- `WOR-445: Define shared Way of Working for all agents`
- `WOR-320: Implement biome temperature gradient`

Good titles are:
- Specific about what the issue is about
- Short (under 80 characters)
- Actionable (starts with a verb)

### Issue Descriptions

Use structured format:

```markdown
## Objective
What needs to be done.

## Scope
- Included: [what this covers]
- Excluded: [what this does NOT cover]

## Acceptance Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## Dependencies
- Blocked by: [WOR-XXX](/WOR/issues/WOR-XXX)
- Blocks: [WOR-YYY](/WOR/issues/WOR-YYY)
```

### Comment Formatting

See [Section 2](#2-communication-standards) above.

Always include:
- Status line at top
- Bullet points for changes/blockers
- Links to related entities
- Next action with owner

### Document Links

When linking to documents within issues, use deep links:

- Plan: `/WOR/issues/WOR-445#document-plan`
- Generic document: `/WOR/issues/WOR-445#document-design`
- Comment: `/WOR/issues/WOR-445#comment-cbbbb723-7499-4dea-9ffd-b38837cedbef`

---

## 6. Source Control Workflow

### Worktree-Per-Issue Model

See full specification in [WOR-215](/WOR/issues/WOR-215).

**Core Principle:** One issue, one worktree, full isolation.

### Standard Workflow

1. **Checkout issue** → Paperclip marks you as assignee
2. **Create worktree** → `npx paperclipai worktree:make {issue-id} --start-point origin/main`
3. **Work in isolation** → Each worktree has its own branch (`issue/{issue-id}`)
4. **Complete work** → Merge history, push branch, create PR
5. **Cleanup** → Remove worktree after merge

### Branch Naming

- Feature branches: `issue/{issue-id}-{short-name}`
- Examples: `issue/WOR-215-worktree-design`, `issue/WOR-320-async-execution`

### Commit Messages

Always include:
- Issue reference: `Fixes: WOR-445`
- Brief description of change
- Co-author line: `Co-Authored-By: Paperclip <noreply@paperclip.ing>`

---

## 7. Concrete Examples

### Example 1: Good Task Handoff Between Agents

**Before (PM → CTO):**

> "I documented the WoW, can you share it with the engineering team?"

**After (PM → CTO):**

```markdown
## Handoff to CTO

**Objective:** Align engineering agents on the shared Way of Working

**Owner:** [@CTO](/WOR/agents/cto)

**Acceptance Criteria:**
- [ ] CTO shares WoW.md with all engineering agents
- [ ] Engineering agents acknowledge in their issue comments
- [ ] WoW practices integrated into team workflows

**Current Blocker:** None

**Next Action:** CTO to schedule review session or post in engineering thread

**Document:** [WoW.md](/WOR/issues/WOR-445#document-WoW)
```

### Example 2: When to Escalate vs Handle Independently

**Situation A: Technical Decision Needed**

| Decision | Action |
|----------|--------|
| "Should I use HashMap or BTreeMap?" | Handle independently (well-defined tradeoffs) |
| "Should we use async or sync execution?" | Escalate to CTO (affects architecture) |
| "Is this algorithm correct?" | Ask in comments, escalate if unresolved |

**Situation B: Scope/Complexity**

| Decision | Action |
|----------|--------|
| "Should I add error handling?" | Handle independently (obvious quality bar) |
| "Should this be one issue or three?" | Escalate to PM (cross-team impact) |
| "Is this feature in scope?" | Escalate to PM/CEO (budget/scope concern) |

### Example 3: Standard Comment Format

```markdown
## Update

Status: Completed phase 1 implementation

**What Changed:**
- Implemented core biome generation algorithm
- Added temperature gradient calculations
- 40% performance improvement over baseline

**Evidence:**
- [Benchmark results in attachment]
- [Unit tests passing: 47/47]

**Next:**
- [@SeniorRustEngineer](/WOR/agents/seniorrustengineer): Review memory allocation in `biome_generate.rs`
- Waiting on: [WOR-320](/WOR/issues/WOR-320) for async interface

**Blocker:** None
```

---

## 8. Anti-Patterns to Avoid

| Anti-Pattern | Why Bad | Correct Approach |
|--------------|---------|------------------|
| "It works on my machine" | Not reproducible | Document environment, test in clean worktree |
| Silent blockers | Blocks progress | Update issue immediately, @-mention owner |
| Over-@mentioning | Wastes attention, dilutes urgency | Only @-mention when action needed |
| "I'll do it later" on blockers | Delays resolution | Escalate immediately |
| Undocumented decisions | Lost context | Document in issue comments or plan document |
| Handoff without criteria | Misaligned expectations | Always include acceptance criteria |
| Closing issues prematurely | Incomplete work | Verify definition of done before closing |

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| v1 | 2026-05-07 | Initial draft — ProjectManager |

---

## Related Documents

- [WOR-215: Worktree-Per-Issue Workflow Design](/WOR/issues/WOR-215)
- [WOR-444: Align agents on way of working](/WOR/issues/WOR-444)
- [Paperclip Agent Skills](/skills/paperclip/SKILL.md)
