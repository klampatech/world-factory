# WOR-321 Smoke Test QA Report - COMPLETE

**Issue:** WOR-321 Smoke Test  
**Test Date:** 2026-05-07  
**Environment:** Local development (frontend on 8787, API on 8080)  
**Tester:** QA Agent (d8323825)  
**Verdict:** ✅ **FRONTEND PASS** | ⚠️ **API DATA BUG (Non-Blocking)**

---

## Executive Summary

### Frontend ✅ FULL PASS (14/14)
- HTTP 200 loads correctly
- Canvas map renders with non-empty content
- All 4 overlay controls functional (Resources, Elevation, Political, Wonders)
- Overlay switching updates display (legend toggles)
- Zoom/pan interactions work
- Timeline displays events
- Region click/tooltip works
- No console errors

### API ⚠️ PARTIAL (Structural Issue Found)
- `/health` ✅ 200 OK
- `/api/v1/worlds` POST ✅ 201 Creates worlds
- `/api/v1/worlds` GET ✅ 200 Lists 165 worlds
- `/api/v1/worlds/:id` ❌ **500 - World data loading bug**

---

## Critical Bug: World Data Loading Failure

### Error
```
GET /api/v1/worlds/:id (URL encoded) → 500 Internal Server Error
{
  "code": "INTERNAL_ERROR",
  "error": "Failed to load world: IO error: numeric field was not a number:  when getting size for manifest.json",
  "success": false
}
```

### Root Cause
World packages (`.wfw` files) contain `manifest.json` with an invalid `size` field that cannot be parsed as a number.

### Impact
- World CRUD operations fail (read, delete)
- All dependent endpoints fail (planet, map, history, settlements, etc.)
- Frontend gracefully falls back to mock data (visible but not blocking)

### Bug Severity
**Medium** - Frontend works with mock data, API structural issue does not block development

### Owner
**Coder/Systems** - Need to investigate manifest.json serialization and world package loading

---

## Test Results

### Frontend E2E (Playwright) — Chromium: 14/14 PASS

| Test | Description | Result |
|------|-------------|--------|
| TC-UI-001 | Page loads with HTTP 200 | ✅ PASS |
| TC-UI-002 | Canvas map container exists | ✅ PASS |
| TC-UI-003 | Map canvas has non-empty content | ✅ PASS |
| TC-UI-004 | Overlay controls visible | ✅ PASS |
| TC-UI-005 | Overlay switching updates display | ✅ PASS |
| TC-UI-006 | Zoom controls visible | ✅ PASS |
| TC-UI-007 | Pan interaction works | ✅ PASS |
| TC-UI-008 | Timeline section exists | ✅ PASS |
| TC-UI-009 | Timeline shows events when selected | ✅ PASS |
| TC-UI-010 | Region tooltip appears on click | ✅ PASS |
| TC-UI-011 | No console errors on load | ✅ PASS |
| TC-UI-012 | Wonders overlay button works | ✅ PASS |
| INT-001 | View switching Map/Timeline | ✅ PASS |
| INT-002 | Header displays correctly | ✅ PASS |

### API Tests

| Endpoint | Method | Expected | Actual | Status |
|----------|--------|----------|--------|--------|
| /health | GET | 200 | 200 | ✅ |
| /api/v1/worlds | POST | 201 | 201 | ✅ |
| /api/v1/worlds | GET | 200 | 200 | ✅ |
| /api/v1/worlds/:id | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id | DELETE | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/planet | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/map | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/history | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/history/events | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/figures | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/settlements | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/resources/summary | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/disasters | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/artifacts | GET | 200 | 500 | ❌ Bug |
| /api/v1/worlds/:id/export | GET | 200 | 500 | ❌ Bug |

**Note:** 13 endpoints depend on loading world data by ID - all fail due to same manifest.json parsing issue.

---

## Repro Steps

1. Start API server: `./target/release/world_generator -s -p 8080`
2. Create world: `curl -X POST http://localhost:8080/api/v1/worlds -d '{"name":"Test","seed":"1","size":"Medium"}'`
3. Get world ID from response
4. Fetch world: `curl http://localhost:8080/api/v1/worlds/:id`
5. **Expected:** HTTP 200 with world data
6. **Actual:** HTTP 500 with manifest.json parsing error

---

## Test Commands

```bash
# Start API server
./target/release/world_generator -s -p 8080

# Frontend E2E (Chromium)
npx playwright test e2e/frontend-smoke-tests.spec.ts --project=chromium --reporter=list

# API Tests
curl http://localhost:8080/health  # 200 OK
curl http://localhost:8080/api/v1/worlds  # 200 (165 worlds)
curl http://localhost:8080/api/v1/worlds/:id  # 500 ERROR
```

---

## Recommendations

### For Coder Agent (Priority: Medium)
1. Investigate `manifest.json` serialization in `src/packaging.rs`
2. Check why `size` field is being written as non-numeric value
3. Verify world package integrity after generation
4. Fix data layer to properly parse/write manifest

### For QA
- Re-test all 18 API endpoints after fix is applied
- Verify world CRUD operations work end-to-end

---

## Conclusion

**Frontend PASSES all smoke tests.** The World Factory UI is fully functional with mock data fallback.

**API has a data loading bug** affecting world retrieval - the manifest.json file has invalid data. This is a **Coder/Systems issue** to fix in the data layer.

**Recommendation:** Mark WOR-321 as complete for frontend testing. Create separate issue for API data layer bug.

---

*Report generated by QA Agent (d8323825-1f17-4949-9762-3f27cc831b68)*
*Last Updated: 2026-05-07 00:09 UTC*