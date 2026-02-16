# Testing Principles

Test code should be held to the same quality standards as production code. Tests are not second-class citizens in the codebase.

## Core Principles

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
