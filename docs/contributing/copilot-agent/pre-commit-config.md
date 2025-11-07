# Setting up TORRUST_TD_SKIP_SLOW_TESTS for GitHub Copilot Agent

This document explains how to configure the `TORRUST_TD_SKIP_SLOW_TESTS` environment variable for GitHub Copilot coding agent to skip slow tests during pre-commit checks.

## Why This Is Needed

GitHub Copilot coding agent has a hardcoded ~5-6 minute timeout for command execution. Our full pre-commit verification (including E2E tests and coverage checks) takes ~5.5 minutes, causing the agent to timeout and retry infinitely.

**Related Issues:**

- GitHub Issue: #121
- Community Discussion: https://github.com/orgs/community/discussions/178998

## Solution

We use an environment variable (`TORRUST_TD_SKIP_SLOW_TESTS=true`) to skip slow tests (E2E tests and code coverage) when Copilot agent runs pre-commit checks. This keeps checks under the timeout limit while maintaining full verification for local development.

**Note:** The variable name follows the project's environment variable naming convention documented in [Environment Variable Prefix ADR](../../decisions/environment-variable-prefix.md). All Torrust Tracker Deployer environment variables use the `TORRUST_TD_` prefix.

## Pre-commit Timing Breakdown

Understanding where time is spent helps explain why we skip certain tests for the agent:

### Individual Task Timings

| Task          | Time        | % of Total | Category | Skipped in Fast Mode? |
| ------------- | ----------- | ---------- | -------- | --------------------- |
| cargo machete | 0.08s       | 0.04%      | Instant  | ❌ No                 |
| All linters   | 18.75s      | 5.7%       | Fast     | ❌ No                 |
| Unit tests    | 1m 16s      | 22.9%      | Medium   | ❌ No                 |
| cargo doc     | 44s         | 13.4%      | Medium   | ❌ No                 |
| E2E provision | 44s         | 13.4%      | Medium   | ✅ **Yes**            |
| E2E config    | 48s         | 14.4%      | Medium   | ✅ **Yes**            |
| Coverage      | 1m 29s      | 26.9%      | Slowest  | ✅ **Yes**            |
| **TOTAL**     | **~5m 30s** | **100%**   | -        | -                     |

**Fast Mode Total: ~3m 48s** (31% time reduction, ~2 minute safety margin below timeout)

### Unit Tests Breakdown (cargo test)

The unit tests (`cargo test`) complete in **1m 16s** and include:

| Test Suite             | Time        | Tests    | Description                          |
| ---------------------- | ----------- | -------- | ------------------------------------ |
| Unit tests (lib)       | 12.24s      | 1200     | Core library unit tests              |
| e2e_create_command     | 13.45s      | 4        | End-to-end create command workflow   |
| e2e_destroy_command    | 0.65s       | 4        | End-to-end destroy command workflow  |
| file_lock_multiprocess | 6.05s       | 8        | Multi-process file locking tests     |
| logging_integration    | 0.13s       | 11       | Logging system integration tests     |
| ssh_client_integration | 11.31s      | 9        | SSH client integration tests         |
| template_integration   | 0.01s       | 4        | Template rendering integration tests |
| Doc tests              | 15.44s      | 289      | Documentation example tests          |
| **TOTAL**              | **~1m 16s** | **1529** | All test suites                      |

**Key Insight:** Even though unit tests take 1m 16s (22.9% of total), they're **NOT skipped** in fast mode because:

- They validate correctness across the entire codebase
- Many are fast unit tests (12.24s for 1200 tests)
- Integration tests provide critical coverage
- Doc tests ensure documentation examples work

**What We Skip:** E2E tests (1m 32s) and coverage (1m 29s) are skipped because:

- They're the slowest checks (54.8% of total time combined)
- E2E tests run in CI workflows after PR creation
- Coverage runs in CI and provides informational metrics
- Skipping them provides ~3 minute time savings

## How to Configure

### Step 1: Navigate to Repository Settings

1. Go to your GitHub repository: https://github.com/torrust/torrust-tracker-deployer
2. Click **Settings** (requires admin access)
3. In the left sidebar, click **Environments**

### Step 2: Configure the Copilot Environment

1. Click on the `copilot` environment (it should already exist)
2. Scroll down to **Environment variables**
3. Click **Add environment variable**

### Step 3: Add the Variable

1. **Name**: `TORRUST_TD_SKIP_SLOW_TESTS`
2. **Value**: `true`
3. Click **Add variable**

## How It Works

### For Local Development (Full Verification)

When developers run `./scripts/pre-commit.sh` locally:

- `TORRUST_TD_SKIP_SLOW_TESTS` is **not set** (defaults to `false`)
- All checks run, including E2E tests and coverage (~5.5 minutes)
- Full quality verification maintained

### For Copilot Agent (Fast Verification)

When Copilot agent runs the pre-commit hook:

- `TORRUST_TD_SKIP_SLOW_TESTS=true` is injected from the `copilot` environment
- E2E tests and coverage checks are **skipped** (~3.8 minutes total)
- CI workflows will still run all tests after the PR is created

## Verification

To verify the configuration is working:

1. Check that the variable exists in Settings > Environments > copilot
2. Wait for Copilot agent to create a PR
3. Check the session logs - you should see: `⚠️  Running in fast mode (skipping slow tests)`

## What Gets Skipped

When `TORRUST_TD_SKIP_SLOW_TESTS=true`:

**Skipped (saves ~3 minutes):**

- ❌ E2E provision and destroy tests (~44s)
- ❌ E2E configuration tests (~48s)
- ❌ Code coverage check (~1m 29s)

**Still runs (maintains quality):**

- ✅ Cargo machete - unused dependencies (~0.08s)
- ✅ All linters - markdown, YAML, Rust, shellcheck (~19s)
- ✅ Unit tests - 1529 tests across all suites (~1m 16s)
- ✅ Cargo documentation build (~44s)

**Total time: ~3m 48s** (vs ~5m 30s in full mode)

## CI Safety Net

Even though slow tests are skipped in pre-commit for Copilot agent, they still run:

- In GitHub Actions workflows on PR creation
- In the full CI pipeline before merging

This ensures no regressions slip through while keeping Copilot agent functional.

## Running Skipped Tests Manually

The pre-commit script will inform you about skipped tests and provide commands to run them:

**For AI agents:** You can run these commands separately (each completes in < 5 minutes):

```bash
# Run E2E provision tests (~44s)
cargo run --bin e2e-provision-and-destroy-tests

# Run E2E config tests (~48s)
cargo run --bin e2e-config-tests

# Run coverage check (~1m 29s)
cargo cov-check
```

**For developers:** Test the fast mode locally:

```bash
TORRUST_TD_SKIP_SLOW_TESTS=true ./scripts/pre-commit.sh
```

To run the full verification locally (default):

```bash
./scripts/pre-commit.sh
```

## References

- [GitHub Docs: Setting environment variables in Copilot's environment](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/coding-agent/customize-the-agent-environment#setting-environment-variables-in-copilots-environment)
- [Community Discussion: Copilot timeout issue](https://github.com/orgs/community/discussions/178998)
