# Getting Started with Testing

When writing new tests:

- Always use the `it_should_` prefix and describe the specific behavior being validated
- Use `MockClock` for any time-dependent tests instead of `Utc::now()`
- Follow the AAA pattern for clear test structure
- Ensure tests are isolated and don't interfere with each other
- **Keep test output clean** - Use `TestContext::new()` or `TestUserOutput::wrapped_silent()` to avoid user-facing messages
- Use test builders for command testing to simplify setup
- Test commands at multiple levels: unit, integration, and E2E
