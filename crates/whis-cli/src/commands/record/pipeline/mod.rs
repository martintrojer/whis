//! Pipeline phases for the record command
//!
//! The record command follows a clear pipeline:
//! 1. Record/Load - Get audio from source
//! 2. Transcribe - Convert audio to text
//! 3. Process - Apply post-processing and presets
//! 4. Output - Display or copy to clipboard

pub mod output;
pub mod process;
pub mod transcribe;

pub use output::{OutputMode, output};
pub use process::{ProcessingConfig, process};
pub use transcribe::{TranscriptionConfig, transcribe};
