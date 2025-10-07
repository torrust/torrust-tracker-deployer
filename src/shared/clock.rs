//! Clock abstraction for testable time management
//!
//! This module provides a clock abstraction that allows controlling time
//! in tests. Time is treated as an infrastructure concern, similar to
//! database or filesystem access.
//!
//! # Design Philosophy
//!
//! Direct use of `Utc::now()` throughout the codebase makes tests
//! non-deterministic and harder to maintain. By abstracting time behind
//! a trait, we can:
//!
//! - Control time in tests (set specific timestamps)
//! - Make tests deterministic and reproducible
//! - Test time-dependent behavior (timeouts, retries, etc.)
//! - Mock time progression without actual delays
//!
//! # Usage
//!
//! ## In Production Code
//!
//! ```rust
//! use torrust_tracker_deploy::shared::Clock;
//!
//! fn record_event(clock: &dyn Clock) {
//!     let timestamp = clock.now();
//!     println!("Event occurred at: {}", timestamp);
//! }
//! ```
//!
//! ## In Tests
//!
//! ```rust,no_run
//! // Note: MockClock is only available in test builds
//! # #[cfg(test)]
//! # {
//! use torrust_tracker_deploy::testing::MockClock;
//! use chrono::{DateTime, TimeZone, Utc};
//!
//! let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
//! let clock = MockClock::new(fixed_time);
//!
//! // Time is now fixed at the specified timestamp
//! assert_eq!(clock.now(), fixed_time);
//!
//! // Advance time by 5 seconds
//! clock.advance_secs(5);
//! assert_eq!(
//!     clock.now(),
//!     Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 5).unwrap()
//! );
//! # }
//! ```

use chrono::{DateTime, Utc};

/// Clock trait for obtaining the current time
///
/// This trait abstracts time acquisition, making it mockable in tests.
/// All time-dependent code should use this trait instead of calling
/// `Utc::now()` directly.
pub trait Clock: Send + Sync {
    /// Returns the current time in UTC
    fn now(&self) -> DateTime<Utc>;
}

/// System clock implementation using real system time
///
/// This is the production implementation that uses `Utc::now()`
/// to get the actual current time.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deploy::shared::{Clock, SystemClock};
///
/// let clock = SystemClock;
/// let now = clock.now();
/// println!("Current time: {}", now);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_current_system_time() {
        let clock = SystemClock;
        let before = Utc::now();
        let now = clock.now();
        let after = Utc::now();

        // Verify the returned time is between before and after
        assert!(now >= before);
        assert!(now <= after);
    }

    #[test]
    fn it_should_return_different_times_on_subsequent_calls() {
        let clock = SystemClock;
        let first = clock.now();

        // Small delay to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_millis(10));

        let second = clock.now();
        assert!(second > first);
    }
}
