# WOR-312 Smoke Test - BLOCKED

## Issue
WOR-312 Smoke Test cannot proceed due to build infrastructure issues.

## Findings

### 1. API Smoke Test Script Exists
- `api_smoke_tests.py` contains 20 test cases (TC-API-001 through TC-API-020)
- Tests cover all `/api/v1` endpoints: health, worlds CRUD, generation, map, timeline, events, history, figures, societies, planet, tectonics, artifacts, cataclysms, wonders
- Tests require server running at `http://localhost:8080`

### 2. Build Environment Issues

#### Problem A: Cargo Lock File Version Incompatibility
```
error: failed to parse lock file at: /build/Cargo.lock
Caused by: lock file version `4` was found, but this version of Cargo does not understand this lock file
```
- The main `Dockerfile` uses `rust:1.75` which has older Cargo
- Current `Cargo.lock` is version 4 (from newer Rust toolchain)
- **Resolution**: Update Dockerfile to use a newer Rust image (e.g., `rust:latest`)

#### Problem B: Missing Cargo/Rust Toolchain
- Local machine has no `cargo` or `rustc` installed
- Cannot build locally without Docker

#### Problem C: Example File Missing in Docker Context
```
error: can't find example `resource_spawning` at path `/workspace/examples/resource_spawning.rs`
```
- `Dockerfile.test` copies `./tests` but not `./examples`
- The `Cargo.toml` references `examples/resource_spawning.rs` which doesn't exist in Docker context

### 3. Server Binary Status
- `./target/release/world_generator` exists but was built without `--features api`
- `./target/debug/world_generator` exists but also built without `--features api`
- Both binaries print: `Error: API feature not enabled. Rebuild with --features api`

## Blocker Details

| Blocker | Owner | Action Required |
|---------|-------|-----------------|
| Dockerfile uses old Rust image | Coder/Systems | Update `Dockerfile` FROM rust:1.75 to rust:latest |
| Cargo.lock version 4 incompatible | Coder/Systems | Update Rust image in Dockerfile |
| No pre-built API binary | Coder | Build and publish image with API feature |
| Example path mismatch | Coder | Fix Dockerfile.test to copy examples directory |

## Next Actions

1. **For CTO/Systems**: Update `Dockerfile` to use `rust:latest` instead of `rust:1.75`
2. **For Coder**: Rebuild Docker image with `docker compose build`
3. **For QA**: Re-trigger smoke test after image is published

## Test Script Ready
The `api_smoke_tests.py` is complete and ready to execute once a working server is available.
