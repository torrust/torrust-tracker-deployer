//! Verbosity level control for user output
//!
//! This module provides verbosity level configuration and filtering logic
//! to control the amount of detail shown to users.

/// Verbosity levels for user output
///
/// Controls the amount of detail shown to users. Higher verbosity levels include
/// all output from lower levels.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::cli::views::VerbosityLevel;
///
/// let level = VerbosityLevel::Normal;
/// assert!(level >= VerbosityLevel::Quiet);
/// assert!(level < VerbosityLevel::Verbose);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum VerbosityLevel {
    /// No output - suppress all user-facing messages
    Silent,
    /// Minimal output - only errors and final results
    Quiet,
    /// Default level - essential progress and results
    #[default]
    Normal,
    /// Detailed progress including intermediate steps
    Verbose,
    /// Very detailed including decisions and retries
    VeryVerbose,
    /// Maximum detail for troubleshooting
    Debug,
}

/// Determines what messages should be displayed based on verbosity level
///
/// This struct encapsulates verbosity filtering logic, making it testable
/// independently from output formatting.
pub(super) struct VerbosityFilter {
    level: VerbosityLevel,
}

impl VerbosityFilter {
    /// Create a new verbosity filter with the specified level
    pub(super) fn new(level: VerbosityLevel) -> Self {
        Self { level }
    }

    /// Check if messages at the given level should be shown
    pub(super) fn should_show(&self, required_level: VerbosityLevel) -> bool {
        self.level >= required_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_show_only_quiet_messages_when_filter_is_set_to_quiet_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);

        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(!filter.should_show(VerbosityLevel::Normal));
        assert!(!filter.should_show(VerbosityLevel::Verbose));
    }

    #[test]
    fn it_should_show_quiet_and_normal_messages_when_filter_is_set_to_normal_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);

        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(filter.should_show(VerbosityLevel::Normal));
        assert!(!filter.should_show(VerbosityLevel::Verbose));
    }

    #[test]
    fn it_should_show_messages_up_to_verbose_when_filter_is_set_to_verbose_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Verbose);

        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(filter.should_show(VerbosityLevel::Normal));
        assert!(filter.should_show(VerbosityLevel::Verbose));
        assert!(!filter.should_show(VerbosityLevel::VeryVerbose));
    }

    #[test]
    fn it_should_show_all_messages_when_filter_is_set_to_debug_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Debug);

        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(filter.should_show(VerbosityLevel::Normal));
        assert!(filter.should_show(VerbosityLevel::Verbose));
        assert!(filter.should_show(VerbosityLevel::VeryVerbose));
        assert!(filter.should_show(VerbosityLevel::Debug));
    }

    #[test]
    fn it_should_order_verbosity_levels_from_quiet_to_debug() {
        assert!(VerbosityLevel::Quiet < VerbosityLevel::Normal);
        assert!(VerbosityLevel::Normal < VerbosityLevel::Verbose);
        assert!(VerbosityLevel::Verbose < VerbosityLevel::VeryVerbose);
        assert!(VerbosityLevel::VeryVerbose < VerbosityLevel::Debug);
    }

    #[test]
    fn it_should_default_to_normal_level_when_using_default() {
        let level = VerbosityLevel::default();

        assert_eq!(level, VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_compare_equal_when_verbosity_levels_are_same() {
        assert_eq!(VerbosityLevel::Normal, VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_compare_not_equal_when_verbosity_levels_are_different() {
        assert_ne!(VerbosityLevel::Normal, VerbosityLevel::Verbose);
    }

    #[test]
    fn it_should_support_ordering_comparisons_with_greater_or_equal() {
        let normal = VerbosityLevel::Normal;

        assert!(normal >= VerbosityLevel::Quiet);
        assert!(normal >= VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_support_ordering_comparisons_with_less_than() {
        let normal = VerbosityLevel::Normal;

        assert!(normal < VerbosityLevel::Verbose);
    }
}
