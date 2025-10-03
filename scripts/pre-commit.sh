#!/bin/bash
# Pre-commit verification script
# Run all mandatory checks before committing changes

set -euo pipefail

echo "🔍 Running pre-commit checks..."
echo

# 1. Check for unused dependencies
echo "1️⃣ Checking for unused dependencies (cargo machete)..."
cargo machete
echo "✅ No unused dependencies found"
echo

# 2. Run all linters (comprehensive - stable & nightly toolchains)
echo "2️⃣ Running linters..."
cargo run --bin linter all
echo "✅ All linters passed"
echo

# 3. Run tests
echo "3️⃣ Running tests..."
cargo test
echo "✅ All tests passed"
echo

# 4. Test cargo docs
echo "4️⃣ Testing cargo documentation..."
cargo doc --no-deps --bins --examples --workspace --all-features
echo "✅ Documentation builds successfully"
echo

# 5. Run comprehensive E2E tests
echo "5️⃣ Running comprehensive E2E tests..."
cargo run --bin e2e-tests-full
echo "✅ All E2E tests passed"
echo

echo "✅ All pre-commit checks passed successfully!"
echo "You can now safely stage and commit your changes."
