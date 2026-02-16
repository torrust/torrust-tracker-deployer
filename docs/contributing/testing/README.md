# Testing Guide

Quick navigation for testing in Torrust Tracker Deployer.

## 🚀 Quick Start

New to testing this project? Start here:

- [Getting Started](./getting-started.md) - Your first test
- [Principles](./principles.md) - Core testing philosophy

## 📚 By Testing Level

### Unit Testing

- [Overview](./unit-testing/README.md)
- [Naming Conventions](./unit-testing/naming-conventions.md) - `it_should_` pattern
- [AAA Pattern](./unit-testing/aaa-pattern.md) - Arrange-Act-Assert
- [Parameterized Tests](./unit-testing/parameterized-tests.md) - Using rstest
- [Mock Clock](./unit-testing/mock-clock.md) - Deterministic time testing
- [Temp Directories](./unit-testing/temp-directories.md) - Cleanup and isolation

### Integration Testing

- [Overview](./integration-testing/README.md)
- [Command Testing](./integration-testing/command-testing.md) - Testing commands
- [Test Builders](./integration-testing/test-builders.md) - Command builders
- [Mocking Strategies](./integration-testing/mocking-strategies.md) - When to mock
- [Idempotency Testing](./integration-testing/idempotency-testing.md) - Safe retries

### E2E Testing

- [Overview](./e2e-testing/README.md) - Links to main E2E docs

## ✅ Quality Standards

- [Coverage](./quality/coverage.md) - Coverage targets and tools
- [Clean Output](./quality/clean-output.md) - Test output standards

## 🔗 Related Documentation

- [E2E Testing Guide](../../e2e-testing/README.md) - End-to-end testing setup and usage
- [Error Handling](../error-handling.md) - Testing error scenarios
- [Module Organization](../module-organization.md) - How to organize test code
