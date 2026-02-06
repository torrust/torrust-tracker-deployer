//! Purge Command Module
//!
//! This module implements the delivery-agnostic `PurgeCommandHandler`
//! for removing all local environment data.
//!
//! ## Architecture
//!
//! The `PurgeCommandHandler` implements the Command Pattern and uses Dependency Injection
//! to interact with infrastructure services through interfaces:
//!
//! - **Repository Pattern**: Accesses environment state via `EnvironmentRepository`
//! - **Clock Abstraction**: Provides deterministic time for testing via `Clock` trait
//! - **Domain-Driven Design**: Uses domain objects from `domain::environment`
//!
//! ## Design Principles
//!
//! - **Delivery-Agnostic**: Works with CLI, REST API, or any delivery mechanism
//! - **Synchronous**: Follows existing patterns (no async/await)
//! - **Explicit Errors**: All errors implement `.help()` with actionable guidance
//! - **Idempotent**: Can be safely executed multiple times on the same environment
//! - **State-Independent**: Works on environments in any state
//!
//! ## Purge Workflow
//!
//! The command handler orchestrates a simple workflow:
//!
//! 1. **Verify environment exists** - Ensure the environment is present in repository
//! 2. **Remove data directory** - Delete `data/{env-name}/` including all environment state
//! 3. **Remove build directory** - Delete `build/{env-name}/` including generated templates
//!
//! ## State Management
//!
//! Unlike other commands, purge **does not transition environment state**:
//!
//! - Accepts environment in any state (via environment name lookup)
//! - Removes all local data regardless of current state
//! - Does not persist state after purge (the environment data is gone)
//!
//! This is intentional: purge is meant to clean up local storage, not manage
//! the environment lifecycle.
//!
//! ## Idempotency
//!
//! The purge operation is idempotent - running it multiple times on the same
//! environment will:
//! - Succeed if the directories are already removed
//! - Report appropriate status to the user
//! - Not fail due to missing directories
//!
//! ## Use Cases
//!
//! - Free up disk space after destroying environments
//! - Reuse environment names after cleanup
//! - Remove local state when infrastructure was destroyed independently
//! - Clean up failed deployments
//!
//! ## Important Notes
//!
//! - **Does NOT destroy infrastructure**: Only removes local files
//! - **Irreversible operation**: All local environment data is permanently deleted
//! - **Works in any state**: Can purge environments that are Created, Provisioned, Running, etc.
//! - **No state preservation**: The environment entry is removed from the repository

pub mod errors;
pub mod handler;

#[cfg(test)]
mod tests;
