# WOR-919 Smoke Test Report

## Test Execution
- **Date:** 2026-05-09T20:01:09.408Z
- **Branch:** main (latest)
- **Commit:** 90aba14074f05383e8ede0b08774d56f4c2c0cf5

## Results Summary
- **Status:** FAIL ❌
- **API Endpoints:** 0/2 passed
- **Frontend Tests:** 2/4 passed
- **Total:** 2/6 passed

## API Endpoint Results
- ❌ POST /api/v1/worlds: ERROR (fetch failed)
- ❌ GET /api/v1/worlds: ERROR (fetch failed)

## Frontend UI Results
- ✅ World creation form
- ✅ World list display
- ❌ Map view (No world ID available)
- ❌ Zero console errors (Failed to load resource: net::ERR_CONNECTION_REFUSED)

## Console Errors
- Failed to load resource: net::ERR_CONNECTION_REFUSED

## Screenshots
- 01_landing_page: screenshots/smoke-test-WOR-919/01_landing_page.png
- 02_world_form: screenshots/smoke-test-WOR-919/02_world_form.png
- 03_form_filled: screenshots/smoke-test-WOR-919/03_form_filled.png
- 04_after_submit: screenshots/smoke-test-WOR-919/04_after_submit.png
- 05_world_list: screenshots/smoke-test-WOR-919/05_world_list.png

## Bug Reports
Bugs detected - see results above.
