#!/bin/bash

# Parallel Linting Script
#
# This script demonstrates running linters in parallel by calling the linter binary
# with individual file type arguments. Each linter runs in a separate process.
#
# Usage: ./scripts/lint-parallel.sh
#
# This approach provides parallel execution without modifying the linter binary,
# but has trade-offs:
#
# Pros:
# - No code changes required - uses existing CLI interface
# - Simple to implement and understand
# - Real parallel execution with performance gains
# - Easy to adjust grouping strategy
#
# Cons:
# - Output may be interleaved (mixed messages from different linters)
# - Multiple Rust compilation/startup overhead if binary not cached
# - Less control over output formatting
# - Harder to aggregate errors cleanly

set -e

echo "========================================"
echo "Running Linters in Parallel"
echo "========================================"
echo ""

# Build the linter binary once (release mode for better performance)
echo "Building linter binary..."
cargo build --release --bin linter --quiet
LINTER_BIN="./target/release/linter"

echo ""

# Track overall success/failure
FAILED=0

# Temporary directory for storing results
RESULT_DIR=$(mktemp -d)
trap 'rm -rf "$RESULT_DIR"' EXIT

# Function to run a linter and capture its exit code
run_linter() {
    local linter_name=$1
    local result_file="$RESULT_DIR/$linter_name.result"
    
    echo "Starting $linter_name linter..."
    
    if "$LINTER_BIN" "$linter_name" > "$result_file.log" 2>&1; then
        echo "0" > "$result_file"
        echo "✓ $linter_name completed successfully"
    else
        echo "1" > "$result_file"
        echo "✗ $linter_name failed"
    fi
}

# Start timing
START_TIME=$(date +%s)

# Group 1: Run linters in parallel (different file types - no conflicts)
echo "Group 1: Running linters in parallel (markdown, yaml, toml, shellcheck, rustfmt)..."
echo ""

run_linter "markdown" &
PID_MARKDOWN=$!

run_linter "yaml" &
PID_YAML=$!

run_linter "toml" &
PID_TOML=$!

run_linter "shellcheck" &
PID_SHELLCHECK=$!

run_linter "rustfmt" &
PID_RUSTFMT=$!

# Wait for Group 1 to complete
wait $PID_MARKDOWN $PID_YAML $PID_TOML $PID_SHELLCHECK $PID_RUSTFMT

echo ""
echo "Group 1 completed."
echo ""

# Group 2: Run clippy sequentially (may conflict with rustfmt on .rs files)
echo "Group 2: Running clippy (sequential)..."
echo ""

run_linter "clippy"

echo ""
echo "Group 2 completed."
echo ""

# Separate group: Run cspell (checks all files, read-only)
echo "Running cspell (read-only checker)..."
echo ""

run_linter "cspell"

echo ""

# Calculate elapsed time
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))

# Display results
echo ""
echo "========================================"
echo "Linting Results"
echo "========================================"
echo ""

for linter in markdown yaml toml shellcheck rustfmt clippy cspell; do
    result_file="$RESULT_DIR/$linter.result"
    log_file="$RESULT_DIR/$linter.result.log"
    
    if [ -f "$result_file" ]; then
        exit_code=$(cat "$result_file")
        if [ "$exit_code" -eq 0 ]; then
            echo "✓ $linter: PASSED"
        else
            echo "✗ $linter: FAILED"
            FAILED=1
            
            # Show the error log
            if [ -f "$log_file" ]; then
                echo "  Log:"
                sed 's/^/    /' "$log_file"
            fi
        fi
    fi
done

echo ""
echo "========================================"
echo "Total time: ${ELAPSED}s"
echo "========================================"

# Exit with error if any linter failed
if [ $FAILED -eq 1 ]; then
    echo ""
    echo "❌ Some linters failed. Please fix the issues above."
    exit 1
else
    echo ""
    echo "✅ All linters passed!"
    exit 0
fi
