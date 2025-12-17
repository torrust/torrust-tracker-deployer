# Introduce Secret Type for Sensitive Data

**Issue**: #TBD
**Related**: [ADR: Secrecy Crate for Sensitive Data](../decisions/secrecy-crate-for-sensitive-data.md)

## Overview

Replace all primitive `String` types used for sensitive data (API tokens, passwords, database credentials) with the industry-standard `secrecy` crate's `Secret<String>` type. This enhances security by preventing accidental exposure through logging/debug output and enabling automatic memory zeroing.

## Refactor Plan

See detailed refactor plan: [docs/refactors/plans/secret-type-introduction.md](../refactors/plans/secret-type-introduction.md)

The plan includes:

- 10 proposals across 4 phases
- Comprehensive inventory of 16 secret fields across all DDD layers
- Progress tracking with detailed implementation checklists
- Timeline and testing strategy

## Goals

- [ ] Replace 16 string-based secret fields with `Secret<String>` type
- [ ] Prevent accidental secret exposure in logs and debug output
- [ ] Enable secure memory zeroing for sensitive data
- [ ] Update documentation with secret handling guidelines

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] All 16 secret fields converted to `Secret<String>` (tracked in refactor plan)
- [ ] No secrets appear in debug/display output
- [ ] All unit tests pass with updated secret types
- [ ] All E2E tests pass with secret handling
- [ ] AGENTS.md updated with secret handling rule
- [ ] Documentation includes examples of proper secret usage
