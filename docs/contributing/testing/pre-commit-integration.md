# Pre-commit Integration Testing

The project includes integration tests that validate all components of the pre-commit script to ensure they work correctly in any environment (including GitHub Copilot's environment).

## How It Works

**By default, `cargo test` runs expensive integration tests** that validate:

- **Dependency check**: `cargo-machete` for unused dependencies
- **Linting**: `cargo run --bin linter all` for code quality
- **Documentation**: `cargo doc` for documentation builds
- **E2E tests**: `cargo run --bin e2e-config-tests` and `cargo run --bin e2e-provision-and-destroy-tests` for end-to-end validation

These tests ensure that when someone runs `./scripts/pre-commit.sh`, all the tools and dependencies are available and working.

## Skipping Expensive Tests During Development

If you need faster test cycles during development, you can skip the AI enforcement tests:

```bash
# Skip AI enforcement tests
SKIP_AI_ENFORCEMENT=1 cargo test
```

**Default Behavior**: AI enforcement tests **run by default** when `SKIP_AI_ENFORCEMENT` is not set. This ensures AI assistants like GitHub Copilot always validate quality requirements.

**When to skip**:

- ✅ Rapid development cycles where you're running tests frequently
- ✅ Working on isolated code that doesn't affect pre-commit tools
- ✅ CI environments that run pre-commit checks separately

**When NOT to skip**:

- ❌ Before creating a PR (let the full tests run at least once)
- ❌ When modifying anything that could affect linting, dependencies, or documentation
- ❌ When testing in a new environment or after dependency changes

## Why This Approach

This integration testing strategy helps with:

- **✅ Environment validation**: Catches missing tools or configuration issues early
- **✅ Copilot compatibility**: Ensures GitHub Copilot's environment has all necessary dependencies
- **✅ Fast feedback**: Developers see pre-commit issues during normal test cycles
- **✅ Flexible development**: Can be disabled when needed for faster iteration

## Running Tests

```bash
# Default: Run all tests including AI enforcement checks
cargo test

# Fast development: Skip AI enforcement
SKIP_AI_ENFORCEMENT=1 cargo test

# Explicitly run only AI enforcement tests
cargo test ai_enforcement
```

This makes the test suite more readable, maintainable, and reliable for all contributors.

## 🤖 AI Assistant Integration

The project includes a dedicated test file `tests/ai_enforcement.rs` that ensures AI assistants (like GitHub Copilot) run all necessary quality checks before committing code.

### Purpose

AI assistants often work in remote environments where they don't have access to local Git hooks or pre-commit scripts. These integration tests force the AI to validate all quality checks during the normal test execution.
