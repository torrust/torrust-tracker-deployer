//! Verbose progress listener implementation
//!
//! This module provides the presentation-layer implementation of the
//! `CommandProgressListener` trait. It translates application-layer progress
//! events into user-facing output through `UserOutput`.

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::ports::CommandProgressListener;
use crate::presentation::views::UserOutput;

/// Presentation layer implementation of `CommandProgressListener`.
///
/// Translates application-layer progress events into user-facing output
/// through `UserOutput`. The verbosity filtering is handled automatically
/// by the `VerbosityFilter` inside `UserOutput` — this listener simply
/// emits all messages at their appropriate verbosity level.
///
/// # DDD Layer Placement
///
/// - **Defined in**: Presentation layer (`src/presentation/views/progress/`)
/// - **Implements**: Application layer trait (`CommandProgressListener`)
/// - **Dependency direction**: Presentation → Application (correct)
///
/// # Examples
///
/// ```rust,ignore
/// use torrust_tracker_deployer_lib::presentation::views::progress::VerboseProgressListener;
///
/// let listener = VerboseProgressListener::new(user_output_arc.clone());
/// listener.on_step_started(1, 9, "Rendering OpenTofu templates");
/// // Outputs: 📋   [Step 1/9] Rendering OpenTofu templates...
/// ```
pub struct VerboseProgressListener {
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
}

impl VerboseProgressListener {
    /// Create a new `VerboseProgressListener` wrapping a shared `UserOutput`
    ///
    /// # Arguments
    ///
    /// * `user_output` - Shared `UserOutput` instance for emitting messages
    #[must_use]
    pub fn new(user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        Self { user_output }
    }

    /// Access the shared `UserOutput` and execute a closure with mutable access
    fn with_output<F>(&self, f: F)
    where
        F: FnOnce(&mut UserOutput),
    {
        let guard = self.user_output.lock();
        let mut output = guard.borrow_mut();
        f(&mut output);
    }
}

impl CommandProgressListener for VerboseProgressListener {
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str) {
        self.with_output(|output| {
            output.detail(&format!(
                "  [Step {step_number}/{total_steps}] {description}..."
            ));
        });
    }

    fn on_step_completed(&self, _step_number: usize, _description: &str) {
        // Steps complete silently — the next on_step_started provides
        // visual progress. Completion details go through on_detail().
    }

    fn on_detail(&self, message: &str) {
        self.with_output(|output| {
            output.detail(&format!("     → {message}"));
        });
    }

    fn on_debug(&self, message: &str) {
        self.with_output(|output| {
            output.debug_detail(&format!("     → {message}"));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::views::testing::TestUserOutput;
    use crate::presentation::views::VerbosityLevel;

    fn create_listener(
        verbosity: VerbosityLevel,
    ) -> (VerboseProgressListener, Arc<parking_lot::Mutex<Vec<u8>>>) {
        let test_output = TestUserOutput::new(verbosity);
        let stderr_buffer = Arc::clone(&test_output.stderr_buffer);
        let (wrapped, _stdout, _stderr) = test_output.into_reentrant_wrapped();
        let listener = VerboseProgressListener::new(wrapped);
        (listener, stderr_buffer)
    }

    #[test]
    fn it_should_emit_step_started_at_verbose_level() {
        let (listener, stderr_buffer) = create_listener(VerbosityLevel::Verbose);

        listener.on_step_started(1, 9, "Rendering OpenTofu templates");

        let output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
        assert!(
            output.contains("[Step 1/9] Rendering OpenTofu templates..."),
            "Expected step message in output, got: {output}"
        );
    }

    #[test]
    fn it_should_not_emit_step_started_at_normal_level() {
        let (listener, stderr_buffer) = create_listener(VerbosityLevel::Normal);

        listener.on_step_started(1, 9, "Rendering OpenTofu templates");

        let output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
        assert!(
            output.is_empty(),
            "Expected no output at Normal level, got: {output}"
        );
    }

    #[test]
    fn it_should_emit_detail_at_verbose_level() {
        let (listener, stderr_buffer) = create_listener(VerbosityLevel::Verbose);

        listener.on_detail("Template directory: build/test/tofu");

        let output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
        assert!(
            output.contains("→ Template directory: build/test/tofu"),
            "Expected detail message in output, got: {output}"
        );
    }

    #[test]
    fn it_should_emit_debug_only_at_debug_level() {
        let (listener, stderr_buffer) = create_listener(VerbosityLevel::Verbose);

        listener.on_debug("Command: tofu init");

        let output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
        assert!(
            output.is_empty(),
            "Expected no debug output at Verbose level, got: {output}"
        );
    }

    #[test]
    fn it_should_emit_debug_at_debug_level() {
        let (listener, stderr_buffer) = create_listener(VerbosityLevel::Debug);

        listener.on_debug("Command: tofu init");

        let output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
        assert!(
            output.contains("→ Command: tofu init"),
            "Expected debug message in output, got: {output}"
        );
    }

    #[test]
    fn it_should_format_all_nine_steps_correctly() {
        let (listener, stderr_buffer) = create_listener(VerbosityLevel::Verbose);

        let steps = [
            "Rendering OpenTofu templates",
            "Initializing OpenTofu",
            "Validating infrastructure configuration",
            "Planning infrastructure changes",
            "Applying infrastructure changes",
            "Retrieving instance information",
            "Rendering Ansible templates",
            "Waiting for SSH connectivity",
            "Waiting for cloud-init completion",
        ];

        for (i, description) in steps.iter().enumerate() {
            listener.on_step_started(i + 1, 9, description);
        }

        let output = String::from_utf8(stderr_buffer.lock().clone()).unwrap();
        for (i, description) in steps.iter().enumerate() {
            let expected = format!("[Step {}/9] {description}...", i + 1);
            assert!(
                output.contains(&expected),
                "Missing step {}: {expected}\nFull output:\n{output}",
                i + 1
            );
        }
    }
}
