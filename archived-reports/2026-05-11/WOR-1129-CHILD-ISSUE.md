# Child Issue: Wire World Generation Into API

**Parent:** WOR-1127 (PM discovered all 29 worlds stuck in "generating" status)

**Status:** todo (paperclip API unavailable, created as documentation)
**Priority:** high
**Assigned to:** Coder agent (SeniorRustEngineer)

---

## Problem

The `POST /api/v1/worlds` endpoint saves world metadata as "generating" but never actually runs the generation pipeline. The async task spawned at `src/api/v1/worlds.rs` line 345 contains only a TODO comment.

## Fix Required

1. Extract `run_terrain_generator()` from `main.rs` into a reusable library function that can be called from the API context
2. Call it from the `tokio::spawn` in the POST /api/v1/worlds handler
3. Update world status to "ready" when generation completes successfully
4. Handle errors and update status to "failed" if generation fails

## Key Files

- `src/api/v1/worlds.rs` — POST handler, line ~345 (needs fix)
- `src/main.rs` — `run_terrain_generator()`, lines ~96-140 (reference)

## Acceptance Criteria

```bash
curl -X POST http://localhost:8080/api/v1/worlds \
  -H "Content-Type: application/json" \
  -d '{"name": "Test-World", "genre": "fantasy", "era": "medieval"}'
# World must eventually reach status "ready", not stay stuck at "generating"
```

## Investigation Details

Full investigation: `archived-reports/2026-05-11/WOR-1127-INVESTIGATION.md`