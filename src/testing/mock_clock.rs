//! Mock clock for testing
//!
//! This module provides a mock clock implementation that allows
//! controlling time in tests for deterministic behavior.

use chrono::{DateTime, Duration, Utc};
use std::sync::{Arc, Mutex};

use crate::shared::Clock;

/// Mock clock for testing that allows controlling time
///
/// This clock implementation allows tests to:
/// - Set a fixed time point
/// - Advance time manually without actual delays
/// - Make time-dependent tests deterministic
///
/// The clock uses interior mutability to allow advancing time
/// while implementing the `Clock` trait which takes `&self`.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer::testing::MockClock;
/// use chrono::{TimeZone, Utc};
///
/// let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
/// let clock = MockClock::new(fixed_time);
///
/// // Time is fixed
/// assert_eq!(clock.now(), fixed_time);
/// assert_eq!(clock.now(), fixed_time); // Still the same
///
/// // Advance time
/// clock.advance_secs(60);
/// let expected = Utc.with_ymd_and_hms(2025, 10, 7, 12, 1, 0).unwrap();
/// assert_eq!(clock.now(), expected);
/// ```
#[derive(Debug, Clone)]
pub struct MockClock {
    /// Current time maintained by the mock clock
    current_time: Arc<Mutex<DateTime<Utc>>>,
}

impl MockClock {
    /// Create a new mock clock with a fixed starting time
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer::testing::MockClock;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    /// let clock = MockClock::new(fixed_time);
    /// ```
    #[must_use]
    pub fn new(initial_time: DateTime<Utc>) -> Self {
        Self {
            current_time: Arc::new(Mutex::new(initial_time)),
        }
    }

    /// Advance the clock by the specified duration
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (which would indicate a panic occurred
    /// while holding the lock in another thread).
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer::testing::MockClock;
    /// use chrono::{Duration, TimeZone, Utc};
    ///
    /// let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    /// let clock = MockClock::new(initial);
    ///
    /// clock.advance(Duration::hours(2));
    /// let expected = Utc.with_ymd_and_hms(2025, 10, 7, 14, 0, 0).unwrap();
    /// assert_eq!(clock.now(), expected);
    /// ```
    pub fn advance(&self, duration: Duration) {
        let mut time = self.current_time.lock().expect("MockClock mutex poisoned");
        *time += duration;
    }

    /// Advance the clock by the specified number of seconds
    ///
    /// Convenience method for advancing time without creating a `Duration`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer::testing::MockClock;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    /// let clock = MockClock::new(initial);
    ///
    /// clock.advance_secs(30);
    /// let expected = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 30).unwrap();
    /// assert_eq!(clock.now(), expected);
    /// ```
    pub fn advance_secs(&self, secs: i64) {
        self.advance(Duration::seconds(secs));
    }

    /// Set the clock to a specific time
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (which would indicate a panic occurred
    /// while holding the lock in another thread).
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer::testing::MockClock;
    /// use chrono::{TimeZone, Utc};
    ///
    /// let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    /// let clock = MockClock::new(initial);
    ///
    /// let new_time = Utc.with_ymd_and_hms(2025, 12, 25, 18, 30, 0).unwrap();
    /// clock.set_time(new_time);
    /// assert_eq!(clock.now(), new_time);
    /// ```
    pub fn set_time(&self, time: DateTime<Utc>) {
        let mut current = self.current_time.lock().expect("MockClock mutex poisoned");
        *current = time;
    }
}

impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        *self.current_time.lock().expect("MockClock mutex poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn it_should_return_fixed_time_when_not_advanced() {
        let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = MockClock::new(fixed_time);

        assert_eq!(clock.now(), fixed_time);
        assert_eq!(clock.now(), fixed_time);
        assert_eq!(clock.now(), fixed_time);
    }

    #[test]
    fn it_should_advance_time_by_duration() {
        let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = MockClock::new(initial);

        clock.advance(Duration::hours(2) + Duration::minutes(30));

        let expected = Utc.with_ymd_and_hms(2025, 10, 7, 14, 30, 0).unwrap();
        assert_eq!(clock.now(), expected);
    }

    #[test]
    fn it_should_advance_time_by_seconds() {
        let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = MockClock::new(initial);

        clock.advance_secs(90);

        let expected = Utc.with_ymd_and_hms(2025, 10, 7, 12, 1, 30).unwrap();
        assert_eq!(clock.now(), expected);
    }

    #[test]
    fn it_should_set_time_to_specific_point() {
        let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = MockClock::new(initial);

        let new_time = Utc.with_ymd_and_hms(2025, 12, 25, 18, 30, 0).unwrap();
        clock.set_time(new_time);

        assert_eq!(clock.now(), new_time);
    }

    #[test]
    fn it_should_support_multiple_advances() {
        let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock = MockClock::new(initial);

        clock.advance_secs(30);
        clock.advance_secs(30);
        clock.advance_secs(30);

        let expected = Utc.with_ymd_and_hms(2025, 10, 7, 12, 1, 30).unwrap();
        assert_eq!(clock.now(), expected);
    }

    #[test]
    fn it_should_be_clonable() {
        let initial = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
        let clock1 = MockClock::new(initial);
        let clock2 = clock1.clone();

        // Both clones share the same time
        clock1.advance_secs(60);
        assert_eq!(clock1.now(), clock2.now());
    }
}
