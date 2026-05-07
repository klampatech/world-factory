# WOR-287: Coverage Failure - llvm-cov fix

## Problem
CI coverage job was failing with exit code 101 due to `cargo llvm-cov` failure.

## Root Cause
The `--workspace` flag was incorrectly used in both workflow files. This flag is not valid for `cargo llvm-cov` and was causing the command to fail.

## Fix Applied
Removed `--workspace` flag from `cargo llvm-cov` commands in:
- `.github/workflows/ci.yml` (coverage job)
- `.github/workflows/test.yml` (coverage job)

### Before
```yaml
run: cargo llvm-cov --workspace --lcov --output-path lcov.info
```

### After
```yaml
run: cargo llvm-cov --lcov --output-path lcov.info
```

## Commit
- Hash: `f7fd135`
- Message: "fix(WOR-287): remove invalid --workspace flag from llvm-cov commands"

## Verification
The fix can be verified by triggering a CI run - the coverage job should now complete successfully without exit code 101.
