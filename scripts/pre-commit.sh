#!/bin/bash
# Pre-commit verification script
# Run all mandatory checks before committing changes

set -euo pipefail

# ============================================================================
# CONFIGURATION: Define all pre-commit check steps
# ============================================================================
# Each step is defined as: "description|success_message|special_note|env_vars|command"
# - description: What the step does
# - success_message: Message shown on success
# - special_note: Optional note shown before running (use empty string if none)
# - env_vars: Optional environment variables (use empty string if none)
# - command: The command to execute
# ============================================================================

# Determine which steps to run based on environment
# When TORRUST_TD_SKIP_SLOW_TESTS=true (set for Copilot agent), skip slow tests to avoid timeout issues
# Slow tests include: E2E tests (~1m 32s) and code coverage (~1m 29s)
if [ "${TORRUST_TD_SKIP_SLOW_TESTS:-false}" = "true" ]; then
    echo "⚠️  Running in fast mode (skipping slow tests)"
    echo ""
    echo "The following tests are SKIPPED to stay within the 5-minute timeout limit:"
    echo "  • E2E provision and destroy tests (~44 seconds)"
    echo "  • E2E configuration tests (~48 seconds)"
    echo ""
    echo "💡 These tests will run automatically in CI after PR creation."
    echo "Note: Code coverage is also checked automatically in CI."
    echo ""
    echo "If you want to run them manually before committing, use these commands:"
    echo "  cargo run --bin e2e-provision-and-destroy-tests  # ~44s"
    echo "  cargo run --bin e2e-config-and-release-tests     # ~48s"
    echo "  cargo cov-check                                  # For coverage check"
    echo ""
    echo "Fast mode execution time: ~2 minutes 30 seconds"
    echo ""
    
    declare -a STEPS=(
        "Checking for unused dependencies (cargo machete)|No unused dependencies found|||cargo machete"
        "Running linters|All linters passed|||cargo run --bin linter all"
        "Running tests|All tests passed|||cargo test"
        "Testing cargo documentation|Documentation builds successfully|||cargo doc --no-deps --bins --examples --workspace --all-features"
    )
else
    declare -a STEPS=(
        "Checking for unused dependencies (cargo machete)|No unused dependencies found|||cargo machete"
        "Running linters|All linters passed|||cargo run --bin linter all"
        "Running tests|All tests passed|||cargo test"
        "Testing cargo documentation|Documentation builds successfully|||cargo doc --no-deps --bins --examples --workspace --all-features"
        "Running E2E provision and destroy tests|Provision and destroy tests passed|(Testing infrastructure lifecycle - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-provision-and-destroy-tests"
        "Running E2E configuration and release tests|Configuration and release tests passed|(Testing software installation, configuration, and release)|RUST_LOG=warn|cargo run --bin e2e-config-and-release-tests"
    )
fi

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

# Function to format elapsed time
format_time() {
    local total_seconds=$1
    local minutes=$((total_seconds / 60))
    local seconds=$((total_seconds % 60))
    if [ $minutes -gt 0 ]; then
        echo "${minutes}m ${seconds}s"
    else
        echo "${seconds}s"
    fi
}

# Function to format elapsed time

# Function to run a step with timing
run_step() {
    local step_number=$1
    local total_steps=$2
    local description=$3
    local success_message=$4
    local special_note=$5
    local env_vars=$6
    local command=$7

    echo "[Step $step_number/$total_steps] $description..."
    if [ -n "$special_note" ]; then
        echo "           $special_note"
        echo
    fi
    
    local step_start=$SECONDS
    if [ -n "$env_vars" ]; then
        env "$env_vars" bash -c "$command"
    else
        eval "$command"
    fi
    local step_elapsed=$((SECONDS - step_start))
    
    if [ -n "$special_note" ]; then
        echo
    fi
    echo "PASSED: $success_message ($(format_time $step_elapsed))"
    echo
}

# Trap errors and show failure message
trap 'echo ""; echo "=========================================="; echo "FAILED: Pre-commit checks failed!"; echo "Fix the errors above before committing."; echo "=========================================="; exit 1' ERR

# ============================================================================
# MAIN EXECUTION
# ============================================================================

# Record total start time
TOTAL_START=$SECONDS
TOTAL_STEPS=${#STEPS[@]}

echo "Running pre-commit checks..."
echo

# Execute all steps
for i in "${!STEPS[@]}"; do
    IFS='|' read -r description success_message special_note env_vars command <<< "${STEPS[$i]}"
    run_step $((i + 1)) "$TOTAL_STEPS" "$description" "$success_message" "$special_note" "$env_vars" "$command"
done

# Calculate and display total time
TOTAL_ELAPSED=$((SECONDS - TOTAL_START))
echo "=========================================="
echo "SUCCESS: All pre-commit checks passed!"
echo "Total time: $(format_time $TOTAL_ELAPSED)"
echo "=========================================="
echo
echo "You can now safely stage and commit your changes."
