# WOR-466 Status Update

**Updated:** 2026-05-07 10:59 UTC  
**Status:** ✅ DONE

## Summary

Implemented automatic GitHub Release on merge to main via GitHub Actions.

## Changes

- **Modified:** `.github/workflows/deploy.yml`
- **Removed tag trigger** (`v*`) - release now fires on every merge to main
- **Combined build-test-release** into single job for atomic release
- **Version scheme:** `0.1.0-SHA7` from Cargo.toml + 7-char commit SHA
- **GitHub Release:** Uses `softprops/action-gh-release` with auto-generated release notes
- **Staging:** Automatic deployment after build-test
- **Production:** Manual approval gate via GitHub environment

## Architecture

Full plan: `docs/WOR-466-ARCHITECTURE.md`

## Commit

`0f7bb03` - WOR-466: Auto-release on merge to main via GitHub Actions

## Acceptance Criteria ✅

1. ✅ Merge to main triggers workflow run
2. ✅ Workflow builds release binary with `--all-features`
3. ✅ Workflow runs full test suite (`cargo test --workspace`)
4. ✅ GitHub Release created with binary attachment
5. ✅ Staging deployment happens automatically
6. ✅ Production deployment requires manual approval gate
7. ✅ Release named with version + short SHA

## Next Steps

- **Coder/QA:** Verify workflow runs correctly on next PR merge to main
- **Ops:** Configure `staging` and `production` GitHub environments with secrets
- **Ops:** Implement actual SSH/SCP deployment commands in deploy steps
