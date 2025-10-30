# Testing Conventions

This document outlines the testing conventions for the Torrust Tracker Deployer project.

## 🎯 Principles

Test code should be held to the same quality standards as production code. Tests are not second-class citizens in the codebase.

### Core Principles

- **Maintainability**: Tests should be easy to update when requirements change
- **Readability**: Tests should be clear and understandable at first glance
- **Reliability**: Tests should be deterministic and not flaky
- **Isolation**: Each test should be independent and not affect other tests
- **Documentation**: Tests serve as living documentation of the system's behavior

Just like production code, tests should follow:

- **DRY (Don't Repeat Yourself)**: Extract common setup logic into helpers and builders
- **Single Responsibility**: Each test should verify one behavior
- **Clear Intent**: Test names and structure should make the purpose obvious
- **Clean Code**: Apply the same refactoring and quality standards as production code

Remember: **If the test code is hard to read or maintain, it will become a burden rather than an asset.**

## ✅ Good Practices

### AAA Pattern (Arrange-Act-Assert)

All tests should follow the AAA pattern, also known as Given-When-Then:

- **Arrange (Given)**: Set up the test data and preconditions
- **Act (When)**: Execute the behavior being tested
- **Assert (Then)**: Verify the expected outcome

This pattern makes tests:

- Easy to read and understand
- Clear about what is being tested
- Simple to maintain and modify

#### Example

```rust
#[test]
fn it_should_create_ansible_host_with_valid_ipv4() {
    // Arrange: Set up test data
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

    // Act: Execute the behavior
    let host = AnsibleHost::new(ip);

    // Assert: Verify the outcome
    assert_eq!(host.as_ip_addr(), &ip);
}
```

#### Benefits

- **Clarity**: Each section has a clear purpose
- **Structure**: Consistent test organization across the codebase
- **Debugging**: Easy to identify which phase is failing
- **Maintenance**: Simple to modify specific parts of the test

### Parameterized Tests Over Loops

When testing the same behavior with different inputs and expected outputs, prefer parameterized tests over loops in the test body.

**Why?** Parameterized tests provide:

- **Better Test Isolation**: Each parameter combination runs as a separate test case
- **Clearer Test Output**: Individual test cases show up separately in test results
- **Parallel Execution**: Test framework can run each case in parallel
- **Easier Debugging**: When a test fails, you know exactly which parameter combination caused it
- **Better IDE Support**: Modern IDEs can run individual parameterized test cases

**How?** Use the `rstest` crate for parameterized testing.

#### ❌ Avoid: Loop in Test Body

```rust
#[test]
fn it_should_create_state_file_in_environment_specific_subdirectory() {
    let test_cases = vec![
        ("e2e-config", "e2e-config/state.json"),
        ("e2e-full", "e2e-full/state.json"),
        ("e2e-provision", "e2e-provision/state.json"),
    ];

    for (env_name, expected_path) in test_cases {
        // Test logic here...
        // If one case fails, you don't know which one without debugging
    }
}
```

**Problem**: If the second iteration fails, the test output only shows the test name, not which specific case failed.

#### ✅ Good: Parameterized Test with rstest

```rust
use rstest::rstest;

#[rstest]
#[case("e2e-config", "e2e-config/state.json")]
#[case("e2e-full", "e2e-full/state.json")]
#[case("e2e-provision", "e2e-provision/state.json")]
fn it_should_create_state_file_in_environment_specific_subdirectory(
    #[case] env_name: &str,
    #[case] expected_path: &str,
) {
    // Test logic here...
    // Each case runs as a separate test with clear identification
}
```

**Benefits**: Test output shows individual cases:

- `it_should_create_state_file_in_environment_specific_subdirectory::case_1` ✅
- `it_should_create_state_file_in_environment_specific_subdirectory::case_2` ✅
- `it_should_create_state_file_in_environment_specific_subdirectory::case_3` ✅

#### When to Use Parameterized Tests

Use parameterized tests when:

- ✅ Testing the same behavior with multiple input/output combinations
- ✅ Validating edge cases with different values
- ✅ Testing configuration variations
- ✅ Verifying data transformation with various inputs

Don't use parameterized tests when:

- ❌ Each case tests fundamentally different behavior (use separate tests)
- ❌ The test logic differs significantly between cases
- ❌ You only have one or two cases (just write separate tests)

#### Setup

Add `rstest` to your `Cargo.toml`:

```toml
[dev-dependencies]
rstest = "0.23"
```

Then import it in your test module:

```rust
#[cfg(test)]
mod tests {
    use rstest::rstest;
    // ... other imports
}
```

## 🚀 Getting Started

When writing new tests:

- Always use the `it_should_` prefix and describe the specific behavior being validated
- Use `MockClock` for any time-dependent tests instead of `Utc::now()`
- Follow the AAA pattern for clear test structure
- Ensure tests are isolated and don't interfere with each other
- Use test builders for command testing to simplify setup
- Test commands at multiple levels: unit, integration, and E2E

## 📚 Documentation Index

This section provides links to specialized testing documentation organized by topic:

- **[Unit Testing](./unit-testing.md)** - Naming conventions, behavior-driven testing
- **[Resource Management](./resource-management.md)** - TempDir usage, test isolation, cleanup
- **[Testing Commands](./testing-commands.md)** - Command test patterns, builders, mocks, E2E
- **[Clock Service](./clock-service.md)** - MockClock usage for deterministic time tests
- **[Pre-commit Integration](./pre-commit-integration.md)** - AI enforcement tests, SKIP_AI_ENFORCEMENT flag
- **[Coverage](./coverage.md)** - Code coverage targets, tools, CI/CD workflow, and PR guidelines

## 🔗 Related Documentation

- [E2E Testing Guide](../../e2e-testing.md) - End-to-end testing setup and usage
- [Error Handling](../error-handling.md) - Testing error scenarios
- [Module Organization](../module-organization.md) - How to organize test code
