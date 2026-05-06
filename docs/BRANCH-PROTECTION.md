# Branch Protection Configuration

This document describes the required branch protection settings for the `main` branch.

## Required Settings

### General Protection Rules for `main`

| Setting | Value | Purpose |
|---------|-------|---------|
| Require pull request reviews before merging | ✅ Enabled | Prevent direct pushes to main |
| Required reviewers | **2** | Must include PM + QA approval |
| Dismiss stale reviews | ✅ Enabled | Re-review required when commits change |
| Require approval from code owners | ✅ Enabled | Enforce CODEOWNERS rules |
| Require status checks to pass before merging | ✅ Enabled | CI must pass |
| Required status checks | `ci`, `test` | Must match GitHub Actions workflow names |
| Require branches to be up to date before merging | Optional | Can be enabled for stricter merge hygiene |
| Do not allow bypassing the above rules | ✅ Enabled (for admins) | Even admins must follow PR process |

### Why These Rules?

1. **2 Approvals (PM + QA)**: Ensures both project management oversight and quality assurance validation before any code lands on main
2. **CI must pass**: Guarantees all automated tests succeed before merge
3. **No direct pushes**: All changes go through PR review process

## GitHub Admin Setup (Manual)

To configure these settings in GitHub:

1. Go to **Settings → Branches → Branch protection rules → Add rule**
2. Set **Branch name pattern**: `main`
3. Check the following:
   - ✅ Require a pull request before merging
   - ✅ Require approvals (set to 2)
   - ✅ Dismiss stale reviews
   - ✅ Require approval from code owners
   - ✅ Require status checks to pass before merging
   - Add required status checks: `ci`, `test`
   - ✅ Do not allow bypassing the above rules

## GitHub CLI / Automation Script

You can also use the GitHub CLI to set this up:

```bash
# Install gh if not already installed
# brew install gh

# Set up branch protection (requires GitHub admin token with repo:admin scope)
gh api repos/{owner}/{repo}/branches/main/protection -X PUT \
  -f required_status_checks='{"strict":true,"contexts":["ci","test"]}' \
  -f enforce_admins=true \
  -f required_pull_request_reviews='{"required_approving_review_count":2,"dismiss_stale_reviews":true,"require_code_owners_reviews":true}'
```

## Verification

After setup, verify the configuration:

```bash
# Check current branch protection
gh api repos/{owner}/{repo}/branches/main/protection
```

Expected output should show:
- `required_approving_review_count: 2`
- `dismiss_stale_reviews: true`
- `require_code_owners_reviews: true`
- `strict: true` (if requiring up-to-date branches)
- `enforce_admins: true`

## CODEOWNERS Integration

The `.github/CODEOWNERS` file works with these protection rules:

- Any PR targeting `main` will automatically request review from designated CODEOWNERS
- CODEOWNERS approval satisfies one of the required 2 approvals
- CODEOWNERS must still be supplemented by second approval (PM + QA)

## Related Files

- [`.github/CODEOWNERS`](../.github/CODEOWNERS) - Code ownership rules
- [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) - CI workflow
- [`.github/PULL_REQUEST_TEMPLATE.md`](../.github/PULL_REQUEST_TEMPLATE.md) - PR template
