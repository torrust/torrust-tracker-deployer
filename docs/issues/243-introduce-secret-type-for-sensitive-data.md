# Introduce Secret Type for Sensitive Data

**Issue**: #243
**Related**: [ADR: Secrecy Crate for Sensitive Data](../decisions/secrecy-crate-for-sensitive-data.md)

## Overview

Replace all primitive `String` types used for sensitive data (API tokens, passwords, database credentials) with wrapper types based on the industry-standard `secrecy` crate's `SecretString` type. This enhances security by preventing accidental exposure through logging/debug output and enabling automatic memory zeroing.

**Implementation Approach**: We use `secrecy::SecretString` as the foundation (which is a type alias for `SecretBox<str>`) and wrap it in domain-specific types (`ApiToken`, `Password`) that add:

- Serialization support (secrecy intentionally doesn't serialize secrets by default)
- PartialEq/Eq for config comparison in tests
- Domain-specific type safety

## Refactor Plan

See detailed refactor plan: [docs/refactors/plans/secret-type-introduction.md](../refactors/plans/secret-type-introduction.md)

The plan includes:

- 10 proposals across 4 phases
- Comprehensive inventory of 16 secret fields across all DDD layers
- Progress tracking with detailed implementation checklists
- Timeline and testing strategy

## Goals

- [ ] Replace 16 string-based secret fields with `ApiToken`/`Password` wrapper types
- [ ] Leverage `SecretString` for automatic debug redaction and memory zeroing
- [ ] Add serialization support for config file generation (deployment tool needs this)
- [ ] Update documentation with secret handling guidelines

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] All 16 secret fields converted to `ApiToken`/`Password` types (tracked in refactor plan)
- [ ] Wrapper types use `SecretString` internally for debug redaction and memory zeroing
- [ ] No secrets appear in debug/display output
- [ ] All unit tests pass with updated secret types
- [ ] All E2E tests pass with secret handling
- [ ] AGENTS.md updated with secret handling rule
- [ ] Documentation includes examples of proper secret usage
