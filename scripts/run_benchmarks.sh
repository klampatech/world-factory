#!/bin/bash
# Run performance benchmarks and collect results
# Usage: ./run_benchmarks.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "=== World Factory Performance Benchmarks ==="
echo ""

# Check if cargo bench is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Install Rust to run benchmarks."
    exit 1
fi

# Create benchmark results directory
RESULTS_DIR="target/criterion"
mkdir -p "$RESULTS_DIR"

# Run cargo bench
echo "Running cargo benchmarks..."
cargo bench --no-run 2>/dev/null || {
    echo "Warning: Benchmarks may not be configured"
    echo "Add benchmarks to src/ using #[bench] attribute"
}

# If criterion reports exist, summarize them
if [ -d "$RESULTS_DIR" ]; then
    echo ""
    echo "=== Benchmark Results ==="
    
    for report in "$RESULTS_DIR"/**/new/*; do
        if [ -f "$report" ]; then
            echo "Report: $report"
        fi
    done
    
    # Look for benchmark summaries
    if command -v cargo &> /dev/null; then
        echo ""
        echo "Running benchmark comparison..."
        cargo bench 2>&1 | tail -20 || true
    fi
fi

echo ""
echo "Benchmark results saved to: $RESULTS_DIR"
echo "Upload to CI artifacts."