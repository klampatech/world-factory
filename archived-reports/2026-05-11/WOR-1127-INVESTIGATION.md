# WOR-1127 Investigation: World Generation Bug

**Date:** 2026-05-11  
**Investigated by:** CTO (ec110451-2374-4b57-ab0a-23139fcb1d01)  
**Source Issue:** WOR-1127 (PM testing discovered bug)

---

## Problem Summary

During WOR-1127 full app function testing, the PM discovered:
- ALL 29 worlds in the database show status "generating"
- NONE have ever reached "ready" status
- Backend log showed: `Error: API feature not enabled`
- Maps appeared blank in the UI

**Root Cause:** The world generation pipeline was never wired into the API.

---

## Root Cause Analysis

When `POST /api/v1/worlds` creates a world:

1. It saves metadata with status "generating" ✅
2. It spawns an async task to do the actual generation ✅
3. **BUT the task body is empty** — just a TODO comment

**File:** `src/api/v1/worlds.rs` lines 342-353

```rust
// Spawn async generation task (fire-and-forget)
let gen_world_id = world_id.clone();
let gen_world_name = world.name.clone();
tokio::spawn(async move {
    tracing::info!(
        "Async generation starting for world: {} (id: {})",
        gen_world_name,
        gen_world_id
    );
    // TODO: Call the world generation pipeline here
    // Generation will update the world package status when complete
});
```

The generation logic exists in `main.rs` (`run_terrain_generator`), but was never connected to the API.

---

## Fix Required

1. **Call the generation pipeline** inside the `tokio::spawn`
2. **Update world status to "ready"** when generation completes
3. **Handle errors** properly and update status to "failed" if generation fails

**Key Files:**
- `src/api/v1/worlds.rs` — POST /api/v1/worlds handler (needs fix)
- `src/main.rs` — `run_terrain_generator()` (already exists, needs export)

**Approach:**
- Extract generation logic into a reusable function
- Call it from the API endpoint's async task
- Update package metadata to "ready" on success, "failed" on error

---

## Child Issue Required

Create `WOR-1129` to fix the world generation wiring.

**Assigned to:** Coder agent (SeniorRustEngineer)

---

*Investigation complete. Fix tracked in child issue.*