# WOR-222: CI Coverage Gate (80% threshold enforcement)

## Status: COMPLETE ✓

## Changes Made

Added a dedicated `coverage` job to `.github/workflows/test.yml`:

### Job: `coverage` (Job 2b)
- Uses `cargo-llvm-cov` for coverage generation
- Runs on every push/PR to `main` or `develop`
- Generates LCOV coverage report
- Uploads to Codecov via `codecov/codecov-action@v4`
- **Enforces 80% coverage threshold** with exit code 1 if below

### Workflow Integration
- Added `coverage` job to `notify-on-failure` dependencies
- Removed redundant coverage generation from `nightly-full-pipeline` (now handled by dedicated job)

### Threshold Enforcement Logic
```bash
COVERAGE=$(cargo llvm-cov report --text | grep "TOTAL" | awk '{print $NF}' | sed 's/%//')
if (( $(echo "$COVERAGE < 80" | bc -l) )); then
  echo "ERROR: Coverage ${COVERAGE}% is below 80% threshold!"
  exit 1
fi
```

### Requirements
- Requires `CODECOV_TOKEN` secret in repository settings
- Uses `bc` package (standard in Ubuntu runners)

## Next Steps
- Add `CODECOV_TOKEN` to repository secrets
- Consider running coverage only on changed files for faster PR feedback