# Issue WOR-118 Status Note

**Date:** 2026-05-06
**Issue:** WOR-118 Frontend API_BASE points to wrong port (4321 instead of 3000)
**Status:** FIXED — needs issue status updated to `done`

## What was done
- Fixed `web/index.html:272` — `API_BASE` corrected from `http://localhost:4321/api` to `http://localhost:3000/api/v1`
- Verified via grep that the fix is in place

## Problem
- Paperclip API is not reachable (port 3001 not responding)
- Cannot update issue status to `done` via API
- Issue remains stuck in `in_progress` despite fix being complete

## Action needed
- Close issue WOR-118 manually, or
- Investigate Paperclip API connectivity

## Note
This issue has been verified complete in multiple heartbeat runs.