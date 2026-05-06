# WOR-291: Add missing toolchain input to dtolnay/rust-toolchain in Lint jobs

**Date**: 2026-05-06  
**Status**: ✅ COMPLETED

## Summary

Fixed the missing `toolchain` input in the Lint job of the CI workflow.

## Problem

The `dtolnay/rust-toolchain@v1` action requires the `toolchain` input to be specified. The Lint job in `.github/workflows/ci.yml` was missing this input, causing CI failures.

## Solution

Added `toolchain: stable` to the Lint job's rust-toolchain setup:

```yaml
- name: Setup Rust
  uses: dtolnay/rust-toolchain@v1
  with:
    toolchain: stable
    components: clippy
```

## Verification

- PR #8 created: https://github.com/klampatech/world-factory/pull/8
- YAML syntax verified
- CI workflow structure is correct

## Next Steps

- Wait for CI to pass on PR #8
- Merge the PR
- Verify the Lint job runs successfully in CI