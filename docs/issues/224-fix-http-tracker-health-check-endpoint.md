# Fix HTTP Tracker Health Check Endpoint

**Issue**: #224
**Parent Epic**: #216 - Release and Run Commands
**Related**: #220 - Tracker Slice Release and Run Commands

## Overview

The HTTP tracker health check validation is using the wrong endpoint URL (`/api/health_check`) and producing warnings instead of errors when validation fails. The correct endpoint is `/health_check` (without the `/api` prefix), and validation failures should be treated as errors to ensure proper deployment validation.

## Goals

- [x] Fix HTTP tracker health check endpoint URL from `/api/health_check` to `/health_check`
- [x] Convert HTTP tracker validation warnings to errors (make it a required check)
- [x] Include the attempted URL in error messages for better debugging
- [x] Update documentation to reflect the correct endpoint and validation behavior

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure
**Module Path**: `src/infrastructure/external_validators/`
**Pattern**: External Validator (Remote Action)

### Module Structure Requirements

- [x] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [x] Respect dependency flow rules (infrastructure can depend on domain)
- [x] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))

### Architectural Constraints

- [x] External validators run from test runner/deployment machine (not via SSH)
- [x] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [x] Validation failures must be actionable with clear troubleshooting steps

### Anti-Patterns to Avoid

- ❌ Using warnings for required validation checks
- ❌ Generic error messages without context (e.g., missing attempted URL)
- ❌ Incorrect endpoint URLs that don't match actual service endpoints

## Specifications

### Current Issue

The HTTP tracker health check in `src/infrastructure/external_validators/running_services.rs` has two problems:

1. **Wrong URL**: Uses `/api/health_check` but should use `/health_check`
2. **Warning vs Error**: Logs warnings on failure instead of returning errors

```rust
// Current (incorrect) implementation
let url = format!("http://{server_ip}:{port}/api/health_check");
// ... logs warning on failure, doesn't propagate error
```

### Expected Behavior

```rust
// Correct implementation
let url = format!("http://{server_ip}:{port}/health_check");
let response = reqwest::get(&url).await.map_err(|e| {
    RemoteActionError::ValidationFailed {
        action_name: self.name().to_string(),
        message: format!(
            "HTTP Tracker external health check failed for URL '{url}': {e}. \n\
             Check that HTTP tracker is running and firewall allows port {port}."
        ),
    }
})?;

if !response.status().is_success() {
    return Err(RemoteActionError::ValidationFailed {
        action_name: self.name().to_string(),
        message: format!(
            "HTTP Tracker returned HTTP {} for URL '{url}': {}. Service may not be healthy.",
            response.status(),
            response.status().canonical_reason().unwrap_or("Unknown")
        ),
    });
}
```

### Affected Files

1. **`src/infrastructure/external_validators/running_services.rs`**

   - Fix URL in `check_http_tracker_external` method
   - Change return type from `()` to `Result<(), RemoteActionError>`
   - Convert warning logs to error returns with URL context

2. **`docs/user-guide/commands/run.md`**

   - Update HTTP tracker health check endpoint documentation
   - Change status from "optional" to "required"

3. **`docs/console-commands.md`**

   - Update health check endpoint documentation

4. **`src/application/command_handlers/test/handler.rs`**
   - Update documentation comments to reflect correct endpoint

## Implementation Plan

### Phase 1: Fix HTTP Tracker Health Check Method (15 minutes)

- [x] Update `check_http_tracker_external` method URL to use `/health_check`
- [x] Change return type from `()` to `Result<(), RemoteActionError>`
- [x] Add URL to error messages for debugging
- [x] Update `validate_external_accessibility` to propagate errors with `?` operator

### Phase 2: Update Documentation (10 minutes)

- [x] Update module documentation header to reflect correct endpoint
- [x] Update `docs/user-guide/commands/run.md` endpoint URL
- [x] Update `docs/console-commands.md` health check description
- [x] Update `src/application/command_handlers/test/handler.rs` comments

### Phase 3: Testing and Verification (10 minutes)

- [x] Run pre-commit checks: `./scripts/pre-commit.sh`
- [x] Verify linting passes (especially clippy and rustfmt)
- [ ] Run manual E2E test to verify HTTP tracker health check works
- [ ] Confirm error messages display the attempted URL

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [x] HTTP tracker health check uses `/health_check` endpoint (not `/api/health_check`)
- [x] Validation failures return errors instead of logging warnings
- [x] Error messages include the attempted URL for debugging
- [x] `check_http_tracker_external` returns `Result<(), RemoteActionError>`
- [x] `validate_external_accessibility` properly propagates HTTP tracker errors
- [x] Documentation accurately reflects the correct endpoint URL
- [x] Documentation describes HTTP tracker check as "required" not "optional"
- [ ] Manual E2E test confirms health check works with deployed tracker

## Related Documentation

- [docs/codebase-architecture.md](../codebase-architecture.md) - Project architecture
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling conventions
- [docs/user-guide/commands/run.md](../user-guide/commands/run.md) - Run command documentation
- [src/infrastructure/external_validators/running_services.rs](../../src/infrastructure/external_validators/running_services.rs) - Implementation file

## Notes

### Why This Fix Is Important

1. **Correct Endpoint**: The Torrust Tracker HTTP tracker exposes health checks at `/health_check`, not `/api/health_check`. Using the wrong endpoint causes all validations to fail with 404 errors.

2. **Error Visibility**: Converting warnings to errors ensures that deployment failures are properly reported and CI/CD pipelines can detect issues. Warnings can be easily missed in logs.

3. **Debugging Support**: Including the attempted URL in error messages helps developers quickly identify configuration issues or endpoint mismatches.

### Verification Steps

After implementation, verify with:

```bash
# Run pre-commit checks
./scripts/pre-commit.sh

# Deploy tracker and test health check
cargo run -- create e2e-test --env-file envs/e2e-test.json
cargo run -- provision e2e-test
cargo run -- configure e2e-test
cargo run -- release e2e-test
cargo run -- run e2e-test  # Should succeed without warnings

# Manual health check test
INSTANCE_IP=$(cat data/e2e-test/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
curl http://$INSTANCE_IP:7070/health_check  # Should return success
```
