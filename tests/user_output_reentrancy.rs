//! Integration tests for `UserOutput` reentrancy fix
//!
//! These tests verify that the `ReentrantMutex` solution prevents deadlocks when
//! acquisitions of `UserOutput` locks would cause deadlocks.
//!
//! ## References
//!
//! - ADR: Use `ReentrantMutex` Pattern for `UserOutput` Reentrancy (docs/decisions/reentrant-mutex-useroutput-pattern.md)
//!
//! ## Problem Context
//!
//! Controller → `ProgressReporter` → `UserOutput` (same thread, multiple acquisitions)
//!
//! ## Solution Components
//!
//! - `Arc<ReentrantMutex<RefCell<UserOutput>>>`: Allows same-thread reentrancy
//! - `RefCell`: Provides interior mutability for `&mut self` `UserOutput` methods
//! - `ReentrantMutex`: Enables same-thread multiple lock acquisitions without deadlock

use std::cell::RefCell;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use torrust_tracker_deployer_lib::presentation::user_output::{UserOutput, VerbosityLevel};

/// Test that verifies same-thread reentrancy works without deadlocking
///
/// This test simulates the exact deadlock scenario that occurred in issue #164:
/// 1. Thread acquires `UserOutput` lock (simulating controller context)
/// 2. Same thread tries to acquire `UserOutput` lock again (simulating `ProgressReporter`)
///
/// **Before the fix**: This would deadlock with `std::sync::Mutex`
/// **After the fix**: This works correctly with `parking_lot::ReentrantMutex`
///
/// **Related:**
/// - Issue: <https://github.com/torrust/torrust-tracker-deployer/issues/164>
/// - ADR: Remove `UserOutput` Mutex (docs/decisions/user-output-mutex-removal.md)
#[test]
fn user_output_allows_same_thread_multiple_acquisitions() {
    // Create UserOutput using the production pattern that fixed issue #164
    let user_output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(
        VerbosityLevel::Silent,
    ))));

    // FIRST ACQUISITION: Simulate controller/error handling context
    let lock1 = user_output.lock(); // First lock acquisition
    {
        let mut output1 = lock1.borrow_mut();
        output1.error("Controller acquired lock for error handling");
    } // Drop RefCell borrow but keep ReentrantMutex lock

    // SECOND ACQUISITION: Simulate ProgressReporter needing the same lock
    // This is the critical test - would deadlock with std::sync::Mutex
    let lock2 = user_output.lock(); // Same thread, second acquisition
    {
        let mut output2 = lock2.borrow_mut();
        output2.success("ProgressReporter successfully acquired lock - no deadlock!");
    }

    // If we reach this point without hanging, reentrancy is working correctly
    println!("✅ Issue #164 verification: Same-thread multiple acquisitions work");
}

/// Test that verifies `RefCell` provides necessary interior mutability
///
/// `UserOutput` methods require `&mut self`, but `ReentrantMutex` only provides `&T`.
/// `RefCell` bridges this gap by providing interior mutability through runtime borrow checking.
///
/// **Technical Details:**
/// - `ReentrantMutex<UserOutput>` → `&UserOutput` (shared reference)  
/// - `RefCell<UserOutput>` → `RefMut<UserOutput>` → `&mut UserOutput` (mutable access)
///
/// **Related:**
/// - Issue: <https://github.com/torrust/torrust-tracker-deployer/issues/164>
/// - ADR: Remove `UserOutput` Mutex (docs/decisions/user-output-mutex-removal.md)
#[test]
fn refcell_enables_mutable_useroutput_methods() {
    let user_output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(
        VerbosityLevel::Silent,
    ))));

    // Verify we can call mutable UserOutput methods through RefCell
    let lock = user_output.lock();
    let mut output = lock.borrow_mut();

    // These calls require `&mut self` - would fail without RefCell interior mutability
    output.data("RefCell provides interior mutability");
    output.warn("Multiple mutable operations work correctly");
    output.result("UserOutput methods accessible through shared ReentrantMutex reference");

    // Test passes if compilation succeeds and runtime borrows work
    println!("✅ RefCell verification: Interior mutability enables &mut self methods");
}
