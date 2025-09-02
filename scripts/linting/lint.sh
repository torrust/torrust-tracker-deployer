#!/bin/bash
# Unified linting commands wrapper

set -euo pipefail

# Get the script directory to ensure relative paths work
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Function to run all linters with clean output
run_all_linters() {
    echo "🔍 Running All Linters"
    echo "======================"
    
    local FAILED=0
    
    # Run markdown linter
    echo ""
    if "${SCRIPT_DIR}/markdown.sh"; then
        echo "✅ Markdown linting: PASSED"
    else
        echo "❌ Markdown linting: FAILED"
        FAILED=1
    fi
    
    # Run YAML linter
    echo ""
    if "${SCRIPT_DIR}/yaml.sh"; then
        echo "✅ YAML linting: PASSED"
    else
        echo "❌ YAML linting: FAILED"
        FAILED=1
    fi
    
    # Run Rust clippy linter
    echo ""
    if "${SCRIPT_DIR}/clippy.sh"; then
        echo "✅ Rust clippy linting: PASSED"
    else
        echo "❌ Rust clippy linting: FAILED"
        FAILED=1
    fi
    
    # Run Rust formatter check
    echo ""
    if "${SCRIPT_DIR}/rustfmt.sh"; then
        echo "✅ Rust formatting: PASSED"
    else
        echo "❌ Rust formatting: FAILED"
        FAILED=1
    fi
    
    # Run ShellCheck linter
    echo ""
    if "${SCRIPT_DIR}/shellcheck.sh"; then
        echo "✅ Shell script linting: PASSED"
    else
        echo "❌ Shell script linting: FAILED"
        FAILED=1
    fi
    
    echo ""
    echo "======================"
    if [ $FAILED -eq 0 ]; then
        echo "🎉 All linters passed!"
        exit 0
    else
        echo "💥 Some linters failed!"
        exit 1
    fi
}

case "${1:-help}" in
    "md"|"markdown")
        echo "🔍 Running markdown linter..."
        "${SCRIPT_DIR}/markdown.sh"
        ;;
    "yaml")
        echo "🔍 Running YAML linter..."
        "${SCRIPT_DIR}/yaml.sh"
        ;;
    "clippy")
        echo "🔍 Running Rust clippy linter..."
        "${SCRIPT_DIR}/clippy.sh"
        ;;
    "rustfmt"|"fmt")
        echo "🔍 Running Rust formatter check..."
        "${SCRIPT_DIR}/rustfmt.sh"
        ;;
    "shellcheck"|"shell")
        echo "🔍 Running ShellCheck linter..."
        "${SCRIPT_DIR}/shellcheck.sh"
        ;;
    "all")
        run_all_linters
        ;;
    "help"|*)
        echo "Linting Commands:"
        echo "  ./scripts/linting/lint.sh md         - Run markdown linter"
        echo "  ./scripts/linting/lint.sh yaml       - Run YAML linter" 
        echo "  ./scripts/linting/lint.sh clippy     - Run Rust clippy linter"
        echo "  ./scripts/linting/lint.sh rustfmt    - Run Rust formatter check"
        echo "  ./scripts/linting/lint.sh shellcheck - Run ShellCheck linter"
        echo "  ./scripts/linting/lint.sh all        - Run all linters"
        echo "  ./scripts/linting/lint.sh help       - Show this help"
        echo ""
        echo "Direct script execution:"
        echo "  ./scripts/linting/markdown.sh"
        echo "  ./scripts/linting/yaml.sh"
        echo "  ./scripts/linting/clippy.sh"
        echo "  ./scripts/linting/rustfmt.sh"
        echo "  ./scripts/linting/shellcheck.sh"
        ;;
esac
