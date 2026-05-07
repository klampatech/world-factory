#!/bin/bash
# Script to check build status for WOR-67

export PATH="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH"

echo "=== Checking API module build ==="
cargo build --features api 2>&1 | grep -E "^error\[E" | head -20

echo ""
echo "=== Checking for API-specific errors ==="
cargo build --features api 2>&1 | grep -E "(api/mod.rs|api/v1/species.rs)" | head -10

echo ""
echo "=== Build summary ==="
cargo build --features api 2>&1 | tail -5