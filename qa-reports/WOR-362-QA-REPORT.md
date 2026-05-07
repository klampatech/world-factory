# WOR-362 QA Report: World Retrieve by ID Returns 404

## Bug Status: CONFIRMED — ROOT CAUSE IDENTIFIED

## Test Date: 2026-05-07

## Issue
GET /api/v1/worlds/:id returns 404 for worlds that exist in the world list.

## Steps to Reproduce
1. Create world: `POST /api/v1/worlds` with `{"name": "Test"}`
2. Response: `{"id": "world:da00ca49-a805-454e-9031-acb2696dce05", ...}`
3. List worlds: `GET /api/v1/worlds` → Shows the world
4. Retrieve world: `GET /api/v1/worlds/world:da00ca49-a805-454e-9031-acb2696dce05`
5. Expected: 200 with world data
6. **Actual: 404 NOT_FOUND**

## Evidence

### API Response
```bash
$ curl -s -X POST http://localhost:8080/api/v1/worlds \
  -H 'Content-Type: application/json' \
  -d '{"name": "QA Test World WOR-362"}'
{"success":true,"data":{"id":"world:da00ca49-a805-454e-9031-acb2696dce05","name":"QA Test World WOR-362",...}}

$ curl -s http://localhost:8080/api/v1/worlds/world:da00ca49-a805-454e-9031-acb2696dce05
{"code":"NOT_FOUND","error":"World 'da00ca49-a805-454e-9031-acb2696dce05' not found","success":false}
```

### Container Investigation
```bash
$ docker exec <container> sh -c 'find /root -name "*.wfw"'
/root/.local/share/world-factory/generated/world:da00ca49-a805-454e-9031-acb2696dce05/world.wfw

$ docker exec <container> sh -c 'cat /root/.local/share/world-factory/generated/world:da00ca49-a805-454e-9031-acb2696dce05/metadata.json'
{
  "id": "world:da00ca49-a805-454e-9031-acb2696dce05",
  "name": "QA Test World WOR-362",
  "status": "Generating"
}
```

**The world file EXISTS at the correct location.** The create flow works correctly.

## Root Cause

**File:** `src/storage.rs` line 165

The `default_base_dir()` function uses the wrong directory name:

```rust
// Line 165 - WRONG:
.join("WorldFactory")

// Should be:
.join("world-factory")
```

The code saves worlds to `~/.local/share/world-factory/` but looks for them at `~/.local/share/WorldFactory/` (capital letters, no hyphen).

### Code Path Analysis

1. **Create flow** (`src/api/v1/worlds.rs:300`):
   - `create_world()` saves package via `state.storage.world_package_path(&world.id)`
   - Uses `StorageManager::world_package_path()` which calls `world_dir()` → `generated_dir()` → `base_dir()`
   - **Saves to:** `/root/.local/share/world-factory/generated/world:xxx/world.wfw`

2. **Get flow** (`src/api/v1/worlds.rs:360`):
   - `get_world()` checks `state.storage.world_exists(&world_id)`
   - `world_exists()` checks `world_package_path().exists()`
   - Uses same `base_dir()` → **Looks at wrong path**

3. **Resolution:** Both save and retrieve use the same `StorageManager`, but the path is inconsistent with the actual directory created by the app.

## Fix Required

**File:** `src/storage.rs`
**Line:** 165 (in the Linux cfg block of `default_base_dir()`)

Change:
```rust
.join("WorldFactory")
```
To:
```rust
.join("world-factory")
```

## Fix Verification (2026-05-07)

### Fixed By: WOR-358 (Storage layer path fix)

### Verification Tests

| Test | World ID | Expected | Actual | Result |
|------|----------|----------|--------|--------|
| Create world | New | 201 Created | 201 Created | ✅ PASS |
| Get new world | `88a05ff5-...` | 200 OK | 200 OK | ✅ PASS |
| Get existing world | `07446da1-...` | 200 OK | 200 OK | ✅ PASS |
| Get non-existent UUID | `00000000-...` | 404 | 404 | ✅ PASS |

### Verdict: **PASS** — Issue Resolved

All acceptance criteria met:
- [x] Newly created worlds are retrievable by ID
- [x] Existing worlds from list are retrievable
- [x] Non-existent IDs properly return 404

**Status: CLOSED**