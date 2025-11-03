# Update CI Workflows and Remove Bash Scripts

**Type**: Sub-issue (Task)
**Issue**: [#119](https://github.com/torrust/torrust-tracker-deployer/issues/119)
**Parent Epic**: [#112](https://github.com/torrust/torrust-tracker-deployer/issues/112) - Refactor and Improve E2E Test Execution
**Depends On**: [#113](https://github.com/torrust/torrust-tracker-deployer/issues/113) - Create Dependency Installation Package for E2E Tests

## Summary

Update GitHub Actions workflows to use the new `dependency-installer` binary instead of bash scripts, and remove the obsolete bash scripts from `scripts/setup/`. This completes the migration from bash-based dependency management to the Rust-based solution.

## Objectives

- [ ] Update all GitHub Actions workflows to use `dependency-installer` binary
- [ ] Remove obsolete bash scripts from `scripts/setup/`
- [ ] Update documentation to reference the new dependency installation method
- [ ] Verify CI workflows pass with the new approach

## Background

### Current State

GitHub Actions workflows currently use bash scripts for dependency installation:

```yaml
# Example from .github/workflows/test.yml
- name: Install dependencies
  run: |
    ./scripts/setup/install-opentofu.sh
    ./scripts/setup/install-ansible.sh
    ./scripts/setup/install-lxd-ci.sh
```

These bash scripts are in `scripts/setup/`:

- `install-opentofu.sh`
- `install-ansible.sh`
- `install-lxd-ci.sh`

### Desired State

Workflows should use the Rust binary:

```yaml
- name: Install dependencies
  run: cargo run --bin dependency-installer install
```

Bash scripts should be removed after verifying the Rust implementation works correctly in CI.

## Scope

This issue covers only the CI workflow updates and bash script removal. The package creation and E2E test integration are handled in separate issues.

## Technical Approach

### Workflow Files to Update

The following GitHub Actions workflow files currently use bash installation scripts:

1. **`.github/workflows/test-e2e-provision.yml`**

   - Uses: `./scripts/setup/install-lxd-ci.sh`
   - Uses: `./scripts/setup/install-opentofu.sh`

2. **`.github/workflows/test-e2e-config.yml`**

   - Uses: `./scripts/setup/install-ansible.sh`

3. **`.github/workflows/test-lxd-provision.yml`**

   - Uses: `./scripts/setup/install-lxd-ci.sh`
   - Uses: `./scripts/setup/install-opentofu.sh`

**Note**: Other workflow files (`.github/workflows/testing.yml`, `.github/workflows/coverage.yml`, `.github/workflows/linting.yml`) do NOT use bash scripts and do not need modification.

### Workflow Update Strategy

1. **Update the 3 workflows listed above** to use `dependency-installer` binary
2. **Test workflows** in CI to ensure they work
3. **Remove bash scripts** after successful CI runs
4. **Update documentation** to reference the new approach

### Workflow Changes

Update GitHub Actions workflows to:

```yaml
- name: Set up Rust toolchain
  uses: actions-rs/toolchain@v1
  with:
    toolchain: stable

- name: Install development dependencies
  run: |
    cargo build --bin dependency-installer
    cargo run --bin dependency-installer install

- name: Run E2E tests
  run: cargo run --bin e2e-tests-full
```

## Implementation Plan

### Phase 1: Identify Workflows (30 minutes)

- [ ] List all GitHub Actions workflow files in `.github/workflows/`
- [ ] Identify which workflows use bash installation scripts
- [ ] Document the current dependency installation approach in each workflow

### Phase 2: Update Workflows (1-2 hours)

- [ ] Update workflows to use `cargo run --bin dependency-installer install`
- [ ] Remove calls to bash scripts (`./scripts/setup/install-*.sh`)
- [ ] Add appropriate caching for the binary build if needed
- [ ] Test changes in a feature branch

### Phase 3: Verify CI and Remove Bash Scripts (1 hour)

- [ ] Create PR with workflow changes
- [ ] Verify all CI workflows pass with the new approach
- [ ] After successful CI run, remove bash scripts:
  - [ ] Delete `scripts/setup/install-opentofu.sh`
  - [ ] Delete `scripts/setup/install-ansible.sh`
  - [ ] Delete `scripts/setup/install-lxd-ci.sh`
- [ ] Update `scripts/setup/README.md` if it exists

### Phase 4: Update Documentation (30 minutes - 1 hour)

- [ ] Update `docs/e2e-testing.md` to reference the new installation method
- [ ] Update `README.md` if it references bash scripts
- [ ] Update any other documentation that mentions bash installation scripts
- [ ] Add migration notes to the EPIC issue

## Acceptance Criteria

### Functional Requirements

- [ ] All GitHub Actions workflows use `dependency-installer` binary instead of bash scripts
- [ ] CI workflows pass successfully with the new approach
- [ ] Bash scripts are removed from `scripts/setup/`
- [ ] No references to bash scripts remain in workflow files

### Code Quality

- [ ] Workflow YAML files are properly formatted
- [ ] All linters pass (`cargo run --bin linter all`)
- [ ] No broken links in documentation

### Documentation

- [ ] `docs/e2e-testing.md` updated to reference new installation method
- [ ] `README.md` updated if it referenced bash scripts
- [ ] Migration notes added to EPIC issue

### CI Verification

- [ ] All CI workflows pass with the new dependency installation approach
- [ ] No regressions in test execution or coverage

## Dependencies

This sub-issue depends on:

- **#TBD** - Create Dependency Installation Package for E2E Tests (must be completed first)

## Related Documentation

- [docs/e2e-testing.md](../e2e-testing.md) - E2E testing documentation
- [GitHub Actions Documentation](https://docs.github.com/en/actions)

## Estimated Time

**3-4 hours** total:

- Phase 1: 30 minutes (identifying workflows)
- Phase 2: 1-2 hours (updating workflows)
- Phase 3: 1 hour (CI verification and cleanup)
- Phase 4: 30 minutes - 1 hour (documentation updates)

## Notes

### Design Considerations

**Incremental approach**: Update workflows first and verify they work before removing bash scripts. This ensures we can roll back if issues arise.

**CI caching**: Consider caching the compiled `dependency-installer` binary to speed up CI runs, though compilation is fast.

**Backward compatibility**: Once bash scripts are removed, developers working on older branches may need to rebase to get the new installation method.

### Migration Path

1. Complete the dependency-installer package (#TBD)
2. Update workflows to use the new binary
3. Run full CI suite to verify everything works
4. Only then remove bash scripts
5. Update documentation to reflect the change

### Rollback Strategy

If CI fails with the new approach:

1. Revert workflow changes
2. Keep bash scripts until issues are resolved
3. Fix issues in the dependency-installer package
4. Retry workflow updates

### Future Enhancements

- Add workflow caching for faster CI runs
- Consider creating a composite GitHub Action for dependency installation
- Add dependency version checking to ensure specific versions are installed
