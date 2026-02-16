//! Recording progress listener for testing
//!
//! This module provides a `RecordingProgressListener` that captures all progress
//! events emitted by command handlers for test assertions. It implements the
//! `CommandProgressListener` trait and stores events in a thread-safe `Vec`.

use std::sync::Arc;

use parking_lot::Mutex;

use crate::application::ports::CommandProgressListener;

/// A recorded progress event from a command handler.
///
/// Each variant captures the arguments passed to the corresponding
/// `CommandProgressListener` method, enabling precise test assertions
/// about the sequence and content of progress reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEvent {
    /// Recorded from `on_step_started(step_number, total_steps, description)`
    StepStarted {
        step_number: usize,
        total_steps: usize,
        description: String,
    },
    /// Recorded from `on_step_completed(step_number, description)`
    StepCompleted {
        step_number: usize,
        description: String,
    },
    /// Recorded from `on_detail(message)`
    Detail { message: String },
    /// Recorded from `on_debug(message)`
    Debug { message: String },
}

/// A test double that records all progress events for later assertion.
///
/// Uses interior mutability (`Arc<Mutex<Vec<ProgressEvent>>>`) to allow
/// shared ownership and mutation while implementing the `Send + Sync`
/// trait bounds required by `CommandProgressListener`.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::testing::RecordingProgressListener;
/// use torrust_tracker_deployer_lib::testing::ProgressEvent;
/// use torrust_tracker_deployer_lib::application::ports::CommandProgressListener;
///
/// let listener = RecordingProgressListener::new();
///
/// listener.on_step_started(1, 3, "First step");
/// listener.on_detail("some detail");
/// listener.on_step_completed(1, "First step");
///
/// let events = listener.events();
/// assert_eq!(events.len(), 3);
/// assert_eq!(events[0], ProgressEvent::StepStarted {
///     step_number: 1,
///     total_steps: 3,
///     description: "First step".to_string(),
/// });
/// ```
#[derive(Debug, Clone)]
pub struct RecordingProgressListener {
    events: Arc<Mutex<Vec<ProgressEvent>>>,
}

impl RecordingProgressListener {
    /// Creates a new empty recording listener.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns a snapshot of all recorded events.
    #[must_use]
    pub fn events(&self) -> Vec<ProgressEvent> {
        self.events.lock().clone()
    }

    /// Returns only `StepStarted` events, filtered from all recorded events.
    #[must_use]
    pub fn step_started_events(&self) -> Vec<ProgressEvent> {
        self.events
            .lock()
            .iter()
            .filter(|e| matches!(e, ProgressEvent::StepStarted { .. }))
            .cloned()
            .collect()
    }

    /// Returns the number of recorded events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.lock().len()
    }

    /// Clears all recorded events.
    pub fn clear(&self) {
        self.events.lock().clear();
    }
}

impl Default for RecordingProgressListener {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandProgressListener for RecordingProgressListener {
    fn on_step_started(&self, step_number: usize, total_steps: usize, description: &str) {
        self.events.lock().push(ProgressEvent::StepStarted {
            step_number,
            total_steps,
            description: description.to_string(),
        });
    }

    fn on_step_completed(&self, step_number: usize, description: &str) {
        self.events.lock().push(ProgressEvent::StepCompleted {
            step_number,
            description: description.to_string(),
        });
    }

    fn on_detail(&self, message: &str) {
        self.events.lock().push(ProgressEvent::Detail {
            message: message.to_string(),
        });
    }

    fn on_debug(&self, message: &str) {
        self.events.lock().push(ProgressEvent::Debug {
            message: message.to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_record_step_started_events() {
        let listener = RecordingProgressListener::new();

        listener.on_step_started(1, 9, "Rendering templates");
        listener.on_step_started(2, 9, "Initializing infrastructure");

        let events = listener.events();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0],
            ProgressEvent::StepStarted {
                step_number: 1,
                total_steps: 9,
                description: "Rendering templates".to_string(),
            }
        );
        assert_eq!(
            events[1],
            ProgressEvent::StepStarted {
                step_number: 2,
                total_steps: 9,
                description: "Initializing infrastructure".to_string(),
            }
        );
    }

    #[test]
    fn it_should_record_all_event_types_in_order() {
        let listener = RecordingProgressListener::new();

        listener.on_step_started(1, 3, "Step one");
        listener.on_detail("detail info");
        listener.on_debug("debug info");
        listener.on_step_completed(1, "Step one");

        let events = listener.events();
        assert_eq!(events.len(), 4);
        assert!(matches!(&events[0], ProgressEvent::StepStarted { .. }));
        assert!(matches!(&events[1], ProgressEvent::Detail { .. }));
        assert!(matches!(&events[2], ProgressEvent::Debug { .. }));
        assert!(matches!(&events[3], ProgressEvent::StepCompleted { .. }));
    }

    #[test]
    fn it_should_filter_step_started_events() {
        let listener = RecordingProgressListener::new();

        listener.on_step_started(1, 3, "First");
        listener.on_detail("detail");
        listener.on_step_started(2, 3, "Second");
        listener.on_debug("debug");

        let step_events = listener.step_started_events();
        assert_eq!(step_events.len(), 2);
    }

    #[test]
    fn it_should_clear_recorded_events() {
        let listener = RecordingProgressListener::new();

        listener.on_step_started(1, 3, "Step one");
        assert_eq!(listener.event_count(), 1);

        listener.clear();
        assert_eq!(listener.event_count(), 0);
        assert!(listener.events().is_empty());
    }

    #[test]
    fn it_should_work_as_trait_object() {
        let listener = RecordingProgressListener::new();
        let trait_obj: &dyn CommandProgressListener = &listener;

        trait_obj.on_step_started(1, 5, "Test step");
        trait_obj.on_detail("Some detail");

        assert_eq!(listener.event_count(), 2);
    }
}
