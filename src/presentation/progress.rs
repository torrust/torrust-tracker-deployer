//! Progress Reporting for Long-Running Operations
//!
//! This module provides progress reporting functionality for multi-step operations
//! that take significant time to complete. It builds on top of `UserOutput` to
//! provide standardized progress updates with timing information.
//!
//! ## Features
//!
//! - **Step Tracking**: Reports progress through numbered steps (e.g., "[1/5] Loading configuration...")
//! - **Timing Information**: Tracks and reports duration for each completed step
//! - **Sub-step Support**: Shows detailed progress within major steps
//! - **Verbosity Aware**: Respects user verbosity settings through `UserOutput`
//! - **Consistent Format**: Standardized output format across all commands
//!
//! ## Example Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
//! use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
//!
//! let mut output = UserOutput::new(VerbosityLevel::Normal);
//! let mut progress = ProgressReporter::new(output, 3);
//!
//! // Step 1: Load configuration
//! progress.start_step("Loading configuration");
//! // ... perform operation ...
//! progress.complete_step(Some("Configuration loaded: test-env"));
//!
//! // Step 2: Provision with sub-steps
//! progress.start_step("Provisioning infrastructure");
//! progress.sub_step("Creating virtual machine");
//! progress.sub_step("Configuring network");
//! // ... perform operations ...
//! progress.complete_step(Some("Instance created: test-instance"));
//!
//! // Step 3: Finalize
//! progress.start_step("Finalizing environment");
//! // ... perform operation ...
//! progress.complete_step(None);
//!
//! // Complete with summary
//! progress.complete("Environment 'test-env' created successfully");
//! ```
//!
//! ## Output Format
//!
//! The progress reporter generates output like:
//!
//! ```text
//! ⏳ [1/3] Loading configuration...
//!   ✓ Configuration loaded: test-env (took 150ms)
//! ⏳ [2/3] Provisioning infrastructure...
//!     → Creating virtual machine
//!     → Configuring network
//!   ✓ Instance created: test-instance (took 2.3s)
//! ⏳ [3/3] Finalizing environment...
//!   ✓ Done (took 450ms)
//! ✅ Environment 'test-env' created successfully
//! ```

use std::time::{Duration, Instant};

use crate::presentation::user_output::UserOutput;

/// Progress reporter for multi-step operations
///
/// Tracks progress through multiple steps of a long-running operation,
/// providing clear feedback with step numbers, descriptions, and timing.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
///
/// let output = UserOutput::new(VerbosityLevel::Normal);
/// let mut progress = ProgressReporter::new(output, 2);
///
/// progress.start_step("Step 1");
/// progress.complete_step(Some("Step 1 done"));
///
/// progress.start_step("Step 2");
/// progress.complete_step(None);
///
/// progress.complete("All done!");
/// ```
pub struct ProgressReporter {
    output: UserOutput,
    total_steps: usize,
    current_step: usize,
    step_start: Option<Instant>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    ///
    /// # Arguments
    ///
    /// * `output` - User output handler for displaying messages
    /// * `total_steps` - Total number of steps in the operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// let progress = ProgressReporter::new(output, 5);
    /// ```
    #[must_use]
    pub fn new(output: UserOutput, total_steps: usize) -> Self {
        Self {
            output,
            total_steps,
            current_step: 0,
            step_start: None,
        }
    }

    /// Start a new step with a description
    ///
    /// Increments the current step counter and displays a progress message
    /// in the format `[current/total] description...`.
    ///
    /// # Arguments
    ///
    /// * `description` - Human-readable description of what this step does
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// let mut progress = ProgressReporter::new(output, 3);
    ///
    /// progress.start_step("Loading configuration");
    /// // Output: ⏳ [1/3] Loading configuration...
    /// ```
    pub fn start_step(&mut self, description: &str) {
        self.current_step += 1;
        self.step_start = Some(Instant::now());

        self.output.progress(&format!(
            "[{}/{}] {}...",
            self.current_step, self.total_steps, description
        ));
    }

    /// Complete the current step with optional result message
    ///
    /// Displays a completion message with timing information.
    /// The message shows either the provided result or a generic "Done" message.
    ///
    /// # Arguments
    ///
    /// * `result` - Optional description of what was accomplished
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// let mut progress = ProgressReporter::new(output, 2);
    ///
    /// progress.start_step("Loading data");
    /// progress.complete_step(Some("Data loaded successfully"));
    /// // Output: ✓ Data loaded successfully (took 150ms)
    ///
    /// progress.start_step("Processing");
    /// progress.complete_step(None);
    /// // Output: ✓ Done (took 2.3s)
    /// ```
    pub fn complete_step(&mut self, result: Option<&str>) {
        if let Some(start) = self.step_start {
            let duration = start.elapsed();

            if let Some(msg) = result {
                self.output
                    .result(&format!("  ✓ {} (took {})", msg, format_duration(duration)));
            } else {
                self.output
                    .result(&format!("  ✓ Done (took {})", format_duration(duration)));
            }
        }

        self.step_start = None;
    }

    /// Report a sub-step within the current step
    ///
    /// Displays an indented message indicating progress within the current step.
    /// Useful for showing detailed progress without starting a new numbered step.
    ///
    /// # Arguments
    ///
    /// * `description` - What is currently happening within this step
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// let mut progress = ProgressReporter::new(output, 1);
    ///
    /// progress.start_step("Provisioning infrastructure");
    /// progress.sub_step("Creating virtual machine");
    /// progress.sub_step("Configuring network");
    /// progress.sub_step("Setting up storage");
    /// progress.complete_step(Some("Infrastructure ready"));
    /// ```
    pub fn sub_step(&mut self, description: &str) {
        self.output.result(&format!("    → {description}"));
    }

    /// Complete all steps and show summary
    ///
    /// Displays a final success message indicating the entire operation completed.
    /// This should be called after all steps are done.
    ///
    /// # Arguments
    ///
    /// * `summary` - Final success message describing what was accomplished
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// let mut progress = ProgressReporter::new(output, 1);
    ///
    /// progress.start_step("Creating environment");
    /// progress.complete_step(None);
    /// progress.complete("Environment 'test-env' created successfully");
    /// // Output: ✅ Environment 'test-env' created successfully
    /// ```
    pub fn complete(&mut self, summary: &str) {
        self.output.success(summary);
    }

    /// Get a mutable reference to the underlying `UserOutput`
    ///
    /// This allows using other output methods (like `error`, `warn`)
    /// while progress is being tracked.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = UserOutput::new(VerbosityLevel::Normal);
    /// let mut progress = ProgressReporter::new(output, 1);
    ///
    /// progress.start_step("Checking conditions");
    /// progress.output().warn("Some non-critical warning");
    /// progress.complete_step(None);
    /// ```
    #[must_use]
    pub fn output(&mut self) -> &mut UserOutput {
        &mut self.output
    }
}

/// Format duration in a human-readable way
///
/// Converts durations to appropriate units:
/// - Less than 1 second: milliseconds (e.g., "150ms")
/// - 1 second or more: seconds with 1 decimal place (e.g., "2.3s")
///
/// # Arguments
///
/// * `duration` - The duration to format
///
/// # Returns
///
/// A human-readable string representation of the duration
fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1000 {
        format!("{millis}ms")
    } else {
        format!("{:.1}s", duration.as_secs_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::user_output::VerbosityLevel;
    use std::sync::{Arc, Mutex};

    /// Helper to create test `UserOutput` with buffer writers
    ///
    /// Returns: (`UserOutput`, Arc to stdout buffer, Arc to stderr buffer)
    #[allow(clippy::type_complexity)]
    fn create_test_user_output(
        verbosity: VerbosityLevel,
    ) -> (UserOutput, Arc<Mutex<Vec<u8>>>, Arc<Mutex<Vec<u8>>>) {
        let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
        let stderr_buffer = Arc::new(Mutex::new(Vec::new()));

        let stdout_writer = Box::new(SharedWriter(Arc::clone(&stdout_buffer)));
        let stderr_writer = Box::new(SharedWriter(Arc::clone(&stderr_buffer)));

        let output = UserOutput::with_writers(verbosity, stdout_writer, stderr_writer);

        (output, stdout_buffer, stderr_buffer)
    }

    /// A writer that shares a buffer through an Arc<Mutex<Vec<u8>>>
    struct SharedWriter(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for SharedWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.0.lock().unwrap().flush()
        }
    }

    #[test]
    fn it_should_create_progress_reporter_with_total_steps() {
        let output = UserOutput::new(VerbosityLevel::Normal);
        let progress = ProgressReporter::new(output, 5);

        assert_eq!(progress.total_steps, 5);
        assert_eq!(progress.current_step, 0);
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_start_step_and_increment_counter() {
        let (output, _stdout, stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 3);

        progress.start_step("Loading configuration");

        assert_eq!(progress.current_step, 1);
        assert!(progress.step_start.is_some());

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("[1/3] Loading configuration..."));
    }

    #[test]
    fn it_should_track_multiple_steps() {
        let (output, _stdout, stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 3);

        progress.start_step("Step 1");
        assert_eq!(progress.current_step, 1);

        progress.start_step("Step 2");
        assert_eq!(progress.current_step, 2);

        progress.start_step("Step 3");
        assert_eq!(progress.current_step, 3);

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("[1/3] Step 1..."));
        assert!(stderr_content.contains("[2/3] Step 2..."));
        assert!(stderr_content.contains("[3/3] Step 3..."));
    }

    #[test]
    fn it_should_complete_step_with_result_message() {
        let (output, stdout, _stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 1);

        progress.start_step("Loading data");
        progress.complete_step(Some("Data loaded successfully"));

        let stdout_content = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(stdout_content.contains("✓ Data loaded successfully"));
        assert!(stdout_content.contains("took"));
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_complete_step_without_result_message() {
        let (output, stdout, _stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 1);

        progress.start_step("Processing");
        progress.complete_step(None);

        let stdout_content = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(stdout_content.contains("✓ Done"));
        assert!(stdout_content.contains("took"));
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_report_sub_steps() {
        let (output, stdout, _stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 1);

        progress.start_step("Provisioning");
        progress.sub_step("Creating VM");
        progress.sub_step("Configuring network");
        progress.complete_step(None);

        let stdout_content = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(stdout_content.contains("→ Creating VM"));
        assert!(stdout_content.contains("→ Configuring network"));
    }

    #[test]
    fn it_should_display_completion_summary() {
        let (output, _stdout, stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 1);

        progress.start_step("Creating environment");
        progress.complete_step(None);
        progress.complete("Environment created successfully");

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("✅ Environment created successfully"));
    }

    #[test]
    fn it_should_provide_access_to_output() {
        let (output, _stdout, stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 1);

        progress.output().warn("Test warning");

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("⚠️  Test warning"));
    }

    #[test]
    fn it_should_respect_verbosity_levels() {
        let (output, _stdout, stderr) = create_test_user_output(VerbosityLevel::Quiet);
        let mut progress = ProgressReporter::new(output, 1);

        progress.start_step("Step 1");
        progress.complete_step(Some("Done"));

        // At Quiet level, progress messages should not appear
        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert_eq!(stderr_content, "");
    }

    #[test]
    fn it_should_format_milliseconds_correctly() {
        let duration = Duration::from_millis(150);
        assert_eq!(format_duration(duration), "150ms");

        let duration = Duration::from_millis(999);
        assert_eq!(format_duration(duration), "999ms");
    }

    #[test]
    fn it_should_format_seconds_correctly() {
        let duration = Duration::from_millis(1000);
        assert_eq!(format_duration(duration), "1.0s");

        let duration = Duration::from_millis(2345);
        assert_eq!(format_duration(duration), "2.3s");

        let duration = Duration::from_secs(10);
        assert_eq!(format_duration(duration), "10.0s");
    }

    #[test]
    fn it_should_handle_full_workflow() {
        let (output, stdout, stderr) = create_test_user_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 3);

        // Step 1
        progress.start_step("Loading configuration");
        progress.complete_step(Some("Configuration loaded: test-env"));

        // Step 2 with sub-steps
        progress.start_step("Provisioning infrastructure");
        progress.sub_step("Creating virtual machine");
        progress.sub_step("Configuring network");
        progress.complete_step(Some("Instance created: test-instance"));

        // Step 3
        progress.start_step("Finalizing environment");
        progress.complete_step(None);

        // Complete
        progress.complete("Environment 'test-env' created successfully");

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("[1/3] Loading configuration..."));
        assert!(stderr_content.contains("[2/3] Provisioning infrastructure..."));
        assert!(stderr_content.contains("[3/3] Finalizing environment..."));
        assert!(stderr_content.contains("✅ Environment 'test-env' created successfully"));

        let stdout_content = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(stdout_content.contains("✓ Configuration loaded: test-env"));
        assert!(stdout_content.contains("→ Creating virtual machine"));
        assert!(stdout_content.contains("→ Configuring network"));
        assert!(stdout_content.contains("✓ Instance created: test-instance"));
        assert!(stdout_content.contains("✓ Done"));
    }
}
