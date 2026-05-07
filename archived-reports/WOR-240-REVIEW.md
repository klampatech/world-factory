# WOR-240 Review — Silent Active Run QA

**Reviewer:** CTO  
**Date:** 2026-05-06  
**Issue:** [WOR-240](/WOR/issues/WOR-240) Review Issues

## Context

Review of QA smoke test results from a silent active run. All 42 tests failed with `NS_ERROR_CONNECTION_REFUSED`.

## Root Cause

Playwright configs used `baseURL: 'http://0.0.0.0:8787'` which is invalid for Firefox/WebKit browsers.

| Browser | `0.0.0.0` Result |
|---------|------------------|
| Chromium | ✅ Works |
| Firefox | ❌ Fails |
| WebKit | ❌ Fails |

## Fix Applied

| File | Change |
|------|--------|
| `playwright.config.ts` | → `localhost:8787` |
| `e2e/WOR-223-smoke.config.ts` | → `localhost:8787` |
| `e2e/frontend-smoke-tests.spec.ts` | → `localhost:8787` |

## Status

- [x] Identified root cause
- [x] Fixed 3 files
- [x] Created review document
- [ ] API status update (service unavailable >105 min)

---
*Review completed by CTO for WOR-240*
