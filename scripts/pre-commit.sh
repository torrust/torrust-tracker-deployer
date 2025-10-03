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

declare -a STEPS=(
    "Checking for unused dependencies (cargo machete)|No unused dependencies found|||cargo machete"
    "Running linters|All linters passed|||cargo run --bin linter all"
    "Running tests|All tests passed|||cargo test"
    "Testing cargo documentation|Documentation builds successfully|||cargo doc --no-deps --bins --examples --workspace --all-features"
    "Running comprehensive E2E tests|All E2E tests passed|(Filtering logs to WARNING level and above - this may take a few minutes)|RUST_LOG=warn|cargo run --bin e2e-tests-full"
)

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
