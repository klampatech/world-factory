# Contributing to World Factory

## Development Workflow

### Issue Assignment & Worktrees

1. **Claim an issue**: Before starting work, ensure you're assigned to the issue in the project tracker.
2. **Create a worktree**: Each issue gets its own Git worktree to prevent cross-contamination.
   ```bash
   # Create worktree at ../worktrees/wor-{number}
   cd /home/kyle/projects/world-generator
   git worktree add ../worktrees/wor-XXX -b wor-XXX/description-slug
   ```
   - Worktrees are created as siblings of the main repository directory
   - Branch name format: `wor-{number}/{description-slug}`

### Branch Naming Convention

```
wor-{issue_number}/{type}-{short-description}
```

Examples:
- `wor-201/fix-map-tiles`
- `wor-215/feat-worktree-workflow`
- `wor-216/docs-pr-review-process`

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting, no code change
- `refactor`: Code restructuring
- `test`: Adding/updating tests
- `chore`: Build, tooling, dependencies

**Examples:**
```bash
feat(map-view): correct vertex coordinate scale in calculateTransform
fix(api): handle nil world ID in GET /api/worlds/:id/map
docs(readme): add installation prerequisites section
test(voronoi): add edge case for single-cell generation
```

---

## Pull Request Process

### Creating a PR

1. **Push your branch** from the worktree:
   ```bash
   cd ../worktrees/wor-XXX
   git push -u origin wor-XXX/description-slug
   ```

2. **PR Title Format**:
   ```
   [{ISSUE-ID}] {Type}: {Short description}
   ```
   Examples:
   - `[WOR-201] Fix: Map tiles rendering as scattered squares`
   - `[WOR-215] Feat: Worktree-per-issue workflow implementation`

3. **PR Description** must include:
   - **Link to issue**: `[WOR-XXX](/WOR/issues/WOR-XXX)`
   - **Summary**: What changed and why
   - **Testing**: How to verify the change works
   - **Screenshots** (if UI changes): Before/after comparisons

4. **PR Body Template**:
   ```markdown
   ## Summary
   <!-- What does this PR do? -->

   ## Changes
   <!-- List the specific changes made -->

   ## Testing
   <!-- How was this tested? -->

   - [ ] Unit tests added/updated
   - [ ] Manual smoke test performed
   - [ ] No regressions in existing tests

   ## Related Issues
   Closes [WOR-XXX](/WOR/issues/WOR-XXX)
   ```

### PR Review Requirements

All PRs require **two approvals** before merge:

| Reviewer | Focus Area |
|----------|------------|
| **PM (Project Manager)** | Correctness, completeness, alignment with goals |
| **QA** | Test coverage, edge cases, smoke tests |

#### Review Checklist

**For Author (before requesting review):**
- [ ] Tests pass locally (`cargo test` and/or `npm test`)
- [ ] Code follows project style guidelines
- [ ] PR description is complete
- [ ] Self-reviewed the diff

**For Reviewer:**
- [ ] Logic is correct and handles edge cases
- [ ] Tests are adequate
- [ ] No obvious bugs or security issues
- [ ] Documentation updated if needed
- [ ] Performance implications considered

### Merge Criteria

A PR can be merged when:
- All CI checks pass (lint, tests, build)
- Two approvals received (PM + QA)
- No unresolved comments or change requests
- Branch is up-to-date with `main`

### Merge Strategy

**Squash and Merge** is the default merge strategy:
- Combines all commits into a single commit on `main`
- Commit message follows PR title format
- Keeps `main` history clean

### Branch Cleanup

After PR is merged:
1. Delete the remote branch:
   ```bash
   git push origin --delete wor-XXX/description-slug
   ```
2. Remove the worktree:
   ```bash
   cd /home/kyle/projects/world-generator
   git worktree remove ../worktrees/wor-XXX
   ```
3. Update issue status to `done` in the project tracker

---

## Code Quality Standards

### Rust Code (Backend)
- Run `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- All public functions must have doc comments

### TypeScript/Playwright (Tests)
- Follow existing patterns in test files
- Use descriptive test names: `describe("Map generation")` not `describe("tests")`
- Include inline comments for complex test logic

### General
- No TODO comments left in code (create issues instead)
- No commented-out code in PRs
- Keep PRs focused (one concern per PR)

---

## Project Structure

```
world-factory/
├── src/              # Rust source code
├── tests/            # Integration tests
├── web/              # Frontend code
├── e2e/              # End-to-end Playwright tests
├── docs/             # API documentation
├── scripts/          # Utility scripts
└── dist/             # Build outputs
```

---

## Getting Help

- **Issues**: Create a new issue in the project tracker
- **Questions**: Ask in the issue comments with tag `@PM`
- **Urgent**: Tag the relevant agent directly in comments