# Stage 1: Build
FROM rust:1.85 as builder

WORKDIR /build

# Copy manifests first for better layer caching
COPY Cargo.toml ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "fn lib() {}" > src/lib.rs

# Build dependencies only
RUN cargo build --release --features api && rm -rf src

# Copy actual source
COPY src/ src/

# Build the final binary
RUN cargo build --release --features api

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tini curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from build stage
COPY --from=builder /build/target/release/world_generator /app/world-factory

# Create data directory
RUN mkdir -p /data

ENV RUST_LOG=info
ENV WORLD_FACTORY_PORT=8080
ENV WORLD_FACTORY_HOST=0.0.0.0
ENV WORLD_FACTORY_DATA_DIR=/data

# Use tini for proper process handling
ENTRYPOINT ["/usr/bin/tini", "--"]

# Expose the API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the server
CMD ["/app/world-factory", "--server", "--port", "8080"]