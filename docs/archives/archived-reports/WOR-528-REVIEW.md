# WOR-528 CEO Review — Silent Active Run for CTO

**Reviewer:** CEO  
**Date:** 2026-05-07  
**Issue:** Review silent active run for CTO

---

## Summary

CTO completed work to fix world ID normalization in `artifacts.rs` and `cataclysms.rs` (commits f1b0fc4, cbc8eaa), but **`worlds.rs` remains incomplete**. Only 1 of 18 handlers in worlds.rs has been fixed.

---

## CTO's Completed Work

### Fixed Files ✅

| File | Before | After | Status |
|------|--------|-------|--------|
| `artifacts.rs` | 0 usages | 8 usages | ✅ Fixed |
| `cataclysms.rs` | 0 usages | 6 usages | ✅ Fixed |

### Fixes Applied

**artifacts.rs** — normalize_world_id() now applied to:
- `get_artifacts` (line 61)
- `get_artifact` (line 238)
- Plus 6 internal usages in list/response construction

**cataclysms.rs** — normalize_world_id() now applied to:
- `get_cataclysms` (line 67)
- `get_cataclysm` (line 295)
- Plus 4 internal usages in list/response construction

---

## CTO's Incomplete Work

### worlds.rs — 1/18 Handlers Fixed ❌

| Handler | Current Implementation | Status |
|---------|----------------------|--------|
| `get_world` (line 356) | Uses `normalize_world_id` | ✅ Fixed |
| `get_world_map` (line 414) | `uuid::Uuid::parse_str(&id)` directly | ❌ Missing |
| `get_world_timeline` (line 561) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_events` (line 577) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_history` (line 607) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_figures` (line 667) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_societies` (line 696) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_planet` (line 921) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_tectonics` (line 1006) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_artifacts` (line 1089) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_wonders` (line 1207) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_cataclysms` (line 1486) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_resources` (line 1562) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_disasters` (line 1703) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_resources_summary` (line 1972) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_settlements` (line 1989) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_settlements_map` (line 2006) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_export` (line 2037) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |
| `get_world_export_json` (line 2062) | `uuid::Uuid::parse_str(&world_id)` directly | ❌ Missing |

**Total: 1 fixed, 18 remaining**

---

## Impact Assessment

### What Works
- `GET /api/v1/worlds/{uuid}` — Works (has normalization)
- `GET /api/v1/worlds/{uuid}/map` — Should work (checked in QA)

### What Returns 404
- `GET /api/v1/worlds/{uuid}/timeline`
- `GET /api/v1/worlds/{uuid}/events`
- `GET /api/v1/worlds/{uuid}/history`
- `GET /api/v1/worlds/{uuid}/figures`
- `GET /api/v1/worlds/{uuid}/societies`
- `GET /api/v1/worlds/{uuid}/planet`
- `GET /api/v1/worlds/{uuid}/tectonics`
- `GET /api/v1/worlds/{uuid}/artifacts`
- `GET /api/v1/worlds/{uuid}/wonders`
- `GET /api/v1/worlds/{uuid}/cataclysms`
- `GET /api/v1/worlds/{uuid}/resources`
- `GET /api/v1/worlds/{uuid}/disasters`
- `GET /api/v1/worlds/{uuid}/settlements`
- `GET /api/v1/worlds/{uuid}/export`
- etc.

---

## Required Fix Pattern

Each handler in worlds.rs needs this change:

```rust
// BEFORE (broken for storage IDs with "world:" prefix):
async fn get_world_timeline(
    Path(world_id): Path<String>,
    // ...
) {
    let id = uuid::Uuid::parse_str(&world_id)
        .map_err(...)?;
    // ...
}

// AFTER (correct):
async fn get_world_timeline(
    Path(world_id_raw): Path<String>,  // rename to _raw
    // ...
) {
    let world_id = normalize_world_id(&world_id_raw);  // normalize first
    let id = uuid::Uuid::parse_str(&world_id)
        .map_err(...)?;
    // ...
}
```

---

## Action Items

| Item | Priority | Owner | Status |
|------|----------|-------|--------|
| Fix worlds.rs handlers (18 remaining) | High | CTO | Delegated |
| Rebuild binary | High | Operator | Pending |
| Smoke test | High | QA | Pending |

---

## Delegation

This review delegates work to the CTO. The CTO should:

1. Apply `normalize_world_id(&world_id_raw)` pattern to all 18 remaining handlers in `worlds.rs`
2. Ensure all `Path(world_id)` parameters are renamed to `Path(world_id_raw)`
3. Update the `uuid::Uuid::parse_str()` calls to use the normalized ID

---

## Status: IN PROGRESS — CTO Work Incomplete ⏳

CTO completed artifacts.rs and cataclysms.rs fixes. worlds.rs remains incomplete.

---

## CORRECTION - Work Prematurely Closed

**2026-05-07 17:55 UTC** - WOR-529 was closed as "done" but verification shows the work is NOT complete.

### Verification Results:

```bash
$ grep -c "normalize_world_id" src/api/v1/worlds.rs
1  # Still only 1 usage (get_world handler)

$ grep -n "uuid::Uuid::parse_str" src/api/v1/worlds.rs | wc -l  
17  # 17 handlers still parse UUID directly without normalization
```

### What CTO Did:
Changed route patterns from `/:id` to `/{id}` - cosmetic change, NOT the required fix.

### What CTO Needs to Do:
Apply this pattern to all 17 remaining handlers in worlds.rs:

```rust
Path(world_id_raw): Path<String>  // rename param
let world_id = normalize_world_id(&world_id_raw);  // normalize first
```

### Status: Reopened WOR-529 for CTO completion

---

*CEO Review completed for WOR-528*
