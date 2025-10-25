# Update E2E Full Tests to Use Create Command

**Issue**: [#39](https://github.com/torrust/torrust-tracker-deployer/issues/39)
**Parent Epic**: [#34](https://github.com/torrust/torrust-tracker-deployer/issues/34) - Implement Create Environment Command
**Depends On**: [#36](https://github.com/torrust/torrust-tracker-deployer/issues/36) - Application Layer Command
**Related**: [E2E Testing Guide](../e2e-testing.md), [Full E2E Tests](../../src/bin/e2e_tests_full.rs)

## Overview

Update the `src/bin/e2e_tests_full.rs` to use the new create command handler instead of direct environment creation. This ensures the full E2E test exercises the complete create command functionality as part of the comprehensive test suite.

## Goals

- [ ] Add new function to create environment using CreateCommand handler
- [ ] Update e2e_tests_full.rs to use the new create command for environment creation
- [ ] Maintain existing test flow while exercising create command logic
- [ ] Ensure comprehensive test coverage of create command in full E2E context
- [ ] Preserve existing test reliability and performance

## Implementation Summary

- **Location**: `src/bin/e2e_tests_full.rs`
- **Approach**: Add `create_environment_via_command()` function that uses CreateCommand handler
- **Integration**: Replace direct environment creation with command-based creation
- **Testing**: Not black-box like Subissue 5 - this uses the command handler directly

**Estimated Time**: 1-2 hours

## Acceptance Criteria

- [ ] E2E full tests use CreateCommand handler for environment creation
- [ ] All existing E2E test functionality remains intact
- [ ] Create command is properly exercised in comprehensive test suite
- [ ] Test execution time and reliability maintained
- [ ] Proper error handling and logging preserved
