# Quick Reference

Quick reference guide for common testing patterns.

## Test Naming

```rust
it_should_{expected_behavior}_when_{condition}
it_should_{expected_behavior}_given_{state}
```

## AAA Pattern

```rust
// Arrange: Set up test data
let data = setup();

// Act: Execute the behavior
let result = perform_action(data);

// Assert: Verify the outcome
assert_eq!(result, expected);
```

## Parameterized Tests

```rust
use rstest::rstest;

#[rstest]
#[case(input1, expected1)]
#[case(input2, expected2)]
fn it_should_handle_various_inputs(#[case] input: Type, #[case] expected: Type) {
    // Test logic
}
```

## Using MockClock

```rust
let clock = Arc::new(MockClock::new(fixed_time));
let component = Component::new(clock);
```

## Using TempDir

```rust
let temp_dir = TempDir::new().unwrap();
let path = temp_dir.path();
// TempDir automatically cleans up when dropped
```

## Test Builders

```rust
let (command, _temp_dir) = CommandTestBuilder::new().build();
```

## Silent Test Output

```rust
let context = TestContext::new(); // Uses silent verbosity by default
let output = TestUserOutput::wrapped_silent();
```
