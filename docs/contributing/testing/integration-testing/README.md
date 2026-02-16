# Integration Testing

Integration tests verify that multiple components work together correctly, testing interactions between application layers and external systems.

## Quick Navigation

- [Command Testing](./command-testing.md) - Testing application commands
- [Test Builders](./test-builders.md) - Command test builders
- [Mocking Strategies](./mocking-strategies.md) - When and how to use mocks
- [Idempotency Testing](./idempotency-testing.md) - Testing safe retry behavior

## Key Principles

- **Real Dependencies**: Use actual implementations when possible
- **State Validation**: Verify actual state changes, not just API calls
- **Cleanup**: Always clean up resources, even on test failure
- **Realistic Scenarios**: Test real-world usage patterns
