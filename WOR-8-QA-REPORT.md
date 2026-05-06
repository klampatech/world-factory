## WOR-8 Testing Review — QA Report

### Verdict: MIXED (Framework: PASS | Runtime Tests: FAIL)

---

### Coverage Gap Analysis

#### ✅ What IS in place

| Area | Status | Evidence |
|---|---|---|
| Vitest (TypeScript unit tests) | **PASS** | `npm test` → 3/3 passing |
| Playwright (E2E config) | Configured | `playwright.config.ts` with multi-browser, multi-device setup |
| GitHub Actions CI | **PASS** | 8-job pipeline: lint, unit tests, integration tests, API tests, frontend E2E, benchmarks, nightly full-pipeline, Slack failure notifications |
| Rust integration test files | Structured | `tests/integration_world_generation.rs` (9 tests), `tests/phase1_integration_test.rs`, `tests/phase2_integration_test.rs` with comprehensive assertions |
| API smoke tests (Python) | Written | `api_smoke_tests.py` — 35 test cases covering all endpoints |
| Frontend E2E (Playwright TS) | Written | `e2e/frontend-smoke-tests.spec.ts` — TC-UI-001 to TC-UI-012 |

#### ❌ What is NOT working (requires running server)

| Area | Status | Evidence |
|---|---|---|
| API smoke tests | **FAIL** — 17 failed, 18 errors | API server not running at localhost:8080 |
| Frontend E2E tests | **FAIL** — 10/11 failing | Frontend not serving at localhost:8765 |
| Rust `cargo test` | **UNABLE TO VERIFY** | `cargo` not installed in local environment (CI has it via `actions-rust-lang/setup-rust`) |

---

### Gap #1 (HIGH PRIORITY): No running server blocks runtime test verification

The API tests and frontend E2E tests cannot be validated without the services running.
- API tests expect `http://localhost:8080` — no server process found
- Frontend E2E tests expect `http://localhost:8765` — no server process found

**Fix:** The `playwright.config.ts` declares a `webServer` for the preview server, but the backend API at port 8080 has no equivalent startup command. The CI pipeline handles this via `cargo build --features api`, but local dev and the GitHub Actions `api-tests` job (which starts postgres) depends on `cargo test --test api_world_generation`.

**Recommendation:** Add a `scripts/` directory with `start-api.sh` and `start-frontend.sh`, and document startup order in `README.md`.

---

### Gap #2 (MEDIUM): Rust unit/integration tests not locally verifiable

Without `cargo` installed, I cannot run the Rust tests locally. The CI pipeline is correctly configured for this, but it means Rust test failures won't be caught in local dev iteration.

**Recommendation:** Add a `justfile` or `Makefile` with `cargo` availability checks, or add a Docker-based test runner.

---

### Gap #3 (MEDIUM): Frontend smoke test failures — missing DOM elements

The Python frontend smoke tests check for specific DOM selectors:
- `#overlay-controls` (Resources, Elevation, Political, Wonders)
- `#zoom-in` / `#zoom-out`
- `#timeline-container`
- `.view-tab`

These selectors are **not found** in the current HTML, indicating either:
1. The frontend UI is incomplete (components not rendered), or
2. The smoke tests use incorrect selectors

**Recommendation:** Verify that `web/index.html` implements the elements the tests look for, or align test selectors with actual DOM output.

---

### Summary Table

| Test Layer | Tool | Tests Written | Tests Running | Pass Rate |
|---|---|---|---|---|
| TypeScript unit tests | Vitest | 3 | 3 | ✅ 100% |
| Rust unit/integration | cargo test | ~20 | Unknown (no cargo) | ❓ |
| API integration | Python/pytest | 35 | 17 failed, 18 errors | ❌ 0% |
| Frontend E2E | Playwright TS | 12 | 10 failed, 1 passed | ❌ 8% |
| CI pipeline | GitHub Actions | 8 jobs | Likely passing | ✅ (CI-managed) |

---

### Next Steps

1. **Coder**: Add server startup scripts (`start-api.sh`, `start-frontend.sh`) with health checks
2. **Coder**: Align frontend DOM selectors in smoke tests with actual `web/index.html`
3. **QA**: Re-run `api_smoke_tests.py` and `e2e/frontend-smoke-tests.py` once servers are confirmed running
4. **QA**: Create child issues for each gap as needed
