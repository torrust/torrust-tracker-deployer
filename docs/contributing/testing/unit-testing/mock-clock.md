# Using the Clock Service for Deterministic Time Tests

Time is treated as an infrastructure concern. Always use the `Clock` trait instead of calling `Utc::now()` directly to make time-dependent code testable and deterministic.

## Why Use Clock Service?

Direct use of `Utc::now()` makes tests:

- **Non-deterministic**: Tests produce different results on each run
- **Hard to test**: Cannot control time progression or test specific timestamps
- **Difficult to debug**: Time-related issues are hard to reproduce

The Clock service solves these problems by:

- **Controlling time in tests**: Set specific timestamps for predictable behavior
- **Making tests deterministic**: Same test always produces same result
- **Testing time-dependent logic**: Simulate time progression without actual delays
- **Enabling edge case testing**: Test timeouts, expiration, and time-based conditions

## Production Code

In production code, inject the `Clock` dependency:

```rust
use crate::shared::Clock;
use chrono::{DateTime, Utc};

pub struct EventRecorder {
    clock: Arc<dyn Clock>,
}

impl EventRecorder {
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self { clock }
    }

    pub fn record_event(&self) -> DateTime<Utc> {
        // Use clock.now() instead of Utc::now()
        let timestamp = self.clock.now();
        println!("Event recorded at: {}", timestamp);
        timestamp
    }
}
```

## Test Code

In tests, use `MockClock` for full control over time:

```rust
use crate::testing::MockClock;
use chrono::{TimeZone, Utc};
use std::sync::Arc;

#[test]
fn it_should_record_event_with_specific_timestamp() {
    // Arrange: Set up mock clock with fixed time
    let fixed_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let clock = Arc::new(MockClock::new(fixed_time));
    let recorder = EventRecorder::new(clock.clone());

    // Act: Record event
    let recorded_time = recorder.record_event();

    // Assert: Verify exact timestamp
    assert_eq!(recorded_time, fixed_time);
}

#[test]
fn it_should_handle_time_progression() {
    // Arrange: Set up mock clock
    let start_time = Utc.with_ymd_and_hms(2025, 10, 7, 12, 0, 0).unwrap();
    let clock = Arc::new(MockClock::new(start_time));
    let recorder = EventRecorder::new(clock.clone());

    // Act: Record first event
    let first_event = recorder.record_event();

    // Simulate 5 minutes passing
    clock.advance_secs(300);

    // Record second event
    let second_event = recorder.record_event();

    // Assert: Verify time difference
    let expected_second = Utc.with_ymd_and_hms(2025, 10, 7, 12, 5, 0).unwrap();
    assert_eq!(first_event, start_time);
    assert_eq!(second_event, expected_second);
}
```

## Key Benefits

- **Deterministic Tests**: Tests always produce the same results
- **Fast Execution**: No need for actual time delays with `sleep()`
- **Edge Case Testing**: Easily test timeouts, expirations, and time boundaries
- **Improved Debugging**: Failures are reproducible with exact timestamps
- **Better Test Coverage**: Can test time-dependent scenarios that would be impractical otherwise

## When to Use

Use `MockClock` when testing:

- Timestamp generation and recording
- Timeout and expiration logic
- Time-based retries and backoff strategies
- Duration calculations and measurements
- Time-series data processing
- Scheduled operations and cron-like behavior
