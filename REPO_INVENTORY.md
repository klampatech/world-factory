# World-Factory Repository Inventory Report

## Overview
Repository at `~/Projects/world-factory` — a procedural world and history generation engine in Rust with TypeScript frontend.

---

## 1. Root-Level Files & Directories

### Source Code (Needed in Repo)

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `Cargo.toml` | Rust package manifest | Tracked | **KEEP** |
| `Cargo.lock` | Rust dependency lock file | **Ignored** (in .gitignore) | **CONSIDER TRACKING** (see §9) |
| `src/` | Rust source code (18 subdirs) | Tracked | **KEEP** |
| `justfile` | Task runner recipes | Tracked | **KEEP** |
| `Dockerfile` | Container build (updated May 7) | Tracked | **KEEP** |
| `Dockerfile.test` | Test container | Tracked | **KEEP** |
| `docker-compose.yml` | Local dev services | Tracked | **KEEP** |
| `.env.example` | Env var template | Tracked | **KEEP** |
| `.dockerignore` | Docker build exclusion | Tracked | **KEEP** |

### Documentation

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `README.md` | Project overview | Tracked | **KEEP** |
| `CONTRIBUTING.md` | Contribution guidelines | Tracked | **KEEP** |
| `docs/` | Specs, API contracts, guides | Tracked | **KEEP** |
| `docs/SPEC.md` | Main specification (79KB) | Tracked | **KEEP** |
| `docs/archive/` | Old spec versions | Tracked | **KEEP** |

### Web Frontend

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `web/` | Standalone HTML demos + TypeScript source | Tracked | **KEEP** (see §2) |
| `web/dist/` | Built web assets | Tracked | **KEEP** (see §2) |
| `dist/` | TypeScript build from `src/` | Tracked | **KEEP** (see §2) |
| `web/index.html` | Main web UI (89KB) | Tracked | **KEEP** |
| `demo.html` | Map overlays demo | **Untracked** | **ARCHIVE** (stale) |
| `demo-society-dashboard.html` | Society dashboard demo | **Untracked** | **ARCHIVE** (stale) |
| `hex-test.html`, `hex-tiling-verification.html` | Verification tools | Tracked | **KEEP** (QA tools) |

### Tests

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `tests/` | Rust integration tests (.rs) | Tracked | **KEEP** |
| `target-test/` | Rust compilation artifacts | **Ignored** | **GITIGNORE** (correct) |
| `e2e/` | Playwright E2E specs | Tracked | **KEEP** |
| `playwright.config.ts` | Root Playwright config | Tracked | **KEEP** |
| `smoke-test-wor179.spec.ts` | Smoke test specs | Tracked | **KEEP** |
| `smoke-test-wor206.spec.ts` | Smoke test specs | Tracked | **KEEP** |
| `test-results/` | Playwright test results | **Ignored** | **GITIGNORE** (correct) |
| `playwright-report/` | Playwright HTML report | **Ignored** | **GITIGNORE** (correct) |

### QA Artifacts

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `qa-reports/` | QA reports for recent WORs | Tracked | **KEEP** (active work) |
| `screenshots/` | Screenshot captures | **Ignored** | **GITIGNORE** (correct) |
| `archived-reports/` | Completed QA reports (100+ files) | Tracked | **KEEP** (historical record) |
| `config-archive/` | Old Playwright configs | Tracked | **CONSIDER REMOVING** (see §5) |

### CI/Deploy

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `.github/workflows/ci.yml` | Lint + build + test gate | Tracked | **KEEP** |
| `.github/workflows/test.yml` | Full test pipeline (327 lines) | Tracked | **KEEP** |
| `.github/workflows/deploy.yml` | Staging/production deploy | Tracked | **KEEP** |

### Operational Scripts

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `ops/` | Operational scripts | Tracked | **KEEP** |
| `scripts/` | Utility scripts | Tracked | **KEEP** |
| `examples/` | Example code | Tracked | **KEEP** |
| `species_templates/` | Data templates | Tracked | **KEEP** |

### Node/NPM

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `package.json` | Root npm manifest (playwright dep) | Tracked | **KEEP** |
| `package-lock.json` | npm lock file | Tracked | **KEEP** |
| `tsconfig.json` | TypeScript config | Tracked | **KEEP** |
| `web/package.json` | Web build manifest (puppeteer) | Tracked | **KEEP** |
| `web/package-lock.json` | Web npm lock | Tracked | **KEEP** |
| `__pycache__/` | Python cache | **Ignored** | **GITIGNORE** (correct) |

### WOR Documents (Work Orders)

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| WOR-296, WOR-356, WOR-358, WOR-360, WOR-371 | Recent REVIEW/QA | Tracked (May 7) | **KEEP** (active) |
| WOR-378, WOR-384, WOR-391, WOR-393, WOR-394 | Recent REVIEW | Tracked (May 7) | **KEEP** (active) |
| WOR-399, WOR-400, WOR-409, WOR-413, WOR-416 | Recent REVIEW | Tracked (May 7) | **KEEP** (active) |
| WOR-901, WOR-902, WOR-915, WOR-916, WOR-922, WOR-935, WOR-941 | CTO Review cycle | Tracked (May 9) | **KEEP** (active) |
| WOR-245, WOR-284, WOR-393 STATUS files | Older STATUS | Tracked (May 6) | **ARCHIVE** (complete) |

### Binary Artifacts

| Item | Purpose | Git Status | Recommendation |
|------|---------|------------|----------------|
| `test_arc` | ELF binary (4.3MB) | **Untracked** | **REMOVE FROM WORKDIR** (see §3) |

---

## 2. Relationship Between `web/`, `web/dist/`, and `dist/`

**Three separate artifact streams:**

| Directory | Content | Origin | Git |
|-----------|---------|--------|-----|
| `dist/` | TypeScript build output (app.js, components/, events/, models/, routes/, terrain/, types/) | Built from `src/` via `tsc` | Tracked |
| `web/` | Standalone HTML/JS demos + TypeScript source | Hand-written + `web/dist/` built into it | Tracked |
| `web/dist/` | Built web assets (index.html, api-integration.js, wor205-qa-test.html) | Built from `web/` source | Tracked |

**Verification:** `web/dist/index.html` and `web/index.html` have identical MD5 checksums (`682b1f559a4196bd586614b0b69bac6b`), confirming `web/index.html` IS the built artifact from `web/dist/`.

**CI Usage:** The test.yml workflow runs `npm run build` which executes `cd web && npm run build`, building `web/dist/` from `web/` sources. The preview server serves from `web/dist/`.

**Conclusion:** `web/dist/` is the build output; `web/index.html` is a copy of that build output for serving. The `dist/` directory is a completely separate TypeScript compilation of the `src/` directory.

---

## 3. `test_arc` Binary

**What it is:** ELF 64-bit LSB executable (4,337,056 bytes), built with debug info.

**Not in git:** Only exists in working tree, ignored by `.gitignore`.

**Likely purpose:** Archive test binary — appears to be a cargo test artifact or custom test harness.

**Git history:** Last touched by commit `df8cb08` ("WOR-227: Disable mock API fallback, use real backend"), but the binary itself is not in git.

**Recommendation:** **REMOVE from working tree** — it's a generated artifact that shouldn't be in the repo. If it's needed for testing, it should be generated via `cargo build` or a script, not stored in the repo.

---

## 4. `target-test/` vs `tests/`

| | `target-test/` | `tests/` |
|---|----------------|----------|
| **Content** | Rust debug artifacts (.rustc_info.json, CACHEDIR.TAG, debug/) | Rust integration test source files (.rs) |
| **Nature** | Compilation/cache artifacts | Test source code |
| **In git** | No | Yes |
| **gitignore** | Already ignored (by `target/`) | N/A — source code |

**Conclusion:** These are completely different things. `target-test/` is to `tests/` what `target/` is to `src/` — generated artifacts vs source code. The `.gitignore` correctly ignores `target-test/`.

---

## 5. `config-archive/` Contents & Need

**Contents:** 14 old Playwright config files:
- `phase4-web-ui.config.ts`
- `playwright.config.ts`, `playwright.e2e.config.ts`, `playwright-headless.config.ts`, `playwright-qa.config.ts`
- `playwright-wor75.config.ts`, `playwright-wor75-v2.config.ts`
- `smoke-wor167.config.ts`, `smoke-wor179.config.ts`, `smoke-wor206.config.ts`, `smoke-wor261.config.ts`
- `wor134-screenshot.config.ts`, `smoke-wor167.config.ts`

**Purpose:** These configs were used for specific smoke tests that have since been reorganized into the `e2e/` directory.

**Need:** **No** — these are stale configs. The active configs are in `e2e/` and at the root level. The .gitignore pattern `wor*-*.config.ts` and `smoke*.config.ts` correctly ignores any that might be regenerated.

**Recommendation:** **REMOVE from tracked git** — move to `archived-reports/` if historical record is needed, or delete entirely. The same configs are already available in `e2e/` with the actual test specs.

---

## 6. `demo.html` Files — Live or Stale?

| File | Size | Last Modified | In Git | Status |
|------|------|---------------|--------|--------|
| `demo.html` | 18,472 bytes | May 6 20:32 | **No** | **STALE** |
| `demo-society-dashboard.html` | 29,100 bytes | May 6 20:32 | **No** | **STALE** |
| `web/hex-test.html` | 3,337 bytes | May 6 20:32 | **Yes** | **KEEP** (QA tool) |
| `web/hex-tiling-verification.html` | 5,394 bytes | May 6 20:32 | **Yes** | **KEEP** (QA tool) |
| `web/wor205-qa-test.html` | 8,480 bytes | May 6 20:32 | Tracked | **KEEP** (QA tool) |

**Analysis:** The root `demo.html` and `demo-society-dashboard.html` are:
1. **Not tracked in git** (only in working tree)
2. **Not built by CI** (`npm run build` doesn't touch them)
3. **Stale** — last touched May 6, no recent commits

**Recommendation:** **ARCHIVE** — move to `archived-reports/` or delete. They are standalone demos that are not part of any automated pipeline.

---

## 7. WOR-* Files: Active vs Should Archive

**Recent (Active — May 7, 2026):**
- WOR-296-SMOKE-TEST-FINAL.md
- WOR-356-REVIEW.md, WOR-358-QA-REPORT.md, WOR-360-REVIEW.md, WOR-371-REVIEW.md
- WOR-378-REVIEW.md, WOR-384-REVIEW.md, WOR-391-REVIEW.md
- WOR-394-REVIEW.md, WOR-399-QA-REPORT.md, WOR-400-REVIEW.md
- WOR-409-REVIEW.md, WOR-413-REVIEW.md, WOR-416-REVIEW.md
- WOR-284-STATUS.txt, WOR-284-WAKE-LOG.txt, WOR-284-WAKES.txt

**Older (Consider Archiving — May 6 or earlier):**
- WOR-245-QA-STATUS.txt (May 6 16:02)
- WOR-284-STATUS.txt, WOR-284-WAKE-LOG.txt, WOR-284-WAKES.txt (May 6 20:09)
- WOR-393-STATUS.txt (May 6 20:24)

**Criteria for archiving:**
1. STATUS files where the work order is CLOSED/COMPLETE → archive
2. REVIEW/QA-REPORT files for completed work → archive to `archived-reports/`

**Recommendation:**
- **Archive** the 3 WOR-284 completion files (wake logs, final status) — WOR-284 is complete
- **Archive** WOR-245-QA-STATUS.txt and WOR-393-STATUS.txt if those WORs are closed
- **Keep** all REVIEW/QA files for active in-progress work

---

## 8. CI Workflow: What It Does with `web/dist`

**test.yml `frontend-e2e` job:**
```yaml
- npm ci                    # Install root dependencies
- npx playwright install --with-deps chromium
- npm run build             # Runs: cd web && npm run build
- npm run preview &         # Serves from web/ dir (port 8765)
- npx playwright test --reporter=html
```

**Key insight:** `npm run build` runs `cd web && npm run build` which invokes `web/scripts/build.js`. This builds `web/dist/` from `web/` sources. The preview server then serves from `web/` directory (which contains the built `web/dist/`).

**The root `dist/` is NOT used by CI** — it's TypeScript compiled from `src/`, likely for a different purpose (maybe server-side rendering or API).

**deploy.yml:** Uses `cargo build --release --all-features` to build the Rust binary, uploads `target/release/world_generator` as an artifact. Does NOT use `web/dist`.

---

## 9. Should Cargo.lock Be Tracked?

**Current state:** `Cargo.lock` is in `.gitignore` (line 6: `# Rust lock file (regenerated)`).

**Arguments FOR tracking:**
1. **CI explicitly uses `Cargo.lock` for caching:**
   ```yaml
   key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
   ```
   Without `Cargo.lock` in git, the cache key becomes unstable.
2. **Reproducibility:** Lock file ensures `cargo build` produces byte-identical binaries across machines.
3. **Cargo docs recommend** tracking `Cargo.lock` for libraries, and it's common practice for binaries too.

**Arguments AGAINST (current practice):**
1. Comment says "regenerated" — implying `cargo update` should be run to update dependencies intentionally.
2. Lock file bloat (45KB) with no real value if you regenerate regularly.

**Recommendation:** **TRACK Cargo.lock**. The CI workflow explicitly hashes it for caching, and reproducible builds are valuable. Remove it from `.gitignore`. The comment about being "regenerated" is incorrect — it's generated by cargo but should be committed.

---

## Summary Recommendations

### Immediate Actions

| Action | Item | Reason |
|--------|------|--------|
| **REMOVE** | `test_arc` binary | Generated artifact, shouldn't be in repo |
| **REMOVE** | `config-archive/` | Stale Playwright configs, no longer needed |
| **ARCHIVE** | `demo.html`, `demo-society-dashboard.html` | Stale untracked demos |
| **TRACK** | `Cargo.lock` | CI needs it for caching; reproducible builds |
| **ARCHIVE** | WOR-284-\* files if closed | Completed work order |
| **ARCHIVE** | WOR-245, WOR-393 STATUS files | Older completed work orders |

### Gitignore Adjustments

Current `.gitignore` correctly ignores most things. Minor improvements:
- `Cargo.lock` should be **removed from gitignore** (track it)
- `test_arc` is already correctly ignored

### .gitignore Correctly Set

- `target/` — Rust build artifacts ✓
- `dist/` — Comment says "Build artifacts" but it's tracked... consider clarifying
- `node_modules/` — npm deps ✓
- `playwright-report/`, `test-results/`, `screenshots/`, `qa-reports/` ✓
- `playwright*.config.ts`, `smoke*.config.ts`, `wor*-*.config.ts` ✓
