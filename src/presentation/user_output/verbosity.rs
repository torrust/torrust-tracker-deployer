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
/// use torrust_tracker_deployer_lib::presentation::user_output::VerbosityLevel;
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

    /// Progress messages require Normal level
    #[allow(dead_code)]
    pub(super) fn should_show_progress(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Success messages require Normal level
    #[allow(dead_code)]
    pub(super) fn should_show_success(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Warning messages require Normal level
    #[allow(dead_code)]
    pub(super) fn should_show_warnings(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Errors are always shown regardless of verbosity level
    #[allow(clippy::unused_self)]
    #[allow(dead_code)]
    pub(super) fn should_show_errors(&self) -> bool {
        true
    }

    /// Blank lines require Normal level
    pub(super) fn should_show_blank_lines(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Steps require Normal level
    #[allow(dead_code)]
    pub(super) fn should_show_steps(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }

    /// Info blocks require Normal level
    #[allow(dead_code)]
    pub(super) fn should_show_info_blocks(&self) -> bool {
        self.should_show(VerbosityLevel::Normal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbosity_filter_at_quiet_level_should_only_show_required_quiet() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(!filter.should_show(VerbosityLevel::Normal));
        assert!(!filter.should_show(VerbosityLevel::Verbose));
    }

    #[test]
    fn verbosity_filter_at_normal_level_should_show_quiet_and_normal() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(filter.should_show(VerbosityLevel::Normal));
        assert!(!filter.should_show(VerbosityLevel::Verbose));
    }

    #[test]
    fn verbosity_filter_at_verbose_level_should_show_all_up_to_verbose() {
        let filter = VerbosityFilter::new(VerbosityLevel::Verbose);
        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(filter.should_show(VerbosityLevel::Normal));
        assert!(filter.should_show(VerbosityLevel::Verbose));
        assert!(!filter.should_show(VerbosityLevel::VeryVerbose));
    }

    #[test]
    fn verbosity_filter_at_debug_level_should_show_all_messages() {
        let filter = VerbosityFilter::new(VerbosityLevel::Debug);
        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(filter.should_show(VerbosityLevel::Normal));
        assert!(filter.should_show(VerbosityLevel::Verbose));
        assert!(filter.should_show(VerbosityLevel::VeryVerbose));
        assert!(filter.should_show(VerbosityLevel::Debug));
    }

    #[test]
    fn verbosity_levels_should_be_ordered() {
        assert!(VerbosityLevel::Quiet < VerbosityLevel::Normal);
        assert!(VerbosityLevel::Normal < VerbosityLevel::Verbose);
        assert!(VerbosityLevel::Verbose < VerbosityLevel::VeryVerbose);
        assert!(VerbosityLevel::VeryVerbose < VerbosityLevel::Debug);
    }

    #[test]
    fn default_verbosity_level_should_be_normal() {
        let level = VerbosityLevel::default();
        assert_eq!(level, VerbosityLevel::Normal);
    }

    #[test]
    fn verbosity_filter_should_show_errors_at_any_level() {
        let quiet_filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        let normal_filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(quiet_filter.should_show_errors());
        assert!(normal_filter.should_show_errors());
    }

    #[test]
    fn verbosity_filter_should_show_progress_only_at_normal_or_higher() {
        let quiet_filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        let normal_filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(!quiet_filter.should_show_progress());
        assert!(normal_filter.should_show_progress());
    }

    #[test]
    fn verbosity_filter_should_show_blank_lines_only_at_normal_or_higher() {
        let quiet_filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        let normal_filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(!quiet_filter.should_show_blank_lines());
        assert!(normal_filter.should_show_blank_lines());
    }

    #[test]
    fn it_should_show_progress_at_verbose_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Verbose);
        assert!(filter.should_show_progress());
    }

    #[test]
    fn it_should_always_show_errors_regardless_of_level() {
        assert!(VerbosityFilter::new(VerbosityLevel::Quiet).should_show_errors());
        assert!(VerbosityFilter::new(VerbosityLevel::Normal).should_show_errors());
        assert!(VerbosityFilter::new(VerbosityLevel::Verbose).should_show_errors());
        assert!(VerbosityFilter::new(VerbosityLevel::VeryVerbose).should_show_errors());
        assert!(VerbosityFilter::new(VerbosityLevel::Debug).should_show_errors());
    }

    #[test]
    fn it_should_show_success_at_normal_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(filter.should_show_success());
    }

    #[test]
    fn it_should_not_show_success_at_quiet_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(!filter.should_show_success());
    }

    #[test]
    fn it_should_show_warnings_at_normal_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(filter.should_show_warnings());
    }

    #[test]
    fn it_should_not_show_warnings_at_quiet_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(!filter.should_show_warnings());
    }

    #[test]
    fn it_should_show_steps_at_normal_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(filter.should_show_steps());
    }

    #[test]
    fn it_should_not_show_steps_at_quiet_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(!filter.should_show_steps());
    }

    #[test]
    fn it_should_show_info_blocks_at_normal_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(filter.should_show_info_blocks());
    }

    #[test]
    fn it_should_not_show_info_blocks_at_quiet_level() {
        let filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(!filter.should_show_info_blocks());
    }

    #[test]
    fn it_should_show_when_level_meets_requirement() {
        let filter = VerbosityFilter::new(VerbosityLevel::Normal);
        assert!(filter.should_show(VerbosityLevel::Quiet));
        assert!(filter.should_show(VerbosityLevel::Normal));
        assert!(!filter.should_show(VerbosityLevel::Verbose));
    }

    #[test]
    fn it_should_handle_all_verbosity_levels_in_should_show() {
        let quiet_filter = VerbosityFilter::new(VerbosityLevel::Quiet);
        assert!(quiet_filter.should_show(VerbosityLevel::Quiet));
        assert!(!quiet_filter.should_show(VerbosityLevel::Normal));

        let debug_filter = VerbosityFilter::new(VerbosityLevel::Debug);
        assert!(debug_filter.should_show(VerbosityLevel::Quiet));
        assert!(debug_filter.should_show(VerbosityLevel::Normal));
        assert!(debug_filter.should_show(VerbosityLevel::Verbose));
        assert!(debug_filter.should_show(VerbosityLevel::VeryVerbose));
        assert!(debug_filter.should_show(VerbosityLevel::Debug));
    }

    // ========================================================================
    // VerbosityLevel Tests
    // ========================================================================

    #[test]
    fn it_should_use_normal_as_default_verbosity() {
        let default = VerbosityLevel::default();
        assert_eq!(default, VerbosityLevel::Normal);
    }

    #[test]
    fn it_should_order_verbosity_levels_correctly() {
        assert!(VerbosityLevel::Quiet < VerbosityLevel::Normal);
        assert!(VerbosityLevel::Normal < VerbosityLevel::Verbose);
        assert!(VerbosityLevel::Verbose < VerbosityLevel::VeryVerbose);
        assert!(VerbosityLevel::VeryVerbose < VerbosityLevel::Debug);
    }

    #[test]
    fn it_should_support_equality_comparison() {
        assert_eq!(VerbosityLevel::Normal, VerbosityLevel::Normal);
        assert_ne!(VerbosityLevel::Normal, VerbosityLevel::Verbose);
    }

    #[test]
    fn it_should_support_ordering_comparison() {
        let normal = VerbosityLevel::Normal;
        assert!(normal >= VerbosityLevel::Quiet);
        assert!(normal >= VerbosityLevel::Normal);
        assert!(normal < VerbosityLevel::Verbose);
    }
}
