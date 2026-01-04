//! Recording mode strategies
//!
//! Different ways to capture audio input:
//! - Microphone: Record from system microphone
//! - File: Load audio from file
//! - Stdin: Read audio from standard input

pub mod file;
pub mod microphone;
pub mod stdin;

pub use file::FileMode;
pub use microphone::MicrophoneConfig;
pub use stdin::StdinMode;
