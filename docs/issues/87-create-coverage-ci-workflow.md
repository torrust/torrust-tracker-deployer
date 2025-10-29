# Create Coverage CI Workflow

**Issue**: #87
**Parent Epic**: #85 - Coverage & Reporting EPIC
**Related**: [85-epic-coverage-and-reporting.md](./85-epic-coverage-and-reporting.md)

## Overview

Create a GitHub Actions workflow to generate code coverage reports on every push to any branch. The workflow should generate coverage in two formats: a text summary for immediate viewing in the workflow output, and an HTML report uploaded as an artifact for detailed inspection.

**Note**: This issue does NOT include Codecov or other external coverage service integration. The focus is on generating and storing coverage reports within GitHub Actions artifacts.

## Goals

- [ ] Generate coverage reports automatically in CI/CD for all branches
- [ ] Display coverage summary in workflow output (text format)
- [ ] Make detailed HTML coverage reports available as artifacts
- [ ] Add `.coverage/` directory to `.gitignore`

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (CI/CD)
**Module Path**: `.github/workflows/coverage.yml`
**Pattern**: GitHub Actions workflow

### Module Structure Requirements

- [ ] Follow existing GitHub Actions workflow patterns in the project
- [ ] Use consistent naming conventions for workflows and jobs
- [ ] Leverage existing Rust toolchain setup if available

### Architectural Constraints

- [ ] Workflow must run on push to any branch
- [ ] Must generate text summary (visible in workflow output)
- [ ] Must generate HTML report (uploaded as artifact)
- [ ] Should use cargo-llvm-cov for coverage generation
- [ ] Must run on Ubuntu (latest stable)
- [ ] Must add `.coverage/` to `.gitignore`

### Anti-Patterns to Avoid

- ❌ Duplicating toolchain setup (reuse existing setup actions)
- ❌ Hardcoding coverage thresholds in CI (use `cargo cov-check` instead)
- ❌ Blocking PRs on coverage failures (coverage is informational)
- ❌ Forgetting to add `.coverage/` to `.gitignore`

## Specifications

### Workflow Trigger Events

```yaml
on:
  push:
  pull_request:
```

**Note**: Following the pattern from `testing.yml`, we trigger on both push and pull_request events without branch restrictions. This ensures coverage is generated for all development work.

### Job Structure

The workflow should have a single job: `coverage`

#### Steps Required

1. **Checkout code** - Use `actions/checkout@v4` with id `checkout`
2. **Setup Rust toolchain** - Use `dtolnay/rust-toolchain@stable` with id `setup`
3. **Enable caching** - Use `Swatinem/rust-cache@v2` with id `cache`
4. **Install cargo-llvm-cov** - Use `taiki-e/install-action@v2` with id `install-llvm-cov`
5. **Generate text coverage summary** - `cargo cov` with id `coverage-text` (outputs to stdout)
6. **Generate HTML report** - `cargo cov-html` with id `coverage-html` (outputs to `target/llvm-cov/html/`)
7. **Upload HTML artifact** - Use `actions/upload-artifact@v4` with id `upload-coverage` to make HTML report downloadable from workflow runs

**Note**: Following the pattern from `testing.yml`, all steps have both an `id` and a `name` field for clear identification.

### Coverage Formats

#### Text Summary (for workflow output)

```bash
cargo llvm-cov --all-features --workspace
```

- **Use case**: Immediate visibility in workflow logs
- **Output**: stdout (displayed in workflow run)
- **Alias**: `cargo cov`

#### HTML Report (for detailed inspection)

```bash
cargo llvm-cov --all-features --workspace --html
```

- **Use case**: Detailed, browsable coverage report
- **Output**: `target/llvm-cov/html/index.html`
- **Alias**: `cargo cov-html`

### Artifact Upload

Upload the HTML coverage report as a workflow artifact:

```yaml
- name: Upload HTML coverage report
  uses: actions/upload-artifact@v4
  with:
    name: coverage-html-report
    path: target/llvm-cov/html/
    retention-days: 30
```

### .gitignore Update

Add the `.coverage/` directory to `.gitignore`:

```gitignore
# Coverage reports
.coverage/
```

**Note**: The `target/` directory is typically already in `.gitignore` for Rust projects, so `target/llvm-cov/` is already excluded.

### Example Workflow File

Location: `.github/workflows/coverage.yml`

```yaml
name: Coverage

on:
  push:
  pull_request:

env:
  CARGO_TERM_COLOR: always

jobs:
  coverage:
    name: Coverage Report
    runs-on: ubuntu-latest

    steps:
      - id: checkout
        name: Checkout Repository
        uses: actions/checkout@v4

      - id: setup
        name: Setup Toolchain
        uses: dtolnay/rust-toolchain@stable

      - id: cache
        name: Enable Workflow Cache
        uses: Swatinem/rust-cache@v2

      - id: install-llvm-cov
        name: Install cargo-llvm-cov
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov

      - id: coverage-text
        name: Generate Text Coverage Summary
        run: cargo cov

      - id: coverage-html
        name: Generate HTML Coverage Report
        run: cargo cov-html

      - id: upload-coverage
        name: Upload HTML Coverage Report
        uses: actions/upload-artifact@v4
        with:
          name: coverage-html-report
          path: target/llvm-cov/html/
          retention-days: 30
```

**Key Patterns from testing.yml**:

- Uses `dtolnay/rust-toolchain@stable` instead of `actions-rust-lang/setup-rust-toolchain`
- Includes `Swatinem/rust-cache@v2` for dependency caching
- Uses `taiki-e/install-action@v2` for tool installation (faster and more reliable than `cargo install`)
- Sets `CARGO_TERM_COLOR: always` environment variable
- All steps have both `id` and `name` fields
- Triggers on both `push` and `pull_request` events

## Implementation Plan

### Phase 1: Update .gitignore (5 minutes)

- [ ] Open `.gitignore` file
- [ ] Add `.coverage/` directory to ignore list
- [ ] Add comment: `# Coverage reports`
- [ ] Save and verify `target/` is already in `.gitignore`

### Phase 2: Create Basic Workflow (15 minutes)

- [ ] Create `.github/workflows/coverage.yml` file
- [ ] Add workflow trigger for push and pull_request events
- [ ] Add `env: CARGO_TERM_COLOR: always` at workflow level
- [ ] Set up basic job structure with Ubuntu runner
- [ ] Add checkout step with `id: checkout`

### Phase 3: Configure Rust Toolchain (10 minutes)

- [ ] Add Rust toolchain setup step using `dtolnay/rust-toolchain@stable` with `id: setup`
- [ ] Add caching with `Swatinem/rust-cache@v2` and `id: cache`
- [ ] Install cargo-llvm-cov using `taiki-e/install-action@v2` with `id: install-llvm-cov`

### Phase 4: Generate Coverage Reports (15 minutes)

- [ ] Add step to generate text summary (`cargo cov`) with `id: coverage-text`
- [ ] Add step to generate HTML report (`cargo cov-html`) with `id: coverage-html`
- [ ] Verify text output appears in workflow logs
- [ ] Verify HTML is generated in `target/llvm-cov/html/`

### Phase 5: Upload HTML Artifact (10 minutes)

- [ ] Add artifact upload step for HTML report only with `id: upload-coverage`
- [ ] Set name to `coverage-html-report`
- [ ] Set retention period to 30 days
- [ ] Test artifact availability after workflow run

### Phase 6: Testing (15 minutes)

- [ ] Push workflow to repository
- [ ] Trigger workflow by creating a test commit on a feature branch or PR
- [ ] Verify text coverage summary appears in workflow output
- [ ] Download and inspect HTML coverage artifact
- [ ] Verify workflow doesn't block on coverage failures
- [ ] Test on main branch to ensure it runs correctly

## Acceptance Criteria

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `.coverage/` directory added to `.gitignore`
- [ ] Workflow file exists at `.github/workflows/coverage.yml`
- [ ] Workflow triggers on push to any branch (`branches: ['**']`)
- [ ] Workflow generates text coverage summary (visible in workflow output)
- [ ] Workflow generates HTML coverage report
- [ ] HTML coverage artifact is uploaded and available for download (30-day retention)
- [ ] Workflow does NOT block CI/CD on coverage failures
- [ ] Workflow runs successfully on test commit (any branch)
- [ ] Coverage text summary is readable in workflow logs
- [ ] HTML report is accessible and browsable after download

## Related Documentation

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Pre-commit Process](../contributing/commit-process.md) - Pre-commit workflow

## Notes

- **No external integrations**: This issue does NOT include Codecov, Coveralls, or other external coverage services
- The workflow is non-blocking by design - coverage failures don't fail CI/CD
- Running on all branches enables coverage tracking for feature development
- Text summary in workflow output provides immediate feedback without downloading artifacts
- HTML report provides detailed line-by-line coverage analysis for deeper inspection
- Artifact retention is set to 30 days as a reasonable balance between storage and accessibility
- The workflow uses the `cov` and `cov-html` aliases from `.cargo/config.toml` for consistency
- `.coverage/` directory must be in `.gitignore` to avoid committing temporary files
- Future enhancement: External coverage service integration can be added later if needed
