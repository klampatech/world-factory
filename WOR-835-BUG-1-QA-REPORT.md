# WOR-836 QA Report: `/history/events` endpoint returns 404

## Issue
`GET /api/v1/worlds/:id/history/events` returns HTTP 404 despite the endpoint being defined in code.

## Reproduction Steps

1. Create a world: `POST /api/v1/worlds`
2. Wait for world status to be `ready`
3. Call: `GET /api/v1/worlds/{world_id}/history/events`

## Expected Behavior
- Returns HTTP 200 with empty events array: `{"success":true,"data":{"worldId":"...","totalEvents":0,"events":[]...}}`

## Actual Behavior
- Returns HTTP 404 Not Found (empty body)

## Evidence

### API Response
```
$ curl -s -D - "http://localhost:8080/api/v1/worlds/246040e4-b331-4ecc-ae3d-4b5e4dc448ce/history/events"

HTTP/1.1 404 Not Found
vary: origin, access-control-request-method, access-control-request-headers
access-control-expose-headers: *
content-length: 0
date: Sat, 09 May 2026 02:07:19 GMT
```

### Comparison with working endpoints
| Endpoint | Status |
|----------|--------|
| `/api/v1/worlds/:id/history` | ✅ 200 OK |
| `/api/v1/worlds/:id/events` | ✅ 200 OK |
| `/api/v1/worlds/:id/history/events` | ❌ 404 NOT FOUND |

### Code Analysis
The route is registered correctly in `src/api/v1/worlds.rs`:
```rust
.route("/{id}/history/events", get(get_history_events))
```

The handler `get_history_events` exists with proper world existence check (lines 708-758).

## Status Timeline

### Initial Finding (2026-05-09T02:07)
**Root Cause: STALE DEPLOYMENT**

| Artifact | Date | Contains fix? |
|----------|------|---------------|
| Running Docker container `test-run` | Created 2026-05-09T00:04 UTC | ❌ No |
| Latest commit to `src/api/v1/worlds.rs` | 2026-05-08T16:35 UTC | ✅ Yes (commit `2a1a769`) |

**The Docker image was built before the fix was merged.**

### Escalation (2026-05-09T03:49)
**New Blocker: BUILD FAILURE**

Attempted to rebuild the Docker container but discovered **the latest code does not compile**:

```
error[E0560]: struct `Artifact` has no field named `activations_used`
  --> src/api/v1/artifacts.rs:88:13
```

5 errors in total - all referencing non-existent field `activations_used` on the `Artifact` struct.

### Created Blocker Issue
**WOR-846**: Build failure: Artifact struct missing activations_used field
- Status: todo
- Priority: critical
- Blocks: WOR-836

## Required Actions

1. **Coder (WOR-846)**: Fix `activations_used` field mismatch in `src/artifacts.rs` or `src/api/v1/artifacts.rs`
2. **DevOps**: Rebuild Docker container after code fix
3. **QA**: Verify `/api/v1/worlds/:id/history/events` returns 200

## Current Status Chain

1. Original bug: `/history/events` returns 404 ❌
2. Root cause identified: Stale deployment
3. Attempted fix: Rebuild container
4. New blocker: Code does not compile ❌ **← YOU ARE HERE**
5. Required: Fix WOR-846 first
6. Then: Rebuild and redeploy
7. Finally: Verify endpoint returns 200
