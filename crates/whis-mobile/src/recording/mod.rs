//! Recording and transcription business logic.
//!
//! This module contains the core recording and transcription functionality,
//! separated from the Tauri command handlers in `commands/recording.rs`.
//!
//! ## Modules
//!
//! - `config` - Load transcription configuration from Tauri store
//! - `pipeline` - Post-processing, clipboard, and event handling

pub mod config;
pub mod pipeline;
