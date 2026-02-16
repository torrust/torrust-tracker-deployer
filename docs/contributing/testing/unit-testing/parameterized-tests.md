# Parameterized Tests

When testing the same behavior with different inputs and expected outputs, prefer parameterized tests over loops in the test body.

**Why?** Parameterized tests provide:

- **Better Test Isolation**: Each parameter combination runs as a separate test case
- **Clearer Test Output**: Individual test cases show up separately in test results
- **Parallel Execution**: Test framework can run each case in parallel
- **Easier Debugging**: When a test fails, you know exactly which parameter combination caused it
- **Better IDE Support**: Modern IDEs can run individual parameterized test cases

**How?** Use the `rstest` crate for parameterized testing.

## ❌ Avoid: Loop in Test Body

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

## ✅ Good: Parameterized Test with rstest

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

## When to Use Parameterized Tests

Use parameterized tests when:

- ✅ Testing the same behavior with multiple input/output combinations
- ✅ Validating edge cases with different values
- ✅ Testing configuration variations
- ✅ Verifying data transformation with various inputs

Don't use parameterized tests when:

- ❌ Each case tests fundamentally different behavior (use separate tests)
- ❌ The test logic differs significantly between cases
- ❌ You only have one or two cases (just write separate tests)

## Setup

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
