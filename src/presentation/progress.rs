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
//! use std::sync::{Arc, Mutex};
//! use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
//! use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
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

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use thiserror::Error;

use crate::presentation::user_output::UserOutput;

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
/// use std::sync::{Arc, Mutex};
/// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
/// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
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
    output: Arc<Mutex<UserOutput>>,
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
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let progress = ProgressReporter::new(output, 5);
    /// ```
    #[must_use]
    pub fn new(output: Arc<Mutex<UserOutput>>, total_steps: usize) -> Self {
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
    /// # Errors
    ///
    /// Returns `ProgressReporterError::UserOutputMutexPoisoned` if the mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
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

        self.output
            .lock()
            .map_err(|_| ProgressReporterError::UserOutputMutexPoisoned)?
            .progress(&format!(
                "[{}/{}] {}...",
                self.current_step, self.total_steps, description
            ));

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
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
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
            let mut output = self
                .output
                .lock()
                .map_err(|_| ProgressReporterError::UserOutputMutexPoisoned)?;

            if let Some(msg) = result {
                output.result(&format!("  ✓ {} (took {})", msg, format_duration(duration)));
            } else {
                output.result(&format!("  ✓ Done (took {})", format_duration(duration)));
            }
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
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
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
        self.output
            .lock()
            .map_err(|_| ProgressReporterError::UserOutputMutexPoisoned)?
            .result(&format!("    → {description}"));
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
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
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
        self.output
            .lock()
            .map_err(|_| ProgressReporterError::UserOutputMutexPoisoned)?
            .success(summary);
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
    /// use std::sync::{Arc, Mutex};
    /// use torrust_tracker_deployer_lib::presentation::progress::ProgressReporter;
    /// use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};
    ///
    /// let output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Normal)));
    /// let mut progress = ProgressReporter::new(output.clone(), 1);
    ///
    /// progress.start_step("Checking conditions");
    /// progress.output().lock().unwrap().warn("Some non-critical warning");
    /// progress.complete_step(None);
    /// ```
    #[must_use]
    pub fn output(&self) -> &Arc<Mutex<UserOutput>> {
        &self.output
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

    /// Helper to create wrapped test `UserOutput` for `ProgressReporter`
    ///
    /// Returns: (`Arc<Mutex<UserOutput>>`, Arc to stdout buffer, Arc to stderr buffer)
    #[allow(clippy::type_complexity)]
    fn create_wrapped_test_output(
        verbosity: VerbosityLevel,
    ) -> (
        Arc<Mutex<UserOutput>>,
        Arc<Mutex<Vec<u8>>>,
        Arc<Mutex<Vec<u8>>>,
    ) {
        let (output, stdout, stderr) = create_test_user_output(verbosity);
        (Arc::new(Mutex::new(output)), stdout, stderr)
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
        let (output, _stdout, _stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
        let progress = ProgressReporter::new(output, 5);

        assert_eq!(progress.total_steps, 5);
        assert_eq!(progress.current_step, 0);
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_start_step_and_increment_counter() {
        let (output, _stdout, stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 3);

        progress
            .start_step("Loading configuration")
            .expect("Failed to start step");

        assert_eq!(progress.current_step, 1);
        assert!(progress.step_start.is_some());

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("[1/3] Loading configuration..."));
    }

    #[test]
    fn it_should_track_multiple_steps() {
        let (output, _stdout, stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
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

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("[1/3] Step 1..."));
        assert!(stderr_content.contains("[2/3] Step 2..."));
        assert!(stderr_content.contains("[3/3] Step 3..."));
    }

    #[test]
    fn it_should_complete_step_with_result_message() {
        let (output, stdout, _stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 1);

        progress
            .start_step("Loading data")
            .expect("Failed to start step");
        progress
            .complete_step(Some("Data loaded successfully"))
            .expect("Failed to complete step");

        let stdout_content = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(stdout_content.contains("✓ Data loaded successfully"));
        assert!(stdout_content.contains("took"));
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_complete_step_without_result_message() {
        let (output, stdout, _stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
        let mut progress = ProgressReporter::new(output, 1);

        progress
            .start_step("Processing")
            .expect("Failed to start step");
        progress
            .complete_step(None)
            .expect("Failed to complete step");

        let stdout_content = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(stdout_content.contains("✓ Done"));
        assert!(stdout_content.contains("took"));
        assert!(progress.step_start.is_none());
    }

    #[test]
    fn it_should_report_sub_steps() {
        let (output, stdout, _stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
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

        let stdout_content = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
        assert!(stdout_content.contains("→ Creating VM"));
        assert!(stdout_content.contains("→ Configuring network"));
    }

    #[test]
    fn it_should_display_completion_summary() {
        let (output, _stdout, stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
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

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("✅ Environment created successfully"));
    }

    #[test]
    fn it_should_provide_access_to_output() {
        let (output, _stdout, stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
        let progress = ProgressReporter::new(output, 1);

        progress
            .output()
            .lock()
            .expect("UserOutput mutex poisoned")
            .warn("Test warning");

        let stderr_content = String::from_utf8(stderr.lock().unwrap().clone()).unwrap();
        assert!(stderr_content.contains("⚠️  Test warning"));
    }

    #[test]
    fn it_should_respect_verbosity_levels() {
        let (output, _stdout, stderr) = create_wrapped_test_output(VerbosityLevel::Quiet);
        let mut progress = ProgressReporter::new(output, 1);

        progress.start_step("Step 1").expect("Failed to start step");
        progress
            .complete_step(Some("Done"))
            .expect("Failed to complete step");

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
        let (output, stdout, stderr) = create_wrapped_test_output(VerbosityLevel::Normal);
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
