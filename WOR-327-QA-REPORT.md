# WOR-327 Smoke Test Report

**Test Date:** 2026-05-07  
**Test Engineer:** QA Agent  
**Test Scope:** Full application stack (Backend API + Frontend UI)

---

## Environment Status

| Service | Port | Status |
|---------|------|--------|
| Backend API | 8080 | ✅ Running |
| Frontend (web) | 8765 | ✅ Running |
| Frontend (app) | 8787 | ✅ Running |

---

## API Endpoint Smoke Test Results

**Total:** 18 endpoints tested  
**Passed:** 7 ✅  
**Failed:** 11 ❌

### ✅ PASSING Endpoints

| Endpoint | Method | Status |
|----------|--------|--------|
| /health | GET | ✅ 200 |
| /api/v1/worlds | GET | ✅ 200 |
| /api/v1/worlds | POST | ✅ 201 |
| /api/v1/species | GET | ✅ 200 |
| /api/v1/worlds/:id/map | GET | ✅ 200 |
| /api/v1/worlds/:id/history | GET | ✅ 200 |
| /api/v1/worlds/:id/figures | GET | ✅ 200 |
| /api/v1/worlds/:id/disasters | GET | ✅ 200 |
| /api/v1/worlds/:id/artifacts | GET | ✅ 200 (requires `?limit=N`) |

### ❌ FAILING Endpoints

| Endpoint | Expected | Actual | Root Cause |
|----------|----------|--------|------------|
| GET /api/v1/worlds/:id | 200 | 500 | **Tar archive corruption** - `world.wfw` files have malformed headers (size=0 in tar header, but content is 5276 bytes) |
| GET /api/v1/worlds/:id/planet | 200 | 404 | **ID format mismatch** - endpoint expects `UUID` but receives `world:UUID` |
| GET /api/v1/worlds/:id/history/events | 200 | 404 | **Missing route** - endpoint not registered |
| GET /api/v1/worlds/:id/figures/:id | 200 | 404 | **ID format mismatch** |
| GET /api/v1/worlds/:id/settlements | 200 | 404 | **Missing/incomplete data** - route exists but returns 404 |
| GET /api/v1/worlds/:id/settlements/map | 200 | 404 | **Missing route** |
| GET /api/v1/worlds/:id/resources/summary | 200 | 404 | **Missing route** |
| GET /api/v1/worlds/:id/export | 200 | 404 | **Missing route** |
| GET /api/v1/worlds/:id/export.json | 200 | 404 | **Missing route** |
| DELETE /api/v1/worlds/:id | 204 | 400/404 | **ID format + corruption** |
| GET /api/v1/species_templates | 200 | 400 | **Missing endpoint** |

---

## Frontend UI Smoke Test Results

**Test File:** `e2e/frontend-smoke-tests.spec.ts` (12 TC-UI tests + 2 integration tests)

| Browser | Result | Notes |
|---------|--------|-------|
| Chromium | ✅ 14/14 PASS | All UI tests pass |
| Mobile Chrome | ✅ 14/14 PASS | All UI tests pass |
| Firefox | ⚠️ 14/14 FAIL | Config issue: baseURL=8787, app running on 8765 |
| WebKit | ⚠️ 14/14 FAIL | Same as Firefox |
| Mobile Safari | ⚠️ 14/14 FAIL | Same as Firefox |

**Chrome-based browsers: 100% pass rate (28/28 tests)**

---

## Root Cause Analysis

### BUG 1: Corrupted .wfw Tar Archives (Critical)

**Affected:** All worlds created in the current backend session  
**File:** `/home/kyle/.local/share/world-factory/generated/world:*/world.wfw` (only 350-370 bytes, should be ~66KB)  
**Symptom:** `tar -tzf world.wfw` → "Skipping to next header" for manifest.json  

**Root Cause:** The tar archive header for manifest.json has `size=0` in the header bytes 124-135, but the actual content is 5276 bytes. This causes `load_world()` in `packaging.rs` to read 0 bytes for manifest.json, then fail to parse.

**Evidence:**
```
$ ls -la world:2ea6f8c2-*/world.wfw
-rw-rw-r-- 1 kyle kyle 362 May  6 20:07 world.wfw  ← Only 362 bytes (truncated)

$ tar -tzf world:2ea6f8c2-*/world.wfw
manifest.json
tar: Skipping to next header  ← Manifest header is malformed
```

Compare with a properly-saved world (before bug was introduced):
```
$ ls -la world:3ae46d22-*/world.wfw
-rw-rw-r-- 1 kyle kyle 65958 world.wfw  ← Proper size

$ tar -tzf world:3ae46d22-*/world.wfw
manifest.json
world.json  ← Both entries readable
```

**Impact:** `GET /api/v1/worlds/:id` returns HTTP 500 with "Failed to load world: IO error: numeric field was not a number: when getting size for manifest.json"

---

### BUG 2: ID Format Inconsistency (API Contract Bug)

**Affected:** All `/api/v1/worlds/:id/*` endpoints  
**Issue:** The `/api/v1/worlds` list endpoint returns IDs as `"world:UUID"`, but individual endpoints expect plain `UUID`.

```
GET /api/v1/worlds  →  returns {"id": "world:uuid-here", ...}
GET /api/v1/worlds/world:uuid-here  →  500 (tar corruption)
GET /api/v1/worlds/uuid-here  →  404 (not found, wrong path)
```

---

### BUG 3: Missing API Routes

Several documented endpoints in the scope don't exist:
- `/api/v1/worlds/:id/history/events` → 404 (separate route not registered)
- `/api/v1/worlds/:id/settlements/map` → 404
- `/api/v1/worlds/:id/resources/summary` → 404
- `/api/v1/worlds/:id/export` → 404
- `/api/v1/worlds/:id/export.json` → 404
- `/api/v1/species_templates` → 400

---

### BUG 4: Test Configuration Mismatch

The `e2e/frontend-smoke.config.ts` sets `baseURL: 'http://localhost:8787'` but the frontend server runs on port 8765. This causes Firefox/WebKit/Mobile Safari tests to fail with connection refused.

---

## Verdict

**❌ SMOKE TEST FAILED**

**Pass Criteria:** All 18 API endpoints return expected responses + all frontend UI paths render without errors + zero browser console errors.

**Reality:** 11 of 18 API endpoints failing, most due to backend bugs (not environment issues). Chrome-based browser UI tests pass.

**Severity:**
- 🔴 **CRITICAL:** Tar archive corruption prevents loading any individual world
- 🔴 **HIGH:** ID format mismatch means the API contract is broken
- 🟡 **MEDIUM:** Missing routes for documented endpoints
- 🟢 **LOW:** Test config mismatch (not an app bug)

---

## Recommendations

1. **BUG-327-1:** Fix `save_world()` in `packaging.rs` - tar header size field not being set correctly
2. **BUG-327-2:** Standardize world ID format across all API endpoints (use `world:UUID` everywhere, or UUID only)
3. **BUG-327-3:** Implement missing routes or document them as unimplemented
4. **BUG-327-4:** Fix `e2e/frontend-smoke.config.ts` baseURL to use port 8765
