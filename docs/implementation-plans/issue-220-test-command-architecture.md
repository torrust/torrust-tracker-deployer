# Implementation Plan: Test Command Improvements

**Issue**: [#220] - Enhance test command to validate all deployed tracker services  
**Branch**: `220-tracker-slice-release-run-commands`  
**Date**: December 10, 2025

## Overview

This plan addresses four improvements to maintain architectural consistency and enhance service validation:

1. **Architecture Fix** - Separate application DTOs from domain types for TrackerConfig (follows Provider pattern)
2. **Port 0 Validation** - Prevent dynamic port assignment (not supported)
3. **Multiple HTTP Trackers** - Validate all configured HTTP trackers, not just the first
4. **Service Location** - Move health checking from infrastructure to application layer

## Architectural Issue Identified

**Problem**: `TrackerConfig` domain types are used directly in application layer, violating DDD layering.

**Current State** (Incorrect):

```text
Application Layer: EnvironmentCreationConfig
  └─> tracker: TrackerConfig (DOMAIN TYPE - should be DTO!)
```

**Correct Pattern** (Like Provider):

```text
Application Layer: EnvironmentCreationConfig
  └─> tracker: TrackerSection (DTO with String primitives)
      └─> converts to TrackerConfig (Domain with SocketAddr, validated types)
```

**Solution**: Create application DTOs (`TrackerSection`, etc.) and enhance domain types with richer types.

## Progress Tracking

Use this checklist to track implementation progress. **Mark as done after each step commits successfully.**

```text
Phase 0: Architecture Fix
  [x] Step 0.1: Create tracker DTO module structure
  [x] Step 0.2: Implement UdpTrackerSection DTO
  [x] Step 0.3: Implement HttpTrackerSection DTO
  [x] Step 0.4: Implement HttpApiSection DTO
  [x] Step 0.5: Implement TrackerCoreSection DTO
  [x] Step 0.6: Implement TrackerSection DTO
  [x] Step 0.7: Update domain types to use SocketAddr
  [x] Step 0.8: Update EnvironmentCreationConfig
  [x] Step 0.9: Update all application imports (already correct)

Phase 1: Port 0 Validation
  [x] Step 1.1: Create ADR document
  [x] Step 1.2: Add DynamicPortNotSupported error
  [x] Step 1.3: Add port 0 validation in conversions
  [x] Step 1.4: Add validation tests

Phase 2: Multiple HTTP Trackers
  [x] Step 2.1: Update RunningServicesValidator signature
  [x] Step 2.2: Update validation logic for multiple ports
  [x] Step 2.3: Update test command handler
  [x] Step 2.4: Update E2E test task
  [x] Step 2.5: Add multiple tracker tests

Phase 3: Service Location
  [ ] Step 3.1: Create TrackerHealthService
  [ ] Step 3.2: Update application services module
  [ ] Step 3.3: Remove old validator
  [ ] Step 3.4: Update all imports
  [ ] Step 3.5: Update error types and documentation

Phase 4: Documentation
  [x] Step 4.1: Update command documentation
  [x] Step 4.2: Update architecture docs
  [ ] Step 4.3: Run full E2E test suite
  [x] Step 4.4: Final verification and summary
```

## Pre-Commit Protocol

**After EVERY step**:

1. **Run tests**: `cargo test`
2. **Run linters**: `cargo run --bin linter all`
3. **If both pass**: `git add . && git commit -m "<commit message from step>"`
4. **If either fails**: Fix issues before proceeding to next step
5. **Update progress**: Mark the step as done in the checklist above

**Important**: Never skip the pre-commit protocol. Each step must be verified before proceeding.

---

## Phase 0: Architecture Fix - Separate Application DTOs from Domain Types

**Priority**: Critical | **Effort**: High | **Time**: 2 hours  
**Incremental Commits**: 9 commits (one per step)

### Step 0.1: Create Tracker DTO Module Structure

**Commit**: `step: [#220] create tracker config DTO module structure`

**Actions**:

1. Create directory: `src/application/command_handlers/create/config/tracker/`
2. Create file: `tracker/mod.rs` with module documentation:

   ```rust
   //! Tracker Configuration DTOs (Application Layer)
   //!
   //! This module contains DTO types for tracker configuration used in
   //! environment creation. These types use raw primitives (String) for
   //! JSON deserialization and convert to rich domain types (SocketAddr).
   ```

3. Update: `src/application/command_handlers/create/config/mod.rs`
   - Add: `pub mod tracker;`

**Pre-commit**: Run tests, run linters, commit

---

### Step 0.2: Implement UdpTrackerSection DTO

**Commit**: `step: [#220] implement UdpTrackerSection DTO with conversion`

**Actions**:

1. Create: `tracker/udp_tracker_section.rs`
2. Implement:

   ```rust
   use serde::{Deserialize, Serialize};
   use std::net::SocketAddr;
   use crate::application::command_handlers::create::config::CreateConfigError;
   use crate::domain::tracker::UdpTrackerConfig;

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   pub struct UdpTrackerSection {
       pub bind_address: String,
   }

   impl UdpTrackerSection {
       pub fn to_udp_tracker_config(&self) -> Result<UdpTrackerConfig, CreateConfigError> {
           let bind_address = self.bind_address.parse::<SocketAddr>()
               .map_err(|e| CreateConfigError::InvalidBindAddress {
                   address: self.bind_address.clone(),
                   source: e,
               })?;
           Ok(UdpTrackerConfig { bind_address })
       }
   }
   ```

3. Export in `tracker/mod.rs`: `pub use udp_tracker_section::UdpTrackerSection;`

**Pre-commit**: Run tests, run linters, commit

---

### Step 0.3: Implement HttpTrackerSection DTO

**Commit**: `step: [#220] implement HttpTrackerSection DTO with conversion`

**Actions**:

1. Create: `tracker/http_tracker_section.rs`
2. Implement similar to UdpTrackerSection with `HttpTrackerConfig`
3. Export in `tracker/mod.rs`

**Pre-commit**: Run tests, run linters, commit

---

### Step 0.4: Implement HttpApiSection DTO

**Commit**: `step: [#220] implement HttpApiSection DTO with conversion`

**Actions**:

1. Create: `tracker/http_api_section.rs`
2. Implement with both `bind_address` and `admin_token` fields
3. Export in `tracker/mod.rs`

**Pre-commit**: Run tests, run linters, commit

---

### Step 0.5: Implement TrackerCoreSection DTO

**Commit**: `step: [#220] implement TrackerCoreSection DTO with conversion`

**Actions**:

1. Create: `tracker/tracker_core_section.rs`
2. Include `database` (use existing `DatabaseConfig`) and `private` fields
3. Implement `to_tracker_core_config()` method
4. Export in `tracker/mod.rs`

**Pre-commit**: Run tests, run linters, commit

---

### Step 0.6: Implement TrackerSection DTO

**Commit**: `step: [#220] implement TrackerSection top-level DTO with full conversion`

**Actions**:

1. Create: `tracker/tracker_section.rs`
2. Implement:

   ```rust
   pub struct TrackerSection {
       pub core: TrackerCoreSection,
       pub udp_trackers: Vec<UdpTrackerSection>,
       pub http_trackers: Vec<HttpTrackerSection>,
       pub http_api: HttpApiSection,
   }

   impl TrackerSection {
       pub fn to_tracker_config(&self) -> Result<TrackerConfig, CreateConfigError> {
           // Convert all sections to domain types
       }
   }
   ```

3. Export in `tracker/mod.rs` and `config/mod.rs`

**Pre-commit**: Run tests, run linters, commit

---

### Step 0.7: Update Domain Types to Use SocketAddr

**Commit**: `step: [#220] enhance domain tracker config with SocketAddr types`

**Actions**:

1. Edit: `src/domain/tracker/config.rs`
2. Change all `bind_address` fields from `String` to `SocketAddr`:
   - `UdpTrackerConfig::bind_address`
   - `HttpTrackerConfig::bind_address`
   - `HttpApiConfig::bind_address`
3. Update `Default` impl to use parsed SocketAddr
4. Update all doctests and unit tests
5. Add `use std::net::SocketAddr;`

**Note**: This will break compilation - that's expected and documented

**Pre-commit**: Run tests (expect some failures), run linters, commit

---

### Step 0.8: Update EnvironmentCreationConfig

**Commit**: `step: [#220] use TrackerSection DTO in EnvironmentCreationConfig`

**Actions**:

1. Edit: `src/application/command_handlers/create/config/environment_config.rs`
2. Change: `pub tracker: TrackerConfig` → `pub tracker: TrackerSection`
3. Add import: `use super::tracker::TrackerSection;`
4. Update methods accessing `tracker` field to call `tracker.to_tracker_config()`
5. Update all tests and examples

**Pre-commit**: Run tests, run linters, commit

---

### Step 0.9: Update All Application Imports

**Commit**: `step: [#220] update application layer imports for tracker DTOs`

**Actions**:

1. Find all application layer files importing domain `TrackerConfig`
2. Update to use `TrackerSection` from config module
3. Add conversion calls where needed: `tracker_section.to_tracker_config()?`
4. Files likely affected:
   - `src/application/command_handlers/create/handler.rs`
   - `src/application/command_handlers/create/tests/*.rs`

**Pre-commit**: Run tests, run linters, commit

---

## Phase 1: Port 0 Validation (Fail Fast)

**Priority**: High | **Effort**: Low | **Time**: 30 minutes  
**Incremental Commits**: 4 commits (one per step)

### Step 1.1: Create ADR Document

**Commit**: `docs: [#220] add ADR for port zero not supported in bind addresses`

**Actions**:

1. Create: `docs/decisions/port-zero-not-supported.md`
2. Follow ADR template from `docs/decisions/README.md`
3. Content sections:
   - **Status**: Accepted
   - **Context**: Port 0 conflicts with firewall configuration in `configure` command
   - **Decision**: Reject port 0 during environment creation (DTO→Domain conversion)
   - **Consequences**: Clear error, users must specify explicit ports
   - **Alternatives Considered**: Parse Docker logs, query Docker mappings (future)

**Pre-commit**: Run linters (markdown, cspell), commit

---

### Step 1.2: Add DynamicPortNotSupported Error

**Commit**: `step: [#220] add DynamicPortNotSupported error variant`

**Actions**:

1. Edit: `src/application/command_handlers/create/errors.rs`
2. Add error variant:

   ```rust
   #[error("Dynamic port assignment (port 0) is not supported in bind address '{bind_address}'")]
   DynamicPortNotSupported { bind_address: String },
   ```

3. Implement `help()` method with detailed guidance

**Pre-commit**: Run tests, run linters, commit

---

### Step 1.3: Add Port 0 Validation in Conversions

**Commit**: `step: [#220] add port 0 validation in DTO to domain conversions`

**Actions**:

1. Edit all `*_section.rs` files with `bind_address` fields
2. In each `to_*_config()` method, after parsing to SocketAddr:

   ```rust
   if bind_address.port() == 0 {
       return Err(CreateConfigError::DynamicPortNotSupported {
           bind_address: self.bind_address.clone(),
       });
   }
   ```

**Pre-commit**: Run tests, run linters, commit

---

### Step 1.4: Add Validation Tests

**Commit**: `test: [#220] add port 0 validation tests for tracker sections`

**Actions**:

1. Add test modules in `tracker/*_section.rs` files
2. Tests to add:
   - `test_rejects_port_zero()`
   - `test_accepts_valid_port()`
3. Test both UDP, HTTP tracker, and HTTP API sections

**Pre-commit**: Run tests, run linters, commit

---

## Phase 2: Support Multiple HTTP Trackers

**Priority**: High | **Effort**: Medium | **Time**: 1 hour  
**Incremental Commits**: 5 commits (one per step)

### Step 2.1: Update RunningServicesValidator Signature

**Commit**: `step: [#220] update validator to accept multiple HTTP tracker ports`

**Actions**:

1. Edit: `src/infrastructure/remote_actions/validators/running_services.rs`
2. Change struct field: `http_tracker_port: Option<u16>` → `http_tracker_ports: Vec<u16>`
3. Update both constructors: `new()` and `with_deploy_dir()`
4. Update module documentation

**Note**: This breaks callers - expected

**Pre-commit**: Run tests (expect failures), run linters, commit

---

### Step 2.2: Update Validation Logic for Multiple Ports

**Commit**: `step: [#220] implement validation for multiple HTTP tracker ports`

**Actions**:

1. Edit: `validate_external_accessibility` method
2. Replace optional port check with loop:

   ```rust
   for (index, port) in self.http_tracker_ports.iter().enumerate() {
       info!("Validating HTTP Tracker #{} on port {}", index + 1, port);
       // validation logic
   }
   ```

**Pre-commit**: Run tests, run linters, commit

---

### Step 2.3: Update Test Command Handler

**Commit**: `step: [#220] collect all HTTP tracker ports in test command`

**Actions**:

1. Edit: `src/application/command_handlers/test/handler.rs`
2. Replace:

   ```rust
   // OLD
   let tracker_api_port = Self::extract_port_from_bind_address(...);
   let http_tracker_port = tracker_config.http_trackers.first()...;

   // NEW
   let tracker_api_port = tracker_config.http_api.bind_address.port();
   let http_tracker_ports: Vec<u16> = tracker_config
       .http_trackers
       .iter()
       .map(|t| t.bind_address.port())
       .collect();
   ```

3. Remove `extract_port_from_bind_address()` helper method
4. Update constructor call

**Pre-commit**: Run tests, run linters, commit

---

### Step 2.4: Update E2E Test Task

**Commit**: `step: [#220] update E2E run validation to use multiple ports`

**Actions**:

1. Edit: `src/testing/e2e/tasks/run_run_validation.rs`
2. Update validator instantiation to pass `Vec<u16>`

**Pre-commit**: Run tests, run linters, commit

---

### Step 2.5: Add Multiple Tracker Tests

**Commit**: `test: [#220] add tests for multiple HTTP tracker validation`

**Actions**:

1. Add tests:
   - `test_validates_multiple_http_trackers()`
   - `test_validates_zero_http_trackers()`
   - `test_validates_single_http_tracker()`

**Pre-commit**: Run tests, run linters, commit

---

## Phase 3: Move to Application Services Layer

**Priority**: Medium | **Effort**: Low | **Time**: 45 minutes  
**Incremental Commits**: 5 commits (one per step)

### Step 3.1: Create TrackerHealthService

**Commit**: `step: [#220] create TrackerHealthService in application layer`

**Actions**:

1. Create: `src/application/services/tracker_health_service.rs`
2. Copy content from `running_services.rs`
3. Rename: `RunningServicesValidator` → `TrackerHealthService`
4. Update module docs to reflect application service

**Pre-commit**: Run tests, run linters, commit

---

### Step 3.2: Update Application Services Module

**Commit**: `step: [#220] export TrackerHealthService from services module`

**Actions**:

1. Edit: `src/application/services/mod.rs`
2. Add:

   ```rust
   mod tracker_health_service;
   pub use tracker_health_service::TrackerHealthService;
   ```

**Pre-commit**: Run tests, run linters, commit

---

### Step 3.3: Remove Old Validator

**Commit**: `step: [#220] remove RunningServicesValidator from infrastructure`

**Actions**:

1. Delete: `src/infrastructure/remote_actions/validators/running_services.rs`
2. Edit: `src/infrastructure/remote_actions/validators/mod.rs` - remove export
3. Edit: `src/infrastructure/remote_actions/mod.rs` - remove re-export

**Note**: This breaks imports - expected

**Pre-commit**: Run tests (expect failures), run linters, commit anyway

---

### Step 3.4: Update All Imports

**Commit**: `step: [#220] update imports to use TrackerHealthService`

**Actions**:

1. Files to update:
   - `src/application/command_handlers/test/handler.rs`
   - `src/testing/e2e/tasks/run_run_validation.rs`
2. Replace all `RunningServicesValidator` → `TrackerHealthService`
3. Update import paths

**Pre-commit**: Run tests, run linters, commit

---

### Step 3.5: Update Error Types and Documentation

**Commit**: `step: [#220] update error types and docs for health service`

**Actions**:

1. Update error type names if needed
2. Update all doc comments from "validator" → "health service"
3. Update method documentation

**Pre-commit**: Run tests, run linters, commit

---

## Phase 4: Documentation and Final Validation

**Priority**: Medium | **Effort**: Low | **Time**: 30 minutes  
**Incremental Commits**: 4 commits (one per step)

### Step 4.1: Update Command Documentation

**Commit**: `docs: [#220] update test command documentation`

**Actions**:

1. Edit: `docs/user-guide/commands/test.md`
2. Add: Note about all HTTP trackers being validated
3. Add: Note about port 0 not supported (link to ADR)

**Pre-commit**: Run linters (markdown, cspell), commit

---

### Step 4.2: Update Architecture Documentation

**Commit**: `docs: [#220] update architecture docs for health service`

**Actions**:

1. Edit: `docs/codebase-architecture.md`
2. Update: Application services section to mention `TrackerHealthService`
3. Update: Remote actions section to clarify SSH-only validators

**Pre-commit**: Run linters (markdown, cspell), commit

---

### Step 4.3: Run Full E2E Test Suite

**Commit**: `test: [#220] verify all E2E tests pass with changes` (if fixes needed)

**Actions**:

1. Run: `cargo test`
2. Run: `cargo run --bin e2e-infrastructure-lifecycle-tests`
3. Run: `cargo run --bin e2e-deployment-workflow-tests`
4. Fix any failures
5. Only commit if fixes were needed

**Pre-commit**: Tests already run, commit only if fixes made

---

### Step 4.4: Final Verification and Summary

**Commit**: `chore: [#220] final linting and validation` (if needed)

**Actions**:

1. Run: `cargo run --bin linter all`
2. Run: `cargo machete` (check unused dependencies)
3. Verify: All checkboxes in progress tracking are marked
4. Only commit if fixes needed

**Pre-commit**: Linters already run, commit only if fixes made

---

## Commit Strategy Summary

**Total Expected Commits**: ~27 incremental commits

**Commit Prefixes**:

- `step:` - Implementation step (code changes)
- `test:` - Test additions
- `docs:` - Documentation only
- `chore:` - Tooling/cleanup
- `fix:` - Bug fixes (if needed during implementation)

**Phase Breakdown**:

- Phase 0: 9 commits
- Phase 1: 4 commits
- Phase 2: 5 commits
- Phase 3: 5 commits
- Phase 4: 4 commits

---

## Important Execution Guidelines

### Protocol Compliance

1. **Never skip pre-commit checks** - Each step must pass tests + linters
2. **Commit after every step** - Don't batch multiple steps
3. **Update progress tracking** - Mark checkboxes as you complete steps
4. **Expected failures are OK** - Some steps intentionally break compilation (documented)

### Phase Dependencies

- **Phase 0 must complete first** - All other phases depend on it
- **Phases 1-3 are independent** - Can be reordered after Phase 0
- **Phase 4 must be last** - Final documentation and validation

### Recovery Strategy

If a step fails unexpectedly:

1. Read error message carefully
2. Check if it's documented as expected
3. Fix the issue
4. Re-run tests + linters
5. Commit with `fix:` prefix
6. Continue to next step

### Time Management

- Each step: 5-15 minutes
- Each phase: 30 minutes - 2 hours
- Total: ~5-6 hours with breaks
- **Take breaks between phases**

---

## Files Summary

### Files to Create (9 new files)

**Phase 0**:

1. `src/application/command_handlers/create/config/tracker/mod.rs`
2. `src/application/command_handlers/create/config/tracker/udp_tracker_section.rs`
3. `src/application/command_handlers/create/config/tracker/http_tracker_section.rs`
4. `src/application/command_handlers/create/config/tracker/http_api_section.rs`
5. `src/application/command_handlers/create/config/tracker/tracker_core_section.rs`
6. `src/application/command_handlers/create/config/tracker/tracker_section.rs`

**Phase 1**: 7. `docs/decisions/port-zero-not-supported.md`

**Phase 3**: 8. `src/application/services/tracker_health_service.rs`

### Files to Delete (1 file)

**Phase 3**:

1. `src/infrastructure/remote_actions/validators/running_services.rs`

### Files to Modify (~15 files)

**Phase 0**:

- `src/domain/tracker/config.rs` - SocketAddr types
- `src/application/command_handlers/create/config/mod.rs` - exports
- `src/application/command_handlers/create/config/environment_config.rs` - use TrackerSection
- Multiple application layer files - update imports

**Phase 1**:

- `src/application/command_handlers/create/errors.rs` - new error
- All tracker section files - add validation

**Phase 2**:

- `src/infrastructure/remote_actions/validators/running_services.rs` - Vec ports
- `src/application/command_handlers/test/handler.rs` - collect all ports
- `src/testing/e2e/tasks/run_run_validation.rs` - update validator usage

**Phase 3**:

- `src/application/services/mod.rs` - exports
- `src/infrastructure/remote_actions/mod.rs` - remove exports
- Files with imports - update paths

**Phase 4**:

- `docs/user-guide/commands/test.md`
- `docs/codebase-architecture.md`

---

## Success Criteria

✅ All 27 steps completed and checked off  
✅ All unit tests pass (`cargo test`)  
✅ All E2E tests pass  
✅ All linters pass (`cargo run --bin linter all`)  
✅ No unused dependencies (`cargo machete`)  
✅ ADR document created  
✅ Documentation updated  
✅ Clean git history with descriptive commits

---

**Ready to start? Begin with Phase 0, Step 0.1!**
