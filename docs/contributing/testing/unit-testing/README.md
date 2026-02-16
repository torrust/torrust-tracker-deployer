# Unit Testing

Unit tests validate individual components in isolation, ensuring correct behavior without external dependencies.

## Quick Navigation

- [Naming Conventions](./naming-conventions.md) - `it_should_` pattern with examples
- [AAA Pattern](./aaa-pattern.md) - Arrange-Act-Assert structure
- [Parameterized Tests](./parameterized-tests.md) - Using rstest for multiple test cases
- [Mock Clock](./mock-clock.md) - Deterministic time testing with MockClock
- [Temp Directories](./temp-directories.md) - TempDir usage and cleanup
- [Test Builders](../integration-testing/test-builders.md) - Builder pattern for test setup

## Key Principles

- **Isolation**: Tests should not depend on external state or other tests
- **Determinism**: Same input always produces same output
- **Fast Execution**: Unit tests should run quickly
- **Clear Intent**: Test names and structure should be self-documenting
