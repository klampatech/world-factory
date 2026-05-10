# Priority Fix 2: Integration Test Broken Import

> **Issue:** `tests/export_endpoint_test.rs` fails to compile
> **Severity:** MEDIUM - blocks `cargo test` from running integration tests
> **Error:** `world_factory::storage::{StorageManager, StorageConfig}` do not exist at that import path

---

## Problem Description

The file `tests/export_endpoint_test.rs` imports:
```rust
use world_factory::storage::{StorageManager, StorageConfig};
```

But `StorageManager` and `StorageConfig` are either:
1. Named differently in the actual storage module
2. Located in a different module path
3. Not public exports of the `world_factory` crate

---

## Investigation Steps

1. Find the actual storage types in the codebase:
   ```bash
   grep -rn "struct StorageManager\|struct StorageConfig\|pub struct Storage" src/
   ```

2. Check what's actually exported from `src/storage.rs` and `src/lib.rs`

3. Check `tests/export_endpoint_test.rs` to understand what functionality it tests

4. Verify the correct import path or create the missing types

---

## Required Fix

Either:

**Option A (Preferred):** Fix the import path to match the actual type names/paths in `src/storage.rs`

**Option B:** If the storage types don't exist, implement them:
- `StorageManager` — manages world persistence to `.wfw` tarballs
- `StorageConfig` — configuration for storage paths, max worlds, etc.

---

## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|--------------|
| 1 | `cargo test --tests` compiles without errors | No "cannot find type" errors |
| 2 | `tests/export_endpoint_test` runs (may pass or fail, but must compile) | `cargo test --test export_endpoint_test` |
| 3 | If types were missing, they are implemented with basic functionality | Code review |
| 4 | No other tests broken by the change | `cargo test --lib` still passes |

---

## Files Reference

- `tests/export_endpoint_test.rs` — the broken test file
- `src/storage.rs` — the actual storage implementation
- `src/lib.rs` — crate public exports

---

## Notes

- This is a **compilation fix**, not a logic fix — the test logic itself is presumably correct
- If `StorageManager`/`StorageConfig` exist but are named differently, just fix the import path
- Do not delete the test file — it tests important functionality