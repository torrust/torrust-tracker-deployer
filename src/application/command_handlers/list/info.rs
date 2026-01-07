//! Data Transfer Objects for environment list display
//!
//! These DTOs encapsulate the lightweight information extracted from environments
//! for list display purposes. They provide a clean separation between the domain
//! model and the presentation layer.

/// Lightweight environment summary for list display
///
/// This DTO contains minimal information about an environment suitable for
/// display in a list view. It is designed to be fast to extract and small
/// in memory footprint.
#[derive(Debug, Clone)]
pub struct EnvironmentSummary {
    /// Name of the environment
    pub name: String,

    /// Current state of the environment (e.g., "Created", "Provisioned", "Running", "Destroyed")
    pub state: String,

    /// Provider name (e.g., "LXD", "Hetzner Cloud")
    pub provider: String,

    /// When the environment was created (ISO 8601 format)
    pub created_at: String,
}

impl EnvironmentSummary {
    /// Create a new `EnvironmentSummary`
    #[must_use]
    pub fn new(name: String, state: String, provider: String, created_at: String) -> Self {
        Self {
            name,
            state,
            provider,
            created_at,
        }
    }
}

/// Collection of environment summaries with metadata
///
/// This DTO wraps a list of environment summaries along with metadata
/// about the listing operation, including any partial failures encountered.
#[derive(Debug, Clone)]
pub struct EnvironmentList {
    /// Successfully loaded environment summaries
    pub environments: Vec<EnvironmentSummary>,

    /// Total count of environments found
    pub total_count: usize,

    /// Environments that failed to load (name, error message)
    pub failed_environments: Vec<(String, String)>,

    /// Path to the data directory that was scanned
    pub data_directory: String,
}

impl EnvironmentList {
    /// Create a new `EnvironmentList`
    #[must_use]
    pub fn new(
        environments: Vec<EnvironmentSummary>,
        failed_environments: Vec<(String, String)>,
        data_directory: String,
    ) -> Self {
        let total_count = environments.len();
        Self {
            environments,
            total_count,
            failed_environments,
            data_directory,
        }
    }

    /// Check if the list is empty (no environments found)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.environments.is_empty() && self.failed_environments.is_empty()
    }

    /// Check if there were any partial failures
    #[must_use]
    pub fn has_failures(&self) -> bool {
        !self.failed_environments.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_environment_summary() {
        let summary = EnvironmentSummary::new(
            "test-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-01-05T10:30:00Z".to_string(),
        );

        assert_eq!(summary.name, "test-env");
        assert_eq!(summary.state, "Running");
        assert_eq!(summary.provider, "LXD");
        assert_eq!(summary.created_at, "2026-01-05T10:30:00Z");
    }

    #[test]
    fn it_should_create_environment_list() {
        let summaries = vec![
            EnvironmentSummary::new(
                "env1".to_string(),
                "Running".to_string(),
                "LXD".to_string(),
                "2026-01-05T10:30:00Z".to_string(),
            ),
            EnvironmentSummary::new(
                "env2".to_string(),
                "Created".to_string(),
                "Hetzner".to_string(),
                "2026-01-06T14:15:30Z".to_string(),
            ),
        ];

        let list = EnvironmentList::new(summaries, vec![], "/path/to/data".to_string());

        assert_eq!(list.total_count, 2);
        assert!(!list.is_empty());
        assert!(!list.has_failures());
    }

    #[test]
    fn it_should_detect_empty_list() {
        let list = EnvironmentList::new(vec![], vec![], "/path/to/data".to_string());

        assert!(list.is_empty());
        assert_eq!(list.total_count, 0);
    }

    #[test]
    fn it_should_detect_partial_failures() {
        let summaries = vec![EnvironmentSummary::new(
            "env1".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "2026-01-05T10:30:00Z".to_string(),
        )];

        let failures = vec![("broken-env".to_string(), "Invalid JSON".to_string())];

        let list = EnvironmentList::new(summaries, failures, "/path/to/data".to_string());

        assert!(!list.is_empty());
        assert!(list.has_failures());
        assert_eq!(list.failed_environments.len(), 1);
    }

    #[test]
    fn it_should_not_be_empty_when_only_failures_exist() {
        let failures = vec![("broken-env".to_string(), "Invalid JSON".to_string())];

        let list = EnvironmentList::new(vec![], failures, "/path/to/data".to_string());

        // Not empty because we found something (even though it failed to load)
        assert!(!list.is_empty());
        assert!(list.has_failures());
    }
}
