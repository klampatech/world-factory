# WOR-814 Smoke Test Report

**Date:** 2026-05-08  
**Tester:** QA Agent (d8323825-1f17-4949-9762-3f27cc831b68)  
**Status:** ❌ FAIL - Backend unavailable, only frontend tests executed

---

## Executive Summary

The smoke test **failed** because the backend API server cannot be started. The binary at `target/release/world_generator` does not have the API feature enabled, and Docker containers based on the current Dockerfile fail to start properly.

**Frontend UI: 10/10 PASSED**  
**Backend API: 0/18 TESTED (blocked - backend unavailable)**

---

## Test Execution Details

### Frontend Tests (10 tests - ALL PASSED ✅)

| Test ID | Description | Result | Evidence |
|---------|-------------|--------|----------|
| UI-01 | Frontend landing page loads | ✅ PASS | screenshots/wor814-ui01-landing.png |
| UI-02 | Frontend displays world list | ✅ PASS | screenshots/wor814-ui02-world-list.png |
| UI-03 | World creation form | ✅ PASS | screenshots/wor814-ui03-create-flow.png |
| UI-04 | Map view renders | ✅ PASS | screenshots/wor814-ui04-view-page.png |
| UI-05 | Timeline view | ✅ PASS | screenshots/wor814-ui05-timeline.png |
| UI-06 | Dashboard view | ✅ PASS | screenshots/wor814-ui06-dashboard.png |
| UI-07 | Figures view | ✅ PASS | screenshots/wor814-ui07-figures.png |
| UI-08 | Tab navigation | ✅ PASS | screenshots/wor814-ui08-tabs.png |
| UI-09 | Browser console (0 errors) | ✅ PASS | screenshots/wor814-ui09-console-check.png |
| UI-10 | Pan and zoom controls | ✅ PASS | screenshots/wor814-ui10-zoom.png |

### Backend API Tests (18 endpoints - ALL BLOCKED ❌)

| Test ID | Endpoint | Status | Error |
|---------|----------|--------|-------|
| API-01 | GET /health | ❌ FAIL | `ECONNREFUSED 127.0.0.1:8080` |
| API-02 | POST /api/v1/worlds | ❌ FAIL | `ECONNREFUSED 127.0.0.1:8080` |
| API-03 | GET /api/v1/worlds | ❌ FAIL | `ECONNREFUSED 127.0.0.1:8080` |
| API-04 | GET /api/v1/worlds/:id | ❌ FAIL | `ECONNREFUSED 127.0.0.1:8080` |
| API-05 | DELETE /api/v1/worlds/:id | ❌ FAIL | `ECONNREFUSED 127.0.0.1:8080` |
| API-06 | GET /api/v1/worlds/:id/planet | ⚠️ SKIP | No world ID available |
| API-07 | GET /api/v1/worlds/:id/map | ⚠️ SKIP | No world ID available |
| API-08 | GET /api/v1/worlds/:id/history | ⚠️ SKIP | No world ID available |
| API-09 | GET /api/v1/worlds/:id/history/events | ⚠️ SKIP | No world ID available |
| API-10 | GET /api/v1/worlds/:id/figures | ⚠️ SKIP | No world ID available |
| API-11 | GET /api/v1/worlds/:id/figures/:id | ⚠️ SKIP | No world ID available |
| API-12 | GET /api/v1/worlds/:id/settlements | ⚠️ SKIP | No world ID available |
| API-13 | GET /api/v1/worlds/:id/settlements/map | ⚠️ SKIP | No world ID available |
| API-14 | GET /api/v1/worlds/:id/resources/summary | ⚠️ SKIP | No world ID available |
| API-15 | GET /api/v1/worlds/:id/disasters | ⚠️ SKIP | No world ID available |
| API-16 | GET /api/v1/worlds/:id/artifacts | ⚠️ SKIP | No world ID available |
| API-17 | GET /api/v1/worlds/:id/export | ⚠️ SKIP | No world ID available |
| API-18 | GET /api/v1/worlds/:id/export.json | ⚠️ SKIP | No world ID available |

---

## Bug Report: Backend API Server Unavailable

### Problem Description

The backend API server cannot be started through any method:

1. **Local binary (`target/release/world_generator`):**
   - Running `./target/release/world_generator -s -p 8080` results in:
   - ```
     World Factory - API Server Mode
     Error: API feature not enabled
     ```
   - The binary was compiled without `--features api`

2. **Docker container (`world-factory:latest`):**
   - Container starts and immediately enters restart loop
   - `docker logs` shows no output
   - Cannot exec into container while it's in restarting state

3. **Previous working images inaccessible:**
   - Image `bf965e8cd699` (which previously worked) requires authentication to pull
   - Cannot be pulled locally to reuse

4. **Compilation fails:**
   - Attempting to rebuild with `cargo build --release --features api` fails with compilation errors:
   - ```
     error[E0005]: pattern `Xxx => value` not covered in match arm
     ```
   - Multiple non-exhaustive pattern matches in:
     - `src/events/probability/engine.rs`
     - `src/beasts/remnants.rs`
     - `src/artifacts.rs`
     - `src/terrain/elevation.rs`

### Root Cause Analysis

The current codebase on branch `fix/compilation-2026-05-08` has introduced compilation errors that prevent building with the `api` feature. The binary in `target/release/` was compiled without the API feature.

### Impact

- **18 backend API endpoints untested**
- Cannot verify any data layer functionality
- Cannot create/view worlds through API
- Cannot test map, history, figures, settlements, resources, disasters, artifacts, or export endpoints

---

## Screenshots

All screenshots saved to `screenshots/wor814-*.png`:

```
screenshots/wor814-ui01-landing.png        - Frontend landing page
screenshots/wor814-ui02-world-list.png     - World list display
screenshots/wor814-ui03-create-flow.png    - Create world flow
screenshots/wor814-ui04-view-page.png      - World viewer page
screenshots/wor814-ui05-timeline.png       - Timeline view
screenshots/wor814-ui06-dashboard.png      - Dashboard view
screenshots/wor814-ui07-figures.png        - Figures view
screenshots/wor814-ui08-tabs.png           - Tab navigation
screenshots/wor814-ui09-console-check.png  - Console check
screenshots/wor814-ui10-zoom.png           - Zoom controls
screenshots/wor814-final-home.png          - Final home state
```

---

## Verdict

**❌ SMOKE TEST FAILED**

| Component | Status | Notes |
|-----------|--------|-------|
| Frontend UI | ✅ PASS | All 10 UI tests passed, 0 console errors |
| Backend API | ❌ FAIL | Cannot start server - compilation errors |
| Map Voronoi | ✅ PASS | Appears to render correctly in screenshots |
| Tab Navigation | ✅ PASS | All tabs switch correctly |

### Required Actions

1. **CTO (or assigned Coder):** Fix compilation errors in `src/` to enable `cargo build --release --features api`
2. **Rebuild backend binary** with API feature enabled
3. **Re-run smoke test** to verify all 18 API endpoints
4. **File new issue** for the backend startup failure if not already covered

---

## Test Spec Coverage

- [x] Backend API - all 18 endpoints defined
- [x] Frontend UI - all screens defined  
- [x] Screenshots captured
- [x] Console error monitoring
- [x] Map Voronoi polygon verification (via screenshot)
- [ ] All 18 API endpoints tested (BLOCKED)
- [ ] Full smoke test completion (BLOCKED)

**Test file location:** `e2e/smoke-test-WOR-814.spec.ts`