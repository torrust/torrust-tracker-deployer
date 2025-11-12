# Presentation Layer Testing and Cleanup

**Issue Type**: Subissue (Roadmap Item)
**Issue**: #170
**Parent**: [Epic #154 - Presentation Layer Reorganization](https://github.com/torrust/torrust-tracker-deployer/issues/154)

## Summary

This issue addresses specific presentation layer testing improvements and cleanup tasks identified during Epic #154 work. The focus is on practical improvements to test organization, code coverage, and following established testing conventions to prepare the codebase for future development.

## Goals

### Primary Goals

1. **Split test_support.rs** into multiple focused modules
2. **Clean unused code** in test support modules (unused public functions/methods)
3. **Fix test output** - ensure clean output when run without `--nocapture`
4. **Reorganize tests** in `src/presentation/views/core.rs` (better grouping of integration tests)
5. **Review naming conventions** - ensure all tests follow `it_should_` format from project conventions
6. **Increase code coverage** across presentation layer modules

### Success Criteria

- [ ] test_support.rs split into logical modules with clear responsibilities
- [ ] No unused public methods in test support modules
- [ ] Clean test output without `--nocapture` flag
- [ ] Well-organized test groups in core.rs following project conventions
- [ ] All tests use `it_should_` naming pattern consistently
- [ ] Improved code coverage metrics across presentation layer

## Technical Approach

### Architecture Compliance

The presentation layer follows this four-layer structure established in Epic #154:

```text
src/presentation/
├── input/          - CLI argument parsing and validation
├── dispatch/       - Routing and context management
├── controllers/    - Command orchestration
└── views/          - User output and display
```

### Testing Strategy

Following `docs/contributing/testing/unit-testing.md` and `docs/contributing/module-organization.md` conventions:

1. **Test Naming**: Use `it_should_{expected_behavior}_when_{condition}` pattern
2. **Test Organization**: Tests in dedicated `#[cfg(test)]` modules with proper ordering
3. **Quality Standards**: Three-part structure (What-When-Then), descriptive names, single responsibility
4. **Coverage Requirements**: Comprehensive coverage for all public APIs
5. **Integration Points**: Test interaction between layers

## Implementation Plan

### Task 1: Split test_support.rs into Multiple Modules

**Current State**: Single large file `src/presentation/views/test_support.rs` (438 lines)

**Target Structure**:

```text
src/presentation/views/test_support/
├── mod.rs                  # Module exports and documentation
├── test_writer.rs          # TestWriter implementation
├── test_user_output.rs     # TestUserOutput struct and basic methods
└── test_wrapper.rs         # TestOutputWrapper and convenience methods
```

**Benefits**:

- Clear separation of concerns
- Easier navigation and maintenance
- Better code organization following module organization conventions

### Task 2: Clean Unused Code in test_support modules

**Current Issues**: Unused public functions and methods identified:

- `output_pair()` method marked with `#[allow(dead_code)]`
- `clear()` method marked with `#[allow(dead_code)]`
- Potentially other unused convenience methods

**Actions**:

- Review all public methods for actual usage
- Remove truly unused methods
- Convert useful methods to be used in tests
- Remove all `#[allow(dead_code)]` and `#[allow(unused)]` attributes

### Task 3: Fix Test Output Quality

**Problem**: Tests produce noisy output when run without `--nocapture`

**Expected Behavior**: Clean output following `docs/contributing/testing/unit-testing.md`

**Actions**:

- Review all `println!` and similar output in test code
- Ensure test code doesn't print to stdout/stderr during normal execution
- Use proper test assertions instead of debug prints
- Follow silent test patterns from testing conventions

### Task 4: Reorganize Tests in core.rs

**Current State**: Tests in `src/presentation/views/core.rs` need better organization

**Target Organization**:

- Group related tests into logical modules
- Separate unit tests from integration tests
- Use clear module structure within `#[cfg(test)]`
- Follow the organization patterns from `docs/contributing/module-organization.md`

### Task 5: Review and Fix Test Naming Conventions

**Current Issues**: Not all tests follow `it_should_` naming pattern

**Actions**:

- Review all test names in presentation layer
- Convert tests to use `it_should_{expected_behavior}_when_{condition}` format
- Follow three-part structure (What-When-Then) from `docs/contributing/testing/unit-testing.md`
- Ensure test names are descriptive and behavior-focused

### Task 6: Increase Code Coverage

**Current State**: Various gaps in test coverage across presentation layer

**Focus Areas**:

- Identify untested public methods and functions
- Add missing test cases for edge conditions
- Ensure all public APIs have comprehensive test coverage
- Focus on practical, valuable tests rather than just coverage numbers

## Definition of Done

### Task 1: Module Split

- [ ] test_support.rs split into focused modules (mod.rs, test_writer.rs, test_user_output.rs, test_wrapper.rs)
- [ ] All imports updated across codebase
- [ ] Module organization follows project conventions
- [ ] All existing functionality preserved

### Task 2: Code Cleanup

- [ ] No unused public methods in test support modules
- [ ] No `#[allow(dead_code)]` or `#[allow(unused)]` attributes
- [ ] All public methods are actually used or removed
- [ ] Clean codebase ready for future development

### Task 3: Test Output

- [ ] No unwanted output when running tests without `--nocapture`
- [ ] Tests follow silent execution pattern
- [ ] Only intentional test output appears
- [ ] Clean test execution experience

### Task 4: Test Organization

- [ ] core.rs tests organized into logical groups
- [ ] Clear separation between different test types
- [ ] Module structure follows project conventions
- [ ] Easy navigation for developers

### Task 5: Naming Conventions

- [ ] All tests use `it_should_` naming pattern consistently
- [ ] Test names are descriptive and behavior-focused
- [ ] Three-part structure followed (What-When-Then)
- [ ] Naming aligns with project testing conventions

### Task 6: Code Coverage

- [ ] Improved test coverage across presentation layer
- [ ] All critical paths tested
- [ ] Edge cases covered appropriately
- [ ] Coverage improvements documented

## Timeline

**Estimated Effort**: 3-5 days focused development

**Task Priority**:

1. **Task 1** (Module Split) - Foundation for other work
2. **Task 2** (Clean Unused Code) - Remove technical debt
3. **Task 5** (Naming Conventions) - Align with standards
4. **Task 4** (Test Organization) - Improve maintainability
5. **Task 3** (Test Output) - Developer experience
6. **Task 6** (Code Coverage) - Comprehensive validation

## Risk Mitigation

- Run full test suite after each task
- Preserve all existing functionality during cleanup
- Use feature branches for each major task
- Validate changes through pre-commit checks

**Recovery Plan**: All changes can be reverted easily since they primarily add tests rather than modify core functionality.

## Related Documentation

- [Epic #154 - Presentation Layer Reorganization](https://github.com/torrust/torrust-tracker-deployer/issues/154)
- [Testing Conventions](../contributing/testing/unit-testing.md) - Project testing standards
- [Module Organization](../contributing/module-organization.md) - Code organization patterns
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Development Principles](../development-principles.md)

---

**Implementation Note**: This is a focused cleanup and improvement effort targeting specific testing debt identified during Epic #154. The emphasis is on practical improvements that enhance code quality and developer experience.
