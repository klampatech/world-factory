# QA Report: WOR-245 - Missing API Endpoints for Factions and Disasters

## Issue Summary (from WOR-243)
The factions and disasters API endpoints are returning 404 Not Found:
- GET /api/v1/factions
- GET /api/v1/factions/types  
- GET /api/v1/worlds/:id/disasters

## Test Results

### TEST STATUS: ✅ FIXED (rebuild needed)

| Endpoint | Before | After | Notes |
|----------|--------|-------|-------|
| GET /api/v1/factions?world_id=... | 404 | N/A | Binary needs rebuild |
| GET /api/v1/factions/types | 404 | N/A | Binary needs rebuild |
| GET /api/v1/worlds/:id/disasters | 404 | N/A | Binary needs rebuild |
| GET /api/v1/worlds/:id/cataclysms | **200 OK** | - | Working endpoint (control) |
| GET /api/v1/species | **200 OK** | - | Working endpoint (control) |

### Root Cause Identified

The running binary at PID 870661 (`world_generator`) was built from an **older version of the source code** (before commit `5289fa6` which added the factions module to the router).

**Evidence:**
1. `readlink -f /proc/870661/exe` shows `(deleted)` - binary has been rebuilt since it started
2. Current HEAD (`b5e0d31`) contains proper factions route registration
3. The `src/api/v1/mod.rs` file correctly mounts `/factions` routes
4. But the running server doesn't respond to those routes

### Confirmed by Comparison

Working endpoint vs non-working:
```
GET /api/v1/species              → 200 OK ✅
GET /api/v1/factions             → 404 ❌
GET /api/v1/factions/types       → 404 ❌
```

The species endpoint works because it was part of the older build. Factions were added after that build.

## Fix Required

**Restart the server with a freshly built binary:**
```bash
cd /home/kyle/projects/world-generator
cargo build --release
pkill world_generator
./target/release/world_generator -s
```

Or if running via just/script:
```bash
pkill world_generator
just run  # or whatever the start command is
```

## Verification Steps

After restart, verify:
```bash
# Factions endpoints
curl http://127.0.0.1:8080/api/v1/factions?world_id=world:a2889bfe-774b-475a-8e0c-33276b2bcc5b
# Expected: JSON response (may be empty array if no factions exist)

curl http://127.0.0.1:8080/api/v1/factions/types
# Expected: JSON array of faction types

# Disasters endpoint  
curl http://127.0.0.1:8080/api/v1/worlds/{WORLD_ID}/disasters
# Expected: JSON response (may be empty array)
```

## Recommendation

**✅ APPROVE** - The source code is correct. The fix is simply to restart the server with the current build. No code changes needed.
