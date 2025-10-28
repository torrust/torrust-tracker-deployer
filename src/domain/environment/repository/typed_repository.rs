use super::environment_repository::EnvironmentRepository;
use super::repository_error::RepositoryError;

/// Type-safe repository wrapper for working with generic `Environment<S>`
///
/// This wrapper provides a higher-level abstraction over `EnvironmentRepository`
/// that works directly with typed `Environment<S>` instead of `AnyEnvironmentState`.
///
/// It handles the conversion between typed and untyped representations internally,
/// providing better ergonomics and type safety for command handlers.
///
/// # Example
///
/// ```rust,ignore
/// let typed_repo = TypedEnvironmentRepository::new(repository);
///
/// // No need for .clone().into_any() - just save directly
/// typed_repo.save_provisioning(&environment)?;
/// ```
pub struct TypedEnvironmentRepository {
    repository: std::sync::Arc<dyn EnvironmentRepository>,
}

impl TypedEnvironmentRepository {
    /// Create a new typed repository wrapper
    pub fn new(repository: std::sync::Arc<dyn EnvironmentRepository>) -> Self {
        Self { repository }
    }

    /// Access the underlying untyped repository
    ///
    /// This is useful when you need to use repository methods that don't have
    /// typed equivalents yet (like load, delete, list).
    #[must_use]
    pub fn inner(&self) -> &std::sync::Arc<dyn EnvironmentRepository> {
        &self.repository
    }
}

// Macro to generate save methods for each state type
macro_rules! impl_save_for_state {
    ($method_name:ident, $state_type:ty) => {
        impl TypedEnvironmentRepository {
            #[doc = concat!("Save typed environment in ", stringify!($state_type), " state")]
            ///
            /// This method handles the conversion from typed Environment to `AnyEnvironmentState`
            /// internally and logs the operation for observability.
            ///
            /// # Errors
            ///
            /// Returns `RepositoryError` if the save operation fails
            pub fn $method_name(
                &self,
                environment: &crate::domain::environment::Environment<$state_type>,
            ) -> Result<(), RepositoryError> {
                tracing::debug!(
                    environment = %environment.name(),
                    state = stringify!($state_type),
                    "Persisting typed environment state"
                );

                let any_state = environment.clone().into_any();
                self.repository.save(&any_state)
            }
        }
    };
}

// Implement save methods for all state types
impl_save_for_state!(save_created, crate::domain::environment::state::Created);
impl_save_for_state!(
    save_provisioning,
    crate::domain::environment::state::Provisioning
);
impl_save_for_state!(
    save_provisioned,
    crate::domain::environment::state::Provisioned
);
impl_save_for_state!(
    save_configuring,
    crate::domain::environment::state::Configuring
);
impl_save_for_state!(
    save_configured,
    crate::domain::environment::state::Configured
);
impl_save_for_state!(save_releasing, crate::domain::environment::state::Releasing);
impl_save_for_state!(save_released, crate::domain::environment::state::Released);
impl_save_for_state!(save_running, crate::domain::environment::state::Running);
impl_save_for_state!(
    save_destroying,
    crate::domain::environment::state::Destroying
);
impl_save_for_state!(save_destroyed, crate::domain::environment::state::Destroyed);
impl_save_for_state!(
    save_provision_failed,
    crate::domain::environment::state::ProvisionFailed
);
impl_save_for_state!(
    save_configure_failed,
    crate::domain::environment::state::ConfigureFailed
);
impl_save_for_state!(
    save_release_failed,
    crate::domain::environment::state::ReleaseFailed
);
impl_save_for_state!(
    save_run_failed,
    crate::domain::environment::state::RunFailed
);
impl_save_for_state!(
    save_destroy_failed,
    crate::domain::environment::state::DestroyFailed
);
