# Test Output Cleanliness

**Principle**: Test output should be clean and focused on test results, not cluttered with user-facing messages.

User-facing progress messages (emojis, status indicators, formatting) should **never** appear in test output as they:

- Make test output noisy and difficult to read
- Obscure actual test failures and important information
- Create inconsistent output between test runs
- Interfere with CI/CD log parsing and analysis

## Enforcing Clean Test Output

The project enforces clean test output through:

1. **Silent Verbosity by Default**: `TestContext` uses `VerbosityLevel::Silent` to suppress all user-facing messages
2. **Test-Specific Output Utilities**: Use `TestUserOutput::wrapped_silent()` for clean test output
3. **No User Messages in Tests**: Tests should focus on verifying behavior, not producing user output

## Best Practices

### ✅ Use Silent Verbosity in Tests

```rust
// ✅ Good: Uses silent verbosity by default
let context = TestContext::new();

// ✅ Good: Explicit silent output for API tests
let output = TestUserOutput::wrapped_silent();
```

### ❌ Avoid User-Facing Output in Tests

```rust
// ❌ Bad: Allows user-facing progress messages
let output = TestUserOutput::wrapped(VerbosityLevel::Normal);

// ❌ Bad: User output appears in test stderr
user_output.progress("⏳ Processing..."); // This will show in test output!
```

### Testing User Output Components

When testing user output functionality itself:

```rust
#[test]
fn it_should_format_progress_message_correctly() {
    // Capture output in test buffers, don't let it reach stderr
    let test_output = TestUserOutput::new(VerbosityLevel::Normal);
    test_output.output.progress("⏳ Processing...");

    // Verify the message format in buffers
    let stderr = test_output.stderr();
    assert!(stderr.contains("⏳ Processing..."));
}
```

## Why This Matters

Clean test output ensures:

- **Readable Results**: Developers can quickly identify failing tests
- **Reliable CI/CD**: Automated systems can parse test output correctly
- **Professional Appearance**: Test output looks polished and focused
- **Debugging Efficiency**: Important error messages aren't buried in noise

**Remember**: If you see user-facing messages (emojis, progress indicators) in test output, it indicates a testing infrastructure issue that should be fixed.
