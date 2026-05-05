# justfile - Developer-friendly task runner for World Factory
# Install just: https://github.com/casey/just#installation
# Or: cargo install just

# Default recipe
default: test

# Run Rust tests
test:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo test -- --nocapture; \
    elif command -v docker >/dev/null 2>&1; then \
        echo "cargo not found, using Docker..."; \
        docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo test -- --nocapture; \
    else \
        echo "ERROR: Neither cargo nor docker is available."; \
        echo ""; \
        echo "To run Rust tests, you need one of:"; \
        echo "1. Install Rust (recommended):"; \
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; \
        echo "   Source ~/.cargo/env after installation"; \
        echo ""; \
        echo "2. Use Docker:"; \
        echo "   docker build -f Dockerfile.test -t world-factory:test ."; \
        echo "   docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo test"; \
        echo ""; \
        echo "3. Install just (task runner):"; \
        echo "   cargo install just"; \
        exit 1; \
    fi

# Run only unit tests
test-unit:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo test --lib -- --nocapture; \
    elif command -v docker >/dev/null 2>&1; then \
        docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo test --lib -- --nocapture; \
    else \
        echo "ERROR: Neither cargo nor docker available. Install Rust or Docker."; \
        exit 1; \
    fi

# Run integration tests
test-integration:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo test --test integration_world_generation -- --nocapture; \
        cargo test --test phase1_integration_test -- --nocapture; \
        cargo test --test phase2_integration_test -- --nocapture; \
    elif command -v docker >/dev/null 2>&1; then \
        docker run --rm -v $(pwd):/workspace -w /workspace rust:latest \
            sh -c "cargo test --test integration_world_generation -- --nocapture && \
                   cargo test --test phase1_integration_test -- --nocapture && \
                   cargo test --test phase2_integration_test -- --nocapture"; \
    else \
        echo "ERROR: Neither cargo nor docker available. Install Rust or Docker."; \
        exit 1; \
    fi

# Build the project
build:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo build; \
    elif command -v docker >/dev/null 2>&1; then \
        docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo build; \
    else \
        echo "ERROR: Neither cargo nor docker available. Install Rust or Docker."; \
        exit 1; \
    fi

# Run clippy lints
lint:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo clippy --all-targets --all-features -- -D warnings; \
    elif command -v docker >/dev/null 2>&1; then \
        docker run --rm -v $(pwd):/workspace -w /workspace rust:latest \
            sh -c "cargo clippy --all-targets --all-features -- -D warnings || true"; \
    else \
        echo "ERROR: Neither cargo nor docker available. Install Rust or Docker."; \
        exit 1; \
    fi

# Format code
fmt:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo fmt --all; \
    elif command -v docker >/dev/null 2>&1; then \
        docker run --rm -v $(pwd):/workspace -w /workspace rust:latest cargo fmt --all; \
    else \
        echo "ERROR: Neither cargo nor docker available. Install Rust or Docker."; \
        exit 1; \
    fi

# Run all tests (alias for default)
test-all: test

# Clean build artifacts
clean:
    @if command -v cargo >/dev/null 2>&1; then \
        cargo clean; \
    else \
        rm -rf target; \
    fi
