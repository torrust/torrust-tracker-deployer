#!/bin/bash
# Run linters with both stable and nightly Rust toolchains

set -euo pipefail

echo "Testing with stable toolchain..."
rustup run stable cargo run --bin linter all

echo "Testing with nightly toolchain..."
rustup run nightly cargo run --bin linter all

echo "All linting tests passed!"
