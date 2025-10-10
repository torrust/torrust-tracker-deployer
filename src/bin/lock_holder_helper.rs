//! Helper binary for multi-process lock testing
//!
//! This binary acquires a lock and holds it for a specified duration,
//! allowing integration tests to verify inter-process locking behavior.
//!
//! # Usage
//!
//! ```bash
//! lock_holder_helper <file_path> <duration_seconds>
//! ```
//!
//! # Example
//!
//! ```bash
//! # Hold lock on test.json for 5 seconds
//! lock_holder_helper ./test.json 5
//! ```
//!
//! # Exit Codes
//!
//! - `0`: Successfully acquired and released lock
//! - `1`: Invalid arguments or lock acquisition failed

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use torrust_tracker_deployer_lib::infrastructure::persistence::filesystem::file_lock::FileLock;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <duration_seconds>", args[0]);
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);
    let duration_secs: u64 = args[2].parse().expect("Duration must be a positive number");

    // Acquire lock silently (only output on error via Result)
    let _lock = FileLock::acquire(&file_path, Duration::from_secs(10))?;

    // Hold the lock for the specified duration
    thread::sleep(Duration::from_secs(duration_secs));

    // Lock released automatically on drop
    Ok(())
}
