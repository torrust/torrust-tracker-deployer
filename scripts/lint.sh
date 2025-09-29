#!/bin/bash
# Run linters with both stable and nightly Rust toolchains
# This script prevents build artifact conflicts between toolchains by cleaning between runs

set -euo pipefail

# Function to display current toolchain information
show_toolchain_info() {
    local toolchain=$1
    echo "============================================"
    echo "🔧 Current Toolchain: $toolchain"
    echo "============================================"
    rustup run "$toolchain" rustc --version
    rustup run "$toolchain" cargo --version
    echo "Target directory: $(pwd)/target"
    echo "============================================"
    echo
}

echo "🧪 Running linters with both stable and nightly Rust toolchains"
echo

echo "Testing with stable toolchain..."
show_toolchain_info "stable"
rustup run stable cargo run --bin linter all

echo
echo "🧹 Cleaning build artifacts to prevent toolchain conflicts..."
cargo clean
echo "Build artifacts cleaned successfully"
echo

echo "Testing with nightly toolchain..."
show_toolchain_info "nightly"
rustup run nightly cargo run --bin linter all

echo
echo "✅ All linting tests passed!"
