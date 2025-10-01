//! Created State
//!
//! Initial state - Environment has been created but no operations performed
//!
//! This is the entry state for all new environments. From this state, the
//! environment can transition to `Provisioning` when infrastructure setup begins.

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{AnyEnvironmentState, Provisioning, StateTypeError};
use crate::domain::environment::Environment;

/// Initial state - Environment has been created but no operations performed
///
/// This is the entry state for all new environments. From this state, the
/// environment can transition to `Provisioning` when infrastructure setup begins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Created;

// State transition implementations
impl Environment<Created> {
    /// Transitions from Created to Provisioning state
    ///
    /// This method consumes the environment and returns a new one in the
    /// Provisioning state, indicating that infrastructure provisioning has begun.
    #[must_use]
    pub fn start_provisioning(self) -> Environment<Provisioning> {
        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: Provisioning,
        }
    }
}

// Type Erasure: Typed → Runtime conversion (into_any)
impl Environment<Created> {
    /// Converts typed `Environment<Created>` into type-erased `AnyEnvironmentState`
    #[must_use]
    pub fn into_any(self) -> AnyEnvironmentState {
        AnyEnvironmentState::Created(self)
    }
}

// Type Restoration: Runtime → Typed conversion (try_into_created)
impl AnyEnvironmentState {
    /// Attempts to convert `AnyEnvironmentState` to `Environment<Created>`
    ///
    /// # Errors
    ///
    /// Returns `StateTypeError::UnexpectedState` if the environment is not in `Created` state.
    pub fn try_into_created(self) -> Result<Environment<Created>, StateTypeError> {
        match self {
            Self::Created(env) => Ok(env),
            other => Err(StateTypeError::UnexpectedState {
                expected: "created",
                actual: other.state_name().to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_created_state() {
        let state = Created;
        assert_eq!(state, Created);
    }

    #[test]
    fn it_should_clone_created_state() {
        let state = Created;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn it_should_serialize_created_state_to_json() {
        let state = Created;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "null");
    }

    #[test]
    fn it_should_deserialize_created_state_from_json() {
        let json = "null";
        let state: Created = serde_json::from_str(json).unwrap();
        assert_eq!(state, Created);
    }

    #[test]
    fn it_should_debug_format_created_state() {
        let created = format!("{Created:?}");
        assert_eq!(created, "Created");
    }

    mod conversion_tests {
        use super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_ssh_credentials() -> SshCredentials {
            let username = Username::new("test-user".to_string()).unwrap();
            SshCredentials::new(
                PathBuf::from("/tmp/test_key"),
                PathBuf::from("/tmp/test_key.pub"),
                username,
            )
        }

        fn create_test_environment_created() -> Environment<Created> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
        }

        #[test]
        fn it_should_convert_created_environment_into_any() {
            let env = create_test_environment_created();
            let any_env = env.into_any();
            assert!(matches!(any_env, AnyEnvironmentState::Created(_)));
        }

        #[test]
        fn it_should_convert_any_to_created_successfully() {
            let env = create_test_environment_created();
            let any_env = env.into_any();
            let result = any_env.try_into_created();
            assert!(result.is_ok());
        }

        #[test]
        fn it_should_fail_converting_provisioning_to_created() {
            let env = create_test_environment_created().start_provisioning();
            let any_env = env.into_any();
            let result = any_env.try_into_created();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, StateTypeError::UnexpectedState { .. }));
            assert!(err.to_string().contains("created"));
            assert!(err.to_string().contains("provisioning"));
        }

        #[test]
        fn it_should_preserve_data_in_round_trip_conversion() {
            let original_name = "test-env";
            let env = create_test_environment_created();

            // Round-trip: typed -> erased -> typed
            let any_env = env.into_any();
            let env_restored = any_env.try_into_created().unwrap();
            let name_after = env_restored.name().as_str();

            assert_eq!(name_after, original_name);
        }
    }

    mod state_transitions {
        use super::super::*;
        use crate::domain::environment::name::EnvironmentName;
        use crate::domain::environment::state::Provisioning;
        use crate::shared::ssh::SshCredentials;
        use crate::shared::Username;
        use std::path::PathBuf;

        fn create_test_ssh_credentials() -> SshCredentials {
            let username = Username::new("test-user".to_string()).unwrap();
            SshCredentials::new(
                PathBuf::from("/tmp/test_key"),
                PathBuf::from("/tmp/test_key.pub"),
                username,
            )
        }

        fn create_test_environment_created() -> Environment<Created> {
            let name = EnvironmentName::new("test-env".to_string()).unwrap();
            let ssh_creds = create_test_ssh_credentials();
            Environment::new(name, ssh_creds)
        }

        #[test]
        fn it_should_transition_from_created_to_provisioning() {
            let env = create_test_environment_created();
            let env = env.start_provisioning();

            // Verify we can access the state
            assert_eq!(*env.state(), Provisioning);
            // Verify other fields are preserved
            assert_eq!(env.name().as_str(), "test-env");
        }
    }
}
