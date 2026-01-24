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
//! use std::sync::Arc;
//! use std::cell::RefCell;
//! use parking_lot::ReentrantMutex;
//! use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
//! use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
//! let mut progress = ProgressReporter::new(output, 3);
//!
//! // Step 1: Load configuration
//! progress.start_step("Loading configuration")?;
//! // ... perform operation ...
//! progress.complete_step(Some("Configuration loaded: test-env"))?;
//!
//! // Step 2: Provision with sub-steps
//! progress.start_step("Provisioning infrastructure")?;
//! progress.sub_step("Creating virtual machine")?;
//! progress.sub_step("Configuring network")?;
//! // ... perform operations ...
//! progress.complete_step(Some("Instance created: test-instance"))?;
//!
//! // Step 3: Finalize
//! progress.start_step("Finalizing environment")?;
//! // ... perform operation ...
//! progress.complete_step(None)?;
//!
//! // Complete with summary
//! progress.complete("Environment 'test-env' created successfully")?;
//! # Ok(())
//! # }
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

use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::ReentrantMutex;

use thiserror::Error;

use crate::presentation::views::UserOutput;

/// Errors that can occur during progress reporting
#[derive(Debug, Error)]
pub enum ProgressReporterError {
    /// `UserOutput` mutex was poisoned
    ///
    /// The shared `UserOutput` mutex was poisoned by a panic in another thread.
    /// This indicates a critical internal error.
    #[error(
        "Internal error: UserOutput mutex was poisoned
Tip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    UserOutputMutexPoisoned,
}

/// Progress reporter for multi-step operations
///
/// Tracks progress through multiple steps of a long-running operation,
/// providing clear feedback with step numbers, descriptions, and timing.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use std::cell::RefCell;
/// use parking_lot::ReentrantMutex;
/// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
/// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
/// let mut progress = ProgressReporter::new(output, 2);
///
/// progress.start_step("Step 1")?;
/// progress.complete_step(Some("Step 1 done"))?;
///
/// progress.start_step("Step 2")?;
/// progress.complete_step(None)?;
///
/// progress.complete("All done!")?;
/// # Ok(())
/// # }
/// ```
pub struct ProgressReporter {
    output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    total_steps: usize,
    current_step: usize,
    step_start: Option<Instant>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    ///
    /// # Arguments
    ///
    /// * `output` - Shared user output handler for displaying messages
    /// * `total_steps` - Total number of steps in the operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let progress = ProgressReporter::new(output, 5);
    /// ```
    #[must_use]
    pub fn new(output: Arc<ReentrantMutex<RefCell<UserOutput>>>, total_steps: usize) -> Self {
        Self {
            output,
            total_steps,
            current_step: 0,
            step_start: None,
        }
    }

    /// Execute a function with the locked `UserOutput`
    ///
    /// With `ReentrantMutex`, we can safely lock multiple times on the same thread.
    /// The `RefCell` provides interior mutability.
    fn with_output<F, R>(&self, f: F) -> Result<R, ProgressReporterError>
    where
        F: FnOnce(&mut UserOutput) -> R,
    {
        let guard = self.output.lock();
        let mut user_output = guard
            .try_borrow_mut()
            .map_err(|_| ProgressReporterError::UserOutputMutexPoisoned)?;
        Ok(f(&mut user_output))
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
    /// # Errors
    ///
    /// Returns `ProgressReporterError::UserOutputMutexPoisoned` if the mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let mut progress = ProgressReporter::new(output, 3);
    ///
    /// progress.start_step("Loading configuration")?;
    /// // Output: ⏳ [1/3] Loading configuration...
    /// # Ok(())
    /// # }
    /// ```
    pub fn start_step(&mut self, description: &str) -> Result<(), ProgressReporterError> {
        self.current_step += 1;
        self.step_start = Some(Instant::now());

        self.with_output(|output| {
            output.progress(&format!(
                "[{}/{}] {}...",
                self.current_step, self.total_steps, description
            ));
        })?;

        Ok(())
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
    /// # Errors
    ///
    /// Returns `ProgressReporterError::UserOutputMutexPoisoned` if the mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let mut progress = ProgressReporter::new(output, 2);
    ///
    /// progress.start_step("Loading data")?;
    /// progress.complete_step(Some("Data loaded successfully"))?;
    /// // Output: ✓ Data loaded successfully (took 150ms)
    ///
    /// progress.start_step("Processing")?;
    /// progress.complete_step(None)?;
    /// // Output: ✓ Done (took 2.3s)
    /// # Ok(())
    /// # }
    /// ```
    pub fn complete_step(&mut self, result: Option<&str>) -> Result<(), ProgressReporterError> {
        if let Some(start) = self.step_start {
            let duration = start.elapsed();
            self.with_output(|output| {
                if let Some(msg) = result {
                    output.progress(&format!("  ✓ {} (took {})", msg, format_duration(duration)));
                } else {
                    output.progress(&format!("  ✓ Done (took {})", format_duration(duration)));
                }
            })?;
        }

        self.step_start = None;
        Ok(())
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
    /// # Errors
    ///
    /// Returns `ProgressReporterError::UserOutputMutexPoisoned` if the mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let mut progress = ProgressReporter::new(output.clone(), 1);
    ///
    /// progress.start_step("Provisioning infrastructure")?;
    /// progress.sub_step("Creating virtual machine")?;
    /// progress.sub_step("Configuring network")?;
    /// progress.sub_step("Setting up storage")?;
    /// progress.complete_step(Some("Infrastructure ready"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sub_step(&mut self, description: &str) -> Result<(), ProgressReporterError> {
        self.with_output(|output| {
            output.progress(&format!("    → {description}"));
        })?;
        Ok(())
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
    /// # Errors
    ///
    /// Returns `ProgressReporterError::UserOutputMutexPoisoned` if the mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let mut progress = ProgressReporter::new(output.clone(), 1);
    ///
    /// progress.start_step("Creating environment")?;
    /// progress.complete_step(None)?;
    /// progress.complete("Environment 'test-env' created successfully")?;
    /// // Output: ✅ Environment 'test-env' created successfully
    /// # Ok(())
    /// # }
    /// ```
    pub fn complete(&mut self, summary: &str) -> Result<(), ProgressReporterError> {
        self.with_output(|output| output.success(summary))?;
        Ok(())
    }

    /// Get a reference to the shared `UserOutput`
    ///
    /// This allows using other output methods (like `error`, `warn`)
    /// while progress is being tracked.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let mut progress = ProgressReporter::new(output.clone(), 1);
    ///
    /// progress.start_step("Checking conditions");
    /// progress.output().lock().borrow_mut().warn("Some non-critical warning");
    /// progress.complete_step(None);
    /// ```
    #[must_use]
    pub fn output(&self) -> &Arc<ReentrantMutex<RefCell<UserOutput>>> {
        &self.output
    }

    /// Add a blank line to the output
    ///
    /// This is a wrapper around `UserOutput::blank_line()` that handles
    /// mutex acquisition with timeout protection.
    ///
    /// # Errors
    ///
    /// Returns `ProgressReporterError::UserOutputMutexPoisoned` if the mutex is poisoned.
    /// Returns `ProgressReporterError::UserOutputMutexTimeout` if the mutex cannot be acquired within the timeout.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let mut progress = ProgressReporter::new(output, 3);
    ///
    /// progress.blank_line()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn blank_line(&self) -> Result<(), ProgressReporterError> {
        self.with_output(UserOutput::blank_line)?;
        Ok(())
    }

    /// Display a list of steps with a title
    ///
    /// This is a wrapper around `UserOutput::steps()` that handles
    /// mutex acquisition with timeout protection.
    ///
    /// # Arguments
    ///
    /// * `title` - The title for the steps list
    /// * `steps` - Array of step descriptions
    ///
    /// # Errors
    ///
    /// Returns `ProgressReporterError::UserOutputMutexPoisoned` if the mutex is poisoned.
    /// Returns `ProgressReporterError::UserOutputMutexTimeout` if the mutex cannot be acquired within the timeout.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let mut progress = ProgressReporter::new(output, 3);
    ///
    /// progress.steps("Next steps:", &[
    ///     "Edit the configuration file",
    ///     "Review the settings",
    ///     "Run the deploy command"
    /// ])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn steps(&self, title: &str, steps: &[&str]) -> Result<(), ProgressReporterError> {
        self.with_output(|output| output.steps(title, steps))?;
        Ok(())
    }

    /// Output result data to stdout
    ///
    /// Wraps `UserOutput::result()` to write result data to stdout.
    /// Result data goes to stdout (not stderr) so it can be piped or redirected.
    ///
    /// # Arguments
    ///
    /// * `message` - The result data to output
    ///
    /// # Errors
    ///
    /// Returns error if the user output mutex is poisoned
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use std::cell::RefCell;
    /// use parking_lot::ReentrantMutex;
    /// use torrust_tracker_deployer_lib::presentation::views::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let progress = ProgressReporter::new(output, 1);
    ///
    /// progress.result(r#"{"schema": "..."}"#)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn result(&self, message: &str) -> Result<(), ProgressReporterError> {
        self.with_output(|output| output.result(message))?;
        Ok(())
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
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;

    #[test]
    fn it_should_create_progress_reporter_with_total_steps() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, _stderr) = test_output.into_reentrant_wrapped();
        let progress = ProgressReporter::new(output, 5);

        assert_eq!(progress.total_steps, 5);
        assert_eq!(progress.current_step, 0);
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_start_step_and_increment_counter() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 3);

        progress
            .start_step("Loading configuration")
            .expect("Failed to start step");

        assert_eq!(progress.current_step, 1);
        assert!(progress.step_start.is_some());

        let stderr_content = String::from_utf8(stderr.lock().clone()).unwrap();
        assert!(stderr_content.contains("[1/3] Loading configuration..."));
    }

    #[test]
    fn it_should_track_multiple_steps() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 3);

        progress
            .start_step("Step 1")
            .expect("Failed to start step 1");
        assert_eq!(progress.current_step, 1);

        progress
            .start_step("Step 2")
            .expect("Failed to start step 2");
        assert_eq!(progress.current_step, 2);

        progress
            .start_step("Step 3")
            .expect("Failed to start step 3");
        assert_eq!(progress.current_step, 3);

        let stderr_content = String::from_utf8(stderr.lock().clone()).unwrap();
        assert!(stderr_content.contains("[1/3] Step 1..."));
        assert!(stderr_content.contains("[2/3] Step 2..."));
        assert!(stderr_content.contains("[3/3] Step 3..."));
    }

    #[test]
    fn it_should_complete_step_with_result_message() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 1);

        progress
            .start_step("Loading data")
            .expect("Failed to start step");
        progress
            .complete_step(Some("Data loaded successfully"))
            .expect("Failed to complete step");

        let stderr_content = String::from_utf8(stderr.lock().clone()).unwrap();
        assert!(stderr_content.contains("✓ Data loaded successfully"));
        assert!(stderr_content.contains("took"));
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_complete_step_without_result_message() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 1);

        progress
            .start_step("Processing")
            .expect("Failed to start step");
        progress
            .complete_step(None)
            .expect("Failed to complete step");

        let stderr_content = String::from_utf8(stderr.lock().clone()).unwrap();
        assert!(stderr_content.contains("✓ Done"));
        assert!(stderr_content.contains("took"));
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_report_sub_steps() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 1);

        progress
            .start_step("Provisioning")
            .expect("Failed to start step");
        progress
            .sub_step("Creating VM")
            .expect("Failed to report sub-step");
        progress
            .sub_step("Configuring network")
            .expect("Failed to report sub-step");
        progress
            .complete_step(None)
            .expect("Failed to complete step");

        let stderr_content = String::from_utf8(stderr.lock().clone()).unwrap();
        assert!(stderr_content.contains("→ Creating VM"));
        assert!(stderr_content.contains("→ Configuring network"));
    }

    #[test]
    fn it_should_display_completion_summary() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 1);

        progress
            .start_step("Creating environment")
            .expect("Failed to start step");
        progress
            .complete_step(None)
            .expect("Failed to complete step");
        progress
            .complete("Environment created successfully")
            .expect("Failed to complete");

        let stderr_content = String::from_utf8(stderr.lock().clone()).unwrap();
        assert!(stderr_content.contains("✅ Environment created successfully"));
    }

    #[test]
    fn it_should_provide_access_to_output() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let progress = ProgressReporter::new(output.clone(), 1);

        progress
            .with_output(|user_output| user_output.warn("Test warning"))
            .expect("Failed to write to output");

        let stderr_content = String::from_utf8(stderr.lock().clone()).expect("Invalid UTF-8");
        assert!(stderr_content.contains("⚠️  Test warning"));
    }

    #[test]
    fn it_should_respect_verbosity_levels() {
        let test_output = TestUserOutput::new(VerbosityLevel::Quiet);
        let (output, _stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 1);

        progress.start_step("Step 1").expect("Failed to start step");
        progress
            .complete_step(Some("Done"))
            .expect("Failed to complete step");

        // At Quiet level, progress messages should not appear
        let stderr_content = String::from_utf8(stderr.lock().clone()).expect("Invalid UTF-8");
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
        let duration = Duration::from_secs(1);
        assert_eq!(format_duration(duration), "1.0s");

        let duration = Duration::from_millis(2345);
        assert_eq!(format_duration(duration), "2.3s");

        let duration = Duration::from_secs(10);
        assert_eq!(format_duration(duration), "10.0s");
    }

    #[test]
    fn it_should_handle_full_workflow() {
        let test_output = TestUserOutput::new(VerbosityLevel::Normal);
        let (output, stdout, stderr) = test_output.into_reentrant_wrapped();
        let mut progress = ProgressReporter::new(output, 3);

        // Step 1
        progress
            .start_step("Loading configuration")
            .expect("Failed to start step 1");
        progress
            .complete_step(Some("Configuration loaded: test-env"))
            .expect("Failed to complete step 1");

        // Step 2 with sub-steps
        progress
            .start_step("Provisioning infrastructure")
            .expect("Failed to start step 2");
        progress
            .sub_step("Creating virtual machine")
            .expect("Failed to report sub-step");
        progress
            .sub_step("Configuring network")
            .expect("Failed to report sub-step");
        progress
            .complete_step(Some("Instance created: test-instance"))
            .expect("Failed to complete step 2");

        // Step 3
        progress
            .start_step("Finalizing environment")
            .expect("Failed to start step 3");
        progress
            .complete_step(None)
            .expect("Failed to complete step 3");

        // Complete
        progress
            .complete("Environment 'test-env' created successfully")
            .expect("Failed to complete");

        let stderr_content = String::from_utf8(stderr.lock().clone()).expect("Invalid UTF-8");
        assert!(stderr_content.contains("[1/3] Loading configuration..."));
        assert!(stderr_content.contains("[2/3] Provisioning infrastructure..."));
        assert!(stderr_content.contains("[3/3] Finalizing environment..."));
        assert!(stderr_content.contains("✅ Environment 'test-env' created successfully"));
        assert!(stderr_content.contains("✓ Configuration loaded: test-env"));
        assert!(stderr_content.contains("→ Creating virtual machine"));
        assert!(stderr_content.contains("→ Configuring network"));
        assert!(stderr_content.contains("✓ Instance created: test-instance"));
        assert!(stderr_content.contains("✓ Done"));

        let stdout_content = String::from_utf8(stdout.lock().clone()).expect("Invalid UTF-8");
        // stdout should be empty - all progress goes to stderr
        assert!(stdout_content.is_empty());
    }
}
