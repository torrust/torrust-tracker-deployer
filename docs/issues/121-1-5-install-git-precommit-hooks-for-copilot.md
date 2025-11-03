# Install Git Pre-Commit Hooks for Copilot Agent

**Issue**: [#121](https://github.com/torrust/torrust-tracker-deployer/issues/121)
**Parent Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution  
**Depends On**: [#120](https://github.com/torrust/torrust-tracker-deployer/issues/120) - Configure GitHub Copilot Agent Environment (Issue 1-4)  
**Related**: [Removed Integration Test Commit](https://github.com/torrust/torrust-tracker-deployer/commit/e9955b081f2f2b643949fae573955041f989bdd0)

## Overview

Install Git pre-commit hooks in the Copilot agent's environment to **enforce** pre-commit checks deterministically. This ensures the agent cannot commit code without running linting checks, even if the agent forgets or ignores the instruction in `.github/copilot-instructions.md`.

## Objectives

- [ ] Create installation script that symlinks `scripts/pre-commit.sh` to `.git/hooks/pre-commit`
- [ ] Integrate hook installation into Copilot environment setup workflow
- [ ] Test that hooks prevent commits when checks fail
- [ ] Document the enforcement mechanism

## Context

### The Problem

**Current Situation**: The `.github/copilot-instructions.md` includes this rule:

> Before committing: Always run the pre-commit verification script - all checks must pass before staging files or creating commits

**Issue**: Agents don't consistently follow this instruction because:

1. **Context dilution**: As agents work and summarize conversations, instructions get lost
2. **No enforcement**: It's a guideline, not a technical barrier
3. **Forgetfulness**: Agents may skip checks when focused on implementing features

### Previous Attempted Solution

**What was tried**: Integration test executing pre-commit checks as Rust test  
**Why it failed**: Agents ran `cargo test --lib`, skipping integration tests  
**Removed in**: [Commit e9955b0](https://github.com/torrust/torrust-tracker-deployer/commit/e9955b081f2f2b643949fae573955041f989bdd0)

### New Solution: Git Hooks

**Why Git hooks work**:

- ✅ **Deterministic**: Git **always** runs hooks before committing
- ✅ **Enforced**: Technical barrier, not just a guideline
- ✅ **Transparent**: If checks fail, commit is blocked with clear error
- ✅ **Agent-friendly**: Works regardless of agent's context or memory
- ✅ **Standard practice**: Common in development workflows

**Key insight**: If agents use `git commit`, hooks will **automatically** run pre-commit checks.

### Implementation Strategy

We need two components:

1. **Hook installer script** (`scripts/install-git-hooks.sh`) - Creates symlink from `.git/hooks/pre-commit` to `scripts/pre-commit.sh`
2. **Workflow integration** - Call installer in `.github/workflows/copilot-setup-steps.yml`

## 🏗️ Architecture Requirements

**Scripts Location**: `scripts/` directory  
**Hook Location**: `.git/hooks/pre-commit` (symlink to `scripts/pre-commit.sh`)  
**Integration Point**: `.github/workflows/copilot-setup-steps.yml` workflow

## Specifications

### Hook Installer Script (No Separate Hook File Needed)

**Key Insight**: We don't need a separate hook script! We can directly use `scripts/pre-commit.sh` as the Git hook by creating a **symbolic link**.

This is much simpler and ensures the hook always uses the latest version of the pre-commit checks.

### Hook Installer Script

Create `scripts/install-git-hooks.sh`:

```bash
#!/usr/bin/env bash
#
# Install Git hooks for this repository.
# This script creates a symbolic link from .git/hooks/pre-commit to scripts/pre-commit.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
HOOK_SOURCE="$REPO_ROOT/scripts/pre-commit.sh"
HOOK_TARGET="$REPO_ROOT/.git/hooks/pre-commit"

echo "Installing Git hooks..."

# Check if .git directory exists
if [ ! -d "$REPO_ROOT/.git" ]; then
    echo "Error: .git directory not found. Are you in a Git repository?"
    exit 1
fi

# Check if source script exists
if [ ! -f "$HOOK_SOURCE" ]; then
    echo "Error: Pre-commit script not found at $HOOK_SOURCE"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$REPO_ROOT/.git/hooks"

# Remove existing hook if present
if [ -e "$HOOK_TARGET" ]; then
    echo "Removing existing pre-commit hook..."
    rm "$HOOK_TARGET"
fi

# Create symbolic link to pre-commit script
echo "Creating symbolic link to scripts/pre-commit.sh..."
ln -s "$HOOK_SOURCE" "$HOOK_TARGET"

# Verify the hook is executable
if [ ! -x "$HOOK_SOURCE" ]; then
    echo "Warning: Making scripts/pre-commit.sh executable..."
    chmod +x "$HOOK_SOURCE"
fi

echo ""
echo "✓ Git hooks installed successfully"
echo ""
echo "The pre-commit hook is now linked to ./scripts/pre-commit.sh"
echo "Any changes to scripts/pre-commit.sh will automatically affect the Git hook."
```

**Key Features**:

- Validates `.git` directory exists
- Validates source script exists
- Creates `.git/hooks/` if needed
- Creates **symbolic link** (not copy) to `scripts/pre-commit.sh`
- Removes existing hook before creating new link
- Ensures source script is executable
- Clear success/error messages
- Idempotent (safe to run multiple times)

**Why Symlink Instead of Copy**:

- ✅ Hook always uses latest version of `scripts/pre-commit.sh`
- ✅ No need to reinstall when script changes
- ✅ Single source of truth for pre-commit logic
- ✅ Simpler maintenance

### Update Copilot Setup Workflow

Update `.github/workflows/copilot-setup-steps.yml` to add hook installation:

```yaml
name: "Copilot Setup Steps"

on:
  workflow_dispatch:
  push:
    paths:
      - .github/workflows/copilot-setup-steps.yml
  pull_request:
    paths:
      - .github/workflows/copilot-setup-steps.yml

jobs:
  copilot-setup-steps:
    runs-on: ubuntu-latest
    permissions:
      contents: read

    steps:
      - name: Checkout code
        uses: actions/checkout@v5

      - name: Set up Rust toolchain
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable

      - name: Build dependency-installer binary
        run: |
          cd packages/dependency-installer
          cargo build --release --bin dependency-installer

      - name: Install all development dependencies
        run: |
          sudo packages/dependency-installer/target/release/dependency-installer install --yes
        env:
          DEBIAN_FRONTEND: noninteractive

      - name: Verify installations
        run: |
          packages/dependency-installer/target/release/dependency-installer check

      # NEW STEP: Install Git hooks
      - name: Install Git pre-commit hooks
        run: |
          ./scripts/install-git-hooks.sh
```

## Implementation Tasks

### Verify Pre-Commit Script

- [ ] Ensure `scripts/pre-commit.sh` exists and is executable
- [ ] Verify it has proper shebang: `#!/usr/bin/env bash`
- [ ] Test it runs successfully: `./scripts/pre-commit.sh`

### Create Hook Installer Script

- [ ] Create `scripts/install-git-hooks.sh`
- [ ] Add script documentation header
- [ ] Add `set -e` for error handling
- [ ] Implement directory validation (`.git` exists)
- [ ] Implement source script validation (`scripts/pre-commit.sh` exists)
- [ ] Remove existing hook if present
- [ ] Create symbolic link: `ln -s <source> <target>`
- [ ] Ensure source script is executable
- [ ] Add clear success/error messages
- [ ] Make installer executable: `chmod +x scripts/install-git-hooks.sh`
- [ ] Test installer locally

### Update Copilot Workflow

- [ ] Open `.github/workflows/copilot-setup-steps.yml`
- [ ] Add new step: "Install Git pre-commit hooks"
- [ ] Place step **after** dependency installation
- [ ] Run `./scripts/install-git-hooks.sh`
- [ ] Ensure step runs in repository root directory

### Testing

- [ ] **Test hook locally**:

  - [ ] Run `./scripts/install-git-hooks.sh`
  - [ ] Verify symlink created: `ls -la .git/hooks/pre-commit`
  - [ ] Verify symlink points to correct file: `readlink .git/hooks/pre-commit`
  - [ ] Make a bad commit (e.g., unformatted code)
  - [ ] Verify hook blocks the commit
  - [ ] Fix issues and verify commit succeeds
  - [ ] Modify `scripts/pre-commit.sh` (e.g., add echo statement)
  - [ ] Verify hook immediately reflects changes (no reinstall needed)

- [ ] **Test in workflow**:

  - [ ] Commit changes to feature branch
  - [ ] Create pull request
  - [ ] Verify workflow runs successfully
  - [ ] Check logs show hook installation step
  - [ ] Verify "✓ Git hooks installed successfully" message

- [ ] **Test with Copilot agent** (after merge):
  - [ ] Assign a small issue to Copilot agent
  - [ ] Monitor agent's session logs
  - [ ] Verify agent cannot commit bad code
  - [ ] Verify hook provides clear error messages

### Documentation

- [ ] Add section to README about Git hooks
- [ ] Document manual installation: `./scripts/install-git-hooks.sh`
- [ ] Explain automatic installation in Copilot environment
- [ ] Add troubleshooting section for hook failures
- [ ] Document how to bypass hooks (for emergencies): `git commit --no-verify`

## Acceptance Criteria

**Hook Installer Script**:

- [ ] Script created at `scripts/install-git-hooks.sh`
- [ ] Script is executable
- [ ] Validates `.git` directory exists
- [ ] Validates `scripts/pre-commit.sh` exists
- [ ] Creates symbolic link: `.git/hooks/pre-commit` → `scripts/pre-commit.sh`
- [ ] Removes existing hook before creating symlink
- [ ] Ensures source script is executable
- [ ] Provides clear success/error messages
- [ ] Idempotent (safe to run multiple times)

**Workflow Integration**:

- [ ] Step added to `.github/workflows/copilot-setup-steps.yml`
- [ ] Step runs after dependency installation
- [ ] Calls `./scripts/install-git-hooks.sh`
- [ ] Workflow completes successfully
- [ ] Logs show hook installation

**Hook Functionality**:

- [ ] Symlink correctly points to `scripts/pre-commit.sh`
- [ ] Hook blocks commits when pre-commit checks fail
- [ ] Hook allows commits when all checks pass
- [ ] Error messages are clear and actionable (from `scripts/pre-commit.sh`)
- [ ] Changes to `scripts/pre-commit.sh` immediately affect hook
- [ ] Works in both local and Copilot environments

**Testing**:

- [ ] Local testing confirms hook blocks bad commits
- [ ] Workflow runs successfully in PR
- [ ] Hook installed in Copilot agent environment
- [ ] Agent respects pre-commit checks

**Documentation**:

- [ ] README documents Git hooks
- [ ] Manual installation instructions
- [ ] Automatic installation for Copilot
- [ ] Troubleshooting guide
- [ ] Emergency bypass instructions

## How It Works

### For Human Contributors

1. Developer clones repository
2. Runs `./scripts/install-git-hooks.sh` (one-time setup)
3. Symbolic link created: `.git/hooks/pre-commit` → `scripts/pre-commit.sh`
4. Every `git commit` automatically runs `scripts/pre-commit.sh`
5. If checks fail, commit is blocked with clear error
6. Developer fixes issues and tries again

**Bonus**: If `scripts/pre-commit.sh` is updated, the hook immediately reflects changes (no reinstall needed).

### For Copilot Agent

1. Agent environment starts (GitHub Actions)
2. Workflow runs `.github/workflows/copilot-setup-steps.yml`
3. Hook installer runs: `./scripts/install-git-hooks.sh`
4. Symbolic link created in agent's environment
5. When agent runs `git commit`, hook executes `scripts/pre-commit.sh` automatically
6. If checks fail, commit is blocked (agent must fix issues)

### If Pre-Commit Checks Fail

**Example output when hook blocks commit**:

```text
Running pre-commit checks...

Running linter checks...
✗ Markdown linting failed

Error: Pre-commit checks failed
Fix the issues above before committing

✗ Commit blocked by pre-commit hook
```

**Agent must**:

1. See the error message
2. Understand what checks failed
3. Fix the issues
4. Try committing again

## Example Usage

### Manual Installation (Human Contributors)

```bash
# Install hooks once after cloning
./scripts/install-git-hooks.sh

# Output:
# Installing Git hooks...
# Installing pre-commit hook...
# ✓ Pre-commit hook installed
#
# ✓ Git hooks installed successfully
#
# The pre-commit hook will now run ./scripts/pre-commit.sh before every commit.
# This ensures all linting checks pass before code is committed.
```

### Attempting Bad Commit (Blocked)

```bash
# Try to commit unformatted code
git commit -m "feat: add new feature"

# Output:
# Running pre-commit checks...
#
# Running linter: markdown
# ✗ Markdown linting failed
#
# Error: Pre-commit checks failed
#
# ✗ Commit blocked by pre-commit hook
```

### Successful Commit (After Fixing)

```bash
# Fix the issues
cargo run --bin linter markdown

# Try commit again
git commit -m "feat: add new feature"

# Output:
# Running pre-commit checks...
#
# Running linter: markdown
# ✓ Markdown linting passed
# [... all other checks ...]
#
# ✓ All pre-commit checks passed
# [main abc1234] feat: add new feature
```

### Emergency Bypass (Use Sparingly)

```bash
# Only if absolutely necessary (e.g., documenting a breaking change)
git commit --no-verify -m "docs: document breaking change"
```

## Related Documentation

- [Issue 1-4](./1-4-configure-github-copilot-agent-environment.md) - Copilot environment setup (where hooks are installed)
- [Removed Integration Test](https://github.com/torrust/torrust-tracker-deployer/commit/e9955b081f2f2b643949fae573955041f989bdd0) - Previous attempt at enforcement
- [scripts/pre-commit.sh](../../scripts/pre-commit.sh) - Pre-commit checks script that hook calls
- [Git Hooks Documentation](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)

## Notes

### Estimated Time

**2-3 hours** total for this issue.

### Dependencies

**Requires**:

- Issue 1-4 must be completed first (Copilot workflow exists)
- `scripts/pre-commit.sh` must exist and work correctly

**Blocks**:

- None, but significantly improves code quality enforcement

### Design Decisions

**Why symlink approach**: Using symbolic link instead of copying:

- ✅ Hook always uses latest version of `scripts/pre-commit.sh`
- ✅ No need to reinstall when script changes
- ✅ Single source of truth
- ✅ Simpler maintenance

**Why not a wrapper script**: We could create a separate hook that calls `scripts/pre-commit.sh`, but that adds unnecessary indirection. Direct symlink is simpler.

**Why install in workflow**: Copilot's environment is ephemeral. We must install hooks every time.

**Why not Git template directory**: Git template directory is global. Our solution is repository-specific and explicit.

### Hook vs Instruction

**Before (instruction only)**:

- 📝 Rule in `.github/copilot-instructions.md`
- ❌ Agents may forget or ignore
- ❌ No enforcement
- ❌ Bad commits possible

**After (Git hook)**:

- 🔒 Technical enforcement via Git hook
- ✅ Automatic execution
- ✅ Cannot be forgotten
- ✅ Bad commits **blocked**

### Limitations

**Hook can be bypassed**: `git commit --no-verify` skips hooks

**Why this is okay**:

- Human developers sometimes need emergency commits
- Agents typically don't use `--no-verify`
- Still much better than no enforcement
- Can be caught in code review

**What if agent uses --no-verify**:

- Rare occurrence
- Would be visible in commit command
- Can be addressed in feedback to GitHub

### Future Enhancements

- **Pre-push hook**: Run tests before pushing
- **Commit-msg hook**: Enforce conventional commit format
- **Multiple hooks**: Add more quality gates as needed
- **Hook configuration**: Make hooks configurable via config file
