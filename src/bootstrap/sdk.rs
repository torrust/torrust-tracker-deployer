//! Bootstrap helpers for the SDK package
//!
//! This module creates the default infrastructure dependencies that the SDK
//! (`torrust-tracker-deployer-sdk`) needs, without requiring the SDK to import
//! from the infrastructure layer directly.
//!
//! The bootstrap layer is explicitly allowed to cross-cut all DDD layers for
//! dependency injection, which is why infrastructure types are imported here
//! rather than inside the SDK package.

use std::sync::Arc;
use std::time::Duration;

use crate::application::traits::RepositoryProvider;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::SystemClock;
use torrust_deployer_types::Clock;

/// The default file-lock timeout used by the SDK when no custom value is provided.
pub const DEFAULT_SDK_LOCK_TIMEOUT: Duration = Duration::from_secs(30);

/// Create the default repository provider backed by the file-based `RepositoryFactory`.
///
/// The SDK builder calls this to construct its provider without importing any
/// infrastructure type directly.
///
/// # Arguments
///
/// * `lock_timeout` — How long the repository factory waits to acquire a file lock.
#[must_use]
pub fn default_repository_provider(lock_timeout: Duration) -> Arc<dyn RepositoryProvider> {
    Arc::new(RepositoryFactory::new(lock_timeout))
}

/// Create the default system clock.
///
/// Returns a `SystemClock` wrapped in the `Clock` trait object so the SDK
/// builder does not need to import `SystemClock` from `crate::shared`.
#[must_use]
pub fn default_clock() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}
