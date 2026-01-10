//! Logging macros for consistent output across whis crates.
//!
//! # Macros
//!
//! - `verbose!()` - Debug info, only shown when verbose mode enabled
//! - `info!()` - General information messages
//! - `warn!()` - Warning messages
//! - `error!()` - Error messages
//!
//! # Usage
//!
//! ```ignore
//! use whis_core::{verbose, info, warn, error};
//!
//! verbose!("Debug details: {}", value);  // Only if set_verbose(true)
//! info!("Processing file: {}", path);
//! warn!("Deprecated option used");
//! error!("Failed to connect: {}", err);
//! ```

use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Enable or disable verbose logging
pub fn set_verbose(enabled: bool) {
    VERBOSE.store(enabled, Ordering::SeqCst);
}

/// Check if verbose logging is enabled
pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::SeqCst)
}

/// Log a formatted message if verbose mode is enabled
#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        if $crate::verbose::is_verbose() {
            eprintln!("[verbose] {}", format!($($arg)*));
        }
    };
}

/// Log an info message (always printed)
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!("[info] {}", format!($($arg)*));
    };
}

/// Log a warning message (always printed)
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("[warn] {}", format!($($arg)*));
    };
}

/// Log an error message (always printed)
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[error] {}", format!($($arg)*));
    };
}
