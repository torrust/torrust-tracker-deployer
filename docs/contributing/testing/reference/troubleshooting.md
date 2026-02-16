# Troubleshooting

Common testing issues and their solutions.

## Test Fails with "Directory Already Exists"

**Problem**: Tests create real directories in `./data` or `./build`

**Solution**: Use `TempDir` for temporary directories

```rust
let temp_dir = TempDir::new().unwrap();
let path = temp_dir.path();
```

See: [Temp Directories](../unit-testing/temp-directories.md)

## Test Output Shows User-Facing Messages

**Problem**: Emoji and progress messages appear in test output

**Solution**: Use silent verbosity in tests

```rust
let context = TestContext::new(); // Silent by default
let output = TestUserOutput::wrapped_silent();
```

See: [Clean Output](../quality/clean-output.md)

## Time-Dependent Tests Are Flaky

**Problem**: Tests using `Utc::now()` produce inconsistent results

**Solution**: Use `MockClock` for deterministic time

```rust
let clock = Arc::new(MockClock::new(fixed_time));
```

See: [Mock Clock](../unit-testing/mock-clock.md)

## Coverage Check Fails

**Problem**: `cargo cov-check` reports coverage below 70%

**Solution**:

1. Run `cargo cov-html` to see detailed report
2. Identify untested code
3. Add tests or verify if module is excluded from coverage requirements

See: [Coverage](../quality/coverage.md)

## Parameterized Test Shows Wrong Case Number

**Problem**: Can't identify which test case failed

**Solution**: Use `rstest` instead of loops in test body

```rust
#[rstest]
#[case(input1, expected1)]
#[case(input2, expected2)]
fn test_name(#[case] input: Type, #[case] expected: Type) {
    // Each case runs separately
}
```

See: [Parameterized Tests](../unit-testing/parameterized-tests.md)
