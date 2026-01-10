//! Configuration and settings for whis.
//!
//! This module contains:
//! - `TranscriptionProvider` enum (provider selection)
//! - Default values for settings
//! - Preset system for post-processing

mod defaults;
mod preset;
mod provider;

pub use defaults::*;
pub use preset::{Preset, PresetSource};
pub use provider::TranscriptionProvider;
