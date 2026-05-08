# WOR-519 Smoke Test Results

**Date:** 2026-05-07  
**Server:** http://localhost:8080  
**Test Suite:** ops/api_smoke_tests.py  
**Result:** 20 passed, 15 failed

## Summary

Smoke tests ran after server restart. The server is responding on port 8080, but many API endpoints are returning 404, indicating the routes may be missing or world ID normalization changes require route updates.

## Passed Tests (20)

| Test | Status |
|------|--------|
| TC-API-001: Health returns 200 | ✅ PASS |
| TC-API-001: Health returns JSON | ✅ PASS |
| TC-API-001: Health status field | ✅ PASS |
| TC-API-002: Create world returns 201 | ✅ PASS |
| TC-API-002: Create world returns world object | ✅ PASS |
| TC-API-002: World ID generated and unique | ✅ PASS |
| TC-API-002: Create world without name | ✅ PASS |
| TC-API-003: List worlds returns 200 | ✅ PASS |
| TC-API-003: List worlds returns array | ✅ PASS |
| TC-API-003: List worlds pagination | ✅ PASS |
| TC-API-003: List worlds search | ✅ PASS |
| TC-API-004: Get world returns 200 | ✅ PASS |
| TC-API-004: Get world correct fields | ✅ PASS |
| TC-API-005: Invalid ID returns 404 | ✅ PASS |
| TC-API-005: Malformed ID handling | ✅ PASS |
| TC-API-007: Get world map returns 200 | ✅ PASS |
| TC-API-007: Get world map polygons | ✅ PASS |
| TC-API-018: Invalid body returns 400/201 | ✅ PASS |
| TC-API-018: Empty body returns 400/201 | ✅ PASS |
| TC-API-019: Generate non-existent world returns 404/400 | ✅ PASS |

## Failed Tests (15)

| Test | Status | Error |
|------|--------|-------|
| TC-API-006: Trigger generation returns 202 | ❌ FAIL | 404 Not Found |
| TC-API-006: Generate with params | ❌ FAIL | 404 Not Found |
| TC-API-008: Timeline returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-009: Events returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-009: Events pagination | ❌ FAIL | 404 Not Found |
| TC-API-010: History returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-011: Figures returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-012: Societies returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-013: Planet returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-014: Tectonics returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-015: Artifacts returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-016: Cataclysms returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-017: Wonders returns 200 | ❌ FAIL | 404 Not Found |
| TC-API-018: Oversized name returns 400 | ❌ FAIL | Got 201, expected 400 |
| TC-API-020: Concurrent generation | ❌ FAIL | All 404s |

## Key Findings

### Blocker: Most child endpoints return 404

All tests for endpoints like `/timeline`, `/events`, `/history`, `/figures`, `/societies`, `/planet`, `/tectonics`, `/artifacts`, `/cataclysms`, `/wonders`, and `/generate` are returning 404.

**Reproduction:**
```bash
curl http://localhost:8080/api/v1/worlds/a0286f51-c4c3-4c61-9ddc-05d2e0e914fd/timeline
# Returns: 404 Not Found
```

The worlds are being created successfully (`POST /api/v1/worlds` returns 201), but all child endpoints return 404 even with valid world IDs.

### Secondary: Name length validation missing

Oversized names (>100 chars) are accepted with 201 instead of 400.

**Reproduction:**
```bash
curl -X POST http://localhost:8080/api/v1/worlds \
  -d '{"name":"xxx...xxx (200 chars)"}'
# Returns: 201 instead of 400
```

## Conclusion

**FAIL** - Server is running but 15 out of 35 smoke tests are failing. The core world CRUD operations work, but all child resource endpoints (timeline, events, history, figures, societies, planet, tectonics, artifacts, cataclysms, wonders, generate) return 404.

This likely indicates the world ID normalization changes (WOR-459 context) may have broken route matching, or these endpoints were removed/refactored.
