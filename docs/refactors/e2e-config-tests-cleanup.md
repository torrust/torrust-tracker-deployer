# E2E Config Tests Cleanup Refactor Plan

## 📋 Overview

This refactor plan outlines the improvements needed for `src/bin/e2e_config_tests.rs` to match the clean architecture and code quality of `src/bin/e2e_provision_tests.rs`.

## 🎯 Goals

- **Primary**: Improve code readability, maintainability, and consistency
- **Secondary**: Align architectural patterns with `e2e_provision_tests.rs`
- **Tertiary**: Enhance error handling and reduce code duplication

## 🔍 Current Issues Analysis

### High Priority Issues

1. **Inconsistent Async/Sync Handling**

   - **Problem**: Manual `Runtime::new()` creation in sync `main()`
   - **Location**: Lines 165, 180
   - **Impact**: Inefficient runtime management, inconsistent with project patterns

2. **Excessive Function Responsibility**

   - **Problem**: `run_configuration_tests()` handles 7+ different concerns
   - **Location**: Lines 119-183
   - **Impact**: Hard to test, maintain, and understand

3. **Nested Error Handling Anti-patterns**

   - **Problem**: Deep match/Result nesting instead of using `?` operator
   - **Location**: Lines 240-267 in `run_ansible_configuration()`
   - **Impact**: Reduced readability, maintenance burden

4. **Code Duplication**
   - **Problem**: SSH credential creation logic duplicated
   - **Location**: Lines 331-355 and 357-370
   - **Impact**: Maintenance overhead, inconsistency risk

### Medium Priority Issues

1. **Magic Numbers and Hard-coded Values**

   - **Problem**: Scattered throughout without centralization
   - **Examples**: `Duration::from_secs(30)`, `"torrust"`, `"data/templates"`
   - **Impact**: Configuration fragility

2. **Inconsistent Documentation**

   - **Problem**: Missing or incomplete function documentation
   - **Location**: Helper functions lack comprehensive docs
   - **Impact**: Reduced maintainability

3. **Unclear Test State Logic**
   - **Problem**: "Expected failure" comments create confusion
   - **Location**: Lines 240-250
   - **Impact**: Test reliability concerns

### Low Priority Issues

1. **Function Naming Inconsistency**

   - **Problem**: Mixed naming patterns
   - **Impact**: Code consistency

2. **Logging Pattern Variations**
   - **Problem**: Inconsistent structured logging
   - **Impact**: Debugging difficulty

## 🏗️ Refactor Strategy

### Phase 1: Structural Foundation (High Priority)

#### 1.1 Convert to Async Main Pattern ✅ COMPLETE

- **Task**: Replace sync `main()` with `#[tokio::main]` ✅
- **Files**: `src/bin/e2e_config_tests.rs` ✅
- **Changes**:
  - Remove manual `Runtime::new()` calls ✅
  - Make `main()` and primary functions `async` ✅
  - Use consistent `await` pattern throughout ✅

#### 1.2 Function Decomposition ✅ COMPLETE

- **Task**: Break down `run_configuration_tests()` into focused functions ✅
- **Target Functions**: ✅

  ```rust
  async fn setup_test_environment() -> Result<TestEnvironment> ✅
  async fn setup_docker_container() -> Result<RunningProvisionedContainer> ✅
  async fn configure_ssh_connectivity(container: &RunningProvisionedContainer) -> Result<()> ✅
  async fn run_provision_simulation(container: &RunningProvisionedContainer) -> Result<()> ✅ (Already extracted)
  async fn apply_ansible_configuration(container: &RunningProvisionedContainer) -> Result<()> ✅ (Already extracted)
  async fn validate_deployment(container: &RunningProvisionedContainer) -> Result<()> ✅ (Already extracted)
  async fn cleanup_container(container: RunningProvisionedContainer) -> Result<()> ✅
  ```

**Status**: ✅ COMPLETE - Functions organized in execution order, all linters pass, E2E tests work correctly

#### 1.3 Error Handling Simplification ✅ COMPLETE

- **Task**: Replace nested match statements with `?` operator ✅
- **Pattern**: Convert `match result { Ok(x) => ..., Err(e) => return Err(...) }` to direct `?` usage ✅
- **Focus**: `run_ansible_configuration()` and `run_deployment_validation()` ✅

**Status**: ✅ COMPLETE - Replaced nested match patterns with `?` operator in both functions, improved readability

### Phase 2: Code Duplication Removal (High Priority)

#### 2.1 Extract SSH Credential Factory

- **Task**: Create centralized SSH credential creation
- **New Function**:

  ```rust
  fn create_test_ssh_credentials() -> Result<SshCredentials> {
      let project_root = std::env::current_dir().context("Failed to get current directory")?;
      Ok(SshCredentials::new(
          project_root.join("fixtures/testing_rsa"),
          project_root.join("fixtures/testing_rsa.pub"),
          TEST_SSH_USERNAME.to_string(),
      ))
  }
  ```

#### 2.2 Consolidate Configuration Creation

- **Task**: Merge `create_container_config()` and `create_container_ssh_credentials()`
- **Approach**: Single configuration factory with SSH credentials included

### Phase 3: Constants and Configuration (Medium Priority)

#### 3.1 Extract Constants

- **Task**: Centralize magic numbers and strings
- **Constants to Add**:

  ```rust
  const TEST_SSH_USERNAME: &str = "torrust";
  const SSH_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
  const SSH_RETRY_COUNT: u32 = 10;
  const DEFAULT_TEMPLATES_DIR: &str = "./data/templates";
  const TEST_INSTANCE_NAME: &str = "torrust-tracker-container";
  ```

#### 3.2 Configuration Structure

- **Task**: Create test-specific configuration struct
- **Benefits**: Type safety, centralized configuration, easier testing

### Phase 4: Documentation and Consistency (Medium Priority)

#### 4.1 Comprehensive Documentation

- **Task**: Add complete function documentation following project patterns
- **Template**: Match documentation style from `e2e_provision_tests.rs`
- **Include**: Purpose, parameters, returns, errors, examples

#### 4.2 Logging Standardization

- **Task**: Implement consistent structured logging
- **Pattern**: Use tracing with structured fields like `e2e_provision_tests.rs`

#### 4.3 Clarify Test State Logic

- **Task**: Resolve "expected failure" confusion
- **Options**:
  1. Fix underlying inventory issue
  2. Create explicit test-only mode
  3. Document limitations clearly

### Phase 5: Advanced Improvements (Low Priority)

#### 5.1 Function Naming Alignment

- **Task**: Standardize function naming patterns
- **Pattern**: Follow `e2e_provision_tests.rs` conventions

#### 5.2 Trait-based Design

- **Task**: Use traits for better testability
- **Example**: Extract container operations to traits

## 📅 Implementation Timeline

### Week 1: Foundation (Phase 1) ✅ COMPLETE

- [x] Convert to async main pattern
- [x] Decompose `run_configuration_tests()`
- [x] Simplify error handling
- [x] Organize functions in execution order

### Week 2: Deduplication (Phase 2)

- [ ] Extract SSH credential factory
- [ ] Consolidate configuration creation
- [ ] Add constants

### Week 3: Polish (Phases 3-4)

- [ ] Complete documentation
- [ ] Standardize logging
- [ ] Resolve test state issues

### Week 4: Final Improvements (Phase 5)

- [ ] Function naming alignment
- [ ] Optional trait implementations
- [ ] Final testing and validation

## ✅ Success Criteria

### Code Quality Metrics

- [ ] All functions under 50 lines
- [ ] No nested error handling beyond 2 levels
- [ ] 100% function documentation coverage
- [ ] Zero code duplication in SSH credential creation
- [ ] Consistent async/await usage

### Architectural Alignment

- [ ] Matches `e2e_provision_tests.rs` structure
- [ ] Single responsibility principle adherence
- [ ] Consistent error handling patterns
- [ ] Unified logging approach

### Testing

- [ ] All existing tests pass
- [ ] New unit tests for extracted functions
- [ ] E2E tests continue to work
- [ ] Performance maintains or improves

## 🔧 Tools and Commands

### Pre-refactor Validation

```bash
# Run all linters
cargo run --bin linter all

# Run tests
cargo test
cargo run --bin e2e-tests

# Check for unused dependencies
cargo machete
```

### Post-refactor Validation

```bash
# Same commands as pre-refactor
cargo run --bin linter all
cargo test
cargo run --bin e2e-tests
cargo machete

# Additional checks
cargo clippy -- -D warnings
cargo fmt --check
```

## 📝 Notes

- Each phase should be implemented as separate commits following conventional commit format
- All changes must pass existing CI/CD checks
- Maintain backward compatibility for any public interfaces
- Document any breaking changes in commit messages
- Consider creating feature branch for the entire refactor: `{issue-number}-refactor-e2e-config-tests-cleanup`

## 🔗 References

- Source file: `src/bin/e2e_config_tests.rs`
- Reference implementation: `src/bin/e2e_provision_tests.rs`
- Project conventions: `docs/contributing/`
- Testing guidelines: `docs/contributing/testing.md`
