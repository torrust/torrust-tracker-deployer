//! Maintenance cron template context
//!
//! Defines the variables needed for maintenance-backup.cron.tera template rendering.

use serde::Serialize;

use crate::domain::backup::CronSchedule;

/// Context for rendering maintenance-backup.cron.tera template
///
/// Contains the cron schedule needed by the crontab entry template.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::backup::template::wrapper::MaintenanceCronContext;
/// use torrust_tracker_deployer_lib::domain::backup::CronSchedule;
///
/// let schedule = CronSchedule::default();
/// let context = MaintenanceCronContext::new(&schedule);
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct MaintenanceCronContext {
    /// Cron schedule expression (e.g., "0 3 * * *" for 3 AM daily)
    pub schedule: String,
}

impl MaintenanceCronContext {
    /// Creates a new maintenance cron context
    ///
    /// # Arguments
    ///
    /// * `schedule` - The cron schedule for backup execution
    #[must_use]
    pub fn new(schedule: &CronSchedule) -> Self {
        Self {
            schedule: schedule.as_str().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_context_with_default_schedule() {
        let schedule = CronSchedule::default();
        let context = MaintenanceCronContext::new(&schedule);

        assert_eq!(context.schedule, "0 3 * * *");
    }

    #[test]
    fn it_should_create_context_with_custom_schedule() {
        let schedule =
            CronSchedule::new("30 2 * * 0".to_string()).expect("Failed to create schedule");
        let context = MaintenanceCronContext::new(&schedule);

        assert_eq!(context.schedule, "30 2 * * 0");
    }
}
